//#![allow(unused)]
use super::dir_iter::*;
use super::layout::{FATDirEnt,FATDiskInodeType, FATShortDirEnt};
use super::{Cache,PageCache,PageCacheManager};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::convert::TryInto;
use core::panic;
use spin::*;

use super::vfs::Inode;
use super::vfs::InodeLock;
use super::vfs::InodeTime;
macro_rules! div_ceil {
    ($mult:expr,$deno:expr) => {
        ($mult - 1 + $deno) / $deno
    };
}

/// IO
impl Inode {
    /// Read file content into buffer.
    /// It will read from `offset` until the end of the file or buffer can't read more
    /// This operation is ignored if start is greater than or equal to end.
    /// # Arguments    
    /// + `inode_lock`: The lock of inode
    /// + `offset`: The start offset in file
    /// + `buf`: The buffer to receive data
    /// # Return Value
    /// The number of number of bytes read.
    pub fn read_at_block_cache_rlock(
        &self,
        _inode_lock: &RwLockReadGuard<InodeLock>,
        offset: usize,
        buf: &mut [u8],
    ) -> usize {
        let mut start = offset;
        let size = self.file_content.read().size as usize;
        let end = (offset + buf.len()).min(size);
        if start >= end {
            return 0;
        }
        let mut start_cache = start / PageCacheManager::CACHE_SZ;
        let mut read_size = 0;
        loop {
            // calculate end of current block
            let mut end_current_block =
                (start / PageCacheManager::CACHE_SZ + 1) * PageCacheManager::CACHE_SZ;
            end_current_block = end_current_block.min(end);
            // read and update read size
            let lock = self.file_content.read();
            let block_read_size = end_current_block - start;
            self.file_cache_mgr
                .get_cache(
                    start_cache,
                    || -> Vec<usize> { self.get_neighboring_sec(&lock.clus_list, start_cache) },
                    &self.fs.block_device,
                )
                .lock()
                // I know hardcoding 4096 in is bad, but I can't get around Rust's syntax checking...
                .read(0, |data_block: &[u8; 4096]| {
                    let dst = &mut buf[read_size..read_size + block_read_size];
                    let src = &data_block[start % PageCacheManager::CACHE_SZ
                        ..start % PageCacheManager::CACHE_SZ + block_read_size];
                    dst.copy_from_slice(src);
                });
            drop(lock);
            read_size += block_read_size;
            // move to next block
            if end_current_block == end {
                break;
            }
            start_cache += 1;
            start = end_current_block;
        }
        read_size
    }
    /// do same thing but params different
    pub fn read_at_block_cache_wlock(
        &self,
        _inode_lock: &RwLockWriteGuard<InodeLock>,
        offset: usize,
        buf: &mut [u8],
    ) -> usize {
        let mut start = offset;
        let size = self.file_content.read().size as usize;
        let end = (offset + buf.len()).min(size);
        if start >= end {
            return 0;
        }
        let mut start_cache = start / PageCacheManager::CACHE_SZ;
        let mut read_size = 0;
        loop {
            // calculate end of current block
            let mut end_current_block =
                (start / PageCacheManager::CACHE_SZ + 1) * PageCacheManager::CACHE_SZ;
            end_current_block = end_current_block.min(end);
            // read and update read size
            let lock = self.file_content.read();
            let block_read_size = end_current_block - start;
            self.file_cache_mgr
                .get_cache(
                    start_cache,
                    || -> Vec<usize> { self.get_neighboring_sec(&lock.clus_list, start_cache) },
                    &self.fs.block_device,
                )
                .lock()
                // I know hardcoding 4096 in is bad, but I can't get around Rust's syntax checking...
                .read(0, |data_block: &[u8; 4096]| {
                    let dst = &mut buf[read_size..read_size + block_read_size];
                    let src = &data_block[start % PageCacheManager::CACHE_SZ
                        ..start % PageCacheManager::CACHE_SZ + block_read_size];
                    dst.copy_from_slice(src);
                });
            drop(lock);
            read_size += block_read_size;
            // move to next block
            if end_current_block == end {
                break;
            }
            start_cache += 1;
            start = end_current_block;
        }
        read_size
    }
    /// Read file content into buffer.
    /// It will read from `offset` until the end of the file or buffer can't read more
    /// This operation is ignored if start is greater than or equal to end.
    /// # Arguments    
    /// + `offset`: The start offset in file
    /// + `buf`: The buffer to receive data
    /// # Return Value
    /// The number of number of bytes read.
    /// # Warning
    /// This function will lock self's `file_content`, may cause deadlock
    #[inline(always)]
    pub fn read_at_block_cache(&self, offset: usize, buf: &mut [u8]) -> usize {
        self.read_at_block_cache_rlock(&self.read(), offset, buf)
    }

