#![allow(unused)]
use super::dir_iter::*;
use super::layout::{FATDirEnt, FATDiskInodeType, FATLongDirEnt, FATShortDirEnt};
use super::{BlockCacheManager, Cache, PageCache, PageCacheManager};
use super::{DiskInodeType, EasyFileSystem};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::convert::TryInto;
use core::ops::Mul;
use core::panic;
use spin::*;

pub struct FileContent {
    /// For FAT32, size is a value computed from FAT.
    /// You should iterate around the FAT32 to get the size.
    pub size: u32,
    /// The cluster list.
    pub clus_list: Vec<u32>,
    /// If this file is a directory, hint will record the position of last directory entry(the first byte is 0x00).
    pub hint: u32,
}

impl FileContent {
    /// Get file size
    /// # Return Value
    /// The file size
    #[inline(always)]
    pub fn get_file_size(&self) -> u32 {
        self.size
    }
}
macro_rules! div_ceil {
    ($mult:expr,$deno:expr) => {
        ($mult - 1 + $deno) / $deno
    };
}
pub struct InodeTime {
    pub create_time: u64,
    pub access_time: u64,
    pub modify_time: u64,
}
#[allow(unused)]
impl InodeTime {
    /// Set the inode time's create time.
    pub fn set_create_time(&mut self, create_time: u64) {
        self.create_time = create_time;
    }

    /// Get a reference to the inode time's create time.
    pub fn create_time(&self) -> &u64 {
        &self.create_time
    }

    /// Set the inode time's access time.
    pub fn set_access_time(&mut self, access_time: u64) {
        self.access_time = access_time;
    }

    /// Get a reference to the inode time's access time.
    pub fn access_time(&self) -> &u64 {
        &self.access_time
    }

    /// Set the inode time's modify time.
    pub fn set_modify_time(&mut self, modify_time: u64) {
        self.modify_time = modify_time;
    }

    /// Get a reference to the inode time's modify time.
    pub fn modify_time(&self) -> &u64 {
        &self.modify_time
    }
}

pub struct InodeLock;
/* *ClusLi was DiskInode*
 * Even old New York, was New Amsterdam...
 * Why they changed it I can't say.
 * People just like it better that way.*/
/// The functionality of ClusLi & Inode can be merged.
/// The struct for file information
pub struct Inode {
    /// Inode lock: for normal operation
    pub inode_lock: RwLock<InodeLock>,
    /// File Content
    pub file_content: RwLock<FileContent>,
    /// File cache manager corresponding to this inode.
    pub file_cache_mgr: PageCacheManager,
    /// File type
    pub file_type: Mutex<DiskInodeType>,
    /// The parent directory of this inode
    pub parent_dir: Mutex<Option<(Arc<Self>, u32)>>,
    /// file system
    pub fs: Arc<EasyFileSystem>,
    /// Struct to hold time related information
    pub time: Mutex<InodeTime>,
    /// Info Inode to delete file content
    pub deleted: Mutex<bool>,
}

impl Drop for Inode {
    /// Before deleting the inode, the file information should be written back to the parent directory
    fn drop(&mut self) {
        if *self.deleted.lock() {
            // Clear size
            let mut lock = self.file_content.write();
            let length = lock.clus_list.len();
            self.dealloc_clus(&mut lock, length);
        } else {
            if self.parent_dir.lock().is_none() {
                return;
            }
            let par_dir_lock = self.parent_dir.lock();
            let (parent_dir, offset) = par_dir_lock.as_ref().unwrap();

            let par_inode_lock = parent_dir.write();
            let dir_ent = parent_dir.get_dir_ent(&par_inode_lock, *offset).unwrap();
            let mut short_dir_ent = *dir_ent.get_short_ent().unwrap();
            // Modify size
            short_dir_ent.file_size = self.get_file_size();
            // Modify fst cluster
            short_dir_ent.set_fst_clus(
                self.get_first_clus_lock(&self.file_content.read())
                    .unwrap_or(0),
            );
            // Modify time
            // todo!
            log::debug!("[Inode drop]: new_ent: {:?}", short_dir_ent);
            // Write back
            parent_dir
                .set_dir_ent(&par_inode_lock, *offset, dir_ent)
                .unwrap();
        }
    }
}

/// Constructor
impl Inode {
    /// Constructor for Inodes
    /// # Arguments
    /// + `fst_clus`: The first cluster of the file
    /// + `file_type`: The type of the inode determined by the file
    /// + `size`: NOTE: the `size` field should be set to `None` for a directory
    /// + `parent_dir`: parent directory
    /// + `fs`: The pointer to the file system
    /// # Return Value
    /// Pointer to Inode
    pub fn new(
        fst_clus: u32,
        file_type: DiskInodeType,
        size: Option<u32>,
        parent_dir: Option<(Arc<Self>, u32)>,
        fs: Arc<EasyFileSystem>,
    ) -> Arc<Self> {
        let file_cache_mgr = PageCacheManager::new();
        let clus_list = match fst_clus {
            0 => Vec::new(),
            _ => fs.fat.get_all_clus_num(fst_clus, &fs.block_device),
        };

        let size = size.unwrap_or_else(|| clus_list.len() as u32 * fs.clus_size());
        let hint = 0;

        let file_content = RwLock::new(FileContent {
            size,
            clus_list,
            hint,
        });
        let parent_dir = Mutex::new(parent_dir);
        let time = InodeTime {
            create_time: 0,
            access_time: 0,
            modify_time: 0,
        };
        let inode = Arc::new(Inode {
            inode_lock: RwLock::new(InodeLock {}),
            file_content,
            file_cache_mgr,
            file_type: Mutex::new(file_type),
            parent_dir,
            fs,
            time: Mutex::new(time),
            deleted: Mutex::new(false),
        });

        // Init hint
        if file_type == DiskInodeType::Directory {
            inode.set_hint();
        }
        inode
    }
}

