# ARM RSTiny - Rust Bare Metal OS Makefile

# 项目配置
PROJECT_NAME = rstiny

TOOL_PATH = ../tools/orangepi5

kernel_elf = ../target/$(TARGET)/$(MODE)/$(PROJECT_NAME)
wrapper_bin = wrapper.bin
kernel_bin = $(kernel_elf).bin
kernel_img = $(kernel_elf).img
kernel_uimg = $(kernel_elf).uimg
kernel_asm = $(kernel_elf)_asm.txt
kernel_linux = $(kernel_elf)_linux.bin

ifeq ($(MODE), release)
	MODE_ARG := --release
endif

# 编译选项
CARGO_FLAGS = $(MODE_ARG) --target $(TARGET)

# 默认目标
all: build

../axplat-aarch64-d3000m-n80-laptop/tools/wrapper.bin: 
	@echo "Preparing EFI wrapper..."
	aarch64-linux-musl-gcc -c ../axplat-aarch64-d3000m-n80-laptop/tools/wrapper.S -o ../axplat-aarch64-d3000m-n80-laptop/tools/wrapper.o
	aarch64-linux-musl-objcopy -O binary ../axplat-aarch64-d3000m-n80-laptop/tools/wrapper.o ../axplat-aarch64-d3000m-n80-laptop/tools/wrapper.bin

# 编译项目
build: ../axplat-aarch64-d3000m-n80-laptop/tools/wrapper.bin
	@echo "Building $(PROJECT_NAME) with integrated EFI wrapper..."
	cargo build $(CARGO_FLAGS)
	@echo "Build completed: $(kernel_elf)"
	
	@echo "Generating $(kernel_bin) (includes EFI wrapper)..."
	@rust-objcopy --binary-architecture=aarch64 -O binary $(kernel_elf) $(kernel_bin)

	@echo "Dump $(kernel_asm)"
	@rust-objdump -d --print-imm-hex $(kernel_elf) > $(kernel_asm)

	@echo "Generating $(kernel_uimg)..."
	@mkimage -A arm64 -O linux -T kernel -C none -a 0x400000 -e 0x4000000 -n "$(PROJECT_NAME)" -d $(kernel_bin) $(kernel_uimg)
	@echo "Build completed. The kernel binary now includes EFI wrapper support."

	@echo "Generating $(kernel_linux)..."
	@cat ../axplat-aarch64-d3000m-n80-laptop/tools/wrapper.bin $(kernel_bin) > $(kernel_linux)

tftp: $(kernel_linux)
	@echo "Copy $(PROJECT_NAME) to TFTP directory..."
	scp $(kernel_linux) greatwall@10.88.21.33:/tftpboot/arceos_debin.bin