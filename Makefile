all:
#	mv ./os/cargo ./os/.cargo
#	mv ./user/cargo ./user/.cargo
	gzip -dk rootfs-ubifs-ze.img.gz
	mkdir -p easy-fs-fuse
	mv rootfs-ubifs-ze.img easy-fs-fuse/rootfs-ubifs-ze.img
	cd os && make all

run:
	cd os && make run

gdb:
	cd os && make gdb

qemu-download:
	mkdir -p util/qemu-2k1000/tmp
	cd util/qemu-2k1000/tmp && wget https://github.com/LoongsonLab/2k1000-materials/releases/download/qemu-static-20240401/qemu-static-20240401.tar.xz
	cd util/qemu-2k1000/tmp && tar xavf qemu-static-20240401.tar.xz
	rm -rf util/qemu-2k1000/tmp/qemu-static-20240401.tar.xz
	rm -rf util/qemu-2k1000/tmp/qemu/2k1000
	rm -rf util/qemu-2k1000/tmp/qemu/runqemu
	rm -rf util/qemu-2k1000/tmp/qemu/README.md
	rm -rf util/qemu-2k1000/tmp/qemu/include
	rm -rf util/qemu-2k1000/tmp/qemu/var
	mkdir -p easy-fs-fuse
	sudo chmod 777 easy-fs-fuse/
	chmod +x util/mkimage
	chmod +x util/qemu-2k1000/gz/runqemu2k1000
	chmod +x util/qemu-2k1000/tmp/qemu/bin/qemu-system-loongarch64