/// Basic Funtions
impl Inode {
    /// Get self's file content lock
    /// # Return Value
    /// a lock of file content
    #[inline(always)]
    pub fn read(&self) -> RwLockReadGuard<InodeLock> {
        self.inode_lock.read()
    }
    #[inline(always)]
    pub fn write(&self) -> RwLockWriteGuard<InodeLock> {
        self.inode_lock.write()
    }
    pub fn get_file_type_lock(&self) -> MutexGuard<DiskInodeType> {
        self.file_type.lock()
    }
    /// Get file type
    pub fn get_file_type(&self) -> DiskInodeType {
        *self.file_type.lock()
    }
    #[inline(always)]
    pub fn get_file_size_rlock(&self, _inode_lock: &RwLockReadGuard<InodeLock>) -> u32 {
        self.get_file_size()
    }
    pub fn get_file_size_wlock(&self, _inode_lock: &RwLockWriteGuard<InodeLock>) -> u32 {
        self.get_file_size()
    }
    #[inline(always)]
    pub fn get_file_size(&self) -> u32 {
        self.file_content.read().get_file_size()
    }
    /// Check if file type is directory
    /// # Return Value
    /// Bool result
    #[inline(always)]
    pub fn is_dir(&self) -> bool {
        self.get_file_type() == DiskInodeType::Directory
    }
    /// Check if file type is file
    /// # Return Value
    /// Bool result
    #[inline(always)]
    pub fn is_file(&self) -> bool {
        self.get_file_type() == DiskInodeType::File
    }
    /// Get first cluster of inode.
    /// # Arguments
    /// + `lock`: The lock of target file content
    /// # Return Value
    /// If cluster list isn't empty, it will return the first cluster list number.
    /// Otherwise it will return None.
    pub fn get_first_clus_lock(&self, lock: &RwLockReadGuard<FileContent>) -> Option<u32> {
        let clus_list = &lock.clus_list;
        if !clus_list.is_empty() {
            Some(clus_list[0])
        } else {
            None
        }
    }
    /// Get inode number of inode.
    /// For convenience, treat the first sector number as the inode number.
    /// # Arguments
    /// + `lock`: The lock of target file content
    /// # Return Value
    /// If cluster list isn't empty, it will return the first sector number.
    /// Otherwise it will return None.
    #[inline(always)]
    pub fn get_inode_num_lock(&self, lock: &RwLockReadGuard<FileContent>) -> Option<u32> {
        self.get_first_clus_lock(lock)
            .map(|clus| self.fs.first_sector_of_cluster(clus))
    }
    /// Get the number of clusters needed after rounding up according to size.
    /// # Return Value
    /// The number representing the number of clusters
    pub fn total_clus(&self, size: u32) -> u32 {
        //size.div_ceil(self.fs.clus_size())
        let clus_sz = self.fs.clus_size();
        div_ceil!(size, clus_sz)
        //(size - 1 + clus_sz) / clus_sz
    }
    /// Get first block id corresponding to the inner cache index
    /// # Arguments
    /// + `lock`: The lock of target file content
    /// + `inner_cache_id`: The index of inner cache
    /// # Return Value
    /// If `inner_cache_id` is valid, it will return the first block id
    /// Otherwise it will return None
    #[inline(always)]
    fn get_block_id(
        &self,
        lock: &RwLockReadGuard<FileContent>,
        inner_cache_id: u32,
    ) -> Option<u32> {
        let idx = inner_cache_id as usize / self.fs.sec_per_clus as usize;
        let clus_list = &lock.clus_list;
        if idx >= clus_list.len() {
            return None;
        }
        let base = self.fs.first_sector_of_cluster(clus_list[idx]);
        let offset = inner_cache_id % self.fs.sec_per_clus as u32;
        Some(base + offset)
    }
    /// Get a list of `block_id` represented by the given cache index.
    /// # Arguments
    /// + `clus_list`: The cluster list
    /// + `inner_cache_id`: Index of T's file caches (usually 4096 size per cache)
    /// # Return Value
    /// List of `block_id`
    pub fn get_neighboring_sec(&self, clus_list: &Vec<u32>, inner_cache_id: usize) -> Vec<usize> {
        let sec_per_clus = self.fs.sec_per_clus as usize;
        let byts_per_sec = self.fs.byts_per_sec as usize;
        let sec_per_cache = PageCacheManager::CACHE_SZ / byts_per_sec;
        let mut sec_id = inner_cache_id * sec_per_cache;
        let mut block_ids = Vec::with_capacity(sec_per_cache);
        for _ in 0..sec_per_cache {
            let cluster_id = sec_id / sec_per_clus;
            if cluster_id >= clus_list.len() {
                break;
            }
            let offset = sec_id % sec_per_clus;
            let start_block_id = self.fs.first_sector_of_cluster(clus_list[cluster_id]) as usize;
            block_ids.push(start_block_id + offset);
            sec_id += 1;
        }
        block_ids
    }
    /// Open the root directory
    /// # Arguments
    /// + `efs`: The pointer to inner file system
    /// # Return Value
    /// A pointer to Inode
    pub fn root_inode(efs: &Arc<EasyFileSystem>) -> Arc<Self> {
        let rt_clus = efs.root_clus;
        Self::new(
            rt_clus,
            DiskInodeType::Directory,
            None,
            None,
            Arc::clone(efs),
        )
    }
}
