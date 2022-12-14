use std::cell::{Cell, RefCell};

use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    execution_engine::{ExecutionEngine, JitFunction},
    module::Module,
    values::{FunctionValue, PointerValue},
    AddressSpace, OptimizationLevel,
};

use crate::{
    measure_time,
    vm::{opcode::OpCode, VM},
};

use super::Interpreter;

pub type RunFunc = unsafe extern "C" fn(*mut i32, *mut i32);

const MOD_NAME: &str = "vmt_vm_mod";
const FUNC_NAME: &str = "vt_vm";

struct FunctionContext<'ctx> {
    function: FunctionValue<'ctx>,
    acc: PointerValue<'ctx>,
    acc_ptr: PointerValue<'ctx>,
    lc: PointerValue<'ctx>,
    lc_ptr: PointerValue<'ctx>,
    spilled_cnt: Cell<Option<usize>>,
    spilled_bbs: Vec<BasicBlock<'ctx>>,
}

pub struct JittedInterpreter<'ctx> {
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
    fun_context: RefCell<Option<FunctionContext<'ctx>>>,
}

impl<'ctx> JittedInterpreter<'ctx> {
    pub fn new(context: &'ctx Context, opt_level: OptimizationLevel) -> Self {
        let module = context.create_module(MOD_NAME);
        let execution_engine = module.create_jit_execution_engine(opt_level).unwrap();
        let builder = module.get_context().create_builder();

        Self {
            module,
            execution_engine,
            builder,
            fun_context: RefCell::new(None),
        }
    }

    fn setup_jit_function(&self) {
        let i32_type = self.module.get_context().i32_type();
        let i32ptr_type = self
            .module
            .get_context()
            .i32_type()
            .ptr_type(AddressSpace::Generic);
        let fun_type = self
            .module
            .get_context()
            .void_type()
            .fn_type(&[i32ptr_type.into(), i32ptr_type.into()], false);
        let function = self.module.add_function(FUNC_NAME, fun_type, None);

        let basic_block = self
            .module
            .get_context()
            .append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);

        let acc_ptr = self.builder.build_alloca(i32ptr_type, "");
        let lc_ptr = self.builder.build_alloca(i32ptr_type, "");

        let acc = self.builder.build_alloca(i32_type, "acc");
        let lc = self.builder.build_alloca(i32_type, "lc");
        self.fun_context.replace(Some(FunctionContext {
            function,
            acc,
            acc_ptr,
            lc,
            lc_ptr,
            spilled_bbs: vec![],
            spilled_cnt: Cell::new(None),
        }));

        if let Some(fun_context) = self.fun_context.borrow().as_ref() {
            let first_param = fun_context
                .function
                .get_first_param()
                .unwrap()
                .into_pointer_value();
            let second_param = fun_context
                .function
                .get_nth_param(1)
                .unwrap()
                .into_pointer_value();

            self.builder.build_store(acc_ptr, first_param);
            self.builder.build_store(lc_ptr, second_param);

            let acc_ptr = self.builder.build_load(acc_ptr, "");
            let acc_ptr_val = self.builder.build_load(acc_ptr.into_pointer_value(), "");
            self.builder.build_store(acc, acc_ptr_val);

            let lc_ptr = self.builder.build_load(lc_ptr, "");
            let lc_ptr_val = self.builder.build_load(lc_ptr.into_pointer_value(), "");
            self.builder.build_store(lc, lc_ptr_val);

            let basic_block = self.module.get_context().append_basic_block(function, "bb");
            self.builder.build_unconditional_branch(basic_block);
            self.builder.position_at_end(basic_block);
        }
    }

    fn jit_compile(&self) -> Option<JitFunction<RunFunc>> {
        unsafe { self.execution_engine.get_function(FUNC_NAME).ok() }
    }
}

impl<'ctx> Interpreter for JittedInterpreter<'ctx> {
    fn run(&self, vm: &VM) {
        // Prepare function environment
        self.setup_jit_function();

        let mut halt = false;
        let mut has_jump = false;

        let basic_blocks = vm.running_program.build_basic_blocks();

        for basic_block in basic_blocks.iter() {
            // Build branch to next basic block
            for instr in basic_block.iter() {
                // Write LLVM bitcode inside the function environment
                let instr = *instr;
                match OpCode::try_from(instr).unwrap() {
                    OpCode::HALT => {
                        self.halt(vm, instr);
                        halt = true;
                    }
                    OpCode::CLRA => self.clra(vm, instr),
                    OpCode::INC3A => self.inc3a(vm, instr),
                    OpCode::DECA => self.deca(vm, instr),
                    OpCode::SETL => self.setl(vm, instr),
                    OpCode::BACK7 => {
                        self.back7(vm, instr);
                        has_jump = true;
                    }
                    OpCode::SPILL => {
                        self.spill(vm, instr);
                        has_jump = true;
                    }
                }
            }

            if halt || has_jump {
                has_jump = false;
                continue;
            }

            if let Some(fun_context) = self.fun_context.borrow().as_ref() {
                let basic_block = self
                    .module
                    .get_context()
                    .append_basic_block(fun_context.function, "bb");
                self.builder.build_unconditional_branch(basic_block);
                self.builder.position_at_end(basic_block);
            }
        }

        // Print LLVM module to the stderr
        // self.module.print_to_stderr();

        // Verify the module's correctness before executing the result.
        match self.module.verify() {
            Ok(_) => (),
            Err(msg) => panic!(
                "Error while verifying LLVM module: {}",
                msg.to_str().unwrap()
            ),
        }

        // Run the compiled code
        if let Some(fun) = self.jit_compile() {
            let elapsed_time = measure_time!({
                unsafe {
                    let mut acc = vm.registers.acc_value();
                    let mut lc = vm.registers.lc_value();

                    // Call the compiled-in-memory function
                    fun.call(&mut acc as *mut i32, &mut lc as *mut i32);

                    vm.registers.acc.replace(acc);
                    vm.registers.lc.replace(lc);
                }
            });
            vm.running_time.replace(elapsed_time);
        } else {
            panic!("Unable to JIT compile VM code.")
        }
    }

