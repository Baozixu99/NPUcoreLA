//#![allow(unused)]
use super::dir_iter::*;
use super::layout::{FATDirEnt, FATLongDirEnt, FATShortDirEnt};
use super::{Cache, PageCacheManager};
use alloc::vec::Vec;
use core::ops::Mul;
use core::panic;
use spin::*;
use super::vfs::Inode;
use super::vfs::FileContent;
use super::vfs::InodeLock;

macro_rules! div_ceil {
    ($mult:expr,$deno:expr) => {
        ($mult - 1 + $deno) / $deno
    };
}
/// Directory Operation
impl Inode {
    /// A Constructor for `DirIter`(See `dir_iter.rs/DirIter` for details).
    /// # Arguments    
    /// + `inode_lock`: The lock of inode
    /// + `offset`: The start offset of iterator
    /// + `mode`: The mode of iterator
    /// + `forward`: The direction of the iterator iteration
    /// # Return Value
    /// Pointer to iterator
    /// 
   
    pub fn dir_iter<'a, 'b>(
        &'a self,
        inode_lock: &'a RwLockWriteGuard<'b, InodeLock>,
        offset: Option<u32>,
        mode: DirIterMode,
        forward: bool,
    ) -> DirIter<'a, 'b> {
        debug_assert!(self.is_dir(), "this isn't a directory");
        DirIter::new(inode_lock, offset, mode, forward, self)
    }
    /// Set the offset of the last entry in the directory file(first byte is 0x00) to hint
    pub fn set_hint(&self) {
        let inode_lock = self.write();
        let mut iter = self.dir_iter(&inode_lock, None, DirIterMode::Enum, FORWARD);
        loop {
            let dir_ent = iter.next();
            if dir_ent.is_none() {
                // Means iter reachs the end of file
                let mut lock = self.file_content.write();
                lock.hint = lock.size;
                return;
            }
            let dir_ent = dir_ent.unwrap();
            if dir_ent.last_and_unused() {
                let mut lock = self.file_content.write();
                lock.hint = iter.get_offset().unwrap();
                return;
            }
        }
    }
    /// Check if current file is an empty directory
    /// If a file contains only "." and "..", we consider it to be an empty directory
    /// # Arguments    
    /// + `inode_lock`: The lock of inode
    /// # Return Value
    /// Bool result
    pub fn is_empty_dir_lock(&self, inode_lock: &RwLockWriteGuard<InodeLock>) -> bool {
        if !self.is_dir() {
            return false;
        }
        let iter = self
            .dir_iter(inode_lock, None, DirIterMode::Used, FORWARD)
            .walk();
        for (name, _) in iter {
            if [".", ".."].contains(&name.as_str()) == false {
                return false;
            }
        }
        true
    }
    /// Expand directory file's size(a cluster)
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// # Return Value
    /// Default is Ok
    fn expand_dir_size(&self, inode_lock: &RwLockWriteGuard<InodeLock>) -> Result<(), ()> {
        let diff_size = self.fs.clus_size();
        self.modify_size_lock(inode_lock, diff_size as isize, false);
        Ok(())
    }
    /// Shrink directory file's size to fit `hint`.
    /// For directory files, it has at least one cluster, which should be noted.
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// # Return Value
    /// Default is Ok
    fn shrink_dir_size(&self, inode_lock: &RwLockWriteGuard<InodeLock>) -> Result<(), ()> {
        let lock = self.file_content.read();
        let new_size = div_ceil!(lock.hint, self.fs.clus_size())
            .mul(self.fs.clus_size())
            .max(self.fs.clus_size());
        /*lock
        .hint
        .div_ceil(self.fs.clus_size())
        .mul(self.fs.clus_size())
        // For directory file, it has at least one cluster
        .max(self.fs.clus_size());*/
        let diff_size = new_size as isize - lock.size as isize;
        drop(lock);
        self.modify_size_lock(inode_lock, diff_size as isize, false);
        Ok(())
    }
    /// Allocate directory entries required for new file.
    /// The allocated directory entries is a contiguous segment.
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// + `alloc_num`: Required number of directory entries
    /// # Return Value
    /// It will return lock anyway.
    /// If successful, it will also return the offset of the last allocated entry.
    fn alloc_dir_ent(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        alloc_num: usize,
    ) -> Result<u32, ()> {
        let offset = self.file_content.read().hint;
        let mut iter = self.dir_iter(inode_lock, None, DirIterMode::Enum, FORWARD);
        iter.set_iter_offset(offset);
        let mut found_free_dir_ent = 0;
        loop {
            let dir_ent = iter.next();
            if dir_ent.is_none() {
                if self.expand_dir_size(&mut iter.inode_lock).is_err() {
                    log::error!("[alloc_dir_ent]expand directory size error");
                    return Err(());
                }
                continue;
            }
            // We assume that all entries after `hint` are valid
            // That's why we use `hint`. It can reduce the cost of iterating over used entries
            found_free_dir_ent += 1;
            if found_free_dir_ent >= alloc_num {
                let offset = iter.get_offset().unwrap();
                // Set hint
                // Set next entry to last_and_unused
                if iter.next().is_some() {
                    iter.write_to_current_ent(&FATDirEnt::unused_and_last_entry());
                    let mut lock = self.file_content.write();
                    lock.hint = iter.get_offset().unwrap();
                } else {
                    // Means iter reachs the end of file
                    let mut lock = self.file_content.write();
                    lock.hint = lock.size;
                }
                return Ok(offset);
            }
        }
    }
    /// Get a directory entries.
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// + `offset`: The offset of entry
    /// # Return Value
    /// If successful, it will return a `FATDirEnt`(See `layout.rs/FATDirEnt` for details)
    /// Otherwise, it will return Error
    /// # Warning
    /// This function will lock self's `file_content`, may cause deadlock
    pub fn get_dir_ent(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        offset: u32,
    ) -> Result<FATDirEnt, ()> {
        let mut dir_ent = FATDirEnt::empty();
        if self.read_at_block_cache_wlock(inode_lock, offset as usize, dir_ent.as_bytes_mut())
            != dir_ent.as_bytes().len()
        {
            return Err(());
        }
        Ok(dir_ent)
    }
    /// Write the directory entry back to the file contents.
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// + `offset`: The offset of file to write
    /// + `dir_ent`: The buffer needs to write back
    /// # Return Value
    /// If successful, it will return Ok.
    /// Otherwise, it will return Error.
    /// # Warning
    /// This function will lock self's `file_content`, may cause deadlock
    pub fn set_dir_ent(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        offset: u32,
        dir_ent: FATDirEnt,
    ) -> Result<(), ()> {
        if self.write_at_block_cache_lock(inode_lock, offset as usize, dir_ent.as_bytes())
            != dir_ent.as_bytes().len()
        {
            return Err(());
        }
        Ok(())
    }
    /// Get directory entries, including short and long entries
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// + `offset`: The offset of short entry
    /// # Return Value
    /// If successful, it returns a pair of a short directory entry and a long directory entry list.
    /// Otherwise, it will return Error.
    /// # Warning
    /// This function will lock self's `file_content`, may cause deadlock
    fn get_all_dir_ent(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        offset: u32,
    ) -> Result<(FATShortDirEnt, Vec<FATLongDirEnt>), ()> {
        debug_assert!(self.is_dir());
        let short_ent: FATShortDirEnt;
        let mut long_ents = Vec::<FATLongDirEnt>::with_capacity(5);

        let mut iter = self.dir_iter(inode_lock, Some(offset), DirIterMode::Enum, BACKWARD);

        short_ent = *iter.current_clone().unwrap().get_short_ent().unwrap();

        // Check if this directory entry is only a short directory entry
        {
            let dir_ent = iter.next();
            // First directory entry
            if dir_ent.is_none() {
                return Ok((short_ent, long_ents));
            }
            let dir_ent = dir_ent.unwrap();
            // Short directory entry
            if !dir_ent.is_long() {
                return Ok((short_ent, long_ents));
            }
        }

        // Get long dir_ents
        loop {
            let dir_ent = iter.current_clone();
            if dir_ent.is_none() {
                return Err(());
            }
            let dir_ent = dir_ent.unwrap();
            if dir_ent.get_long_ent().is_none() {
                return Err(());
            }
            long_ents.push(*dir_ent.get_long_ent().unwrap());
            if dir_ent.is_last_long_dir_ent() {
                break;
            }
        }
        Ok((short_ent, long_ents))
    }
    /// Delete derectory entries, including short and long entries.
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// + `offset`: The offset of short entry
    /// # Return Value
    /// If successful, it will return Ok.
    /// Otherwise, it will return Error.
    /// # Warning
    /// This function will lock self's `file_content`, may cause deadlock.
    pub fn delete_dir_ent(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        offset: u32,
    ) -> Result<(), ()> {
        debug_assert!(self.is_dir());
        let mut iter = self.dir_iter(inode_lock, Some(offset), DirIterMode::Used, BACKWARD);

        iter.write_to_current_ent(&FATDirEnt::unused_not_last_entry());
        // Check if this directory entry is only a short directory entry
        {
            let dir_ent = iter.next();
            // First directory entry
            if dir_ent.is_none() {
                return Ok(());
            }
            let dir_ent = dir_ent.unwrap();
            // Short directory entry
            if !dir_ent.is_long() {
                return Ok(());
            }
        }
        // Remove long dir_ents
        loop {
            let dir_ent = iter.current_clone();
            if dir_ent.is_none() {
                return Err(());
            }
            let dir_ent = dir_ent.unwrap();
            if !dir_ent.is_long() {
                return Err(());
            }
            iter.write_to_current_ent(&FATDirEnt::unused_not_last_entry());
            iter.next();
            if dir_ent.is_last_long_dir_ent() {
                break;
            }
        }
        // Modify hint
        // We use new iterate mode
        let mut iter = self.dir_iter(
            inode_lock,
            Some(self.file_content.read().hint),
            DirIterMode::Enum,
            BACKWARD,
        );
        loop {
            let dir_ent = iter.next();
            if dir_ent.is_none() {
                // Indicates that the file is empty
                self.file_content.write().hint = 0;
                break;
            }
            let dir_ent = dir_ent.unwrap();
            if dir_ent.unused() {
                self.file_content.write().hint = iter.get_offset().unwrap();
                iter.write_to_current_ent(&FATDirEnt::unused_and_last_entry());
            } else {
                // Represents `iter` pointer to a used entry
                break;
            }
        }
        // Modify file size
        self.shrink_dir_size(inode_lock)
    }
    /// Create new disk space for derectory entries, including short and long entries.
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// + `short_ent`: short entry
    /// + `long_ents`: list of long entries
    /// # Return Value
    /// If successful, it will return Ok.
    /// Otherwise, it will return Error.
    /// # Warning
    /// This function will lock self's `file_content`, may cause deadlock.
    pub fn create_dir_ent(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        short_ent: FATShortDirEnt,
        long_ents: Vec<FATLongDirEnt>,
    ) -> Result<u32, ()> {
        debug_assert!(self.is_dir());
        let short_ent_offset = match self.alloc_dir_ent(inode_lock, 1 + long_ents.len()) {
            Ok(offset) => offset,
            Err(_) => return Err(()),
        };
        // We have graranteed we have alloc enough entries
        // So we use Enum mode
        let mut iter = self.dir_iter(
            inode_lock,
            Some(short_ent_offset),
            DirIterMode::Enum,
            BACKWARD,
        );

        iter.write_to_current_ent(&FATDirEnt {
            short_entry: short_ent,
        });
        for long_ent in long_ents {
            iter.next();
            iter.write_to_current_ent(&FATDirEnt {
                long_entry: long_ent,
            });
        }
        Ok(short_ent_offset)
    }
    /// Modify current directory file's ".." directory entry
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// + `parent_dir_clus_num`: The first cluster number of the parent directory
    /// # Return Value
    /// If successful, it will return Ok.
    /// Otherwise, it will return Error.
    /// # Warning
    /// This function will lock self's `file_content`, may cause deadlock
    pub fn modify_parent_dir_entry(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        parent_dir_clus_num: u32,
    ) -> Result<(), ()> {
        debug_assert!(self.is_dir());
        let mut iter = self.dir_iter(inode_lock, None, DirIterMode::Used, FORWARD);
        loop {
            let dir_ent = iter.next();
            if dir_ent.is_none() {
                break;
            }
            let mut dir_ent = dir_ent.unwrap();
            if dir_ent.get_name() == ".." {
                dir_ent.set_fst_clus(parent_dir_clus_num);
                iter.write_to_current_ent(&dir_ent);
                return Ok(());
            }
        }
        Err(())
    }
}


