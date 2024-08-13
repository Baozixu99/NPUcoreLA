//#![allow(unused)]
use super::dir_iter::*;
use super::layout::{FATDirEnt,FATLongDirEnt, FATShortDirEnt};
use super::DiskInodeType;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::convert::TryInto;
use core::panic;
use spin::*;

use super::vfs::Inode;
use super::vfs::InodeLock;
macro_rules! div_ceil {
    ($mult:expr,$deno:expr) => {
        ($mult - 1 + $deno) / $deno
    };
}

/// Delete
impl Inode {
    /// Delete the short and the long entry of `self` from `parent_dir`
    /// # Return Value
    /// If successful, it will return Ok.
    /// Otherwise, it will return Error.
    /// # Warning
    /// This function will lock self's parent_dir, may cause deadlock
    fn delete_self_dir_ent(&self) -> Result<(), ()> {
        if let Some((par_inode, offset)) = &*self.parent_dir.lock() {
            return par_inode.delete_dir_ent(&par_inode.write(), *offset);
        }
        Err(())
    }
    /// Delete the file from the disk,
    /// This file doesn't be removed immediately(dropped)
    /// deallocating both the directory entries (whether long or short),
    /// and the occupied clusters.
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// + `delete`: Signal of deleting the file content when inode is dropped
    /// # Return Value
    /// If successful, it will return Ok.
    /// Otherwise, it will return Error with error number.
    /// # Warning
    /// This function will lock trash's `file_content`, may cause deadlock
    /// Make sure Arc has a strong count of 1.
    /// Make sure all its caches are not held by anyone else.
    /// Make sure target directory file is empty.
    pub fn unlink_lock(
        &self,
        _inode_lock: &RwLockWriteGuard<InodeLock>,
        delete: bool,
    ) -> Result<(), isize> {
        log::debug!(
            "[delete_from_disk] inode: {:?}, type: {:?}",
            self.get_inode_num_lock(&self.file_content.read()),
            self.file_type
        );
        // Remove directory entries
        if self.parent_dir.lock().is_none() {
            return Ok(());
        }
        if self.delete_self_dir_ent().is_err() {
            panic!()
        }
        if delete {
            *self.deleted.lock() = true;
        }
        *self.parent_dir.lock() = None;
        Ok(())
    }
}

/// Link
impl Inode {
    pub fn link_par_lock(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        parent_dir: &Arc<Self>,
        parent_inode_lock: &RwLockWriteGuard<InodeLock>,
        name: String,
    ) -> Result<(), ()> {
        // Genrate directory entries
        let (short_ent, long_ents) = Self::gen_dir_ent(
            parent_dir,
            parent_inode_lock,
            &name,
            self.get_first_clus_lock(&self.file_content.read())
                .unwrap_or(0),
            *self.file_type.lock(),
        );
        // Allocate new directory entry
        let short_ent_offset =
            match parent_dir.create_dir_ent(parent_inode_lock, short_ent, long_ents) {
                Ok(offset) => offset,
                Err(_) => return Err(()),
            };
        // If this is a directory, modify ".."
        if self.is_dir()
            && self
                .modify_parent_dir_entry(
                    inode_lock,
                    parent_dir
                        .get_first_clus_lock(&parent_dir.file_content.read())
                        .unwrap(),
                )
                .is_err()
        {
            return Err(());
        }
        // Modify parent directory
        *self.parent_dir.lock() = Some((parent_dir.clone(), short_ent_offset));
        Ok(())
    }
}

