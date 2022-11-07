// Distributed Arch
// Use Tcpsocket

// crate
use crate::fs::{FileLike, OpenFlags};
use crate::net::*;
use alloc::collections::BTreeMap;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::convert::TryInto;
use lock::Mutex;
use rcore_fs_dfs::transport::Transport;

// alloc
use alloc::string::String;

// smoltcp
use smoltcp::wire::IpAddress;
use smoltcp::wire::IpEndpoint;

pub struct DistriTran {
    comm: DistriComm,
    store: Mutex<BTreeMap<u64, Vec<u8>>>,
}

impl DistriTran {
    pub async fn new() -> Self {
        let comm = DistriComm::new();
        comm.connect().await.unwrap();
        Self {
            comm,
            store: Default::default(),
        }
    }
}

#[allow(unused_variables)]
impl Transport for DistriTran {
    fn nid(&self) -> u64 {
        self.comm.getid().unwrap().try_into().unwrap()
    }
    fn len(&self) -> u64 {
        unimplemented!()
    }
    fn get(&self, nid: u64, bid: u64, buf: &mut [u8]) -> Result<usize, String> {
        if nid == self.nid() {
            if let Some(val) = self.store.lock().get(&bid) {
                buf[..val.len()].copy_from_slice(&val);
                return Ok(val.len());
            } else {
                return Err("bid not found".to_string());
            }
        }
        unimplemented!()
    }
    fn set(&self, nid: u64, bid: u64, buf: &[u8]) -> Result<(), String> {
        if nid == self.nid() {
            self.store.lock().insert(bid, buf.to_vec());
            return Ok(());
        }
        unimplemented!()
    }
    fn next(&self) -> u64 {
        unimplemented!()
    }
}

/// Distributed Communication
pub struct DistriComm {
    master_endpoint: Endpoint,
    /// DistriComm Inner
    inner: Mutex<DistriCommInner>,
}

/// Distributed Communication Inner
pub struct DistriCommInner {
    /// Distributed OS id
    id: Option<usize>,
    socket: TcpSocketState,
}

impl DistriComm {
    pub fn new() -> Self {
        let end_point = Endpoint::Ip(IpEndpoint::new(IpAddress::v4(10, 0, 2, 16), 1234));
        let socket = TcpSocketState::new();
        DistriComm {
            master_endpoint: end_point,
            inner: Mutex::new(DistriCommInner { id: None, socket }),
        }
    }

    pub fn set_block(&self) -> LxResult {
        let inner = self.inner.lock();
        inner.socket.set_flags(OpenFlags::from_bits_truncate(0))
    }

    pub fn set_nonblock(&self) -> LxResult {
        let inner = self.inner.lock();
        inner.socket.set_flags(OpenFlags::from_bits_truncate(
            SocketType::SOCK_NONBLOCK as usize & !SOCKET_TYPE_MASK,
        ))
    }

    pub fn getid(&self) -> Option<usize> {
        let inner = self.inner.lock();
        inner.id
    }

    pub async fn connect(&self) -> SysResult {
        let mut inner = self.inner.lock();
        inner.socket.connect(self.master_endpoint.clone()).await?;
        let mut data = [0u8; 10];
        if let Ok(len) = FileLike::read(&inner.socket, &mut data).await {
            let str = String::from_utf8(data[0..len].to_vec()).unwrap();
            info!("recv: {} {}", str, str.len());
            let id = str.parse::<usize>().unwrap();
            inner.id = Some(id);
            return Ok(id);
        } else {
            return Err(LxError::ENOTCONN);
        }
    }

    pub fn disconnect(&self) -> SysResult {
        let inner = self.inner.lock();

        // poll
        if let Ok(status) = FileLike::poll(&inner.socket, PollEvents::OUT) {
            if !status.write {
                return Err(LxError::EAGAIN);
            }
        } else {
            return Err(LxError::ENOBUFS);
        }

        // send quit
        let str = "quit";
        let result_write = FileLike::write(&inner.socket, str.as_bytes());
        info!("disconnect result <= {:?}", result_write);
        result_write
    }

    pub fn send(&self, dest_id: usize, data: &[u8]) -> SysResult {
        // max len of send data: 1024, change it if you want
        const MAX_LEN: usize = 1024;
        // check len of data
        if data.len() > MAX_LEN {
            return Err(LxError::ENOBUFS);
        }

        let inner = self.inner.lock();

        // poll
        if let Ok(status) = FileLike::poll(&inner.socket, PollEvents::OUT) {
            if !status.write {
                return Err(LxError::EAGAIN);
            }
        } else {
            return Err(LxError::ENOBUFS);
        }

        // assume dest_id < max_u32
        // dest_id to bytes
        let b1: u8 = (dest_id & 0xFF) as u8;
        let b2: u8 = ((dest_id >> 8) & 0xFF) as u8;
        let b3: u8 = ((dest_id >> 16) & 0xFF) as u8;
        let b4: u8 = ((dest_id >> 24) & 0xFF) as u8;

        // send data len to bytes
        let len: u32 = data.len() as u32;
        let lb1: u8 = (len & 0xFF) as u8;
        let lb2: u8 = ((len >> 8) & 0xFF) as u8;
        let lb3: u8 = ((len >> 16) & 0xFF) as u8;
        let lb4: u8 = ((len >> 24) & 0xFF) as u8;

        // make head
        let head = [b1, b2, b3, b4, lb1, lb2, lb3, lb4];

        // make package
        const SEND_LEN: usize = MAX_LEN + 8;

        let mut send_data = [0u8; SEND_LEN];
        for i in 0..8 {
            send_data[i] = head[i];
        }
        for i in 0..data.len() {
            send_data[i + 8] = data[i];
        }
        // package length
        let slice_len = data.len() + 8;
        // send package
        let result_write = FileLike::write(&inner.socket, &send_data[0..slice_len]);

        info!("send data result <= {:?}", result_write);
        // Ok(send successful len) or Err(LxError)
        result_write
    }

    pub async fn recv(&self, source_id: &mut usize, data: &mut [u8]) -> SysResult {
        let inner = self.inner.lock();

        let mut head = [0u8; 8];
        let len = FileLike::read(&inner.socket, &mut head).await?;
        if len < 8 {
            // panic head error
            return Err(LxError::ENOENT);
        }
        // get source id
        *source_id = head[3] as usize;
        *source_id = (*source_id << 8) | head[2] as usize;
        *source_id = (*source_id << 8) | head[1] as usize;
        *source_id = (*source_id << 8) | head[0] as usize;

        // get data len
        let mut data_len: usize = head[7] as usize;
        data_len = (data_len << 8) | head[6] as usize;
        data_len = (data_len << 8) | head[5] as usize;
        data_len = (data_len << 8) | head[4] as usize;

        // data not enough large, lost the package
        if data.len() < data_len {
            let mut trash = [0u8; 1024];
            while data_len != 0 {
                if data_len <= 1024 {
                    let len = FileLike::read(&inner.socket, &mut trash[0..data_len]).await?;
                    data_len -= len;
                } else {
                    let len = FileLike::read(&inner.socket, &mut trash).await?;
                    data_len -= len;
                }
            }
            return Err(LxError::ENOBUFS);
        }

        // read the package
        let ret_len = data_len;
        while data_len != 0 {
            let len = FileLike::read(&inner.socket, &mut data[0..data_len]).await?;
            data_len -= len;
        }

        // return read length
        return Ok(ret_len);
    }
}
