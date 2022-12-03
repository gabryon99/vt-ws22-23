use std::cell::RefCell;

use inkwell::{
builder::Builder,
    context::{Context},
    execution_engine::{ExecutionEngine, JitFunction},
    module::Module,
    values::{FunctionValue, PointerValue},
    OptimizationLevel,
};

use crate::vm::{VM, opcode::OpCode};

use super::Interpreter;

type RunFunc = unsafe extern "C" fn(i32, i32) -> (i32, i32);

const MOD_NAME: &str = "vmt_vm_mod";
const FUNC_NAME: &str = "vt_vm";

struct FunctionContext<'ctx> {
    function: FunctionValue<'ctx>,
    acc: PointerValue<'ctx>,
    lc: PointerValue<'ctx>,
}

pub struct JittedInterpreter<'ctx> {
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
    fun_context: RefCell<Option<FunctionContext<'ctx>>>
}

impl<'ctx> JittedInterpreter<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let module = context.create_module(MOD_NAME);
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::Default)
            .unwrap();
        let builder = module.get_context().create_builder();

        Self {
            module,
            execution_engine,
            builder,
            fun_context: RefCell::new(None)
        }
    }

    fn setup_jit_function(&self) {

        let i32_type = self.module.get_context().i32_type();
        let i32_x2_type = self.module.get_context().struct_type(&[i32_type.into(), i32_type.into()], false);
        let fun_type = i32_x2_type.fn_type(&[i32_type.into(), i32_type.into()], false);
        let function = self.module.add_function(FUNC_NAME, fun_type, None);
        let basic_block = self.module.get_context().append_basic_block(function, "entry");

        self.builder.position_at_end(basic_block);

        let acc = self.builder.build_alloca(i32_type, "acc");
        let lc = self.builder.build_alloca(i32_type, "lc");

        self.fun_context.replace(Some(FunctionContext { function, acc, lc }));

        if let Some(fun_context) = self.fun_context.borrow().as_ref() {
            let initial_acc = fun_context
                .function
                .get_nth_param(0)
                .unwrap()
                .into_int_value();
            let initial_lc = fun_context
                .function
                .get_nth_param(1)
                .unwrap()
                .into_int_value();

            self.builder.build_store(fun_context.acc, initial_acc);
            self.builder.build_store(fun_context.lc, initial_lc);
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

        // Write LLVM bitcode inside the function environment
        for i in 0..vm.running_program.data.len() {
            let instr = vm.running_program.data[i];
            match OpCode::try_from(instr).unwrap() {
                OpCode::HALT => self.halt(vm, instr),
                OpCode::CLRA => self.clra(vm, instr),
                OpCode::INC3A => self.inc3a(vm, instr),
                OpCode::DECA => self.deca(vm, instr),
                OpCode::SETL => self.setl(vm, instr),
                OpCode::BACK7 => self.back7(vm, instr),
            }
        }

        // self.module.print_to_stderr();
        
        // Verify the module's correctness before executing the result.
        match self.module.verify() {
            Ok(_) => (),
            Err(msg) => panic!("Error while verifying LLVM module: {}", msg.to_str().unwrap()),
        }

        // Run the compiled code
        if let Some(fun) = self.jit_compile() {

            use std::time::Instant;
            let now = Instant::now();

            unsafe {
                let res = fun.call(vm.registers.acc_value(), vm.registers.lc_value());
                vm.registers.acc.replace(res.0);
                vm.registers.lc.replace(res.1);
            }

            let elapsed = now.elapsed();
            println!("[jitted] :: Elapsed: {:.2?}", elapsed);

        } else {
            panic!("Unable to JIT compile VM code.")
        } 
       
    }

    fn halt(&self, _: &VM, _: u8) {
        // Halt should return the new registers
        let i32_type = self.module.get_context().i32_type();
        let i32_x2_type = self.module.get_context().struct_type(&[i32_type.into(), i32_type.into()], false);

        if let Some(fun_context) = self.fun_context.borrow().as_ref() {
            let ret = self.builder.build_alloca(i32_x2_type, "rv");
            // Store acc and lc into ret
            let acc_value = self.builder.build_load(fun_context.acc, "");
            let ret_acc = self.builder.build_struct_gep(ret, 0, "rv_acc").unwrap();
            self.builder.build_store(ret_acc, acc_value);

            let lc_value = self.builder.build_load(fun_context.lc, "");
            let ret_lc = self.builder.build_struct_gep(ret, 1, "rv_lc").unwrap();
            self.builder.build_store(ret_lc, lc_value);

            let val = self.builder.build_load(ret, "");

            // Build return instruction
            self.builder.build_return(Some(&val));
        }
    }

    fn clra(&self, _: &VM, _: u8) {
        let zero = self.module.get_context().i32_type().const_zero();
        if let Some(fun_context) = self.fun_context.borrow().as_ref() {
            self.builder.build_store(fun_context.acc, zero);
        }
    }

    fn inc3a(&self, _: &VM, _: u8) {
        let three = self.module.get_context().i32_type().const_int(3, false);
        if let Some(fun_context) = self.fun_context.borrow().as_ref() {
            let prev_value = self.builder.build_load(fun_context.acc, "");
            let inc = self
                .builder
                .build_int_add(prev_value.into_int_value(), three, "");
            self.builder.build_store(fun_context.acc, inc);
        }
    }

    fn deca(&self, _: &VM, _: u8) {
        let one = self.module.get_context().i32_type().const_int(1, false);
        if let Some(fun_context) = self.fun_context.borrow().as_ref() {
            let prev_value = self.builder.build_load(fun_context.acc, "");
            let inc = self
                .builder
                .build_int_sub(prev_value.into_int_value(), one, "");
            self.builder.build_store(fun_context.acc, inc);
        }
    }

    fn setl(&self, _: &VM, _: u8) {
        if let Some(fun_context) = self.fun_context.borrow().as_ref() {
            let acc_value = self.builder.build_load(fun_context.acc, "");
            self.builder.build_store(fun_context.lc, acc_value);
        }
    }

    fn back7(&self, _: &VM, _: u8) {
        todo!("Jumps are not allowed for the moment.")
    }
}
