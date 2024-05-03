use crate::config::HIGH_BASE_EIGHT;

pub const MMIO: &[(usize, usize)] = &[];

use crate::drivers::block::MemBlockWrapper;
pub type BlockDeviceImpl = MemBlockWrapper;

pub const ROOT_BASE_ADDR: usize = 0x00e0_0000;
pub const BLOCK_SZ: usize = 2048;
pub const UART_BASE: usize = 0x1FE2_0000 + HIGH_BASE_EIGHT;
pub const ACPI_BASE: usize = 0x1FE2_7000;
