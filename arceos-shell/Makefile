# Arceos Shell Makefile

PROJECT_NAME = arceos-shell

PLAT_NAME = axplat-aarch64-d3000m-n80-laptop
PLAT_CONFIG = $(PWD)/axplat-aarch64-d3000m-n80-laptop/axconfig.toml

kernel_elf = $(PWD)/target/$(TARGET)/$(MODE)/$(PROJECT_NAME)
kernel_bin = $(kernel_elf).bin
kernel_img = $(kernel_elf).img
kernel_uimg = $(kernel_elf).uimg
kernel_asm = $(kernel_elf)_asm.txt
kernel_linux = $(PWD)/$(PROJECT_NAME)_linux.bin

ifeq ($(MODE), release)
	MODE_ARG := --release
endif

FEATURE_FLAGS = "axstd/myplat axstd/log-level-trace axstd/bus-mmio axstd/page-alloc-4g axstd/driver-ramdisk"

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

# 编译项目
build:  $(OUT_CONFIG)
	@echo "Building $(PROJECT_NAME) with integrated EFI wrapper..."
	cargo build $(CARGO_FLAGS)
	@echo "Build completed: $(kernel_elf)"
	
	@echo "Generating $(kernel_bin) (includes EFI wrapper)..."
	@rust-objcopy --binary-architecture=aarch64 -O binary $(kernel_elf) $(kernel_bin)

	@echo "Dump $(kernel_asm)"
	@rust-objdump -d --print-imm-hex $(kernel_elf) > $(kernel_asm)

tftp: $(kernel_bin)
	@echo "Copy $(PROJECT_NAME) to TFTP directory..."
	cp $(kernel_bin) arceos.bin
	xtool tftpd .
