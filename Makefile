all:
	gzip -d rootfs-ubifs-ze.img.gz
	mkdir -p easy-fs-fuse
	mv rootfs-ubifs-ze.img easy-fs-fuse/rootfs-ubifs-ze.img

