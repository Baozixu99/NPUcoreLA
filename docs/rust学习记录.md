selfptr: **Mutex**<**Weak**<**Self**>>

- `Mutex<T>` 是 Rust 中的互斥锁，用于在多线程环境中保护数据的并发访问。它可以确保同一时间只有一个线程能够访问被 Mutex 保护的数据。

- `Weak<T>` 是 Rust 中的弱引用类型，用于表示一个弱引用。与强引用不同，弱引用不会增加被引用对象的引用计数，也不会阻止对象被释放。

1. **强引用（Strong Reference）**Rc<T>：
   - 强引用是指对对象的一种正常引用，它会增加对象的引用计数（Reference Counting）。
   - 只要存在至少一个强引用指向对象，对象就会保持在内存中，不会被释放。
   - 当所有强引用都不再指向对象时，对象的引用计数会减少，当引用计数为零时，对象会被系统自动释放并回收内存。
2. **弱引用（Weak Reference）**Weak<T>：
   - 弱引用不会增加对象的引用计数，因此不会阻止对象被释放。
   - 弱引用通常用于解决循环引用（Cyclic References）问题，当两个或多个对象相互引用，并且存在强引用循环时，会导致对象无法被正确释放。
   - 弱引用允许获取对象的引用但不增加引用计数，这样即使存在循环引用，对象也可以在合适的时机被释放，避免内存泄漏。

**Rc<T>只适用单线程，Arc<T>是多线程版本**

**智能指针的使用场景:**

- 当需要在堆上分配内存时，使用 `Box<T>`。
- 当需要多处共享所有权时，使用 `Rc<T>` 或 `Arc<T>`。
- 当需要内部可变性时，使用 `RefCell<T>`。
- 当需要线程安全的共享所有权时，使用 `Arc<T>`。
- 当需要互斥访问数据时，使用 `Mutex<T>`。
- 当需要读取-写入访问数据时，使用 `RwLock<T>`。
- 当需要解决循环引用问题时，使用 `Weak<T>`。

____

children: **RwLock**<**Option**<**BTreeMap**<**String,Arc<Self>>>>**

- `RwLock<T>` 是 Rust 中的读写锁（Read-Write Lock），用于在多线程环境中保护数据的读写操作。`RwLock` 允许多个线程同时获取数据的只读引用，但只允许一个线程获取数据的可写引用，并且在写入时会阻塞其他线程的读取操作，以确保数据的一致性和安全性。

- `Option<T>` 是 Rust 中的可选类型，表示一个值可能存在，也可能不存在。当需要表示一个可能为空的值时，可以使用 `Option<T>` 类型。

- `BTreeMap<String, Arc<Self>>` 是一个键值对容器类型，使用字符串作为键（key），并且值（value）是 `Arc<Self>` 类型的强引用。`BTreeMap` 是 Rust 中的有序映射，可以按键进行排序并快速查找键值对。

综合起来，这段代码定义了一个数据结构 `children`，它使用读写锁保护一个可选的 BTreeMap，其中键为字符串，值为 `Arc<Self>` 类型的强引用。这种设计通常用于多线程环境中共享和管理具有层级关系的数据结构，比如树形结构中的子节点列表。通过读写锁和强引用，可以在多线程环境中安全地访问和操作这些数据结构。

**static ref** DIRECTORY_VEC: **Mutex**<(**Vec**<**Weak**<DirectoryTreeNode >>, **usize**)> = **Mutex**::new((**Vec**::new(), 0));

- `Vec<Weak<DirectoryTreeNode>>` 是一个元素类型为 `Weak<DirectoryTreeNode>` 的向量，表示一组弱引用的节点列表。

Mutex::new()  创建一个新的 Mutex 对象

Vec::new()    这是一个空的 `Vec`，表示一个空的向量（vector）

____

**type** ChildLockType <’a> =**RwLockWriteGuard**<’a, **Option**<**BTreeMap**<**String**, **Arc**<DirectoryTreeNode >>>>;

