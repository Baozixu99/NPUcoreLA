#![no_std]
#![no_main]
#![feature(asm)]
#![feature(linkage)]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(core_intrinsics)]
#![feature(global_asm)]
#![feature(asm_experimental_arch)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
// #![feature(btree_drain_filter)]
// #![feature(drain_filter)]
#![feature(int_roundings)]
#![feature(string_remove_matches)]
#![feature(lang_items)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![feature(option_result_unwrap_unchecked)]
#![feature(const_maybe_uninit_assume_init)]

pub use arch::config;
extern crate alloc;

#[macro_use]
extern crate bitflags;

#[macro_use]
mod console;
mod arch;
mod drivers;
mod fs;
mod lang_items;
mod mm;
mod syscall;
mod task;
mod timer;

use crate::arch::{bootstrap_init, machine_init};
// #[cfg(feature = "board_2k1000")]
use crate::config::DISK_IMAGE_BASE;
#[cfg(feature = "la64")]
core::arch::global_asm!(include_str!("arch/la64/entry.asm"));
// #[cfg(feature = "board_2k1000")]
core::arch::global_asm!(include_str!("load_img.S"));
// core::arch::global_asm!(include_str!("preload_app.S"));

fn mem_clear() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    #[cfg(feature = "zero_init")]
    unsafe {
        core::slice::from_raw_parts_mut(
            sbss as usize as *mut u8,
            crate::config::MEMORY_END - sbss as usize,
        )
        .fill(0);
    }
    #[cfg(not(feature = "zero_init"))]
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

// #[cfg(feature = "board_2k1000")]
fn move_to_high_address() {
    extern "C" {
        fn simg();
        fn eimg();
    }
    unsafe {
        let img = core::slice::from_raw_parts(    // 创建一个不可变切片，用于表示镜像的数据
            simg as usize as *mut u8,
            eimg as usize - simg as usize
        );
        // 从DISK_IMAGE_BASE到MEMORY_END
        let mem_disk = core::slice::from_raw_parts_mut(  // 创建一个可变切片，用于表示目标内存
            DISK_IMAGE_BASE as *mut u8,
            0x800_0000
        );
        mem_disk.fill(0);   // 清空目标内存
        mem_disk[..img.len()].copy_from_slice(img); // 将镜像数据复制到目标内存中
    }
}

#[no_mangle]
pub fn rust_main() -> ! {
    bootstrap_init();
    mem_clear();
    // #[cfg(feature = "board_2k1000")]
    move_to_high_address();
    console::log_init();
    println!("[kernel] Console initialized.");
    mm::init();
    // note that remap_test is currently NOT supported by LA64, for the whole kernel space is RW!
    //mm::remap_test();

    machine_init();
    println!("[kernel] Hello, Welcome to HPU!");

    //machine independent initialization
    fs::directory_tree::init_fs();
    // fs::flush_preload();
    task::add_initproc();
    // note that in run_tasks(), there is yet *another* pre_start_init(),
    // which is used to turn on interrupts in some archs like LoongArch.
    task::run_tasks();
    panic!("Unreachable in rust_main!");
    
}

#[cfg(test)]
fn test_runner(_tests: &[&dyn Fn()]) {}
