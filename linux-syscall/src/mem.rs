use super::*;
use bitflags::bitflags;
use zircon_object::vm::*;

impl Syscall {
    pub fn sys_mmap(
        &self,
        mut addr: usize,
        len: usize,
        prot: usize,
        flags: usize,
        fd: FileDesc,
        offset: usize,
    ) -> SysResult<usize> {
        let prot = MmapProt::from_bits_truncate(prot);
        let flags = MmapFlags::from_bits_truncate(flags);
        info!(
            "mmap: addr={:#x}, size={:#x}, prot={:?}, flags={:?}, fd={}, offset={:#x}",
            addr, len, prot, flags, fd, offset
        );

        let proc = &self.thread.proc;
        let vmar = proc.vmar();

        if addr == 0 {
            // although NULL can be a valid address
            // but in C, NULL is regarded as allocation failure
            // so just skip it
            addr = PAGE_SIZE;
        }
        let addr = if flags.contains(MmapFlags::FIXED) {
            Some(addr)
        } else {
            None
        };
        if flags.contains(MmapFlags::ANONYMOUS) {
            if flags.contains(MmapFlags::SHARED) {
                return Err(SysError::EINVAL);
            }
            let vmo = VMObjectPaged::new(pages(len));
            let addr = vmar.map(addr, vmo.clone(), 0, vmo.len(), prot.to_flags())?;
            Ok(addr)
        } else {
            unimplemented!()
        }
    }

    pub fn sys_mprotect(&self, addr: usize, len: usize, prot: usize) -> SysResult<usize> {
        let prot = MmapProt::from_bits_truncate(prot);
        info!(
            "mprotect: addr={:#x}, size={:#x}, prot={:?}",
            addr, len, prot
        );
        unimplemented!()
    }

    pub fn sys_munmap(&self, addr: usize, len: usize) -> SysResult<usize> {
        info!("munmap: addr={:#x}, size={:#x}", addr, len);
        let proc = &self.thread.proc;
        let vmar = proc.vmar();
        vmar.unmap(addr, len)?;
        Ok(0)
    }
}

bitflags! {
    pub struct MmapFlags: usize {
        #[allow(clippy::identity_op)]
        /// Changes are shared.
        const SHARED = 1 << 0;
        /// Changes are private.
        const PRIVATE = 1 << 1;
        /// Place the mapping at the exact address
        const FIXED = 1 << 4;
        /// The mapping is not backed by any file. (non-POSIX)
        const ANONYMOUS = MMAP_ANONYMOUS;
    }
}

#[cfg(target_arch = "mips")]
const MMAP_ANONYMOUS: usize = 0x800;
#[cfg(not(target_arch = "mips"))]
const MMAP_ANONYMOUS: usize = 1 << 5;

bitflags! {
    pub struct MmapProt: usize {
        #[allow(clippy::identity_op)]
        /// Data can be read
        const READ = 1 << 0;
        /// Data can be written
        const WRITE = 1 << 1;
        /// Data can be executed
        const EXEC = 1 << 2;
    }
}

impl MmapProt {
    fn to_flags(self) -> MMUFlags {
        let mut flags = MMUFlags::empty();
        if self.contains(MmapProt::READ) {
            flags |= MMUFlags::READ;
        }
        if self.contains(MmapProt::WRITE) {
            flags |= MMUFlags::WRITE;
        }
        if self.contains(MmapProt::EXEC) {
            flags |= MMUFlags::EXECUTE;
        }
        flags
    }
}