- `type ChildLockType<'a>`：这里使用 `type` 关键字定义了一个名为 `ChildLockType` 的类型别名，它有一个泛型参数 `'a`，表示生命周期参数。

- `RwLockWriteGuard<'a, T>`：这是 `RwLock` 的写入守卫类型，用于控制对 `RwLock` 所保护的数据的写访问权限。`'a` 是生命周期参数，表示守卫的引用的生命周期与其保护的数据的生命周期相关联。
- `Option<T>`：这是 Rust 中的枚举类型，它可以是 `Some` 包含具体值的情况，也可以是 `None` 表示空值。在这里，`T` 是 `BTreeMap<String, Arc<DirectoryTreeNode>>` 类型，表示一个键为字符串，值为 `Arc<DirectoryTreeNode>` 的有序映射。
- `BTreeMap<String, Arc<DirectoryTreeNode>>`：这是一个基于 B 树实现的有序映射，其中的键是字符串，值是 `Arc<DirectoryTreeNode>` 类型的智能指针，表示一个目录树节点。

____

###  `Arc<MutexGuard<FdTable>>` 和 `MutexGuard<FdTable>` 类型在使用时的不同

**生命周期和所有权**

- `Arc<MutexGuard<FdTable>>`：这表示一个引用计数智能指针，指向一个 `MutexGuard<FdTable>` 对象。`Arc` 表示其引用计数可以在多个地方共享，因此多个线程可以同时拥有对 `MutexGuard<FdTable>` 的共享访问权限。此外，`Arc` 类型确保 `MutexGuard<FdTable>` 在没有任何引用时被销毁，因此不需要手动管理其生命周期。

- `MutexGuard<FdTable>`：这是一个在 `Mutex` 上的智能指针，用于在获取锁后访问被锁定的数据。它的生命周期受到 `Mutex` 的作用域的限制，在超出作用域时会自动释放锁。

  **并发访问**

- `Arc<MutexGuard<FdTable>>` 允许多个线程同时共享对 `MutexGuard<FdTable>` 的访问权限。每个线程都持有 `Arc` 引用，并且可以通过调用 `clone()` 方法来增加引用计数，以防止提前销毁。

- `MutexGuard<FdTable>` 通过互斥锁保证了对 `FdTable` 的互斥访问，即同一时刻只有一个线程能够持有 `MutexGuard<FdTable>`，其他线程需要等待锁释放后才能获取到 `MutexGuard<FdTable>`。

因此，在实际使用中，你需要根据具体的需求来选择使用哪种类型。如果需要在多个线程之间共享对 `FdTable` 的访问权限，并且不希望手动管理生命周期，可以使用 `Arc<MutexGuard<FdTable>>`。如果在单线程中进行访问，并且可以确保锁的正确获取和释放，可以直接使用 `MutexGuard<FdTable>`。

------------------

```rust

pub enum Result<T, E> {
    Ok( /* … */ ),
    Err( /* … */ ),
}

****** if let *******
r = Ok(10000)
if let Ok(v) = r
//if let 表达式是一个特殊的匹配模式，用于从一个 Result 类型中提取 Ok / Err的值并进行处理。
//Ok(v) 是一个模式，表示匹配 Result 类型中的 Ok 分支，并将其中的值绑定到变量 v 上。
//所以，当 r 的值为 Ok(10000) 时，if let Ok(v) = r 会匹配成功，将 Ok 中的值 10000 绑定到变量 v 上

****** 在 Result 对象后添加 ? *******
    let t = Ok(10000)?; // 因为确定 t 不是 Err, t 在这里已经是 i32 类型
//? 符的实际作用是将 Result 类非异常的值直接取出，如果有异常就将异常 Result 返回出去。
//所以，? 符仅用于返回值类型为 Result<T, E> 的函数，其中 E 类型必须和 ? 所处理的 Result 的 E 类型一致。
//如果表达式是 Ok 分支，则返回 Ok 中的值。
//如果表达式是 Err 分支，则立即返回整个函数，并将 Err 中的值作为整个函数的返回值。



```

### where

在 Rust 中，`where` 关键字通常用于在泛型函数、泛型结构体或者 trait 实现中添加一些额外的约束条件。主要用途包括：

