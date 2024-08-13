# Filesystem 接口描述
为了能在Linux上进行一般测试及fuzzing测试，需要实现`Trait fuse::Filesystem`.

必须实现此特征才能通过 FUSE 提供用户空间文件系统。 这些方法对应于libfuse中的fuse_lowlevel_ops。这里提供了合理的默认实现，以获得一个不执行任何操作的可安装文件系统。

参考： https://docs.rs/fuse/latest/fuse/trait.Filesystem.html

``` rust

/// Filesystem trait.
///
/// 要通过 FUSE 提供用户空间文件系统，必须实现该trait特性。
/// 这些方法对应于 libfuse 中的 fuse_lowlevel_ops。
/// 在此提供了合理的默认实现，以获得一个不做任何事情的可挂载文件系统。
pub trait Filesystem {
    /// 初始化文件系统。
    /// 最先调用。
    fn init(&mut self, _req: &Request) -> Result<(), c_int> {
        Ok(())
    }

    /// 清理文件系统。
    /// 在文件系统退出时调用。
    fn destroy(&mut self, _req: &Request) {}

    /// 按名称查找目录条目并获取其属性。
    fn lookup(&mut self, _req: &Request, _parent: u64, _name: &OsStr, reply: ReplyEntry) {
        reply.error(ENOSYS);
    }

    /// 忘记inode索引节点.
    /// nlookup 参数表示之前对该 inode 执行的查找次数。
    /// 如果文件系统实现了inode节点生命周期，
    /// 建议每次查找时，inode 只获得一个引用，每次forget时，丢失 nlookup 引用。
    /// 如果节点不需要有有限的生命周期，文件系统可以忽略forget调用。
    /// 卸载时，不能保证所有引用的 inodes 都会收到一条 forget 消息。
    fn forget(&mut self, _req: &Request, _ino: u64, _nlookup: u64) {}

    /// 获取文件属性。
    fn getattr(&mut self, _req: &Request, _ino: u64, reply: ReplyAttr) {
        reply.error(ENOSYS);
    }

    /// 设置文件属性。
    fn setattr(&mut self, _req: &Request, _ino: u64, _mode: Option<u32>, _uid: Option<u32>, _gid: Option<u32>, _size: Option<u64>, _atime: Option<Timespec>, _mtime: Option<Timespec>, _fh: Option<u64>, _crtime: Option<Timespec>, _chgtime: Option<Timespec>, _bkuptime: Option<Timespec>, _flags: Option<u32>, reply: ReplyAttr) {
        reply.error(ENOSYS);
    }

    /// 读取符号链接。
    fn readlink(&mut self, _req: &Request, _ino: u64, reply: ReplyData) {
        reply.error(ENOSYS);
    }

    /// 创建文件节点。
    /// 创建普通文件、字符设备、块设备、fifo 或套接字节点。
    fn mknod(&mut self, _req: &Request, _parent: u64, _name: &OsStr, _mode: u32, _rdev: u32, reply: ReplyEntry) {
        reply.error(ENOSYS);
    }

    /// 创建目录。
    fn mkdir(&mut self, _req: &Request, _parent: u64, _name: &OsStr, _mode: u32, reply: ReplyEntry) {
        reply.error(ENOSYS);
    }

    /// 删除文件。
    fn unlink(&mut self, _req: &Request, _parent: u64, _name: &OsStr, reply: ReplyEmpty) {
        reply.error(ENOSYS);
    }

    /// 删除一个目录。
    fn rmdir(&mut self, _req: &Request, _parent: u64, _name: &OsStr, reply: ReplyEmpty) {
        reply.error(ENOSYS);
    }

    /// 创建符号链接。
    fn symlink(&mut self, _req: &Request, _parent: u64, _name: &OsStr, _link: &Path, reply: ReplyEntry) {
        reply.error(ENOSYS);
    }

    /// 重命名文件。
    fn rename(&mut self, _req: &Request, _parent: u64, _name: &OsStr, _newparent: u64, _newname: &OsStr, reply: ReplyEmpty) {
        reply.error(ENOSYS);
    }

    /// 创建硬链接。
    fn link(&mut self, _req: &Request, _ino: u64, _newparent: u64, _newname: &OsStr, reply: ReplyEntry) {
        reply.error(ENOSYS);
    }

    /// 打开文件。
    /// 打开文件的flags标志（O_CREAT、O_EXCL、O_NOCTTY 和 O_TRUNC 除外）可在flags标志中提供。
    /// 文件系统可以在 fh 中存储任意文件句柄（pointer, index, etc）,
    /// 并在其他所有文件操作（read, write, flush, release, fsync）中使用.
    /// 文件系统也可以实现无状态文件 I/O，而不在 fh 中存储任何内容。
    /// 文件系统还可以设置一些flags标志（direct_io、keep_cache），以改变打开文件的方式。
    /// 参见<fuse_common.h>文件中的fuse_file_info结构，以了解更多详情。
    fn open(&mut self, _req: &Request, _ino: u64, _flags: u32, reply: ReplyOpen) {
        reply.opened(0, 0);
    }

    /// 读取数据。
    /// 除了 EOF 或出错时，读取数据应准确发送请求的字节数、
    /// 否则，数据的其余部分将以 0 代替。
    /// 一个例外情况是当文件以'direct_io'模式打开时，
    /// 在这种情况下，读取系统调用的返回值，将反映此操作的返回值。
    /// fh 将包含 open 方法设置的值，如果 open 方法没有设置任何值，则为未定义。
    fn read(&mut self, _req: &Request, _ino: u64, _fh: u64, _offset: i64, _size: u32, reply: ReplyData) {
        reply.error(ENOSYS);
    }

    /// 写入数据。
    /// 写入时应准确返回所请求的字节数，出错时除外。
    /// 文件以 "direct_io "模式打开时例外,
    /// 在这种情况下，写入系统调用的返回值将反映此操作的返回值。
    /// fh 将包含 open 方法设置的值，
    /// 或者, 如果 open 方法没有设置任何值，则返回值为未定义值。
    fn write(&mut self, _req: &Request, _ino: u64, _fh: u64, _offset: i64, _data: &[u8], _flags: u32, reply: ReplyWrite) {
        reply.error(ENOSYS);
    }

     /// flush刷新方法。
     /// 这在打开的文件的每次 close() 上调用。 
     /// 由于文件描述符可以被复制(dup, dup2, fork)，对于一个open调用可能有很多flush调用。
     /// 文件系统不应该假设刷新总是会在某个时间后被调用写入，或者根本不会调用。 
     /// fh 将包含由 open 方法设置的值，如果 open 方法没有设置任何值，则该方法将是未定义的。
     /// 注意：该方法的名称具有误导性，因为（与 fsync 不同）
     /// 文件系统不会强制刷新挂起的写入。 刷新数据的原因之一是，
     /// 如果文件系统想要返回写入错误。 如果文件系统支持文件锁定操作（setlk，getlk）,
     /// 它应该删除属于“lock_owner”的所有锁。
    fn flush(&mut self, _req: &Request, _ino: u64, _fh: u64, _lock_owner: u64, reply: ReplyEmpty) {
        reply.error(ENOSYS);
    }

     /// 释放一个打开的文件。
     /// 当不再有对打开文件的引用时调用 Release：
     /// 所有文件描述符被关闭并且所有内存映射都被取消映射。
     /// 对于每一次打开的调用将会有一个release释放调用。
     /// 文件系统可能会回复错误，但错误值不会返回到 触发释放的 close() 或 munmap()
     /// fh 将包含 open 方法设置的值，或者将是未定义的，如果 open 方法没有设置任何值。 
     /// flags 将包含与 open 相同的标志。
    fn release(&mut self, _req: &Request, _ino: u64, _fh: u64, _flags: u32, _lock_owner: u64, _flush: bool, reply: ReplyEmpty) {
        reply.ok();
    }

    /// 同步文件内容。
    /// 如果 datasync 参数为非零，则只刷新用户数据、而不是元数据。
    fn fsync(&mut self, _req: &Request, _ino: u64, _fh: u64, _datasync: bool, reply: ReplyEmpty) {
        reply.error(ENOSYS);
    }

    /// 打开一个目录。
    /// 文件系统可以在 fh 中存储任意文件句柄（指针、索引等），并在其他所有操作中使用该句柄。
    /// 在其他所有目录流操作（readdir, releaseir, fsyncdir）。
    /// 文件系统也可以实现无状态目录 I/O，而不在 fh 中存储任何内容。
    /// 目录 I/O 而不在 fh 中存储任何内容，但这样就无法实现符合标准的目录流操作，
    /// 因为目录内容可能会在 opendir 和 releasedir 之间发生变化。
    fn opendir(&mut self, _req: &Request, _ino: u64, _flags: u32, reply: ReplyOpen) {
        reply.opened(0, 0);
    }

    /// 读取目录。
    /// 发送一个使用 buffer.fill() 填充的缓冲区，其大小不超过请求的大小。
    /// 在流结束时发送一个空缓冲区。
    /// 如果 opendir 方法没有设置任何值，那么 fh 将是未定义的。
    fn readdir(&mut self, _req: &Request, _ino: u64, _fh: u64, _offset: i64, reply: ReplyDirectory) {
        reply.error(ENOSYS);
    }

    /// 释放打开的目录。
    /// 每调用一次 opendir，就会调用一次 releasedir。
    /// fh包含 opendir 方法设置的值，或者如果 opendir 方法没有设置任何值，那么 fh 将是未定义的。
    fn releasedir(&mut self, _req: &Request, _ino: u64, _fh: u64, _flags: u32, reply: ReplyEmpty) {
        reply.ok();
    }

    /// 同步目录内容。
    /// 如果设置了 datasync 参数，则只刷新目录内容，而不刷新 meta data 元数据。
    /// fh 将包含 opendir 方法设置的值， 如果 opendir 方法没有设置任何值，那么 fh 将是未定义的。
    fn fsyncdir (&mut self, _req: &Request, _ino: u64, _fh: u64, _datasync: bool, reply: ReplyEmpty) {
        reply.error(ENOSYS);
    }

    /// 获取文件系统统计数据。
    fn statfs(&mut self, _req: &Request, _ino: u64, reply: ReplyStatfs) {
        reply.statfs(0, 0, 0, 0, 0, 512, 255, 0);
    }

    /// 设置扩展属性。
    fn setxattr(&mut self, _req: &Request, _ino: u64, _name: &OsStr, _value: &[u8], _flags: u32, _position: u32, reply: ReplyEmpty) {
        reply.error(ENOSYS);
    }

    /// 获取扩展属性。
    /// 如果 `size` 为 0，则应使用 `reply.size()` 发送值的大小。
    /// 如果 `size` 不是 0，且值大小合适，则应使用 `reply.data()` 发送，或者 `reply.error(ERANGE)`。
    fn getxattr(&mut self, _req: &Request, _ino: u64, _name: &OsStr, _size: u32, reply: ReplyXattr) {
        reply.error(ENOSYS);
    }

    /// 列出扩展属性名称。
    /// 如果 `size` 为 0，应使用 `reply.size()` 发送值的大小。
    /// 如果 `size` 不是 0，且值大小合适，则应使用 `reply.data()` 发送，或者 `reply.error(ERANGE)`。
    fn listxattr(&mut self, _req: &Request, _ino: u64, _size: u32, reply: ReplyXattr) {
        reply.error(ENOSYS);
    }

    /// 删除扩展属性。
    fn removexattr(&mut self, _req: &Request, _ino: u64, _name: &OsStr, reply: ReplyEmpty) {
        reply.error(ENOSYS);
    }

    /// 检查文件访问权限。
    /// 将调用 access() 系统调用。如果给定了 "default_permissions" mount 选项，则不会调用此方法。
    /// 在 Linux 内核 2.4.x 版本以下，不会调用该方法
    fn access(&mut self, _req: &Request, _ino: u64, _mask: u32, reply: ReplyEmpty) {
        reply.error(ENOSYS);
    }

    /// 创建并打开文件。
    /// 如果文件不存在，首先用指定的模式创建文件，然后打开它。
    /// 打开文件。打开标志 flags（O_NOCTTY 除外）在flags标志中提供。
    /// 文件系统可以将任意文件句柄（指针、索引等）存储在 fh,
    /// 并在其他所有文件操作（read, write, flush, release, fsync）中使用
    /// 文件系统还可以设置一些flags标志（direct_io、keep_cache)
    /// 文件系统可以设置这些标志，以改变打开文件的方式。
    /// 参见<fuse_common.h>的fuse_file_info结构了解更多详情。
    /// 如果此方法未实现 或 Linux 内核版本早于 2.6.15，
    /// 则 mknod() 和 open() 方法将被调用。
    fn create(&mut self, _req: &Request, _parent: u64, _name: &OsStr, _mode: u32, _flags: u32, reply: ReplyCreate) {
        reply.error(ENOSYS);
    }

    /// 测试 POSIX 文件锁。
    fn getlk(&mut self, _req: &Request, _ino: u64, _fh: u64, _lock_owner: u64, _start: u64, _end: u64, _typ: u32, _pid: u32, reply: ReplyLock) {
        reply.error(ENOSYS);
    }

    /// 获取、修改或释放 POSIX 文件锁。
    /// 对于 POSIX 线程（NPTL）来说，pid 和所有者之间是 1-1 的关系，但除此之外，情况并非总是如此。
    /// 为检查锁所有权，必须使用 "fi->owner"。'struct flock' 中的 l_pid 字段只能用于在 getlk() 中填写该字段。
    /// 注意：如果锁定方法没有实现，内核仍将允许文件锁定在本地运行。
    /// 因此，这些方法只对网络文件系统和类似系统有意义。
    fn setlk(&mut self, _req: &Request, _ino: u64, _fh: u64, _lock_owner: u64, _start: u64, _end: u64, _typ: u32, _pid: u32, _sleep: bool, reply: ReplyEmpty) {
        reply.error(ENOSYS);
    }

    /// 将文件中的块索引映射到设备中的块索引。
    /// 注意：使用 "blkdev "选项的文件系统才对块设备支持的文件系统有意义。
    fn bmap(&mut self, _req: &Request, _ino: u64, _blocksize: u32, _idx: u64, reply: ReplyBmap) {
        reply.error(ENOSYS);
    }

    /// 仅限 macOS： 
    /// 重命名加密卷。在启动过程中将 fuse_init_out.flags 设置为 FUSE_VOL_RENAME 以启用
    #[cfg(target_os = "macos")]
    fn setvolname(&mut self, _req: &Request, _name: &OsStr, reply: ReplyEmpty) {
        reply.error(ENOSYS);
    }

    /// 仅限 macOS（未注明）
    #[cfg(target_os = "macos")]
    fn exchange(&mut self, _req: &Request, _parent: u64, _name: &OsStr, _newparent: u64, _newname: &OsStr, _options: u64, reply: ReplyEmpty) {
        reply.error(ENOSYS);
    }

    /// 仅限 macOS： 查询扩展时间（bkuptime 和 crtime）。在启动过程中将 fuse_init_out.flags 设置为 FUSE_XTIMES，以启用
    #[cfg(target_os = "macos")]
    fn getxtimes(&mut self, _req: &Request, _ino: u64, reply: ReplyXTimes) {
        reply.error(ENOSYS);
    }
}
```