/// File Content Operation
impl Inode {
    /// Allocate the required cluster.
    /// It will allocate as much as possible and then append to `clus_list` in `lock`.
    /// # Arguments
    /// + `lock`: The lock of target file content
    /// + `alloc_num`: Required number of clusters
    fn alloc_clus(&self, lock: &mut RwLockWriteGuard<FileContent>, alloc_num: usize) {
        let clus_list = &mut lock.clus_list;
        let mut new_clus_list = self.fs.fat.alloc(
            &self.fs.block_device,
            alloc_num,
            clus_list.last().map(|clus| *clus),
        );
        clus_list.append(&mut new_clus_list);
    }
    /// Release a certain number of clusters from `clus_list` in `lock`.
    /// `clus_list` will be emptied when the quantity to be freed exceeds the available quantity.
    /// # Arguments
    /// + `lock`: The lock of target file content
    /// + `dealloc_num`: The number of clusters that need to be released
    pub fn dealloc_clus(&self, lock: &mut RwLockWriteGuard<FileContent>, dealloc_num: usize) {
        let clus_list = &mut lock.clus_list;
        let dealloc_num = dealloc_num.min(clus_list.len());
        let mut dealloc_list = Vec::<u32>::with_capacity(dealloc_num);
        for _ in 0..dealloc_num {
            dealloc_list.push(clus_list.pop().unwrap());
        }
        self.fs.fat.free(
            &self.fs.block_device,
            dealloc_list,
            clus_list.last().map(|x| *x),
        );
    }
    /// Change the size of current file.
    /// This operation is ignored if the result size is negative
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// + `diff`: The change in file size
    /// # Warning
    /// This function will not modify its parent directory (since we changed the size of the current file),
    /// we will modify it when it is deleted.
    pub fn modify_size_lock(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        diff: isize,
        clear: bool,
    ) {
        let mut lock = self.file_content.write();

        debug_assert!(diff.saturating_add(lock.size as isize) >= 0);

        let old_size = lock.size;
        let new_size = (lock.size as isize + diff) as u32;

        let old_clus_num = self.total_clus(old_size) as usize;
        let new_clus_num = self.total_clus(new_size) as usize;

        if diff > 0 {
            self.alloc_clus(&mut lock, new_clus_num - old_clus_num);
        } else {
            self.dealloc_clus(&mut lock, old_clus_num - new_clus_num);
        }

        lock.size = new_size;
        drop(lock);

        if diff > 0 {
            if clear {
                self.clear_at_block_cache_lock(
                    inode_lock,
                    old_size as usize,
                    (new_size - old_size) as usize,
                );
            }
        } else {
            self.file_cache_mgr.notify_new_size(new_size as usize)
        }
    }

    fn clear_at_block_cache_lock(
        &self,
        _inode_lock: &RwLockWriteGuard<InodeLock>,
        offset: usize,
        length: usize,
    ) -> usize {
        let mut start = offset;
        let end = offset + length;

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
                    let dst = &mut data_block[start % PageCacheManager::CACHE_SZ
                        ..start % PageCacheManager::CACHE_SZ + block_write_size];
                    dst.fill(0);
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

    /// When memory is low, it is called to free its cache
    /// it just tries to lock it's file contents to free memory
    /// # Return Value
    /// The number of freed pages
    pub fn oom(&self) -> usize {
        let neighbor = |inner_cache_id| {
            self.get_neighboring_sec(&self.file_content.read().clus_list, inner_cache_id)
        };
        self.file_cache_mgr.oom(neighbor, &self.fs.block_device)
    }
}
