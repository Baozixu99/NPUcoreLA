### NPUcoreLA

_____

NPUcoreLA来自于2023年操作系统大赛功能赛二等奖作品NPUcore+LA，NPUcore+LA支持在QEMU-2K000，以及龙芯2K0500上运行，我们针对大赛要求，对NPUcore+LA进行了适配，能NPUcoreLA能够成功运行在QEMU-2K1000以及龙芯2K1000上。目前在QEMU-2K1000上已经支持lua、busybox、lmbench和部分系统调用测例， 后续还计划添加2K1000实机开发板的更多测例支持。

### NPUcoreLA系统架构图

------------------


# 基础环境配置
rustc --version

rustc 1.77.0-nightly (bf3c6c5be 2024-02-01)

Loong Arch GCC 12：百度网盘链接: https://pan.baidu.com/s/1xHriNdgcNzzn-X9U73sHlw 提取码: 912v

Loong Arch GCC 13： https://github.com/LoongsonLab/oscomp-toolchains-for-oskernel
## 1.环境准备
`make qemu-download` 
# 2.运行方式
`make all`

`make run`
## 其他
`make clean`: 清理已经编译的项目（包括用户程序， 系统和FAT镜像）

### 目录结构
