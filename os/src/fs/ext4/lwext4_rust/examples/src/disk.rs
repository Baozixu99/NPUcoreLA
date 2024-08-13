use log::*;
use lwext4_rust::KernelDevOp;
use virtio_drivers::{
    device::{
        blk::{VirtIOBlk, SECTOR_SIZE},
        gpu::VirtIOGpu,
        input::VirtIOInput,
    },
    transport::{
        mmio::{MmioTransport, VirtIOHeader},
        DeviceType, Transport,
    },
    Hal,
};

const BLOCK_SIZE: usize = 512;

/// A disk device with a cursor.
pub struct Disk<H: Hal, T: Transport> {
    block_id: usize,
    offset: usize,
    dev: VirtIOBlk<H, T>,
}

impl<H: Hal, T: Transport> Disk<H, T> {
    /// Create a new disk.
    pub fn new(dev: VirtIOBlk<H, T>) -> Self {
        assert_eq!(BLOCK_SIZE, SECTOR_SIZE);
        Self {
            block_id: 0,
            offset: 0,
            dev,
        }
    }

    /// Get the size of the disk.
    /// capacity() 以512 byte为单位
    pub fn size(&self) -> u64 {
        self.dev.capacity() * BLOCK_SIZE as u64
    }

    /// Get the position of the cursor.
    pub fn position(&self) -> u64 {
        (self.block_id * BLOCK_SIZE + self.offset) as u64
    }

    /// Set the position of the cursor.
    pub fn set_position(&mut self, pos: u64) {
        self.block_id = pos as usize / BLOCK_SIZE;
        self.offset = pos as usize % BLOCK_SIZE;
    }

    /// Read within one block, returns the number of bytes read.
    pub fn read_one(&mut self, buf: &mut [u8]) -> Result<usize, i32> {
        // info!("block id: {}", self.block_id);
        let read_size = if self.offset == 0 && buf.len() >= BLOCK_SIZE {
            // whole block
            self.dev
                .read_blocks(self.block_id, &mut buf[0..BLOCK_SIZE])
                .map_err(as_disk_err)?;
            self.block_id += 1;
            BLOCK_SIZE
        } else {
            // partial block
            let mut data = [0u8; BLOCK_SIZE];
            let start = self.offset;
            let count = buf.len().min(BLOCK_SIZE - self.offset);
            if start > BLOCK_SIZE {
                info!("block size: {} start {}", BLOCK_SIZE, start);
            }

            self.dev
                .read_blocks(self.block_id, &mut data)
                .map_err(as_disk_err)?;
            buf[..count].copy_from_slice(&data[start..start + count]);

            self.offset += count;
            if self.offset >= BLOCK_SIZE {
                self.block_id += 1;
                self.offset -= BLOCK_SIZE;
            }
            count
        };
        Ok(read_size)
    }

    /// Write within one block, returns the number of bytes written.
    pub fn write_one(&mut self, buf: &[u8]) -> Result<usize, i32> {
        let write_size = if self.offset == 0 && buf.len() >= BLOCK_SIZE {
            // whole block
            self.dev
                .write_blocks(self.block_id, &buf[0..BLOCK_SIZE])
                .map_err(as_disk_err)?;
            self.block_id += 1;
            BLOCK_SIZE
        } else {
            // partial block
            let mut data = [0u8; BLOCK_SIZE];
            let start = self.offset;
            let count = buf.len().min(BLOCK_SIZE - self.offset);

            self.dev
                .read_blocks(self.block_id, &mut data)
                .map_err(as_disk_err)?;
            data[start..start + count].copy_from_slice(&buf[..count]);
            self.dev
                .write_blocks(self.block_id, &data)
                .map_err(as_disk_err)?;

            self.offset += count;
            if self.offset >= BLOCK_SIZE {
                self.block_id += 1;
                self.offset -= BLOCK_SIZE;
            }
            count
        };
        Ok(write_size)
    }

    pub fn flush(&mut self) -> Result<(), i32> {
        self.dev.flush().map_err(as_disk_err)
    }
}

impl<H: Hal, T: Transport> KernelDevOp for Disk<H, T> {
    //type DevType = Box<Disk>;
    type DevType = Self;

    fn read(dev: &mut Self, mut buf: &mut [u8]) -> Result<usize, i32> {
        debug!("READ block device buf={}", buf.len());
        let mut read_len = 0;
        while !buf.is_empty() {
            match dev.read_one(buf) {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                    read_len += n;
                }
                Err(_e) => return Err(-1),
            }
        }
        debug!("READ rt len={}", read_len);
        Ok(read_len)
    }
    fn write(dev: &mut Self, mut buf: &[u8]) -> Result<usize, i32> {
        debug!("WRITE block device buf={}", buf.len());
        let mut write_len = 0;
        while !buf.is_empty() {
            match dev.write_one(buf) {
                Ok(0) => break,
                Ok(n) => {
                    buf = &buf[n..];
                    write_len += n;
                }
                Err(_e) => return Err(-1),
            }
        }
        debug!("WRITE rt len={}", write_len);
        Ok(write_len)
    }
    fn flush(dev: &mut Self::DevType) -> Result<usize, i32> {
        dev.flush();
        Ok(0)
    }
    fn seek(dev: &mut Self, off: i64, whence: i32) -> Result<i64, i32> {
        let size = dev.size();
        debug!(
            "SEEK block device size:{}, pos:{}, offset={}, whence={}",
            size,
            &dev.position(),
            off,
            whence
        );
        let new_pos = match whence as u32 {
            lwext4_rust::bindings::SEEK_SET => Some(off),
            lwext4_rust::bindings::SEEK_CUR => {
                dev.position().checked_add_signed(off).map(|v| v as i64)
            }
            lwext4_rust::bindings::SEEK_END => size.checked_add_signed(off).map(|v| v as i64),
            _ => {
                error!("invalid seek() whence: {}", whence);
                Some(off)
            }
        }
        .ok_or(-1)?;

        if new_pos as u64 > size {
            warn!("Seek beyond the end of the block device");
        }
        dev.set_position(new_pos as u64);
        Ok(new_pos)
    }
}

const fn as_disk_err(e: virtio_drivers::Error) -> i32 {
    use virtio_drivers::Error::*;
    match e {
        QueueFull => -1,
        NotReady => -2,
        WrongToken => -3,
        AlreadyUsed => -4,
        InvalidParam => -5,
        DmaError => -6,
        IoError => -7,
        Unsupported => -8,
        ConfigSpaceTooSmall => -9,
        ConfigSpaceMissing => -10,
        _ => -127,
    }
}
