#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::string::String;
use user_lib::{close, open, read, OpenFlags};

#[no_mangle]
pub fn main(argc: usize, argv: &[&str]) -> i32 {
    println!("argc: {}", argc);
    println!("argv: {:?}", argv);
    assert!(argc == 2);
    let fd = open(argv[1], OpenFlags::RDONLY);
    if fd == -1 {
        panic!("Error occured when opening file");
    }
    let fd = fd as usize;
    let mut buf = [0u8; 16];
    let mut s = String::new();
    let mut size = 0;
    loop {
        size = read(fd, &mut buf) as usize;
        if size == 0 {
            break;
        }
        s.push_str(core::str::from_utf8(&buf[..size]).unwrap());
    }
    print!("{}", s);
    close(fd);
    0
}
