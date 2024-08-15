# Ext4文件系统实现探索：与FAT32的比较分析

参加“全国大学生计算机系统能力大赛-操作系统设计赛(全国)-OS内核实现赛道”，目前已进入决赛阶段。初赛NPUcoreLA使用的文件系统为FAT32，而决赛要求改为ext4文件系统。由于时间和知识的限制，无法完全实现ext4文件系统。因此，本文档记录了对ext4实现的探索过程，并展示了已完成的工作。

## 1.探索目标

- 探索ext4和FAT32文件系统之间的差异。

- 结合参考内核，讨论ext4的设计和实现，特别是与指令集相关的部分。

- 分析ext4对底层指令集架构的依赖性，比较LoongArch和RISC-V实现。


## 2.FAT32和Ext4文件系统概述

### 2.1 FAT32

#### 2.1.1 设计原理

FAT32文件系统使用文件分配表（FAT）来管理文件和目录。它使用链式分配，每个文件和目录由簇链连接。

#### 2.1.2 结构和组件

- **Boot Sector**: 包含文件系统类型和相关信息。
- **FAT**: 用于记录每个簇的状态。
- **Root Directory**: 存储文件和目录的元数据。
- **Data Area**: 存储实际数据。

#### 2.1.3 优势和限制

- **优势**: 简单易实现，兼容性强。
- **限制**: 不支持大文件，元数据支持有限，性能和扩展性较差。

### 2.2 Ext4

#### 2.2.1 设计原理

Ext4文件系统是Ext3的改进版，采用多种技术提高性能和可靠性，如延迟分配、扩展、日志记录等。

#### 2.2.2 结构和组件

- **Superblock**: 存储文件系统信息。
- **Inode Table**: 存储文件和目录的元数据。
- **Block Group**: 数据被分成多个块组，每个块组包含数据块、inode表和超级块的备份。
- **Journaling**: 用于记录文件系统的变化，以便在系统崩溃时恢复。

#### 2.2.3 优势和限制

- **优势**: 支持大文件和大容量存储，元数据丰富，性能和可靠性高。
- **限制**: 实现复杂度高，设计和实现要求较高。

## 3. FAT32和Ext4的详细比较

### 3.1 文件分配和管理

- **FAT32**: 使用链式分配，简单但效率低。
- **Ext4**: 使用inode和扩展，效率高，支持大文件和动态分配。

### 3.2 元数据处理

- **FAT32**: 元数据支持有限，仅包含基本信息。
- **Ext4**: 支持丰富的元数据，如权限、所有者、时间戳等，并采用日志记录保证数据一致性。

### 3.3 性能和可扩展性

- **FAT32**: 适用于中小型存储，性能和扩展性较差。
- **Ext4**: 设计用于大容量存储，具有更好的性能和可扩展性。

### 3.4 错误处理和恢复

- **FAT32**: 基本的错误处理，缺乏高级恢复机制。
- **Ext4**: 通过日志记录和校验和机制提供可靠的错误处理和恢复功能。

## 4. Ext4的设计和实现

### 4.1 Ext4的核心组件

- **Inode**: 存储文件和目录的元数据。
- **超级块**: 存储文件系统信息。
- **块组描述符**: 描述块组的结构和内容。
- **数据块**: 存储实际数据。

### 4.2 日志机制

日志机制记录文件系统的变化，确保在系统崩溃后能够恢复。Ext4的日志记录支持三种模式：数据模式、元数据模式和订单模式，提供不同级别的数据保护。

### 4.3 扩展和动态分配

Ext4采用扩展（extent）技术来提高文件分配效率。扩展是连续数据块的集合，可以减少文件分配的碎片化，提高文件系统的性能。

### 4.4 目录结构和索引

Ext4使用HTree索引来管理目录，提高目录查找和操作的效率。HTree是一种B+树结构，能够快速定位目录项。

### 4.5 与FAT32设计方面的对比

- **文件分配**: FAT32使用链式分配，Ext4使用inode和扩展。
- **元数据**: FAT32支持有限，Ext4元数据丰富。
- **性能和扩展性**: FAT32性能和扩展性较差，Ext4性能和扩展性优越。
- **错误处理和恢复**: FAT32基本错误处理，Ext4具有日志记录和校验和机制。

## 5. 结合参考内核的分析

### 5.1 参考内核概述

