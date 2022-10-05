//! Implement INode for ShadowINode

use alloc::sync::Arc;
use core::any::Any;

use lock::Mutex;
use rcore_fs::vfs::*;

/// shadow INode data struct
pub struct ShadowINodeData {
    inner: Arc<dyn INode>,
}

/// random INode struct
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
        self.data.lock().inner.create(name, type_, mode)
    }

    fn create2(
        &self,
        name: &str,
        type_: FileType,
        mode: u32,
        data: usize,
    ) -> Result<Arc<dyn INode>> {
        self.data.lock().inner.create2(name, type_, mode, data)
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
        self.data.lock().inner.find(name)
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