/// Create
impl Inode {
    /// Create a file or a directory from the parent.
    /// The parent directory will write the new file directory entries.
    /// # Arguments
    /// + `parent_dir`: the pointer to parent directory inode
    /// + `parent_inode_lock`: the lock of parent's inode
    /// + `name`: new file's name
    /// + `file_type`: new file's file type
    /// # Return Value
    /// If successful, it will return the new file inode
    /// Otherwise, it will return Error.
    /// # Warning
    /// This function will lock the `file_content` of the parent directory, may cause deadlock
    /// The length of name should be less than 256(for ascii), otherwise the file system can not store.
    /// Make sure there are no duplicate names in parent_dir.
    pub fn create_lock(
        parent_dir: &Arc<Self>,
        parent_inode_lock: &RwLockWriteGuard<InodeLock>,
        name: String,
        file_type: DiskInodeType,
    ) -> Result<Arc<Self>, ()> {
        if parent_dir.is_file() || name.len() >= 256 {
            Err(())
        } else {
            log::debug!(
                "[create] par_inode: {:?}, name: {:?}, file_type: {:?}",
                parent_dir.get_inode_num_lock(&parent_dir.file_content.read()),
                &name,
                file_type
            );
            // If file_type is Directory, alloc first cluster
            let fst_clus = if file_type == DiskInodeType::Directory {
                let fst_clus = parent_dir
                    .fs
                    .fat
                    .alloc(&parent_dir.fs.block_device, 1, None);
                if fst_clus.is_empty() {
                    return Err(());
                }
                fst_clus[0]
            } else {
                0
            };
            // Genrate directory entries
            let (short_ent, long_ents) =
                Self::gen_dir_ent(parent_dir, parent_inode_lock, &name, fst_clus, file_type);
            // Create directory entry
            let short_ent_offset =
                match parent_dir.create_dir_ent(parent_inode_lock, short_ent, long_ents) {
                    Ok(offset) => offset,
                    Err(_) => return Err(()),
                };
            // Generate current file
            let current_file = Self::from_ent(&parent_dir, &short_ent, short_ent_offset);
            // If file_type is Directory, set first 3 directory entry
            if file_type == DiskInodeType::Directory {
                // Set hint
                current_file.file_content.write().hint =
                    2 * core::mem::size_of::<FATDirEnt>() as u32;
                // Fill content
                Self::fill_empty_dir(&parent_dir, &current_file, fst_clus);
            }
            Ok(current_file)
        }
    }

    /// Construct a \[u16,13\] corresponding to the `long_ent_num`'th 13-u16 or shorter name slice
    /// _NOTE_: the first entry is of number 0 for `long_ent_num`
    /// # Arguments
    /// + `name`: File name
    /// + `long_ent_index`: The index of long entry(start from 0)
    /// # Return Value
    /// A long name slice
    fn gen_long_name_slice(name: &String, long_ent_index: usize) -> [u16; 13] {
        let mut v: Vec<u16> = name.encode_utf16().collect();
        debug_assert!(long_ent_index * 13 < v.len());
        while v.len() < (long_ent_index + 1) * 13 {
            v.push(0);
        }
        let start = long_ent_index * 13;
        let end = (long_ent_index + 1) * 13;
        v[start..end].try_into().expect("should be able to cast")
    }

    /// Construct a \[u8,11\] corresponding to the short directory entry name
    /// # Arguments
    /// + `parent_dir`: The pointer to parent directory
    /// + `parent_inode_lock`: the lock of parent's inode
    /// + `name`: File name
    /// # Return Value
    /// A short name slice
    /// # Warning
    /// This function will lock the `file_content` of the parent directory, may cause deadlock
    fn gen_short_name_slice(
        parent_dir: &Arc<Self>,
        parent_inode_lock: &RwLockWriteGuard<InodeLock>,
        name: &String,
    ) -> [u8; 11] {
        let short_name = FATDirEnt::gen_short_name_prefix(name.clone());
        if short_name.len() == 0 || short_name.find(' ').unwrap_or(8) == 0 {
            panic!("illegal short name");
        }

        let mut short_name_slice = [0u8; 11];
        short_name_slice.copy_from_slice(&short_name.as_bytes()[0..11]);

        let iter = parent_dir.dir_iter(parent_inode_lock, None, DirIterMode::Short, FORWARD);
        FATDirEnt::gen_short_name_numtail(iter.collect(), &mut short_name_slice);
        short_name_slice
    }
    /// Construct short and long entries name slices
    /// # Arguments
    /// + `parent_dir`: The pointer to parent directory
    /// + `parent_inode_lock`: the lock of parent's inode
    /// + `name`: File name
    /// # Return Value
    /// A pair of a short name slice and a list of long name slices
    /// # Warning
    /// This function will lock the `file_content` of the parent directory, may cause deadlock
    fn gen_name_slice(
        parent_dir: &Arc<Self>,
        parent_inode_lock: &RwLockWriteGuard<InodeLock>,
        name: &String,
    ) -> ([u8; 11], Vec<[u16; 13]>) {
        let short_name_slice = Self::gen_short_name_slice(parent_dir, parent_inode_lock, name);

        let long_ent_num = div_ceil!(name.len(), 13);
        //name.len().div_ceil(13);
        let mut long_name_slices = Vec::<[u16; 13]>::with_capacity(long_ent_num);
        for i in 0..long_ent_num {
            long_name_slices.push(Self::gen_long_name_slice(name, i));
        }

        (short_name_slice, long_name_slices)
    }

