core::arch::global_asm!(include_str!("entry64.asm"));

extern crate linked_list_allocator;

pub mod sbi;

#[macro_use]
pub mod logger;

pub mod lang_items;

use self::linked_list_allocator::LockedHeap;
use log::*;

#[global_allocator]
pub static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub const KERNEL_HEAP_SIZE: usize = 1024 * 1024 * 2;

pub fn init_heap() {
    static mut HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP.as_mut_ptr(), KERNEL_HEAP_SIZE);
    }
    info!("heap init end");
}
