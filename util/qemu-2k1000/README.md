# qemu for 2k1000 board

## install

1. unpack this package in /tmp:

```
cd /tmp
tar xpvf <path-to-xz-file>/qemu-2k1000-static.20240126.tar.xz
```

2. create rootfs image for qemu

you need to have qemu-img (often in package qemu-utils), fdisk and mkfs.ext4
in your system path.

make sure nbd kernel module is loaded: 

```bash
modprobe nbd maxparts=12
```

then run this script(only needed before first run):

```bash
cd /tmp/qemu/2k1000
./create_qemu_image.sh
```

if nothing goes wrong, 2kfs.img should be created in /tmp/qemu/2k1000/.

3. run

```bash
cd /tmp/qemu
./runqemu
```

By default, it will first run u-boot, then a buildroot linux system, type root
to login, no passwd needed.