    /// Construct short and long entries
    /// # Arguments
    /// + `parent_dir`: The pointer to parent directory
    /// + `parent_inode_lock`: the lock of parent's inode
    /// + `name`: File name
    /// + `fst_clus`: The first cluster of constructing file
    /// + `file_type`: The file type of constructing file
    /// # Return Value
    /// A pair of a short directory entry and a list of long name entries
    /// # Warning
    /// This function will lock the `file_content` of the parent directory, may cause deadlock
    fn gen_dir_ent(
        parent_dir: &Arc<Self>,
        parent_inode_lock: &RwLockWriteGuard<InodeLock>,
        name: &String,
        fst_clus: u32,
        file_type: DiskInodeType,
    ) -> (FATShortDirEnt, Vec<FATLongDirEnt>) {
        // Generate name slices
        let (short_name_slice, long_name_slices) =
            Self::gen_name_slice(parent_dir, parent_inode_lock, &name);
        // Generate short entry
        let short_ent = FATShortDirEnt::from_name(short_name_slice, fst_clus, file_type);
        // Generate long entries
        let long_ent_num = long_name_slices.len();
        let long_ents = long_name_slices
            .iter()
            .enumerate()
            .map(|(i, slice)| FATLongDirEnt::from_name_slice(i + 1 == long_ent_num, i + 1, *slice))
            .collect();
        (short_ent, long_ents)
    }

    /// Create a file from directory entry.
    /// # Arguments
    /// + `parent_dir`: the parent directory inode pointer
    /// + `ent`: the short entry as the source of information
    /// + `offset`: the offset of the short directory entry in the `parent_dir`
    /// # Return Value
    /// Pointer to Inode
    pub fn from_ent(parent_dir: &Arc<Self>, ent: &FATShortDirEnt, offset: u32) -> Arc<Self> {
        Self::new(
            ent.get_first_clus(),
            if ent.is_dir() {
                DiskInodeType::Directory
            } else {
                DiskInodeType::File
            },
            if ent.is_file() {
                Some(ent.file_size)
            } else {
                None
            },
            Some((parent_dir.clone(), offset)),
            parent_dir.fs.clone(),
        )
    }

    /// Fill out an empty directory with only the '.' & '..' entries.
    /// # Arguments
    /// + `parent_dir`: the pointer of parent directory inode
    /// + `current_dir`: the pointer of new directory inode
    /// + `fst_clus`: the first cluster number of current file
    fn fill_empty_dir(parent_dir: &Arc<Self>, current_dir: &Arc<Self>, fst_clus: u32) {
        let current_inode_lock = current_dir.write();
        let mut iter = current_dir.dir_iter(&current_inode_lock, None, DirIterMode::Enum, FORWARD);
        let mut short_name: [u8; 11] = [' ' as u8; 11];
        //.
        iter.next();
        short_name[0] = '.' as u8;
        iter.write_to_current_ent(&FATDirEnt {
            short_entry: FATShortDirEnt::from_name(
                short_name,
                fst_clus as u32,
                DiskInodeType::Directory,
            ),
        });
        //..
        iter.next();
        short_name[1] = '.' as u8;
        iter.write_to_current_ent(&FATDirEnt {
            short_entry: FATShortDirEnt::from_name(
                short_name,
                parent_dir
                    .get_first_clus_lock(&parent_dir.file_content.read())
                    .unwrap(),
                DiskInodeType::Directory,
            ),
        });
        //add "unused and last" sign
        iter.next();
        iter.write_to_current_ent(&FATDirEnt::unused_and_last_entry());
    }
}
