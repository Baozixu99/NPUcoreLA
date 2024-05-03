all:
	gzip -d rootfs-ubifs-ze.img.gz
	mkdir -p easy-fs-fuse
	mv rootfs-ubifs-ze.img easy-fs-fuse/rootfs-ubifs-ze.img
	cd os && make all

run:
	cd os && make run

gdb:
	cd os && make gdb

qemu-download:
	mkdir -p util/qemu-2k1000/tmp
	cd util/qemu-2k1000/tmp && wget https://gitlab.educg.net/wangmingjian/os-contest-2024-image/-/raw/master/qemu-2k1000-static.20240126.tar.xz
	cd util/qemu-2k1000/tmp && tar xavf qemu-2k1000-static.20240126.tar.xz
	rm -rf util/qemu-2k1000/tmp/qemu-2k1000-static.20240126.tar.xz
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