``` rust
fn example<T, U>(t: T, u: U) -> usize  //泛型函数或者结构体用 where 关键字来添加额外的约束条件，以确保泛型参数满足特定的条件。
where
    T: SomeTrait,
    U: AnotherTrait,
{
    // 函数体
}

impl<T> SomeTrait for T  //在为某个类型实现 trait 时，也可以使用 where 关键字来添加约束条件，以确保类型满足特定的 trait 实现条件。
where
    T: AnotherTrait,
{
    // 方法实现
}

```



## Option转换

我们知道，在`Rust`中，需要使用到`unwrap()`的方法的对象有`Result`,`Option`对象。我们看下`Option`的大致结构：

``` rust
Option`本身是一个`enum`对象，如果该函数（方法）调用结果值没有值，返回`None`,反之有值返回`Some(T).pub enum Option<T> {
    /// No value
    #[stable(feature = "rust1", since = "1.0.0")]
    None,
    /// Some value `T`
    #[stable(feature = "rust1", since = "1.0.0")]
    Some(#[stable(feature = "rust1", since = "1.0.0")] T),
}
//Option`本身是一个`enum`对象，如果该函数（方法）调用结果值没有值，返回`None`,反之有值返回`Some(T).
```

如果我们想获取`Some(T)`中的`T`,最直接的方式是：`unwrap()`。我们前面说过，使用`unwrap()`的方式太过于暴力，如果出错，程序直接`panic`，这是我们最不愿意看到的结果。

## 避免unwrap()

``` rust
fn main() {
    if let Some(v) = opt_val(60) {
        println!("{}", v);
    }
}

fn opt_val(num: i32) -> Option<String> {
    if num >= 60 {
        return Some("foo bar".to_string());
    }
    None
}
```

是的，我们使用`if let Some(v)`的方式取出值，当前`else`的逻辑就可能需要自己处理了。当然，`Option`可以这样做，`Result`也一定可以:

``` rust
fn main() {
    if let Ok(v) = read_file("./dat") {
        println!("{}", v);
    }
}

fn read_file(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}
```

只不过，在处理`Result`的判断时，使用的是`if let Ok(v)`，这个和`Option`的`if let Some(v)`有所不同。

到这里，`unwrap()`的代码片在项目中应该可以规避了。补充下，这里强调了几次规避，就如前所言：**团队风格统一，方便管理代码，消除潜在危机**。

____

