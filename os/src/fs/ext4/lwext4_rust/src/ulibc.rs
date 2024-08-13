use alloc::alloc::{alloc, dealloc, Layout};
use alloc::slice::from_raw_parts_mut;
use alloc::string::String;
use core::cmp::min;
use core::ffi::{c_char, c_int, c_size_t, c_void};

#[cfg(feature = "print")]
#[linkage = "weak"]
#[no_mangle]
unsafe extern "C" fn printf(str: *const c_char, mut args: ...) -> c_int {
    // extern "C" { pub fn printf(arg1: *const c_char, ...) -> c_int; }
    use printf_compat::{format, output};

    let mut s = String::new();
    let bytes_written = format(str as _, args.as_va_list(), output::fmt_write(&mut s));
    //println!("{}", s);
    info!("{}", s);

    bytes_written
}

#[cfg(not(feature = "print"))]
#[linkage = "weak"]
#[no_mangle]
unsafe extern "C" fn printf(str: *const c_char, mut args: ...) -> c_int {
    use core::ffi::CStr;
    let c_str = unsafe { CStr::from_ptr(str) };
    //let arg1 = args.arg::<usize>();

    info!("[lwext4] {:?}", c_str);
    0
}

#[no_mangle]
pub extern "C" fn ext4_user_malloc(size: c_size_t) -> *mut c_void {
    malloc(size)
}

#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn calloc(m: c_size_t, n: c_size_t) -> *mut c_void {
    let mem = malloc(m * n);

    extern "C" {
        pub fn memset(dest: *mut c_void, c: c_int, n: c_size_t) -> *mut c_void;
    }
    unsafe { memset(mem, 0, m * n) }
}

#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn realloc(memblock: *mut c_void, size: c_size_t) -> *mut c_void {
    if memblock.is_null() {
        warn!("realloc a a null mem pointer");
        return malloc(size);
    }

    let ptr = memblock.cast::<MemoryControlBlock>();
    let old_size = unsafe { ptr.sub(1).read().size };
    info!("realloc from {} to {}", old_size, size);

    let mem = malloc(size);

    unsafe {
        let old_size = min(size, old_size);
        let mbuf = from_raw_parts_mut(mem as *mut u8, old_size);
        mbuf.copy_from_slice(from_raw_parts_mut(memblock as *mut u8, old_size));
    }
    free(memblock);

    mem
}

#[no_mangle]
pub extern "C" fn ext4_user_free(p: *mut c_void) {
    free(p)
}

struct MemoryControlBlock {
    size: usize,
}
const CTRL_BLK_SIZE: usize = core::mem::size_of::<MemoryControlBlock>();

/// Allocate size bytes memory and return the memory address.
#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn malloc(size: c_size_t) -> *mut c_void {
    // Allocate `(actual length) + 8`. The lowest 8 Bytes are stored in the actual allocated space size.
    let layout = Layout::from_size_align(size + CTRL_BLK_SIZE, 8).unwrap();
    unsafe {
        let ptr = alloc(layout);
        assert!(!ptr.is_null(), "malloc failed");
        //debug!("malloc {}@{:p}", size + CTRL_BLK_SIZE, ptr);

        let ptr = ptr.cast::<MemoryControlBlock>();
        ptr.write(MemoryControlBlock { size });
        ptr.add(1).cast()
    }
}

/// Deallocate memory at ptr address
#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn free(ptr: *mut c_void) {
    if ptr.is_null() {
        warn!("free a null pointer !");
        return;
    }
    //debug!("free pointer {:p}", ptr);

    let ptr = ptr.cast::<MemoryControlBlock>();
    assert!(ptr as usize > CTRL_BLK_SIZE, "free a null pointer"); // ?
    unsafe {
        let ptr = ptr.sub(1);
        let size = ptr.read().size;
        let layout = Layout::from_size_align(size + CTRL_BLK_SIZE, 8).unwrap();
        dealloc(ptr.cast(), layout)
    }
}