参考内核[MinotaurOS](https://github.com/Dr-TSNG/MinotaurOS)基于RISC-V架构，实现了Ext4和FAT32文件系统。通过分析参考内核，可以了解Ext4和FAT32的具体实现细节，并与我们内核进行对比。MinotaurOS 实现了类似于 Linux 的虚拟文件系统功能，并为此设计了一个名为 `FileSystem` 的通用接口。这个接口仅包含两个方法：`metadata` 和 `root`，前者用于获取文件系统的元数据（包括文件系统类型和 VFS 标志），后者用于获取文件系统的根目录 inode。

```rust
/// 文件系统元数据
///
/// 一个文件系统在刚创建时不关联任何挂载点，通过 `move_mount` 挂载到命名空间。
pub struct FileSystemMeta {
    /// 文件系统类型
    pub fstype: FileSystemType,

    /// 文件系统标志
    pub flags: VfsFlags,
}

/// 文件系统
pub trait FileSystem: Send + Sync {
    /// 文件系统元数据
    fn metadata(&self) -> &FileSystemMeta;

    /// 根 Inode
    fn root(&self) -> Arc<dyn Inode>;
}

impl FileSystemMeta {
    pub fn new(fstype: FileSystemType, flags: VfsFlags) -> Self {
        Self { fstype, flags }
    }
}
pub struct DevFileSystem {
    vfsmeta: FileSystemMeta,
    ino_pool: AtomicUsize,
    root: LateInit<Arc<RootInode>>,
}
impl FileSystem for DevFileSystem {
    fn metadata(&self) -> &FileSystemMeta {
        &self.vfsmeta
    }

    fn root(&self) -> Arc<dyn Inode> {
        self.root.clone()
    }
}

```

### 5.2 参考内核的Ext4实现

#### 5.2.1 设计和实现细节

参考内核的Ext4实现包括超级块、inode表、块组和日志机制等组件。

``` rust
//inode.rs
/// Ext4Inode 结构体代表一个EXT4文件系统的 inode（文件系统节点）。
/// 它包含了inode的元数据，以及指向文件系统的弱引用和对inode引用的Arc锁。
pub struct Ext4Inode {
    /// metadata字段存储了inode的元数据，如文件权限、修改时间等。
    metadata: InodeMeta,
    /// fs字段是一个弱引用，指向Ext4FileSystem，表示这个inode所属的文件系统。
    /// 使用弱引用是为了避免循环引用导致的内存泄漏。
    fs: Weak<Ext4FileSystem>,
    /// inode_ref字段是一个原子引用计数的Mutex包裹的Ext4InodeRef。
    /// 它用于管理对这个inode的引用计数，并通过Mutex提供线程安全的访问。
    inode_ref: Arc<Mutex<Ext4InodeRef>>,
}

impl Ext4Inode {
    // 创建根目录的 Ext4Inode 实例
    pub fn root(fs: &Arc<Ext4FileSystem>, parent: Option<Arc<dyn Inode>>) -> Arc<Self>;
}

impl Inode for Ext4Inode {
    // 获取 inode 的元数据
    fn metadata(&self) -> &InodeMeta;

    // 获取 inode 所属的文件系统的弱引用
    fn file_system(&self) -> Weak<dyn FileSystem>;
}

#[async_trait]
impl InodeInternal for Ext4Inode {
    // 异步直接读取文件数据到缓冲区
    async fn read_direct(&self, buf: &mut [u8], offset: isize) -> SyscallResult<isize>;

    // 异步直接写入数据从缓冲区到文件
    async fn write_direct(&self, buf: &[u8], offset: isize) -> SyscallResult<isize>;

    // 异步截断文件到指定大小
    async fn truncate_direct(&self, size: isize) -> SyscallResult;

    // 异步加载 inode 的子项
    async fn load_children(self: Arc<Self>, inner: &mut InodeMetaInner) -> SyscallResult;

    // 异步创建文件或目录
    async fn do_create(self: Arc<Self>, mode: InodeMode, name: &str) -> SyscallResult<InodeChild>;

    // 异步将文件或目录移动到当前目录
    async fn do_movein(self: Arc<Self>, name: &str, inode: Arc<dyn Inode>) -> SyscallResult<InodeChild>;

    // 异步删除文件或目录
    async fn do_unlink(self: Arc<Self>, target: &InodeChild) -> SyscallResult;
}
```

``` rust
//mod.rs
pub struct Ext4FileSystem {
    device: Arc<dyn BlockDevice>, // 块设备的引用
    vfsmeta: FileSystemMeta,       // 文件系统的元数据
    ext4: Ext4,                    // 用于操作ext4文件系统的结构
    root: ManuallyDrop<LateInit<Arc<Ext4Inode>>>, // 根Inode的初始化包装
}

impl Ext4FileSystem {
    // 创建一个新的Ext4FileSystem实例
    pub fn new(device: Arc<dyn BlockDevice>, flags: VfsFlags) -> Arc<Self>;
    
    // 初始化根Inode
    fn init(&self, inode: Arc<Ext4Inode>);
}

impl FileSystem for Ext4FileSystem {
    // 获取文件系统的元数据
    fn metadata(&self) -> &FileSystemMeta;

    // 获取文件系统的根Inode
    fn root(&self) -> Arc<dyn Inode>;
}
```

``` rust
//wrapper.rs
pub struct Ext4(Ext4BlockWrapper<Ext4Disk>);

impl Ext4 {
    // 创建一个新的 Ext4 实例
    pub fn new(device: Arc<dyn BlockDevice>) -> Self;
}

pub struct Ext4Disk {
    device: Arc<dyn BlockDevice>,
    offset: usize,
}

impl Ext4Disk {
    // 创建一个新的 Ext4Disk 实例
    pub fn new(device: Arc<dyn BlockDevice>) -> Self;
}

impl KernelDevOp for Ext4Disk {
    // 写入数据到块设备
    fn write(op: &mut Self::DevType, data: &[u8]) -> Result<usize, i32>;

    // 从块设备读取数据
    fn read(op: &mut Self::DevType, data: &mut [u8]) -> Result<usize, i32>;

    // 移动块设备的读写偏移量
    fn seek(op: &mut Self::DevType, off: i64, whence: i32) -> Result<i64, i32>;

    // 刷新块设备的缓存（通常用于确保数据写入）
    fn flush(_: &mut Self::DevType) -> Result<usize, i32>;
}
```

### 5.3 参考内核的FAT32实现

#### 5.3.1 设计和实现细节

分析参考内核的关键代码，如引导扇区初始化、文件分配表管理、目录项管理、数据区管理等。

``` rust
//bpb.rs
pub enum BootSectorOffset {
    /// 此项忽略
    JmpBoot = 0,
    /// 此项忽略
    OEMName = 3,
    /// 此项忽略
    DrvNum = 64,
    /// 保留位
    Reserved1 = 65,
    /// 扩展引导标记，用于指明此后的 3 个域可用
    BootSig = 66,
    /// 此项忽略
    VolID = 67,
    /// 磁盘卷标
    ///
    /// 此项必须与根目录中 11 字节长的卷标一致
    VolLab = 71,
    /// 文件系统类型
    ///
    /// 此项可为 FAT12、FAT16、FAT32 之一
    FilSysType = 82,
}

impl BootSectorOffset {
    // 从启动扇区中获取扩展引导标记
    pub fn boot_sig(sector: &[u8]) -> u8;

    // 从启动扇区中获取磁盘卷标字符串
    pub fn vol_lab(sector: &[u8]) -> String;

    // 从启动扇区中提取指定区间的数据
    fn split(sector: &[u8], start: Self, end: Self) -> &[u8];
}

pub enum BPBOffset {
    /// 每扇区字节数
    ///
    /// 取值只能是以下的几种情况：512、1024、2048 或是 4096
    BytsPerSec = 11,
    /// 每簇扇区数
    ///
    /// 其值必须是 2 的整数次方
    SecPerClus = 13,
    /// 保留区中保留扇区的数目
    RsvdSecCnt = 14,
    /// 此卷中 FAT 表的份数，通常为 2
    NumFATs = 16,
    /// 对于 FAT32，此项必须为 0
    RootEntCnt = 17,
    /// 对于 FAT32，此项必须为 0
    TotSec16 = 19,
    /// 此项忽略
    Media = 21,
    /// 对于 FAT32，此项必须为 0
    FATSz16 = 22,
    /// 此项忽略
    SecPerTrk = 24,
    /// 此项忽略
    NumHeads = 26,
    /// 在此 FAT 卷之前所隐藏的扇区数
    HiddSec = 28,
    /// 该卷总扇区数
    TotSec32 = 32,
    /// 一个 FAT 表包含的扇区数
    FATSz32 = 36,
    /// Bits 0-3：活动 FAT 表，只有在镜像禁止时才有效
    ///
    /// Bits 7：0 表示 FAT 实时镜像到所有的 FAT 表中；1 表示只有一个活动的 FAT 表
    ExtFlags = 40,
    /// FAT32 版本号
    ///
    /// 高位为主版本号，低位为次版本号
    FSVer = 42,
    /// 根目录所在第一个簇的簇号
    RootClus = 44,
    /// 保留区中 FAT32 卷 FSINFO 结构所占的扇区数
    FSInfo = 48,
    /// 此项忽略
    BkBootSec = 50,
    /// 保留位
    Reserved = 52,
}

impl BPBOffset {
    // 从启动扇区中获取每扇区字节数
    pub fn bytes_per_sector(sector: &[u8]) -> u16;

    // 从启动扇区中获取每簇扇区数
    pub fn sector_per_cluster(sector: &[u8]) -> u8;

    // 从启动扇区中获取保留区中保留扇区的数目
    pub fn reserved_sectors(sector: &[u8]) -> u16;

    // 从启动扇区中获取 FAT 表的份数
    pub fn fats_number(sector: &[u8]) -> u8;

    // 从启动扇区中获取该卷总扇区数
    pub fn total_sectors(sector: &[u8]) -> u32;

    // 从启动扇区中获取一个 FAT 表包含的扇区数
    pub fn fat_size(sector: &[u8]) -> u32;

    // 从启动扇区中获取扩展标签
    pub fn extend_flags(sector: &[u8]) -> u16;

    // 从启动扇区中获取根目录所在第一个簇的簇号
    pub fn root_cluster(sector: &[u8]) -> u32;

    // 从启动扇区中获取保留区中 FAT32 卷 FSINFO 结构所占的扇区数
    pub fn fs_info(sector: &[u8]) -> u16;

    // 从启动扇区中提取指定区间的数据
    fn split(sector: &[u8], start: Self, end: Self) -> &[u8];
}
```

这些函数提供了从 FAT 文件系统的启动扇区中读取关键参数的能力。`boot_sig`、`vol_lab`、`bytes_per_sector`、`sector_per_cluster`、`reserved_sectors`、`fats_number`、`total_sectors`、`fat_size`、`extend_flags`、`root_cluster` 和 `fs_info` 等函数允许程序正确解析启动扇区的内容。`split` 函数是一个通用工具函数，用于从启动扇区中提取指定范围的字节切片。

``` rust
//dir.rs定义了与FAT32文件系统目录项和长目录项相关的数据结构和函数，以及用于处理文件属性和时间的辅助工具。
enum DirOffset {
    // ...
}

impl DirOffset {
    // 从目录项中获取最后访问时间的日期部分
    fn acc_time(value: &[u8]) -> (u16, u16);

    // 从目录项中获取最后写入时间的日期和时间部分
    fn wrt_time(value: &[u8]) -> (u16, u16);

    // 从目录项中获取创建时间的日期和时间部分
    fn crt_time(value: &[u8]) -> (u16, u16);

    // 从目录项中获取文件簇号
    fn cluster(value: &[u8]) -> u32;

    // 从目录项中获取文件大小
    fn size(value: &[u8]) -> u32;

    // 从目录项中提取指定区间的数据
    fn split(value: &[u8], start: Self, end: Self) -> &[u8];
}
enum LongDirOffset {
    // ...
}

impl LongDirOffset {
    // 从长目录项中获取文件名
    fn name(value: &[u8]) -> String;

    // 从长目录项中提取指定区间的数据
    fn split(value: &[u8], start: Self, end: Self) -> &[u8];
}
struct FAT32Dirent {
    // ...
}

impl FAT32Dirent {
    // 检查目录项是否是结束项
    pub fn is_end(value: &[u8]) -> bool;

    // 检查目录项是否是空白项
    pub fn is_empty(value: &[u8]) -> bool;

    // 检查目录项是否是长目录项
    pub fn is_long_dirent(value: &[u8]) -> bool;

    // 获取结束项的默认字节序列
    pub fn end() -> [u8; 32];

    // 获取空白项的默认字节序列
    pub fn empty() -> [u8; 32];

    // 创建一个新的FAT32Dirent实例
    pub fn new(name: String, attr: FileAttr, cluster: u32, size: u32) -> Self;

    // 添加长目录项到目录项
    pub fn append_long(&mut self, long_dir: &[u8]);

    // 添加短目录项到目录项
    pub fn append_short(&mut self, short_dir: &[u8]);

    // 将目录项转换为目录项字节序列
    pub fn to_dirs(&self) -> VecDeque<[u8; 32]>;

    // 将长文件名转换为短文件名
    fn short_name(&self) -> [u8; 11];

    // 将FAT32时间戳标准化为TimeSpec
    fn time_normalize(date: u16, hms: u16) -> Option<TimeSpec>;
}
```

这些函数提供了从FAT32目录项字节序列中解析文件属性、时间戳、文件名、簇号和文件大小的功能。`FAT32Dirent` 结构体还提供了方法来创建目录项、添加长目录项和短目录项，以及将目录项转换回字节序列，用于存储回文件系统。

``` rust
//fat.rs定义了与FAT32文件系统元数据和FAT（文件分配表）项相关的数据结构和实现。
enum FATEnt {
    // ...
}

impl From<u32> for FATEnt {
    // 从u32类型转换为FATEnt枚举
    fn from(value: u32) -> Self;
}

impl From<FATEnt> for u32 {
    // 从FATEnt枚举转换为u32类型
    fn from(value: FATEnt) -> Self;
}

struct FAT32Meta {
    // ...
}

impl FAT32Meta {
    // 根据启动扇区创建FAT32Meta实例
    pub fn new(boot_sector: &[u8]) -> SyscallResult<Self>;

    // 根据簇号获取FAT表项的扇区号
    pub fn ent_sector_for_cluster(&self, cluster: usize) -> usize;

    // 根据簇号获取FAT表项的扇区偏移
    pub fn ent_offset_for_cluster(&self, cluster: usize) -> usize;

    // 根据簇号获取数据的起始扇区号
    pub fn data_sector_for_cluster(&self, cluster: usize) -> usize;
}
```

`ATEnt` 枚举表示FAT表项的不同可能值，包括空簇、坏簇、簇链结束和下一个簇号。实现了 `From<u32>` 转换为 `FATEnt` 和 `From<FATEnt>` 转换为 `u32` 的trait，以便于在FAT表项的数值和枚举表示之间转换。

`FAT32Meta` 结构体包含FAT32文件系统的关键参数，如FAT表和数据区的扇区偏移、每扇区字节数、每簇扇区数等。`new` 函数根据启动扇区（boot sector）的内容创建 `FAT32Meta` 实例，并进行一些基本的文件系统有效性检查。`ent_sector_for_cluster` 和 `ent_offset_for_cluster` 函数用于计算给定簇号的FAT表项在扇区中的位置。`data_sector_for_cluster` 函数用于计算给定簇号的数据在数据区中的起始扇区号。

``` rust
//fsinfo.rs定义了FAT32文件系统的FSInfo扇区的结构和相关操作
pub const SIG_START: u32 = 0x41615252; // FSInfo扇区头部标志的值
pub const SIG_END: u32 = 0xAA550000;   // FSInfo扇区尾部标志的值

enum FSInfoOffset {
    // ...
}

impl FSInfoOffset {
    // 从FSInfo扇区中获取头部标志
    pub fn lead_sig(sector: &[u8]) -> u32;

    // 从FSInfo扇区中获取最新的剩余簇数量
    pub fn free_count(sector: &[u8]) -> u32;

    // 从FSInfo扇区中获取驱动程序最后分配出去的簇号
    pub fn next_free(sector: &[u8]) -> u32;

    // 从FSInfo扇区中获取尾部标志
    pub fn trail_sig(sector: &[u8]) -> u32;

    // 从FSInfo扇区中提取指定区间的数据
    fn split(sector: &[u8], start: Self, end: Self) -> &[u8];
}
```

``` rust
//inode.rs实现了FAT32文件系统中的inode结构和相关操作
struct FAT32Inode {
    // ...
}

impl FAT32Inode {
    // 创建根目录的FAT32Inode
    pub async fn root(
        fs: &Arc<FAT32FileSystem>,
        parent: Option<Arc<dyn Inode>>,
        root_cluster: u32,
    ) -> SyscallResult<Arc<Self>>;

    // 创建新的FAT32Inode
    pub async fn new(
        fs: &Arc<FAT32FileSystem>,
        parent: Arc<dyn Inode>,
        dir: FAT32Dirent,
    ) -> SyscallResult<Arc<Self>>;
}

#[async_trait]
impl Inode for FAT32Inode {
    // 获取Inode的元数据
    fn metadata(&self) -> &InodeMeta;

    // 获取Inode所关联的文件系统的弱引用
    fn file_system(&self) -> Weak<dyn FileSystem>;
}

#[async_trait]
impl InodeInternal for FAT32Inode {
    // 异步直接读取文件数据
    async fn read_direct(&self, buf: &mut [u8], offset: isize) -> SyscallResult<isize>;

    // 异步直接写入文件数据
    async fn write_direct(&self, buf: &[u8], offset: isize) -> SyscallResult<isize>;

    // 异步直接截断文件
    async fn truncate_direct(&self, new_size: isize) -> SyscallResult;

    // 异步加载目录项下的子项
    async fn load_children(self: Arc<Self>, inner: &mut InodeMetaInner) -> SyscallResult;

    // 异步创建文件或目录
    async fn do_create(self: Arc<Self>, mode: InodeMode, name: &str) -> SyscallResult<InodeChild>;

    // 异步将文件或目录移动到当前目录
    async fn do_movein(self: Arc<Self>, name: &str, inode: Arc<dyn Inode>) -> SyscallResult<InodeChild>;

    // 异步删除目录项下的文件或目录
    async fn do_unlink(self: Arc<Self>, target: &InodeChild) -> SyscallResult;
}

impl Drop for FAT32Inode {
    // 当FAT32Inode被销毁时，同步page cache中的数据
    fn drop(&mut self);
}

struct FAT32InodeExt {
    dir_occupy: BitVec, // 位图，标记目录项占用情况
    clusters: Vec<usize>, // 文件或目录占用的簇列表
}

struct FAT32ChildExt {
    dir_pos: usize, // 目录项在目录中的起始位置
    dir_len: usize, // 目录项的长度
}

impl FAT32ChildExt {
    // 创建FAT32ChildExt的新实例
    pub fn new(dir_pos: usize, dir_len: usize) -> Box<Self>;
}
```

这些函数和结构体实现了FAT32Inode的初始化、数据读写、大小截断、目录项的加载、文件和目录的创建、移动以及删除，以及在退出时同步page cache中的数据。`read_direct` 和 `write_direct` 方法用于直接对文件数据进行读取和写入操作，而 `truncate_direct` 方法用于改变文件的大小。`load_children` 方法用于加载目录中的子项，`do_create`、`do_movein` 和 `do_unlink` 方法用于操作目录中的文件和目录项。

`FAT32InodeExt` 结构体包含实际的文件数据所在的簇列表和目录项占用的位图，而 `FAT32ChildExt` 结构体则包含了目录项的位置信息。`INO_POOL` 是一个静态的原子变量，用于生成唯一的inode号。`AsyncMutex` 被用来保护数据结构的并发访问。

``` rust
//mod.rs FAT32文件系统的接口，提供了文件系统的创建、数据读写、目录操作等功能
pub struct FAT32FileSystem {
    // 块设备接口
    device: Arc<dyn BlockDevice>,
    // 文件系统元数据
    vfsmeta: FileSystemMeta,
    // FAT32文件系统元数据
    fat32meta: FAT32Meta,
    // 块缓存
    cache: BlockCache<BLOCK_SIZE>,
    // 根目录Inode
    root: ManuallyDrop<LateInit<Arc<FAT32Inode>>>,
}

impl FAT32FileSystem {
    // 创建并初始化新的FAT32文件系统实例
    pub async fn new(device: Arc<dyn BlockDevice>, flags: VfsFlags) -> SyscallResult<Arc<Self>>;

    // 从指定簇读取数据到缓冲区
    pub async fn read_data(&self, cluster: usize, buf: &mut [u8], offset: usize) -> SyscallResult<()>;

    // 将数据从缓冲区写入到指定簇
    pub async fn write_data(&self, cluster: usize, buf: &[u8], offset: usize) -> SyscallResult<()>;

    // 读取目录项，返回子Inode列表
    pub async fn read_dir(
        &self,
        parent: Arc<dyn Inode>,
        clusters: &[usize],
        occupy: &mut BitVec,
    ) -> SyscallResult<Vec<InodeChild>>;

    // 写入单个目录项
    pub async fn write_dir(&self, clusters: &[usize], pos: usize, dirent: &[u8; 32]) -> SyscallResult<()>;

    // 追加目录项到目录中
    pub async fn append_dir(
        &self,
        clusters: &mut Vec<usize>,
        occupy: &mut BitVec,
        dirent: &FAT32Dirent,
    ) -> SyscallResult<(usize, usize)>;

    // 从目录中删除目录项
    pub async fn remove_dir(
        &self,
        clusters: &mut Vec<usize>,
        occupy: &mut BitVec,
        pos: usize,
        len: usize,
    ) -> SyscallResult<()>;

    // 计算FAT表项所在的块号和偏移
    fn ent_block_for_cluster(&self, cluster: usize) -> (usize, usize);

    // 读取FAT表项
    async fn read_fat_ent(&self, cluster: usize) -> SyscallResult<FATEnt>;

    // 写入FAT表项
    async fn write_fat_ent(&self, cluster: usize, ent: FATEnt) -> SyscallResult<()>;

    // 遍历簇链
    async fn walk_fat_ent(&self, ent: FATEnt) -> SyscallResult<Vec<usize>>;

    // 分配一个新的簇
    async fn alloc_cluster(&self) -> SyscallResult<usize>;
}

// 实现FileSystem trait，提供文件系统的标准接口
impl FileSystem for FAT32FileSystem {
    fn metadata(&self) -> &FileSystemMeta;
    fn root(&self) -> Arc<dyn Inode>;
}

// 当FAT32FileSystem实例被销毁时，执行清理工作
impl Drop for FAT32FileSystem {
    fn drop(&mut self);
}
```

这些函数提供了FAT32文件系统的核心功能，包括：

- **文件系统初始化**：`new`函数用于创建文件系统的实例并读取启动扇区。
- **数据读写**：`read_data`和`write_data`函数用于从文件簇中读取或写入数据。
- **目录操作**：`read_dir`、`write_dir`、`append_dir`和`remove_dir`函数用于操作目录项。
- **FAT表操作**：`read_fat_ent`、`write_fat_ent`和`walk_fat_ent`函数用于读取、写入和遍历FAT表项。
- **簇管理**：`alloc_cluster`函数用于分配新的簇。
- **文件系统元数据**：`FileSystem` trait的实现提供了访问文件系统元数据的方法。

### 5.4 与我们内核实现的fat32文件系统进行对比

比较MinotaurOS和我们内核在FAT32实现上的异同点，找出我们内核可以改进的部分，总结参考内核在Ext4和FAT32实现上的经验，如设计思路、代码结构、优化策略等，并分析这些经验对我们内核的借鉴意义，便于后续实现ext4。

- 两者都实现了FAT32文件系统的核心功能，如文件读写、目录管理、文件属性管理等。
- 两者都采用了类似的方法来处理FAT表和目录项。
- MinotaurOS在文件系统实现中采用了异步I/O操作，而我们的文件系统实现是同步的。

``` rust
//bitmap.rs
pub struct Fat {
    /// Cache manager for fat
    fat_cache_mgr: Arc<Mutex<BlockCacheManager>>,
    /// The first block id of FAT.
    /// In FAT32, this is equal to bpb.rsvd_sec_cnt
    start_block_id: usize,
    /// size fo sector in bytes copied from BPB
    byts_per_sec: usize,
    /// The total number of FAT entries
    tot_ent: usize,
    /// The queue used to store known vacant clusters
    vacant_clus: Mutex<VecDeque<u32>>,
    /// The final unused cluster id we found
    hint: Mutex<usize>,
}

impl Fat {
    // 根据当前簇号获取指向的下一个簇号
    pub fn get_next_clus_num(&self, current_clus_num: u32, block_device: &Arc<dyn BlockDevice>) -> u32;

    // 获取当前簇号之后的所有簇号
    pub fn get_all_clus_num(&self, current_clus_num: u32, block_device: &Arc<dyn BlockDevice>) -> Vec<u32>;

    // 构造函数，初始化Fat结构体
    pub fn new(rsvd_sec_cnt: usize, byts_per_sec: usize, clus: usize, fat_cache_mgr: Arc<Mutex<BlockCacheManager>>) -> Self;

    // 计算给定簇号在FAT区域的扇区ID
    #[inline(always)]
    pub fn this_fat_sec_num(&self, clus_num: u32) -> usize;

    // 计算给定簇号在FAT区域扇区内的偏移
    #[inline(always)]
    pub fn this_fat_ent_offset(&self, clus_num: u32) -> usize;

    // 将`current`簇号分配到`next`簇号
    fn set_next_clus(&self, block_device: &Arc<dyn BlockDevice>, current: Option<u32>, next: u32);

    // 分配尽可能多的簇（不超过alloc_num）
    pub fn alloc(&self, block_device: &Arc<dyn BlockDevice>, alloc_num: usize, last: Option<u32>) -> Vec<u32>;

    // 从数据区域找到并分配一个簇
    fn alloc_one(&self, block_device: &Arc<dyn BlockDevice>, last: Option<u32>, hlock: &mut MutexGuard<usize>) -> Option<u32>;

    // 从数据区域找到下一个空闲簇
    fn get_next_free_clus(&self, start: u32, block_device: &Arc<dyn BlockDevice>) -> Option<u32>;

    // 从数据区域释放多个簇
    pub fn free(&self, block_device: &Arc<dyn BlockDevice>, cluster_list: Vec<u32>, last: Option<u32>);
}
```

这些函数提供了管理FAT32文件系统中簇分配的核心功能，包括获取簇链、分配新簇、释放簇等。`get_next_clus_num` 和 `get_all_clus_num` 函数用于获取簇链信息；`new` 函数用于创建 `Fat` 实例；`this_fat_sec_num` 和 `this_fat_ent_offset` 函数用于计算FAT项在存储介质上的位置；`set_next_clus` 函数用于更新FAT项；`alloc` 和 `alloc_one` 函数用于分配簇；`get_next_free_clus` 函数用于查找空闲簇；`free` 函数用于释放簇。

~~~ rust
//dir_iter.rs实现了FAT32文件系统中目录遍历的迭代器
pub enum DirIterMode {
    // ... 迭代模式定义 ...
}

pub struct DirIter<'a, 'b> {
    // ... 迭代器字段定义 ...
}

impl<'a, 'b> DirIter<'a, 'b> {
    /// 创建一个新的目录迭代器实例
    pub fn new(
        inode_lock: &'a RwLockWriteGuard<'b, InodeLock>,
        offset: Option<u32>,
        mode: DirIterMode,
        direction: bool,
        inode: &'a Inode,
    ) -> Self {
        // ... 实例化目录迭代器 ...
    }

    /// 获取迭代器当前的偏移量
    #[inline(always)]
    pub fn get_offset(&self) -> Option<u32> {
        // ... 返回偏移量 ...
    }

    /// 设置迭代器的偏移量，以便首次迭代时直接定位到目标偏移量
    pub fn set_iter_offset(&mut self, offset: u32) {
        // ... 更新偏移量 ...
    }

    /// 获取当前偏移量对应的`FATDirEnt`内容
    pub fn current_clone(&mut self) -> Option<FATDirEnt> {
        // ... 克隆当前目录项 ...
    }

    /// 将`ent`写入到迭代器当前指向的目录项
    pub fn write_to_current_ent(&mut self, ent: &FATDirEnt) {
        // ... 写入目录项 ...
    }

    /// 内部实现的迭代逻辑
    fn step(&mut self) -> Option<FATDirEnt> {
        // ... 移动到下一个目录项 ...
    }

    /// 将`DirIter`转换为`DirWalker`
    pub fn walk(self) -> DirWalker<'a, 'b> {
        // ... 转换为目录遍历器 ...
    }
}

/// 实现`Iterator` trait为`DirIter`
impl Iterator for DirIter<'_, '_> {
    /// 迭代到下一个有效的目录项
    fn next(&mut self) -> Option<Self::Item> {
        // ... 根据迭代模式和方向移动到下一个目录项 ...
    }
}

/// `DirWalker`结构体，用于迭代目录项（长文件名和短文件名的组合）
pub struct DirWalker<'a, 'b> {
    pub iter: DirIter<'a, 'b>,
}

/// 实现`Iterator` trait为`DirWalker`
impl Iterator for DirWalker<'_, '_> {
    /// 迭代到下一个组合的长文件名和短文件名
    fn next(&mut self) -> Option<Self::Item> {
        // ... 处理长文件名和短文件名的组合 ...
    }
}
~~~

``` rust
//efs.rs 定义了一个名为 EasyFileSystem 的结构体，代表一个简化的FAT32文件系统实现
pub struct EasyFileSystem {
    // ... EasyFileSystem字段定义 ...
}

impl EasyFileSystem {
    /// 计算给定簇号在FAT表中的项偏移量
    #[inline(always)]
    pub fn this_fat_ent_offset(&self, n: u32) -> u32 {
        // ... 委托给Fat结构体的同名方法 ...
    }

    /// 计算给定簇号在FAT表中的扇区编号
    #[inline(always)]
    pub fn this_fat_sec_num(&self, n: u32) -> u32 {
        // ... 委托给Fat结构体的同名方法 ...
    }

    /// 获取指定簇号的下一个簇号
    #[inline(always)]
    pub fn get_next_clus_num(&self, result: u32) -> u32 {
        // ... 委托给Fat结构体的同名方法 ...
    }

    /// 获取数据区域的起始扇区号
    pub fn first_data_sector(&self) -> u32 {
        // ... 返回数据区域的起始扇区号 ...
    }

    /// 获取每个簇的大小（字节）
    #[inline(always)]
    pub fn clus_size(&self) -> u32 {
        // ... 计算并返回簇大小 ...
    }

    /// 计算给定簇号的第一个扇区号
    #[inline(always)]
    pub fn first_sector_of_cluster(&self, clus_num: u32) -> u32 {
        // ... 计算并返回簇的第一个扇区号 ...
    }

    /// 打开文件系统对象
    pub fn open(
        block_device: Arc<dyn BlockDevice>,
        index_cache_mgr: Arc<spin::Mutex<BlockCacheManager>>,
    ) -> Arc<Self> {
        // ... 从硬件设备读取超级块，并初始化EasyFileSystem实例 ...
    }

    /// 分配指定数量的块
    pub fn alloc_blocks(&self, blocks: usize) -> Vec<usize> {
        // ... 从文件系统中分配块，并返回分配的块ID列表 ...
    }
}
```

``` rust
//inode.rs  OSInode结构体封装了对文件的各种操作，包括读取、写入、获取文件状态等。
pub struct OSInode {
    // ... OSInode字段定义 ...
}

impl OSInode {
    /// 创建一个新的OSInode实例，通常用于根目录
    pub fn new(root_inode: Arc<InodeImpl>) -> Arc<dyn File> {
        // ... 实例化OSInode，通常用于根目录 ...
    }

    /// 当OSInode是特殊用途时，在析构时减少特殊用途的计数
    fn drop(&mut self) {
        // ... 析构函数，处理特殊用途文件的资源释放 ...
    }
}

impl File for OSInode {
    /// 深拷贝OSInode，创建一个新的文件句柄
    fn deep_clone(&self) -> Arc<dyn File> {
        // ... 实现文件句柄的深拷贝 ...
    }

    /// 检查文件是否可读
    fn readable(&self) -> bool {
        // ... 返回文件是否可读 ...
    }

    /// 检查文件是否可写
    fn writable(&self) -> bool {
        // ... 返回文件是否可写 ...
    }

    /// 从文件中读取数据
    fn read(&self, offset: Option<&mut usize>, buffer: &mut [u8]) -> usize {
        // ... 从文件读取数据到缓冲区 ...
    }

    /// 向文件写入数据
    fn write(&self, offset: Option<&mut usize>, buffer: &[u8]) -> usize {
        // ... 从缓冲区写入数据到文件 ...
    }

    /// 检查文件是否就绪读取
    fn r_ready(&self) -> bool {
        // ... 文件是否就绪读取 ...
    }

    /// 检查文件是否就绪写入
    fn w_ready(&self) -> bool {
        // ... 文件是否就绪写入 ...
    }

    /// 从用户空间读取数据到内核空间
    fn read_user(&self, offset: Option<usize>, buf: UserBuffer) -> usize {
        // ... 从用户空间读取数据 ...
    }

    /// 从内核空间写入数据到用户空间
    fn write_user(&self, offset: Option<usize>, buf: UserBuffer) -> usize {
        // ... 从内核空间写入数据 ...
    }

    /// 获取文件大小
    fn get_size(&self) -> usize {
        // ... 返回文件大小 ...
    }

    /// 获取文件状态
    fn get_stat(&self) -> Stat {
        // ... 返回文件状态信息 ...
    }

    /// 获取文件类型
    fn get_file_type(&self) -> DiskInodeType {
        // ... 返回文件类型 ...
    }

    /// 设置目录树节点的引用
    fn info_dirtree_node(&self, dirnode_ptr: Weak<DirectoryTreeNode>) {
        // ... 设置目录树节点的弱引用 ...
    }

    /// 获取目录树节点
    fn get_dirtree_node(&self) -> Option<Arc<DirectoryTreeNode>> {
        // ... 获取目录树节点 ...
    }

    /// 打开文件，根据标志确定文件的访问方式
    fn open(&self, flags: OpenFlags, special_use: bool) -> Arc<dyn File> {
        // ... 打开文件，返回新的文件句柄 ...
    }

    /// 打开子文件
    fn open_subfile(&self) -> Result<Vec<(String, Arc<dyn File>)>, isize> {
        // ... 返回子文件列表 ...
    }

    /// 创建新文件或目录
    fn create(&self, name: &str, file_type: DiskInodeType) -> Result<Arc<dyn File>, isize> {
        // ... 在目录中创建新文件或目录 ...
    }

    /// 为子文件创建链接
    fn link_child(&self, name: &str, child: &Self) -> Result<(), isize> {
        // ... 在目录中为子文件创建链接 ...
    }

    /// 删除文件或目录
    fn unlink(&self, delete: bool) -> Result<(), isize> {
        // ... 删除文件或目录 ...
    }

    /// 获取目录项
    fn get_dirent(&self, count: usize) -> Vec<Dirent> {
        // ... 获取目录中的目录项 ...
    }

    /// 移动文件读写位置
    fn lseek(&self, offset: isize, whence: SeekWhence) -> Result<usize, isize> {
        // ... 移动文件读写位置 ...
    }

    /// 修改文件大小
    fn modify_size(&self, diff: isize) -> Result<(), isize> {
        // ... 修改文件大小 ...
    }

    /// 截断文件到指定大小
    fn truncate_size(&self, new_size: usize) -> Result<(), isize> {
        // ... 截断文件到指定大小 ...
    }

    /// 设置文件的时间戳
    fn set_timestamp(&self, ctime: Option<usize>, atime: Option<usize>, mtime: Option<usize>) {
        // ... 设置文件的创建、访问和修改时间戳 ...
    }

    /// 获取单个缓存页
    fn get_single_cache(&self, offset: usize) -> Result<Arc<Mutex<PageCache>>, ()> {
        // ... 获取单个缓存页 ...
    }

    /// 获取所有缓存页
    fn get_all_caches(&self) -> Result<Vec<Arc<Mutex<PageCache>>>, ()> {
        // ... 获取所有缓存页 ...
    }

    /// 处理内存不足的情况
    fn oom(&self) -> usize {
        // ... 返回内存不足时的处理 ...
    }

    /// 挂起文件操作
    fn hang_up(&self) -> bool {
        // ... 文件操作是否可以挂起 ...
    }

    /// 文件控制操作
    fn fcntl(&self, cmd: u32, arg: u32) -> isize {
        // ... 执行文件控制命令 ...
    }
}
```

`OSInode` 结构体封装了对文件的各种操作，包括读取、写入、获取文件状态等。它使用 `InodeImpl` 作为内部表示，并通过 `Mutex` 保护对文件偏移量的访问。`OSInode` 还支持特殊用途标志，用于标识如设备文件等特殊类型的文件。

``` rust
//layout.rs用于操作FAT32文件系统的磁盘结构和目录项
/// 磁盘上的分区信息数据结构
pub struct BPB {
    // ... BPB字段定义 ...
}

impl BPB {
    /// 检查BPB是否有效
    pub fn is_valid(&self) -> bool {
        // ... 检查逻辑 ...
    }

    /// 获取数据区域的扇区数
    pub fn data_sector_count(&self) -> u32 {
        // ... 计算数据区域扇区数 ...
    }

    /// 获取簇的数量
    pub fn count_of_cluster(&self) -> u32 {
        // ... 计算簇数量 ...
    }

    /// 获取每个簇的大小（字节）
    pub fn clus_size(&self) -> u32 {
        // ... 计算簇大小 ...
    }

    /// 获取根目录占用的扇区数
    pub fn root_dir_sec(&self) -> u32 {
        // ... 计算根目录扇区数 ...
    }

    /// 获取数据区域开始的第一个扇区号
    pub fn first_data_sector(&self) -> u32 {
        // ... 返回数据区域起始扇区号 ...
    }

    /// 判断FAT类型（FAT12、FAT16或FAT32）
    pub fn fat_type(&self) -> FatType {
        // ... 判断FAT类型 ...
    }
}

/// 文件系统信息结构
pub struct FSInfo {
    // ... FSInfo字段定义 ...
}

/// 目录项类型
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum DiskInodeType {
    File,
    Directory,
}

/// FAT目录项属性
#[derive(PartialEq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum FATDiskInodeType {
    // ... 属性定义 ...
}

/// FAT目录项（可以是长文件名或短文件名）
pub union FATDirEnt {
    pub short_entry: FATShortDirEnt,
    pub long_entry: FATLongDirEnt,
    pub empty: [u8; 32],
}

impl FATDirEnt {
    /// 生成短文件名
    pub fn gen_short_name_prefix(s: String) -> String {
        // ... 生成短文件名 ...
    }

    /// 检查是否是最后一个长文件名目录项
    pub fn is_last_long_dir_ent(&self) -> bool {
        // ... 检查逻辑 ...
    }

    /// 获取目录项序号
    pub fn ord(&self) -> usize {
        // ... 获取序号 ...
    }

    /// 设置文件大小
    pub fn set_size(&mut self, size: u32) {
        // ... 设置文件大小 ...
    }

    /// 获取第一个簇号
    pub fn get_fst_clus(&self) -> u32 {
        // ... 获取第一个簇号 ...
    }

    /// 设置第一个簇号
    pub fn set_fst_clus(&mut self, fst_clus: u32) {
        // ... 设置第一个簇号 ...
    }

    /// 检查是否是长文件名目录项
    pub fn is_long(&self) -> bool {
        // ... 检查逻辑 ...
    }

    /// 检查是否是短文件名目录项
    pub fn is_short(&self) -> bool {
        // ... 检查逻辑 ...
    }

    /// 获取短文件名目录项
    pub fn get_short_ent(&self) -> Option<&FATShortDirEnt> {
        // ... 获取短文件名目录项 ...
    }

    /// 获取长文件名目录项
    pub fn get_long_ent(&self) -> Option<&FATLongDirEnt> {
        // ... 获取长文件名目录项 ...
    }

    /// 获取目录项名称
    pub fn get_name(&self) -> String {
        // ... 获取名称 ...
    }

    /// 设置目录项名称
    pub fn set_name(&mut self, name: [u8; 11]) {
        // ... 设置名称 ...
    }

    /// 检查目录项是否未使用
    pub fn unused(&self) -> bool {
        // ... 检查逻辑 ...
    }

    /// 检查目录项是否是未使用且不是最后的目录项
    pub fn unused_not_last(&self) -> bool {
        // ... 检查逻辑 ...
    }

    /// 检查目录项是否是未使用且是最后的目录项
    pub fn last_and_unused(&self) -> bool {
        // ... 检查逻辑 ...
    }
}

/// FAT32短文件名目录项
#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct FATShortDirEnt {
    // ... 短文件名目录项字段定义 ...
}

impl FATShortDirEnt {
    /// 从名称和簇号创建短文件名目录项
    pub fn from_name(name: [u8; 11], fst_clus: u32, file_type: DiskInodeType) -> Self {
        // ... 创建逻辑 ...
    }

    /// 设置第一个簇号
    pub fn set_fst_clus(&mut self, fst_clus: u32) {
        // ... 设置簇号 ...
    }

    /// 获取第一个簇号
    pub fn get_first_clus(&self) -> u32 {
        // ... 获取簇号 ...
    }

    /// 检查是否是目录
    pub fn is_dir(&self) -> bool {
        // ... 检查逻辑 ...
    }

    /// 检查是否是文件
    pub fn is_file(&self) -> bool {
        // ... 检查逻辑 ...
    }

    /// 获取短文件名
    pub fn name(&self) -> String {
        // ... 获取名称 ...
    }
}

/// FAT32长文件名目录项
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(packed)]
pub struct FATLongDirEnt {
    // ... 长文件名目录项字段定义 ...
}

impl FATLongDirEnt {
    /// 从名称片段创建长文件名目录项
    pub fn from_name_slice(is_last_ent: bool, order: usize, partial_name: [u16; 13]) -> Self {
        // ... 创建逻辑 ...
    }

    /// 获取长文件名
    pub fn name(&self) -> String {
        // ... 获取名称 ...
    }
}
```

定义了FAT32文件系统的启动参数块（BPB）、文件分配表（FAT）信息、FSInfo结构、目录项（包括长文件名和短文件名）等数据结构。这些结构体用于表示磁盘上的文件系统信息和文件/目录的元数据。


``` rust
//vfs.rs
pub struct Inode {
    // ... Inode字段定义 ...
}

impl Inode {
    /// 创建一个新的Inode实例
    pub fn new(
        fst_clus: u32,
        file_type: DiskInodeType,
        size: Option<u32>,
        parent_dir: Option<(Arc<Self>, u32)>,
        fs: Arc<EasyFileSystem>,
    ) -> Arc<Self> {
        // ... 实例化Inode...
    }

    /// 打开根目录
    pub fn root_inode(efs: &Arc<EasyFileSystem>) -> Arc<Self> {
        // ... 打开文件系统的根目录 ...
    }

    /// 读取文件内容到缓冲区
    pub fn read_at_block_cache(&self, offset: usize, buf: &mut [u8]) -> usize {
        // ... 从文件读取数据到缓冲区 ...
    }

    /// 将缓冲区数据写入文件
    pub fn write_at_block_cache(&self, offset: usize, buf: &[u8]) -> usize {
        // ... 从缓冲区写入数据到文件 ...
    }

    /// 获取文件大小
    pub fn get_file_size(&self) -> u32 {
        // ... 返回文件大小 ...
    }

    /// 检查是否是目录
    pub fn is_dir(&self) -> bool {
        // ... 返回是否是目录 ...
    }

    /// 检查是否是文件
    pub fn is_file(&self) -> bool {
        // ... 返回是否是文件 ...
    }

    /// 分配所需的簇
    fn alloc_clus(&self, lock: &mut RwLockWriteGuard<FileContent>, alloc_num: usize) {
        // ... 在文件中分配簇 ...
    }

    /// 释放簇
    fn dealloc_clus(&self, lock: &mut RwLockWriteGuard<FileContent>, dealloc_num: usize) {
        // ... 在文件中释放簇 ...
    }

    /// 修改文件大小
    pub fn modify_size_lock(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        diff: isize,
        clear: bool,
    ) {
        // ... 修改文件大小 ...
    }

    /// 删除Inode
    pub fn unlink_lock(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        delete: bool,
    ) -> Result<(), isize> {
        // ... 从磁盘删除文件 ...
    }

    /// 创建新文件或目录
    pub fn create_lock(
        parent_dir: &Arc<Self>,
        parent_inode_lock: &RwLockWriteGuard<InodeLock>,
        name: String,
        file_type: DiskInodeType,
    ) -> Result<Arc<Self>, ()> {
        // ... 在目录中创建新文件或目录 ...
    }

    /// 链接到父目录
    pub fn link_par_lock(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        parent_dir: &Arc<Self>,
        parent_inode_lock: &RwLockWriteGuard<InodeLock>,
        name: String,
    ) -> Result<(), ()> {
        // ... 将文件或目录链接到父目录 ...
    }

    /// 获取目录项
    pub fn get_dir_ent(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        offset: u32,
    ) -> Result<FATDirEnt, ()> {
        // ... 获取目录项 ...
    }

    /// 设置目录项
    pub fn set_dir_ent(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        offset: u32,
        dir_ent: FATDirEnt,
    ) -> Result<(), ()> {
        // ... 设置目录项 ...
    }

    /// 获取所有文件和目录项
    pub fn get_all_files_lock(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
    ) -> Vec<(String, FATShortDirEnt, u32)> {
        // ... 获取所有文件和目录项 ...
    }

    /// 获取文件状态信息
    pub fn stat_lock(&self, _inode_lock: &RwLockReadGuard<InodeLock>) -> (i64, i64, i64, i64, u64) {
        // ... 获取文件状态信息 ...
    }

    /// 列出目录内容
    pub fn ls_lock(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
    ) -> Result<Vec<(String, FATShortDirEnt)>, ()> {
        // ... 列出目录内容 ...
    }

    /// 在目录中查找文件
    pub fn find_local_lock(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        req_name: String,
    ) -> Result<Option<(String, FATShortDirEnt, u32)>, ()> {
        // ... 在目录中查找文件 ...
    }

    /// 获取目录项信息
    pub fn dirent_info_lock(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        offset: u32,
        length: usize,
    ) -> Result<Vec<(String, usize, u64, FATDiskInodeType)>, ()> {
        // ... 获取目录项信息 ...
    }
}
```

`Inode` 结构体实现了文件和目录的基本操作，包括创建、读写、删除、遍历等。它使用`RwLock`来保护对文件内容和元数据的并发访问。`FileContent`结构体用于存储文件的簇列表和大小信息。`InodeTime`结构体用于存储文件的时间戳信息。

## 6. 挑战和收获

### 6.1 实现挑战

在实现Ext4的过程中，我们面临了多方面的挑战：

- **时间限制**：在规定的时间内完成复杂的文件系统实现需要高效的计划和执行。
- **复杂性**：Ext4文件系统的复杂性远超FAT32，包括块组管理、日志机制、支持大文件等多个方面。
- **知识不足**：团队成员对文件系统的理解和实现经验不足，需要大量的学习和研究。

### 6.2 关键收获

通过探索Ext4的实现，我们收获了很多宝贵的经验和知识：

- **深入理解文件系统架构和设计原理**：从基础概念到具体实现，我们对文件系统有了全面的理解。
- **提升解决复杂问题的能力**：面对复杂的系统设计和实现，我们学会了如何分解问题、制定计划并逐步实现。
- **团队协作与知识分享**：通过团队合作，我们学会了如何高效沟通、共享知识和共同解决问题。

