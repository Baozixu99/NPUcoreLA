# Building
TARGET := loongarch64-unknown-linux-gnu
MODE := debug
CORE_NUM := 1

KERNEL_ELF = target/$(TARGET)/$(MODE)/os
KERNEL_BIN = $(KERNEL_ELF).bin
KERNEL_UIMG = $(KERNEL_ELF).ui

ARCH=${shell echo ${TARGET}|cut -d- -f1 |grep -o "^[a-zA-Z]\+[[:digit:]]*"}
# BOARD
BOARD ?= laqemu

LDBOARD = la2k1000

SDDEV := $(shell lsblk -do NAME | grep sd | tail -1)
SDCARD := /dev/$(SDDEV)

# Disassembly
DISASM ?= -x

ifeq ($(BOARD), 2K1000)
	BOARD = 2k1000
endif

# Binutils
OBJCOPY := loongarch64-linux-gnu-objcopy
OBJDUMP := loongarch64-linux-gnu-objdump
READELF := loongarch64-linux-gnu-readelf

ifndef LOG
	LOG_OPTION := "log_off"
endif

IMG_DIR := ../easy-fs-fuse
FAT32_IMG_LA = rootfs-ubifs-ze.img
U_FAT32_DIR := $(IMG_DIR)
FAT32_IMG_NAME := $(FAT32_IMG_RV)
ifeq ($(ARCH), loongarch64)
	FAT32_IMG_NAME = $(FAT32_IMG_LA)
endif

#ifeq ($(MODE), debug)
#	LA_2K500_DISABLE_EH_FRAME := -D EH_ENABLED
#endif
ifeq ($(MODE), debug)
	LA_2K1000_DISABLE_EH_FRAME := -D EH_ENABLED
endif
# FAT32_2K500_LN -> FAT32_2K1000_LN
U_FAT32 := ${U_FAT32_DIR}/$(FAT32_IMG_NAME)	
FAT32_2K1000_LN = $(shell readlink -f $(U_FAT32_DIR))/$(FAT32_IMG_LA)


# QEMU_2K500_DIR=../util/qemu-2k500/gz
# QEMU_2K500=$(QEMU_2K500_DIR)/runqemu2k500
QEMU_2K1000_DIR=../util/qemu-2k1000/gz
QEMU_2K1000=$(QEMU_2K1000_DIR)/runqemu2k1000
IMG_DIR=../easy-fs-fuse
U_IMG=$(IMG_DIR)/uImage
#LA_2K500_SERIAL_PORT->LA_2K1000_SERIAL_PORT   LA_2K500_PORT_FREQ->LA_2K1000_PORT_FREQ
LA_DEBUGGER_SERIAL_PORT = $$(python3 -m serial.tools.list_ports 1A86:7523 -q | head -n 1)
LA_DEBUGGER_PORT_FREQ = $(LA_DEBUGGER_SERIAL_PORT) 115200
LA_2K1000_SERIAL_PORT = $$(python3 -m serial.tools.list_ports 067B:2303 -q | head -n 1)
LA_2K1000_PORT_FREQ = $(LA_2K1000_SERIAL_PORT) 115200
MINITERM_START_CMD=python3 -m serial.tools.miniterm --dtr 0 --rts 0 --filter direct 

LA_ENTRY_POINT = 0x9000000090000000
LA_LOAD_ADDR =   0x9000000090000000

RUST_LA_TOOLCHAIN = $(shell rustup show |grep "\(la\|loongarch\)\(64\)\?" |cut -d' ' -f1|head -n 1)


# build: env $(KERNEL_BIN) $(FS_IMG) fat32
all: build mv
# rust-user build mv
mv:
	mv $(KERNEL_BIN) ../kernel.bin
build: env $(KERNEL_BIN)
setup:
    export RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
    export RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
env: setup # switch-toolchain
	-(rustup target list | grep "$(TARGET) (installed)") || rustup target add $(TARGET)
	if [ "$(dpkg --list|grep "ii[[:space:]]*expect")"="" ];then true;else sudo apt install expect;fi
ifneq ($(ARCH), loongarch64)
#	-cargo install cargo-binutils --vers ~0.2
endif
#	-rustup component add rust-src
#	-rustup component add llvm-tools-preview

# build all user programs
user: c-user rust-user
c-user:
	@cd ../user && make c-user BOARD=$(BOARD) MODE=$(MODE)
rust-user: env
	@cd ../user && make rust-user BOARD=$(BOARD) MODE=$(MODE)
# dev/zero永远输出0
sdcard:
    ifeq ($(findstring WSL, $(shell uname -a)), )
		@echo "Are you sure write to $(SDCARD) ? \nYou may also specify another drive with the option \`SDCARD=/dev/<drive>\` [y/N] " && read ans && [ $${ans:-N} = y ]
		./buildfs.sh "$(SDCARD)" "qemu" $(MODE)
    else
		@echo "Are you sure write to E:/ ? [y/N] " && read ans && [ $${ans:-N} = y ]
		@sudo mount $(U_FAT32_DIR)/$(FAT32_IMG_NAME) $(U_FAT32_DIR)/fs
		@powershell.exe 'cd $(U_FAT32_DIR); copy -Recurse -path "$$(pwd)/fs/*" -destination "E:/" -force'
		@sudo umount $(U_FAT32_DIR)/fs
    endif
