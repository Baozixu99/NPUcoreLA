
# 基础环境配置
rustc --version
rustc 1.77.0-nightly (bf3c6c5be 2024-02-01)

Loong Arch GCC 12：百度网盘链接: https://pan.baidu.com/s/1xHriNdgcNzzn-X9U73sHlw 提取码: 912v

# 运行方式与运行效果
`cd os && make`即可。 
# uboot加载内核
`cd os && make run` 在easy-fs-fuse目录下生成uImage

`cd ../util/qemu-2k1000/` 进入qemu-2k1000目录

`./runqemu`启动qmeu,并在启动过程中按C进入uboot命令行

`=> tftpboot uImage` 加载内核镜像

`=> bootm` 进入系统

## 其他
`make clean`: 清理已经编译的项目（包括用户程序， 系统和FAT镜像）

