#!/bin/bash

## read the document to prepare the host environment
SCRIPTPATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
if [ -f ./2kfs.img ] ; then
   echo "2kfs.img exist! removed"
   rm -f ./2kfs.img
fi

qemu-img create -f qcow2 2kfs.img 2G

if [ $? -ne 0 ] ; then
  echo "create image failed"
  exit -1
fi

sudo qemu-nbd -c /dev/nbd0 ./2kfs.img

if [ $? -ne 0 ] ; then
  echo "connect image to nbd device failed!"
  echo "please install nbd kernel module first!"
  echo "   modprobe nbd maxparts=12"
  echo "if /dev/nbd0 is already taken, change all nbd0 in this script to another one such as nbd1"
  exit -2
fi

sudo echo -e 'n\n\n\n\n\n\nw\nq\n'| sudo fdisk /dev/nbd0

if [ $? -ne 0 ] ; then
  echo "disk partition failed"
  exit -3
fi

sudo mkfs.ext4 /dev/nbd0p1

if [ $? -ne 0 ] ; then
  echo "mkfs.ext4 failed"
  exit -4
fi

sudo mount /dev/nbd0p1 /mnt

if [ $? -ne 0 ] ; then
  echo "mount /dev/nbd0p1 failed"
  exit -5
fi

sudo bash -c "lzcat ${SCRIPTPATH}/rootfs-la.cpio.lzma | cpio -idmv -D /mnt &> ./cpio.log"

if [ $? -ne 0 ] ; then
  echo "unpack rootfs failed"
  exit -6
fi

sudo mkdir /mnt/boot 

sudo cp ${SCRIPTPATH}/uImage /mnt/boot/

sudo umount /mnt

sudo qemu-nbd -d /dev/nbd0

echo "done"

