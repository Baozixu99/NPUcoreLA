# NPUcoreLA

NPUcoreLA来自于2023年操作系统大赛功能赛二等奖作品[NPUcore+LA](https://gitlab.eduxiji.net/educg-group-17066-1466467/202310699111039-2789)，NPUcore+LA支持在QEMU-2K0500，以及龙芯2K0500上运行，为了满足大赛要求，我们对NPUcore+LA进行了适配工作（[适配qemu-2k1000工作](./docs/适配qemu-2k1000过程.md)），NPUcoreLA能够成功运行在QEMU-2K1000以及龙芯2K1000上。在初次提交测评时，NPUcoreLA得到了79分。经过修复部分测试用例后，我们最终满分通过了初赛的测试。目前，NPUcoreLA在QEMU-2K1000上已支持lua、busybox、lmbench以及部分系统调用测试用例。我们计划进一步添加对2K1000实机开发板的更多测试用例支持，同时探索支持ext4文件系统的过程。
**在NPUcore+LA的基础上，我们做的主要工作内容如下：**
- **适配龙芯2K1000星云板**：NPUcoreLA能够在QEMU-2K1000以及龙芯2K1000上运行。
- **修复测试用例**：将测评分数从79分提升到满分。
- **修复NPUcoreLA中的bug**：包括修复sbrk、pipe、openat系统调用等。
- **模块化内核结构**：拆解arch/register模块、virtio-drivers模块做成crate，便于后续模块化内核的工作。
- **重构核心系统调用**：如sbrk、mmap、munmap、exec、getpid、fork、clone、open、write、fstat、link等系统调用。
- **支持新的用户程序**：如打印三级页表结构，打印TCb信息。
- **增加新的系统调用支持**：statx、getrandom、dup2等系统调用
- **探索ext4实现（没有实现ext4）**：详细分析文档位于/docs中。

## NPUcoreLA系统架构图

<img src="https://gitlab.eduxiji.net/T202410460992502/oskernel2024-npucorela/-/raw/main/docs/picture/NPUcore%E6%9E%B6%E6%9E%84%E5%9B%BE%EF%BC%88%E6%97%A0%E8%89%B2%E7%89%88%EF%BC%89.png?inline=false" width="60%">

## 基础环境配置

```shell
rustc --version
rustc 1.77.0-nightly (bf3c6c5be 2024-02-01)
```

Loong Arch GCC 12：百度网盘链接: https://pan.baidu.com/s/1xHriNdgcNzzn-X9U73sHlw 提取码: 912v

Loong Arch GCC 13： https://github.com/LoongsonLab/oscomp-toolchains-for-oskernel

## 环境准备

```
make qemu-download
```

## 运行方式

```
make all
make run
```

## 其他

`make clean`: 清理已经编译的项目（包括用户程序， 系统和FAT镜像）

## 相关文档

- [适配qemu-2k1000工作](./docs/适配qemu-2k1000过程.md)

- [QEMU运行NPUcoreLA步骤](./docs/qemu运行NPUcoreLA.md)
- [核心系统调用的实现](./docs/核心系统调用的实现.md)
- [rust学习记录](./docs/rust学习记录.md)
- [内存管理](./docs/内存管理.md)
- [进程管理](./docs/进程管理.md)
- [文件系统](./docs/文件系统.md)
- [系统调用](./docs/系统调用.md)
- [NPUcore+使用说明](./docs/NPUcore+操作系统使用说明书.md)
- [Ext4文件系统实现探索：与FAT32的比较分析](./docs/Ext4文件系统实现探索：与FAT32的比较分析.md)
## 目录结构

```shell
NPUcoreLA
├── Makefile            # 顶级Makefile，用于编译和构建整个项目
├── docs                # 相关文件
├── easy-fs-fuse        # 包含文件系统和操作系统内核镜像  
├── os                  # 内核核心代码存放目录  
│   ├── Makefile        # 内核的构建和运行脚本  
│   ├── buildfs.sh      # 脚本，用于构建文件系统  
│   ├── run_script      # 脚本，用于uboot加载内核  
│   ├── src             # 内核源代码目录  
│   │   ├── arch        # 包含了与平台相关的汇编代码和包装函数  
│   │   ├── console.rs  # 控制台日志消息发送的源代码  
│   │   ├── drivers     # 设备驱动程序目录  
│   │   ├── fs          # 文件系统源代码目录  
│   │   ├── linker.ld   # 定义了内存布局和各个段的起始地址  
│   │   ├── load_img.S  # 用于将文件系统镜像插入到.data段  
│   │   ├── main.rs     # 内核的入口函数源代码  
│   │   ├── mm          # 内存管理源代码目录  
│   │   ├── syscal      # 系统调用源代码目录  
│   │   ├── task        # 进程管理源代码目录  
│   │   └── timer.rs    # 时钟中断管理和计时器源代码   
│   └── target          # 内核构建输出的目标文件存放目录  
├── user                # 用户空间程序存放目录  
└── util                # 工具目录  
    ├── mkimage         # 用于创建镜像文件  
    └── qemu-2k1000     # QEMU模拟器配置
```



