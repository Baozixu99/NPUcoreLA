use core::cell::RefCell;

use crate::alloc::string::String;
use crate::disk::Disk;
use crate::vfs_ops::{VfsNodeOps, VfsOps};
use alloc::sync::Arc;
use log::*;
use lwext4_rust::bindings::{
    O_APPEND, O_CREAT, O_RDONLY, O_RDWR, O_TRUNC, O_WRONLY, SEEK_CUR, SEEK_END, SEEK_SET,
};
use lwext4_rust::{Ext4BlockWrapper, Ext4File, InodeTypes, KernelDevOp};
use virtio_drivers::transport::Transport;
use virtio_drivers::Hal;

pub const BLOCK_SIZE: usize = 512;

#[allow(dead_code)]
pub struct Ext4FileSystem<H: Hal, T: Transport> {
    inner: Ext4BlockWrapper<Disk<H, T>>,
    root: Arc<dyn VfsNodeOps>,
}

unsafe impl<H: Hal, T: Transport> Sync for Ext4FileSystem<H, T> {}
unsafe impl<H: Hal, T: Transport> Send for Ext4FileSystem<H, T> {}

impl<H: Hal, T: Transport> Ext4FileSystem<H, T> {
    pub fn new(disk: Disk<H, T>) -> Self {
        info!(
            "Got Disk size:{}, position:{}",
            disk.size(),
            disk.position()
        );
        let inner = Ext4BlockWrapper::<Disk<H, T>>::new(disk)
            .expect("failed to initialize EXT4 filesystem");
        let root = Arc::new(FileWrapper::new("/", InodeTypes::EXT4_DE_DIR));
        Self { inner, root }
    }
}

/// The [`VfsOps`] trait provides operations on a filesystem.
impl<H: Hal, T: Transport> VfsOps for Ext4FileSystem<H, T> {
    fn mount(&self, _path: &str, _mount_point: Arc<dyn VfsNodeOps>) -> Result<usize, i32> {
        Ok(0)
    }

    fn root_dir(&self) -> Arc<dyn VfsNodeOps> {
        debug!("Get root_dir");
        //let root_dir = unsafe { (*self.root.get()).as_ref().unwrap() };
        Arc::clone(&self.root)
    }
}

pub struct FileWrapper(RefCell<Ext4File>);

unsafe impl Send for FileWrapper {}
unsafe impl Sync for FileWrapper {}

impl FileWrapper {
    pub fn new(path: &str, types: InodeTypes) -> Self {
        info!("FileWrapper new {:?} {}", types, path);
        //file.file_read_test("/test/test.txt", &mut buf);

        Self(RefCell::new(Ext4File::new(path, types)))
    }

    fn path_deal_with(&self, path: &str) -> String {
        if path.starts_with('/') {
            warn!("path_deal_with: {}", path);
        }
        let p = path.trim_matches('/'); // 首尾去除
        if p.is_empty() || p == "." {
            return String::new();
        }

        if let Some(rest) = p.strip_prefix("./") {
            //if starts with "./"
            return self.path_deal_with(rest);
        }
        let rest_p = p.replace("//", "/");
        if p != rest_p {
            return self.path_deal_with(&rest_p);
        }

        //Todo ? ../
        //注：lwext4创建文件必须提供文件path的绝对路径
        let file = self.0.borrow_mut();
        let path = file.get_path();
        let fpath = String::from(path.to_str().unwrap().trim_end_matches('/')) + "/" + p;
        info!("dealt with full path: {}", fpath.as_str());
        fpath
    }
}

/// The [`VfsNodeOps`] trait provides operations on a file or a directory.
impl VfsNodeOps for FileWrapper {

    /*
    fn get_attr(&self) -> Result<usize, i32> {
        let mut file = self.0.lock();

        let perm = file.file_mode_get().unwrap_or(0o755);
        let perm = VfsNodePerm::from_bits_truncate((perm as u16) & 0o777);

        let vtype = file.file_type_get();
        let vtype = match vtype {
            InodeTypes::EXT4_INODE_MODE_FIFO => VfsNodeType::Fifo,
            InodeTypes::EXT4_INODE_MODE_CHARDEV => VfsNodeType::CharDevice,
            InodeTypes::EXT4_INODE_MODE_DIRECTORY => VfsNodeType::Dir,
            InodeTypes::EXT4_INODE_MODE_BLOCKDEV => VfsNodeType::BlockDevice,
            InodeTypes::EXT4_INODE_MODE_FILE => VfsNodeType::File,
            InodeTypes::EXT4_INODE_MODE_SOFTLINK => VfsNodeType::SymLink,
            InodeTypes::EXT4_INODE_MODE_SOCKET => VfsNodeType::Socket,
            _ => {
                warn!("unknown file type: {:?}", vtype);
                VfsNodeType::File
            }
        };

        let size = if vtype == VfsNodeType::File {
            let path = file.get_path();
            let path = path.to_str().unwrap();
            file.file_open(path, O_RDONLY)
                .map_err(|e| <i32 as TryInto<AxError>>::try_into(e).unwrap())?;
            let fsize = file.file_size();
            let _ = file.file_close();
            fsize
        } else {
            0 // DIR size ?
        };
        let blocks = (size + (BLOCK_SIZE as u64 - 1)) / BLOCK_SIZE as u64;

        info!(
            "get_attr of {:?} {:?}, size: {}, blocks: {}",
            vtype,
            file.get_path(),
            size,
            blocks
        );

        Ok(VfsNodeAttr::new(perm, vtype, size, blocks))
    }
    */