``` rust
//内存
pub struct MemorySet {
    page_table: PageTable,  //页表，用于存储虚拟地址到物理地址的映射关系。
    areas: Vec<MapArea>,  //表示已映射的区域,每个元素都是一个 MapArea 结构体，用于描述不同段的内存映射情况,也可用于文件映射等其他目的。
}


pub struct PageTable {
    root_ppn: PhysPageNum,  //页表的根物理页号
    frames: Vec<Arc<FrameTracker>>,  //frames 字段则存储了页表所管理的物理内存页面的引用。
}

pub struct MapArea {
    inner: LinearMap,  //用于跟踪物理页帧到虚拟页的映射关系。
    map_type: MapType, //直接映射还是虚拟映射
    map_perm: MapPermission, //表示映射的权限，包括读、写和执行权限。
    pub map_file: Option<Arc<dyn File>>,  //如果为 Some 则表示是从文件映射而来，如果为 None 则表示是一块直接映射的内存块。
}

pub struct LinearMap {  //将虚拟地址映射到物理地址。
    vpn_range: VPNRange,  //虚拟地址的范围
    frames: Vec<Frame>,   //相应的物理页面
}


//进程
pub struct TaskControlBlock {
    // immutable 不可变部分
    pub pid: PidHandle, //进程号
    pub tid: usize,     //线程号
    pub tgid: usize,    //线程组号
    pub kstack: KernelStack, //内核栈
    pub ustack_base: usize,  //用户栈
    pub exit_signal: Signals,//退出信号
    // mutable  可变部分
    inner: Mutex<TaskControlBlockInner>,
    // shareable and mutable  共享可变部分
    pub exe: Arc<Mutex<FileDescriptor>>, // 可执行文件描述符
    pub tid_allocator: Arc<Mutex<RecycleAllocator>>,// 线程 ID 分配器
    pub files: Arc<Mutex<FdTable>>, // 文件描述符表
    pub fs: Arc<Mutex<FsStatus>>, // 文件系统状态   //暂时觉着这个FsStatus有点多余，觉着可以直接换成FileDescriptor
    pub vm: Arc<Mutex<MemorySet>>, // 虚拟内存映射
    pub sighand: Arc<Mutex<Vec<Option<Box<SigAction>>>>>, // 信号处理器
    pub futex: Arc<Mutex<Futex>>, // 用于 futex 系统调用的同步原语
}

pub struct TaskControlBlockInner {
    pub sigmask: Signals,       //信号屏蔽集
    pub sigpending: Signals,    //阻塞信号队列
    pub trap_cx_ppn: PhysPageNum, //将存放trap上下文信息的物理页号拿出来
    pub task_cx: TaskContext,     //进程的上下文 
    pub task_status: TaskStatus,  //
    pub parent: Option<Weak<TaskControlBlock>>, //父进程
    pub children: Vec<Arc<TaskControlBlock>>,   //子进程
    pub exit_code: u32,   //退出码
    pub clear_child_tid: usize,
    pub robust_list: RobustList,
    pub heap_bottom: usize,
    pub heap_pt: usize,
    pub pgid: usize,
    pub rusage: Rusage,
    pub clock: ProcClock,
    pub timer: [ITimerVal; 3],
}


//文件
pub struct OSInode {  //文件的封装的便于操作系统访问的接口
    readable: bool,
    writable: bool,
    /// See `DirectoryTreeNode` for more details
    special_use: bool,
    append: bool,  //是否在文件尾部追加数据
    inner: Arc<InodeImpl>, //指向了该OSInode指向的Inode
    offset: Mutex<usize>,
    dirnode_ptr: Arc<Mutex<Weak<DirectoryTreeNode>>>, //该文件对应的目录结点
}

pub struct Inode {		//文件的实例
    inode_lock: RwLock<InodeLock>,  //读写锁
    file_content: RwLock<FileContent>, //文件内容
    file_cache_mgr: PageCacheManager,  //文件缓存管理器，用于管理文件的缓存
    file_type: Mutex<DiskInodeType>, //文件类型
    parent_dir: Mutex<Option<(Arc<Self>, u32)>>,  //父节点的inode指针以及该文件对应父节点的偏移量
    fs: Arc<EasyFileSystem>, //简单文件系统指针
    time: Mutex<InodeTime>, //描述文件的打开时间等相关信息
    deleted: Mutex<bool>, //删除
}

pub enum DiskInodeType {
    File,
    Directory,
}

pub struct DirectoryTreeNode {
    spe_usage: Mutex<usize>,
    name: String,
    filesystem: Arc<FileSystem>,
    file: Arc<dyn File>,
    selfptr: Mutex<Weak<Self>>,
    father: Mutex<Weak<Self>>,
    children: RwLock<Option<BTreeMap<String, Arc<Self>>>>,
}
```

------------------

### 栈STACK

栈负责管理函数调用过程，栈帧包括函数的参数、局部变量以及返回地址，栈帧的大小是预先确定的（不同函数的栈帧大小不同）。

### 堆HEAP

堆负责管理内存资源，由内存分配器分配内存资源，只管分配，内存回收需要程序员手动执行。



### Rust语言设计目标

系统编程语言——不能使用GC->堆内存由程序管理

安全性——堆内存自动管理  

可靠性

性能

现代化

### 所有权

三个核心规则：

Rust中的每一个值都与一个变量明确关联，这个变量是该值的唯一拥有者。

任一时刻，每个值都只与一个变量有这种关联。（禁止共享）

当这个变量离开作用域，与之关联的值将会被自动回收。（释放资源是安全的）

~~~ rust
fn inspect (list:Vec<i32>){
    println!("The data is:{:?}",list);
}

