CC := clang
CFLAGS := -O3 -std=c11

TARGET_EXEC := generator
BUILD_DIR := ./build

all: $(BUILD_DIR)/$(TARGET_EXEC)
	@mkdir -p ./scenarios/
	@echo "[info] :: Compilation done!"

$(BUILD_DIR)/$(TARGET_EXEC): gen.c
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) $< -o $@

run: all
	@echo "[info] :: Generating scenarios ..."
	@./$(BUILD_DIR)/$(TARGET_EXEC)
	@echo "[info] :: Generation done! Scenarios can be found inside ./scenarios/"

.PHONY: clean
clean:
	rm -rf ./build/
	rm -rf ./scenarios/