    fn halt(&self, _: &VM, _: u8) {
        if let Some(fun_context) = self.fun_context.borrow().as_ref() {
            let acc_value = self.builder.build_load(fun_context.acc, "");
            let acc_ptr = self.builder.build_load(fun_context.acc_ptr, "");
            self.builder
                .build_store(acc_ptr.into_pointer_value(), acc_value);

            let lc_value = self.builder.build_load(fun_context.lc, "");
            let lc_ptr = self.builder.build_load(fun_context.lc_ptr, "");
            self.builder
                .build_store(lc_ptr.into_pointer_value(), lc_value);

            // Build return instruction
            self.builder.build_return(None);
        }
    }

    fn clra(&self, _: &VM, _: u8) {
        if let Some(fun_context) = self.fun_context.borrow().as_ref() {
            let zero = self.module.get_context().i32_type().const_zero();
            self.builder.build_store(fun_context.acc, zero);
        }
    }

    fn inc3a(&self, _: &VM, _: u8) {
        if let Some(fun_context) = self.fun_context.borrow().as_ref() {
            let three = self.module.get_context().i32_type().const_int(3, false);

            let prev_value = self.builder.build_load(fun_context.acc, "");

            let inc = self
                .builder
                .build_int_nsw_add(prev_value.into_int_value(), three, "");

            self.builder.build_store(fun_context.acc, inc);
        }
    }

    fn deca(&self, _: &VM, _: u8) {
        if let Some(fun_context) = self.fun_context.borrow().as_ref() {
            let one = self.module.get_context().i32_type().const_int(1, false);

            let prev_value = self.builder.build_load(fun_context.acc, "");

            let dec = self
                .builder
                .build_int_nsw_sub(prev_value.into_int_value(), one, "");

            self.builder.build_store(fun_context.acc, dec);
        }
    }

    fn setl(&self, _: &VM, _: u8) {
        if let Some(fun_context) = self.fun_context.borrow().as_ref() {
            let acc_value = self.builder.build_load(fun_context.acc, "");
            self.builder.build_store(fun_context.lc, acc_value);
        }
    }

    fn back7(&self, _: &VM, _: u8) {
        if let Some(fun_context) = self.fun_context.borrow_mut().as_mut() {
            // Get current basic block reference
            // let current_bb = fun_context.function.get_last_basic_block().unwrap();
            let zero = self.module.get_context().i32_type().const_int(0, false);
            let one = self.module.get_context().i32_type().const_int(1, false);

            let lc_prev = self.builder.build_load(fun_context.lc, "");

            let dec = self
                .builder
                .build_int_nsw_sub(lc_prev.into_int_value(), one, "");

            self.builder.build_store(fun_context.lc, dec);

            let lc_curr = self.builder.build_load(fun_context.lc, "");

            let comparison = self.builder.build_int_compare(
                inkwell::IntPredicate::SGT,
                lc_curr.into_int_value(),
                zero,
                "",
            );

            // Build branch
            let new_bb = self
                .module
                .get_context()
                .append_basic_block(fun_context.function, "bb");
            let dest_bb = fun_context.spilled_bbs[fun_context.spilled_cnt.get().unwrap()]
                .get_next_basic_block()
                .unwrap();

            fun_context
                .spilled_cnt
                .replace(Some(fun_context.spilled_cnt.get().unwrap() + 1));

            self.builder
                .build_conditional_branch(comparison, dest_bb, new_bb);

            // Modifier the builder's cursor
            self.builder.position_at_end(new_bb);
        }
    }

    fn spill(&self, _vm: &VM, _instr: u8) {
        if let Some(fun_context) = self.fun_context.borrow_mut().as_mut() {
            let current_bb = fun_context.function.get_last_basic_block().unwrap();
            fun_context.spilled_bbs.push(current_bb);

            // Update the counter if None
            match fun_context.spilled_cnt.get() {
                None => {
                    fun_context.spilled_cnt.replace(Some(0));
                }
                _ => (),
            }

            let basic_block = self
                .module
                .get_context()
                .append_basic_block(fun_context.function, "spill.bb");
            self.builder.build_unconditional_branch(basic_block);
            self.builder.position_at_end(basic_block);
        }
    }
}
