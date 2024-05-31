#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::mycall;

#[no_mangle]
pub fn main() -> i32 {
    mycall();
    0
}