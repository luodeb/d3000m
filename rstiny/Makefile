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

wrapper: ../axplat-aarch64-d3000m-n80-laptop/tools/wrapper.bin
	@echo "Preparing EFI wrapper..."
	aarch64-linux-musl-gcc -c ../axplat-aarch64-d3000m-n80-laptop/tools/wrapper.S -o ../axplat-aarch64-d3000m-n80-laptop/tools/wrapper.o
	aarch64-linux-musl-objcopy -O binary ../axplat-aarch64-d3000m-n80-laptop/tools/wrapper.o ../axplat-aarch64-d3000m-n80-laptop/tools/wrapper.bin

# 编译项目
build: wrapper
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

img: build
	@echo "Make $(PROJECT_NAME) on Orange Pi 5..."
	sudo bash $(TOOL_PATH)/make_boot.sh $(kernel_bin) $(kernel_img)

flash: $(kernel_bin)
	@echo "Flash $(PROJECT_NAME) to Orange Pi 5..."
	sudo bash $(TOOL_PATH)/make_flash.sh uimg=$(kernel_bin)

tftp: $(kernel_uimg)
	@echo "Copy $(PROJECT_NAME) to TFTP directory..."
	scp $(kernel_linux) greatwall@10.88.21.33:/tftpboot/arceos_linux.bin

mini: $(kernel_uimg)
	@echo "Copy $(PROJECT_NAME) to MINI directory..."
	scp -o 'ProxyJump web@20.13.0.88' $(kernel_uimg) debin@10.1.135.123:/data/docker/tftpboot/data/