    /// Write buffer into file content.
    /// It will start to write from `offset` until the buffer is written,
    /// and when the write exceeds the end of file, it will modify file's size.
    /// If hard disk space id low, it will try to write as much data as possible.
    /// # Arguments    
    /// + `inode_lock`: The lock of inode
    /// + `offset`: The start offset in file
    /// + `buf`: The buffer to write data
    /// # Return Value
    /// The number of number of bytes write.
    pub fn write_at_block_cache_lock(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        offset: usize,
        buf: &[u8],
    ) -> usize {
        let mut start = offset;
        let old_size = self.get_file_size() as usize;
        let diff_len = buf.len() as isize + offset as isize - old_size as isize;
        if diff_len > 0 as isize {
            // allocate as many clusters as possible.
            self.modify_size_lock(inode_lock, diff_len, false);
        }
        let end = (offset + buf.len()).min(self.get_file_size() as usize);

        debug_assert!(start <= end);

        let mut start_cache = start / PageCacheManager::CACHE_SZ;
        let mut write_size = 0;
        loop {
            // calculate end of current block
            let mut end_current_block =
                (start / PageCacheManager::CACHE_SZ + 1) * PageCacheManager::CACHE_SZ;
            end_current_block = end_current_block.min(end);
            // write and update write size
            let lock = self.file_content.read();
            let block_write_size = end_current_block - start;
            self.file_cache_mgr
                .get_cache(
                    start_cache,
                    || -> Vec<usize> { self.get_neighboring_sec(&lock.clus_list, start_cache) },
                    &self.fs.block_device,
                )
                .lock()
                // I know hardcoding 4096 in is bad, but I can't get around Rust's syntax checking...
                .modify(0, |data_block: &mut [u8; 4096]| {
                    let src = &buf[write_size..write_size + block_write_size];
                    let dst = &mut data_block[start % PageCacheManager::CACHE_SZ
                        ..start % PageCacheManager::CACHE_SZ + block_write_size];
                    dst.copy_from_slice(src);
                });
            drop(lock);
            write_size += block_write_size;
            // move to next block
            if end_current_block == end {
                break;
            }
            start_cache += 1;
            start = end_current_block;
        }
        write_size
    }

    /// Write buffer into file content.
    /// It will start to write from `offset` until the buffer is written,
    /// and when the write exceeds the end of file, it will modify file's size.
    /// If hard disk space id low, it will try to write as much data as possible.
    /// # Arguments    
    /// + `offset`: The start offset in file
    /// + `buf`: The buffer to write data
    /// # Return Value
    /// The number of number of bytes write.
    /// # Warning
    /// This function will lock self's `file_content`, may cause deadlock
    #[inline(always)]
    pub fn write_at_block_cache(&self, offset: usize, buf: &[u8]) -> usize {
        self.write_at_block_cache_lock(&self.write(), offset, buf)
    }

    /// Get a page cache corresponding to `inner_cache_id`.
    /// # Arguments    
    /// + `inner_cache_id`: The index of inner cache
    /// # Return Value
    /// Pointer to page cache
    /// # Warning
    /// This function will lock self's `file_content`, may cause deadlock
    pub fn get_single_cache(&self, inner_cache_id: usize) -> Arc<Mutex<PageCache>> {
        self.get_single_cache_lock(&self.read(), inner_cache_id)
    }

    /// Get a page cache corresponding to `inner_cache_id`.
    /// # Arguments    
    /// + `inode_lock`: The lock of inode
    /// + `inner_cache_id`: The index of inner cache
    /// # Return Value
    /// Pointer to page cache
    pub fn get_single_cache_lock(
        &self,
        _inode_lock: &RwLockReadGuard<InodeLock>,
        inner_cache_id: usize,
    ) -> Arc<Mutex<PageCache>> {
        let lock = self.file_content.read();
        self.file_cache_mgr.get_cache(
            inner_cache_id,
            || -> Vec<usize> { self.get_neighboring_sec(&lock.clus_list, inner_cache_id) },
            &self.fs.block_device,
        )
    }

    /// Get all page caches corresponding to file
    /// # Return Value
    /// List of pointers to the page cache
    pub fn get_all_cache(&self) -> Vec<Arc<Mutex<PageCache>>> {
        let inode_lock = self.read();
        let lock = self.file_content.read();
        let cache_num =
            (lock.size as usize + PageCacheManager::CACHE_SZ - 1) / PageCacheManager::CACHE_SZ;
        let mut cache_list = Vec::<Arc<Mutex<PageCache>>>::with_capacity(cache_num);
        for inner_cache_id in 0..cache_num {
            cache_list.push(self.get_single_cache_lock(&inode_lock, inner_cache_id));
        }
        cache_list
    }
}