let data = vec![1,2,3];
inspect(data);
//现在我们无法再访问data，因为所有权已经被移动到函数内部
~~~

### 引用类型——受限制的指针

限制1：引用类型和类型一一对应（&i32、&bool）

限制2：不可随意修改

限制3：必须有效初始化

~~~rust
let a:&i32; //错误写法，无效初始化

//正确做法
let value = 42;
let a:&i32 = &value;  //在声明的同一行对引用进行初始化
~~~

引入引用类型后，变量共享问题被解决，引用代表某种拥有值的借用。

~~~ rust
fn inspect (list:&Vec<i32>){
    println!("The data is:{:?}",list);
}

let data = vec![1,2,3];
inspect(&data); //使用借用传递data的引用
// inspect函数执行之后，data依然有效切所有权没有改变
print1n!("data can still be accessed:{:?},data");
~~~



### 借用规则

多个不可变借用（读权限）可以共存，但不可变借用和可变借用（写权限）不能同时存在，既不能同时往同一个地址里写数据。

可变引用在任一时刻只能有一个。

引用的生命周期必须在所有者离开作用域之前结束，防止创建悬挂引用。（lifetime）

~~~rust
fn main() {
    let my_ref = dangling_ref();
	//注意：这段代码编译通过，因为my_ref是一个悬垂引用
}

fn dangling_ref() -> &String {  //引用返回内存地址，但是这个内存地址已经无效了
    let temp_string = String::from("我是一个临时字符串");
    &temp_string         //temp_string在dangling_ref函数返回后被销毁
}
~~~



这些借用规则由Rust的借用检查器进行检查，不符合规则的代码将导致编译错误。

### 生命周期

用于解决悬垂引用和其他内存安全问题

``` rust
fn main () {  //编译器通过比较生命周期的长度发现是否出现悬垂引用
    let r ;				//---------------+ --'a
 						//			 	 |
    {					//			 	 |
        let x =5;		//--+ --'b       |
        r = &x;			//  |			 | //通过&借用操作，来确定引用和变量之间的对应关系
    }					//	|		 	 |  //x的生命周期结束
    println!("r:{}",r); //			 	 |  //出现悬垂引用问题，因为r需要在x的生命周期内使用
}						//			 	 |
```

经典的longest例子，这是一个编译错误的代码：

```rust
fn longest (x: &str,y: &str) -> &str {  //因为没有&借用符号，编译器不知道返回值x和y来自哪里，所以报错
    if x.len() > y.len() {
        x
    }else {
        y
    }
}
```

引入生命周期，正确通过编译的做法：

``` rust
// 'a指出返回的引用与输入参数x、y之间有关联。
// 'a只是这个关联关系的代号
fn longest<'a>(x: &'a str,y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    }else {
        y
    }
}
```

~~~rust
//返回结果只与参数x有关
fn some_fun<'a>(x: &'a str,y: &str) -> &'a str

//返回元组，第一个和x有关，第二个和y有关
fn some_fun2<'a,'b>(x: &'a str,y: &'b str) -> (&'a str,&'b str)
~~~

如何简单解读'a的语义：（在解读生命周期的时候，我们只需要知道关联关系即可）

``` rust
fn longest<'a>(x: &'a str,y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    }else {
        y
    }
}

fn main () {
    let string1 = String::from("xyz");
    let result;    
    {
        let string2 = String::from("long string is long");
        //as_str()方法：通常用于从String类型获取字符串切片&str，这种转换不会转移所有权，仅仅是创建一个引用。
        result = longest(string1.as_str(),string2.as_str());
    }
    println!("The longest string is {}",result);
}
```

编译器可以分析出以下几点：

- result变量是引用类型
- 根据longest函数的签名，result可能与参数x、y其中之一相关联
- string1和x关联、string2和y关联
- result可能指向string1或string2，所以result的生命周期长度不能长于他们两个
- 比较result的生命周期与string1和string2中较短者的生命周期
- 发现当result和string2相关时，在代码第16行，result指向了失效的变量string2，出现了悬垂引用的风险
