//! Implement INode for ShadowINode

use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::any::Any;

use lock::Mutex;
use rcore_fs::vfs::*;

pub trait ShadowRPC: Sync + Send {
    // query node number
    fn id(&self) -> Result<usize>;
    // register inode as local
    fn register(&self, inode: usize) -> Result<()>;
    // locate a remote inode
    fn locate(&self, inode: usize) -> Result<Option<usize>>;
}

#[derive(Default)]
pub struct ShadowRPCMock {
    mapping: Mutex<BTreeMap<usize, usize>>,
}

impl ShadowRPC for ShadowRPCMock {
    fn id(&self) -> Result<usize> {
        Ok(1)
    }
    fn register(&self, inode: usize) -> Result<()> {
        self.mapping.lock().insert(inode, 1);
        Ok(())
    }
    fn locate(&self, inode: usize) -> Result<Option<usize>> {
        Ok(self.mapping.lock().get(&inode).map(|i| *i))
    }
}

#[allow(dead_code)]
pub struct ShadowFS {
    store: Arc<dyn FileSystem>,
    master: Arc<dyn ShadowRPC>,
}

impl ShadowFS {
    pub fn new(store: Arc<dyn FileSystem>, master: Arc<dyn ShadowRPC>) -> Arc<Self> {
        Arc::new(Self { store, master })
    }
}

impl FileSystem for ShadowFS {
    fn sync(&self) -> Result<()> {
        // TODO: also sync remote stores
        self.store.sync()
    }
    fn info(&self) -> FsInfo {
        self.store.info()
    }
    fn root_inode(&self) -> Arc<dyn INode> {
        // TODO: fetchfroot inode from master
        Arc::new(ShadowINode::new(self.store.root_inode()))
    }
}

/// shadow INode data struct
pub struct ShadowINodeData {
    inner: Arc<dyn INode>,
}

/// shadow INode struct
#[derive(Clone)]
pub struct ShadowINode {
    data: Arc<Mutex<ShadowINodeData>>,
}

impl ShadowINode {
    /// create a shadow INode
    pub fn new(inner: Arc<dyn INode>) -> ShadowINode {
        ShadowINode {
            data: Arc::new(Mutex::new(ShadowINodeData { inner })),
        }
    }
}

impl INode for ShadowINode {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize> {
        self.data.lock().inner.read_at(offset, buf)
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> Result<usize> {
        self.data.lock().inner.write_at(offset, buf)
    }

    fn poll(&self) -> Result<PollStatus> {
        self.data.lock().inner.poll()
    }

    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn metadata(&self) -> Result<Metadata> {
        self.data.lock().inner.metadata()
    }

    fn set_metadata(&self, metadata: &Metadata) -> Result<()> {
        self.data.lock().inner.set_metadata(metadata)
    }

    fn sync_all(&self) -> Result<()> {
        self.data.lock().inner.sync_all()
    }

    fn sync_data(&self) -> Result<()> {
        self.data.lock().inner.sync_data()
    }

    fn resize(&self, len: usize) -> Result<()> {
        self.data.lock().inner.resize(len)
    }

    fn create(&self, name: &str, type_: FileType, mode: u32) -> Result<Arc<dyn INode>> {
        match self.data.lock().inner.create(name, type_, mode) {
            Ok(i) => Ok(Arc::new(ShadowINode::new(i))),
            e => e,
        }
    }

    fn create2(
        &self,
        name: &str,
        type_: FileType,
        mode: u32,
        data: usize,
    ) -> Result<Arc<dyn INode>> {
        match self.data.lock().inner.create2(name, type_, mode, data) {
            Ok(i) => Ok(Arc::new(ShadowINode::new(i))),
            e => e,
        }
    }

    fn link(&self, name: &str, other: &Arc<dyn INode>) -> Result<()> {
        self.data.lock().inner.link(name, other)
    }

    fn unlink(&self, name: &str) -> Result<()> {
        self.data.lock().inner.unlink(name)
    }

    fn move_(&self, old_name: &str, target: &Arc<dyn INode>, new_name: &str) -> Result<()> {
        self.data.lock().inner.move_(old_name, target, new_name)
    }

    fn find(&self, name: &str) -> Result<Arc<dyn INode>> {
        match self.data.lock().inner.find(name) {
            Ok(i) => Ok(Arc::new(ShadowINode::new(i))),
            e => e,
        }
    }

    fn get_entry(&self, id: usize) -> Result<alloc::string::String> {
        self.data.lock().inner.get_entry(id)
    }

    fn get_entry_with_metadata(&self, id: usize) -> Result<(Metadata, alloc::string::String)> {
        self.data.lock().inner.get_entry_with_metadata(id)
    }

    fn io_control(&self, cmd: u32, data: usize) -> Result<usize> {
        self.data.lock().inner.io_control(cmd, data)
    }

    fn mmap(&self, area: MMapArea) -> Result<()> {
        self.data.lock().inner.mmap(area)
    }

    fn fs(&self) -> Arc<dyn FileSystem> {
        self.data.lock().inner.fs()
    }
}
