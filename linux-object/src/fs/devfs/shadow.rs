//! Implement INode for ShadowINode

use alloc::sync::Arc;
use core::any::Any;

use lock::Mutex;
use rcore_fs::vfs::*;
use rcore_fs_devfs::DevFS;

/// shadow INode data struct
pub struct ShadowINodeData {}

/// random INode struct
#[derive(Clone)]
#[allow(dead_code)]
pub struct ShadowINode {
    inode_id: usize,
    data: Arc<Mutex<ShadowINodeData>>,
}

#[allow(dead_code)]
impl ShadowINode {
    /// create a shadow INode
    pub fn new() -> ShadowINode {
        ShadowINode {
            inode_id: DevFS::new_inode_id(),
            data: Arc::new(Mutex::new(ShadowINodeData {})),
        }
    }
}

impl INode for ShadowINode {
    fn read_at(&self, _offset: usize, buf: &mut [u8]) -> Result<usize> {
        Ok(buf.len())
    }

    fn write_at(&self, _offset: usize, buf: &[u8]) -> Result<usize> {
        Ok(buf.len())
    }

    fn poll(&self) -> Result<PollStatus> {
        Ok(PollStatus {
            read: true,
            write: true,
            error: false,
        })
    }

    fn metadata(&self) -> Result<Metadata> {
        Ok(Metadata {
            dev: 1,
            inode: self.inode_id,
            size: 0,
            blk_size: 0,
            blocks: 0,
            atime: Timespec { sec: 0, nsec: 0 },
            mtime: Timespec { sec: 0, nsec: 0 },
            ctime: Timespec { sec: 0, nsec: 0 },
            type_: FileType::CharDevice,
            mode: 0o666,
            nlinks: 1,
            uid: 0,
            gid: 0,
            rdev: make_rdev(1, 1),
        })
    }

    fn as_any_ref(&self) -> &dyn Any {
        self
    }
}