// ls and find local
impl Inode {
    /// ls - General Purose file filterer
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// # WARNING
    /// The definition of OFFSET is CHANGED for this item.
    /// It should point to the NEXT USED entry whether it as a long entry whenever possible or a short entry if no long ones exist.
    /// # Return value
    /// On success, the function returns `Ok(_)`. On failure, multiple chances exist: either the Vec is empty, or the Result is `Err(())`.
    /// # Implementation Information
    /// The iterator stops at the last available item when it reaches the end,
    /// returning `None` from then on,
    /// so relying on the offset of the last item to decide whether it has reached an end is not recommended.
    #[inline(always)]
    pub fn ls_lock(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
    ) -> Result<Vec<(String, FATShortDirEnt)>, ()> {
        if !self.is_dir() {
            return Err(());
        }
        Ok(self
            .dir_iter(inode_lock, None, DirIterMode::Used, FORWARD)
            .walk()
            .collect())
    }
    /// find `req_name` in current directory file
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// + `req_name`: required file name
    /// # Return value
    /// On success, the function returns `Ok(_)`. On failure, multiple chances exist: either the Vec is empty, or the Result is `Err(())`.
    pub fn find_local_lock(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        req_name: String,
    ) -> Result<Option<(String, FATShortDirEnt, u32)>, ()> {
        if !self.is_dir() {
            return Err(());
        }
        log::debug!("[find_local] name: {:?}", req_name);
        let mut walker = self
            .dir_iter(inode_lock, None, DirIterMode::Used, FORWARD)
            .walk();
        match walker.find(|(name, _)| {
            name.len() == req_name.len() && name.as_str().eq_ignore_ascii_case(req_name.as_str())
        }) {
            Some((name, short_ent)) => {
                log::trace!("[find_local] Query name: {} found", req_name);
                Ok(Some((name, short_ent, walker.iter.get_offset().unwrap())))
            }
            None => {
                log::trace!("[find_local] Query name: {} not found", req_name);
                Ok(None)
            }
        }
    }
}

// metadata
impl Inode {
    /// Return the `time` field of `self`
    pub fn time(&self) -> MutexGuard<InodeTime> {
        self.time.lock()
    }
    /// Return the `stat` structure to `self` file.
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// # Return value
    /// (file size, access time, modify time, create time, inode number)
    pub fn stat_lock(&self, _inode_lock: &RwLockReadGuard<InodeLock>) -> (i64, i64, i64, i64, u64) {
        let time = self.time.lock();
        (
            self.get_file_size() as i64,
            time.access_time as i64,
            time.modify_time as i64,
            time.create_time as i64,
            self.get_inode_num_lock(&self.file_content.read())
                .unwrap_or(0) as u64,
        )
    }

    pub fn get_all_files_lock(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
    ) -> Vec<(String, FATShortDirEnt, u32)> {
        let mut vec = Vec::with_capacity(8);
        let mut walker = self
            .dir_iter(inode_lock, None, DirIterMode::Used, FORWARD)
            .walk();
        loop {
            let ele = walker.next();
            match ele {
                Some((name, short_ent)) => {
                    if name == "." || name == ".." {
                        continue;
                    }
                    vec.push((name, short_ent, walker.iter.get_offset().unwrap()))
                }
                None => break,
            }
        }
        vec
    }

    /// Get a dirent information from the `self` at `offset`
    /// Return `None` if `self` is not a directory.
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// + `offset` The offset within the `self` directory.
    /// + `length` The length of required vector
    /// # Return value
    /// On success, the function returns `Ok(file name, file size, first cluster, file type)`.
    /// On failure, multiple chances exist: either the Vec is empty, or the Result is `Err(())`.
    pub fn dirent_info_lock(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        offset: u32,
        length: usize,
    ) -> Result<Vec<(String, usize, u64, FATDiskInodeType)>, ()> {
        if !self.is_dir() {
            return Err(());
        }
        let size = self.get_file_size();
        let mut walker = self
            .dir_iter(inode_lock, None, DirIterMode::Used, FORWARD)
            .walk();
        walker.iter.set_iter_offset(offset);
        let mut v = Vec::with_capacity(length);

        let (mut last_name, mut last_short_ent) = match walker.next() {
            Some(tuple) => tuple,
            None => return Ok(v),
        };
        for _ in 0..length {
            let next_dirent_offset =
                walker.iter.get_offset().unwrap() as usize + core::mem::size_of::<FATDirEnt>();
            let (name, short_ent) = match walker.next() {
                Some(tuple) => tuple,
                None => {
                    v.push((
                        last_name,
                        size as usize,
                        last_short_ent.get_first_clus() as u64,
                        last_short_ent.attr,
                    ));
                    return Ok(v);
                }
            };
            v.push((
                last_name,
                next_dirent_offset,
                last_short_ent.get_first_clus() as u64,
                last_short_ent.attr,
            ));
            last_name = name;
            last_short_ent = short_ent;
        }
        Ok(v)
    }
}
