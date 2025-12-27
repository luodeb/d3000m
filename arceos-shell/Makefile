# Arceos Shell Makefile

PROJECT_NAME = arceos-shell

TOOL_PATH = $(PWD)/tools/orangepi5
PLAT_NAME = axplat-aarch64-d3000m-n80-laptop
PLAT_CONFIG = $(PWD)/axplat-aarch64-d3000m-n80-laptop/axconfig.toml

kernel_elf = $(PWD)/target/$(TARGET)/$(MODE)/$(PROJECT_NAME)
kernel_bin = $(kernel_elf).bin
kernel_img = $(kernel_elf).img
kernel_uimg = $(kernel_elf).uimg
kernel_asm = $(kernel_elf)_asm.txt
kernel_linux = $(kernel_elf)_linux.bin

ifeq ($(MODE), release)
	MODE_ARG := --release
endif

FEATURE_FLAGS = "axstd/myplat axstd/log-level-warn axstd/bus-mmio axstd/page-alloc-4g axstd/driver-ramdisk axstd/smp"

# 编译选项
CARGO_FLAGS = $(MODE_ARG) --target $(TARGET) --features $(FEATURE_FLAGS)

OUT_CONFIG = $(PWD)/.axconfig.toml

CONFIG_ARGS := \
  $(PWD)/$(APP)/defconfig.toml $(PLAT_CONFIG) \
  -w 'arch="$(ARCH)"' \
  -w 'platform="$(PLAT_NAME)"' \
  -o "$(OUT_CONFIG)"

export AX_ARCH = $(ARCH)
export AX_PLATFORM = $(PLAT_NAME)
export AX_MODE=$(MODE)
export AX_LOG=$(LOG)
export AX_TARGET=$(TARGET)
export AX_CONFIG_PATH=$(OUT_CONFIG)

# 默认目标
all: build

$(OUT_CONFIG): 
	echo "Generating configuration file at $(OUT_CONFIG)..."
	axconfig-gen $(CONFIG_ARGS)

../axplat-aarch64-d3000m-n80-laptop/tools/wrapper.bin: 
	@echo "Preparing EFI wrapper..."
	aarch64-linux-musl-gcc -c ../axplat-aarch64-d3000m-n80-laptop/tools/wrapper.S -o ../axplat-aarch64-d3000m-n80-laptop/tools/wrapper.o
	aarch64-linux-musl-objcopy -O binary ../axplat-aarch64-d3000m-n80-laptop/tools/wrapper.o ../axplat-aarch64-d3000m-n80-laptop/tools/wrapper.bin

# 编译项目
build: $(OUT_CONFIG) ../axplat-aarch64-d3000m-n80-laptop/tools/wrapper.bin
	@echo "Building $(PROJECT_NAME)..."
	cargo build $(CARGO_FLAGS)
	@echo "Build completed: $(kernel_elf)"
	
	@echo "Generating $(kernel_bin)..."
	@rust-objcopy -O binary $(kernel_elf) $(kernel_bin)

	@echo "Dump $(kernel_asm)"
	@rust-objdump -d --print-imm-hex $(kernel_elf) > $(kernel_asm)

	@echo "Generating $(kernel_uimg)..."
	@mkimage -A arm64 -O linux -T kernel -C none -a 0x400000 -e 0x4000000 -n "$(PROJECT_NAME)" -d $(kernel_bin) $(kernel_uimg)

	@echo "Generating $(kernel_linux)..."
	@cat ../axplat-aarch64-d3000m-n80-laptop/tools/wrapper.bin $(kernel_bin) > $(kernel_linux)

tftp: $(kernel_linux)
	@echo "Copy $(PROJECT_NAME) to TFTP directory..."
	scp $(kernel_linux) greatwall@10.88.21.33:/tftpboot/arceos_linux.bin
