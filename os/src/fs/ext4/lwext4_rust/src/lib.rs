#![no_std]
#![feature(linkage)]
#![feature(c_variadic, c_size_t)]
#![feature(associated_type_defaults)]

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate alloc;

#[macro_use]
extern crate log;

mod ulibc;

// include!("bindings.rs");
pub mod bindings;
pub mod blockdev;
pub mod file;

pub use blockdev::*;
pub use file::{Ext4File, InodeTypes};

pub fn add(left: usize, right: usize) -> usize {
    left + right
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
