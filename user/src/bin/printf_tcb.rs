#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::print_tcb;

#[no_mangle]
pub fn main() -> i32 {
    let mut message = [0 as usize; 6];
    let mut message_ptr = &mut message as *mut usize;  //指向message地址的可变指针
    print_tcb(message_ptr);
    println!("pid of the TCB is: {} ",message[0]);
    println!("tid of the TCB is: {} ",message[1]);
    println!("tgid of the TCB is: {} ",message[2]);
    println!("kernel stack of the TCB is: {} ",message[3]);
    println!("user stack of the TCB is: {} ",message[4]);
    println!("trap context physics page number of the TCB is: {} ",message[5]);
    0
}