use alloc::sync::Arc;
use lwext4_rust::{bindings::ext4_direntry, InodeTypes};

/// Filesystem operations.
pub trait VfsOps: Send + Sync {
    /// Do something when the filesystem is mounted.
    fn mount(&self, _path: &str, _mount_point: Arc<dyn VfsNodeOps>) -> Result<usize, i32> {
        Ok(0)
    }

    /// Do something when the filesystem is unmounted.
    fn umount(&self) -> Result<usize, i32> {
        Ok(0)
    }

    /// Format the filesystem.
    fn format(&self) -> Result<usize, i32> {
        unimplemented!()
    }

    /// Get the attributes of the filesystem.
    fn statfs(&self) -> Result<usize, i32> {
        unimplemented!()
    }

    /// Get the root directory of the filesystem.
    fn root_dir(&self) -> Arc<dyn VfsNodeOps>;
}

/// Node (file/directory) operations.
pub trait VfsNodeOps: Send + Sync {
    /// Do something when the node is opened.
    fn open(&self) -> Result<usize, i32> {
        Ok(0)
    }

    /// Do something when the node is closed.
    fn release(&self) -> Result<usize, i32> {
        Ok(0)
    }

    /// Get the attributes of the node.
    fn get_attr(&self) -> Result<usize, i32> {
        unimplemented!()
    }

    // file operations:

    /// Read data from the file at the given offset.
    fn read_at(&self, _offset: u64, _buf: &mut [u8]) -> Result<usize, i32> {
        unimplemented!()
    }

    /// Write data to the file at the given offset.
    fn write_at(&self, _offset: u64, _buf: &[u8]) -> Result<usize, i32> {
        unimplemented!()
    }

    /// Flush the file, synchronize the data to disk.
    fn fsync(&self) -> Result<usize, i32> {
        unimplemented!()
    }

    /// Truncate the file to the given size.
    fn truncate(&self, _size: u64) -> Result<usize, i32> {
        unimplemented!()
    }

    // directory operations:

    /// Get the parent directory of this directory.
    ///
    /// Return `None` if the node is a file.
    fn parent(&self) -> Option<Arc<dyn VfsNodeOps>> {
        None
    }

    /// Lookup the node with given `path` in the directory.
    ///
    /// Return the node if found.
    fn lookup(self: Arc<Self>, _path: &str) -> Result<usize, i32> {
        unimplemented!()
    }

    /// Create a new node with the given `path` in the directory
    ///
    /// Return [`Ok(())`](Ok) if it already exists.
    fn create(&self, _path: &str, _ty: InodeTypes) -> Result<usize, i32> {
        unimplemented!()
    }

    /// Remove the node with the given `path` in the directory.
    fn remove(&self, _path: &str) -> Result<usize, i32> {
        unimplemented!()
    }

    /// Read directory entries into `dirents`, starting from `start_idx`.
    fn read_dir(&self, _start_idx: usize, _dirents: &mut [ext4_direntry]) -> Result<usize, i32> {
        unimplemented!()
    }

    /// Renames or moves existing file or directory.
    fn rename(&self, _src_path: &str, _dst_path: &str) -> Result<usize, i32> {
        unimplemented!()
    }

    /// Convert `&self` to [`&dyn Any`][1] that can use
    /// [`Any::downcast_ref`][2].
    ///
    /// [1]: core::any::Any
    /// [2]: core::any::Any#method.downcast_ref
    fn as_any(&self) -> &dyn core::any::Any {
        unimplemented!()
    }
}