$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) $@ --strip-all -O binary &
	@$(OBJDUMP) $(KERNEL_ELF) -SC > target/$(TARGET)/$(MODE)/asm_all.txt 
	@$(READELF) -ash $(KERNEL_ELF) > target/$(TARGET)/$(MODE)/sec.txt &
#ifeq ($(ARCH), loongarch64)
uimage: $(KERNEL_BIN)
	../util/mkimage -A loongarch -O linux -T kernel -C none -a $(LA_LOAD_ADDR) -e $(LA_ENTRY_POINT) -n NPUcore+ -d $(KERNEL_BIN) $(KERNEL_UIMG)
	-@rm $(U_IMG)
	@cp -f $$(pwd)/target/$(TARGET)/$(MODE)/os.ui $(U_IMG)
#endif
# -@rm src/linker.ld
#	@$(OBJDUMP) -SC $(KERNEL_ELF) > $(KERNEL_ELF).S

fat32:
ifneq ($(BOARD),laqemu)
	./buildfs.sh "$(U_FAT32_DIR)/$(FAT32_IMG_NAME)" "laqemu" $(MODE)
else
	./buildfs.sh "$(U_FAT32_DIR)/$(FAT32_IMG_NAME)" 2k1000 $(MODE)
endif

remake-qemu-flash-fat-img: fat32 qemu-flash-fat-img

qemu-flash-fat-img:
ifeq ($(BOARD), laqemu)
	@./la_fat $(QEMU_2K1000)
else 
	@./la_fat $(MINITERM_START_CMD) $(LA_2K1000_PORT_FREQ) 
endif



src/linker.ld: .FORCE
	-@rm src/linker.ld
	cpp -P $(LA_2K1000_DISABLE_EH_FRAME) -D $(LDBOARD) src/linker.in.ld $@

fsimg-format:
	sudo mkfs.vfat -F 32 ${U_FAT32}
update-usr:user fat32

# change:
# 	mv cargo .cargo
# 	cd ../user && mv cargo .cargo
kernel: #change
	@echo Platform: $(BOARD)
    ifeq ($(MODE), debug)
		@cargo build --no-default-features --features "board_$(BOARD) $(LOG_OPTION)" --target $(TARGET)
    else
		@cargo build --no-default-features --release --features "board_$(BOARD) $(LOG_OPTION)"  --target $(TARGET)
    endif

clean:
	@cargo clean
# -@rm src/linker.ld
	-@rm ../easy-fs-fuse/uImage
	-@rm ../easy-fs-fuse/rootfs-ubifs-ze.img
	-@cd ../user && make clean

run: clean env update-usr run-inner 

run-inner: build uimage do-run

runsimple: do-run
do-run:
#	-ln -s $(FAT32_2K500_LN) $(QEMU_2K500_DIR)/$(FAT32_IMG_LA)
#	echo $(FAT32_2K500_LN)`  
#First, link the image into the directory.
# FAT32_2K1000_LN -> FAT32_2K1000_LN
ifeq ($(BOARD), laqemu)
	-ln -s $(FAT32_2K1000_LN) $(QEMU_2K1000_DIR)/$(FAT32_IMG_LA)
	@echo "========WARNING!========"
	@echo "The next command is expecting a modified runqemu2k1000 script where any potential and implicit \"current working directory\" has been replaced by a generated script storage path."
	@./run_script $(QEMU_2K1000)
else ifeq ($(BOARD), 2k1000)
	@./run_script $(MINITERM_START_CMD) $(LA_2K1000_PORT_FREQ)
endif


new-gdb: build gdb
gdb:
ifeq ($(BOARD),laqemu)
	./run_script $(QEMU_2K1000) "-S"
else ifeq ($(BOARD), 2k1000)
	@./la_gdbserver minicom -D $(LA_DEBUGGER_PORT_FREQ)
endif


# os-doc:
# 	cargo doc --no-deps --release --target riscv64gc-unknown-none-elf --document-private-items
# 	@./clear_doc.sh os
ch-default:
	@DEFAULT_STR=$$(grep -o "default[[:space:]]*=[[:space:]]\[[[:space:]]*\"board_.\+\"[[:space:]]*\]" Cargo.toml);
	@if [ $$(echo "$$DEFAULT_STR"|grep "qemu") ]; then echo "found"; else echo "not found"; fi;

# test_addr:
# 	@echo "ENT" $(LA_ENTRY_POINT) "\nld " $(LA_LOAD_ADDR)

.PHONY: user update gdb new-gdb monitor .FORCE

