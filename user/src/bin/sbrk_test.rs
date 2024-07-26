#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::sbrk;


#[no_mangle]
pub fn main() -> i32 {
    user_lib::println!("old_heap_pt:{:08x}",sbrk(0));
    user_lib::println!("increment:8192, new_heap_pt:{:08x}",sbrk(8192));
    user_lib::println!("increment:-4096, new_heap_pt:{:08x}",sbrk(-4096));
    user_lib::println!("increment:999999999, new_heap_pt:{:08x}",sbrk(999999999));
    user_lib::println!("increment:-8192, new_heap_pt:{:08x}",sbrk(-8192));
    println!("sbrk test pass!!!");
    0
}