    fn create(&self, path: &str, ty: InodeTypes) -> Result<usize, i32> {
        info!("create {:?} on Ext4fs: {}", ty, path);
        let fpath = self.path_deal_with(path);
        let fpath = fpath.as_str();
        if fpath.is_empty() {
            return Ok(0);
        }

        let types = ty;

        let mut file = self.0.borrow_mut();
        if file.check_inode_exist(fpath, types.clone()) {
            Ok(0)
        } else {
            if types == InodeTypes::EXT4_DE_DIR {
                file.dir_mk(fpath)
            } else {
                file.file_open(fpath, O_WRONLY | O_CREAT | O_TRUNC)
                    .expect("create file failed");
                file.file_close()
            }
        }
    }

    fn remove(&self, path: &str) -> Result<usize, i32> {
        info!("remove ext4fs: {}", path);
        let fpath = self.path_deal_with(path);
        let fpath = fpath.as_str();

        assert!(!fpath.is_empty()); // already check at `root.rs`

        let mut file = self.0.borrow_mut();
        if file.check_inode_exist(fpath, InodeTypes::EXT4_DE_DIR) {
            // Recursive directory remove
            file.dir_rm(fpath)
        } else {
            file.file_remove(fpath)
        }
    }

    /// Get the parent directory of this directory.
    /// Return `None` if the node is a file.
    fn parent(&self) -> Option<Arc<dyn VfsNodeOps>> {
        let file = self.0.borrow_mut();
        if file.get_type() == InodeTypes::EXT4_DE_DIR {
            let path = file.get_path();
            let path = path.to_str().unwrap();
            info!("Get the parent dir of {}", path);
            let path = path.trim_end_matches('/').trim_end_matches(|c| c != '/');
            if !path.is_empty() {
                return Some(Arc::new(Self::new(path, InodeTypes::EXT4_DE_DIR)));
            }
        }
        None
    }

    /*
    /// Read directory entries into `dirents`, starting from `start_idx`.
    fn read_dir(&self, start_idx: usize, dirents: &mut [VfsDirEntry]) -> Result<usize, i32> {
        let file = self.0.lock();
        let (name, inode_type) = file.lwext4_dir_entries().unwrap();

        let mut name_iter = name.iter().skip(start_idx);
        let mut inode_type_iter = inode_type.iter().skip(start_idx);

        for (i, out_entry) in dirents.iter_mut().enumerate() {
            let iname = name_iter.next();
            let itypes = inode_type_iter.next();

            match itypes {
                Some(t) => {
                    let ty = if *t == InodeTypes::EXT4_DE_DIR {
                        VfsNodeType::Dir
                    } else if *t == InodeTypes::EXT4_DE_REG_FILE {
                        VfsNodeType::File
                    } else if *t == InodeTypes::EXT4_DE_SYMLINK {
                        VfsNodeType::SymLink
                    } else {
                        error!("unknown file type: {:?}", itypes);
                        unreachable!()
                    };

                    *out_entry =
                        VfsDirEntry::new(core::str::from_utf8(iname.unwrap()).unwrap(), ty);
                }
                _ => return Ok(i),
            }
        }

        Ok(dirents.len())
    }

    /// Lookup the node with given `path` in the directory.
    /// Return the node if found.
    fn lookup(self: Arc<Self>, path: &str) -> Result<usize, i32> {
        info!("lookup ext4fs: {:?}, {}", self.0.get_path(), path);

        let fpath = self.path_deal_with(path);
        let fpath = fpath.as_str();
        if fpath.is_empty() {
            return Ok(self.clone());
        }

        /////////
        let mut file = self.0;
        if file.check_inode_exist(fpath, InodeTypes::EXT4_DE_DIR) {
            debug!("lookup new DIR FileWrapper");
            Ok(Arc::new(Self::new(fpath, InodeTypes::EXT4_DE_DIR)))
        } else if file.check_inode_exist(fpath, InodeTypes::EXT4_DE_REG_FILE) {
            debug!("lookup new FILE FileWrapper");
            Ok(Arc::new(Self::new(fpath, InodeTypes::EXT4_DE_REG_FILE)))
        } else {
            Err(VfsError::NotFound)
        }
    }
    */

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize, i32> {
        debug!("To read_at {}, buf len={}", offset, buf.len());
        let mut file = self.0.borrow_mut();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDONLY)?;

        file.file_seek(offset as i64, SEEK_SET)?;
        let r = file.file_read(buf);

        let _ = file.file_close();
        r
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> Result<usize, i32> {
        debug!("To write_at {}, buf len={}", offset, buf.len());
        let mut file = self.0.borrow_mut();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDWR)?;

        file.file_seek(offset as i64, SEEK_SET)?;
        let r = file.file_write(buf);

        let _ = file.file_close();
        r
    }

    fn truncate(&self, size: u64) -> Result<usize, i32> {
        info!("truncate file to size={}", size);
        let mut file = self.0.borrow_mut();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDWR | O_CREAT | O_TRUNC)?;

        let t = file.file_truncate(size);

        let _ = file.file_close();
        t
    }

    fn rename(&self, src_path: &str, dst_path: &str) -> Result<usize, i32> {
        info!("rename from {} to {}", src_path, dst_path);
        let mut file = self.0.borrow_mut();
        file.file_rename(src_path, dst_path)
    }
}

impl Drop for FileWrapper {
    fn drop(&mut self) {
        let mut file = self.0.borrow_mut();
        debug!("Drop struct FileWrapper {:?}", file.get_path());
        file.file_close().expect("failed to close fd");
        drop(file); // todo
    }
}
