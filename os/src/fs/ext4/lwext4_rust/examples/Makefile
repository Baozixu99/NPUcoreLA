arch ?= riscv64
board ?= qemu
target := $(arch)gc-unknown-none-elf
# target := $(arch)imac-unknown-none-elf
# when using new toolchain after 2024-04, release mode get stuck
# mode := release
mode := debug
kernel := target/$(target)/$(mode)/lwext4_rust_examples
bin := target/$(target)/$(mode)/lwext4_rust_examples.bin
img := target/$(target)/$(mode)/ext4.img

sysroot := $(shell rustc --print sysroot)
objdump := $(shell find $(sysroot) -name llvm-objdump) --arch-name=$(arch)
objcopy := $(shell find $(sysroot) -name llvm-objcopy)

BUILD_ARGS += --target $(target)
ifeq ($(mode), release)
	BUILD_ARGS += --release
endif
ifeq ($(board), fu740)
	BUILD_ARGS += --features board-fu740
endif

.PHONY: kernel build clean qemu run

build: kernel $(bin)

kernel:
	cargo build $(BUILD_ARGS)

$(bin): kernel
	$(objcopy) $(kernel) --strip-all -O binary $(bin)

asm:
	$(objdump) -d $(kernel) | less

sym:
	$(objdump) -t $(kernel) | less

header:
	$(objdump) -x $(kernel) | less

clean:
	cargo clean

qemu: $(img)
	qemu-system-$(arch) \
		-machine virt \
		-no-reboot \
		-serial mon:stdio \
		-display none \
		-bios default \
		-kernel $(kernel) \
		-device ahci,id=ahci0 \
		-drive file=$(img),if=none,format=raw,id=x0 \
		-device virtio-blk-device,drive=x0 \
		-netdev user,id=net0 \
		-device e1000e,netdev=net0

# 8M ext4 fs
$(img):
	dd if=/dev/zero of=$@ bs=512 count=16384
	mkfs.ext4 -F -O ^metadata_csum_seed -L ext4-test $@
	chmod 777 $@

run: build qemu
