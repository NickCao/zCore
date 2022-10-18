#![cfg_attr(not(feature = "libos"), no_std)]
#![deny(warnings)]
#![no_main]
#![feature(naked_functions, asm_sym, asm_const)]
#![feature(default_alloc_error_handler)]

use core::sync::atomic::{AtomicBool, Ordering};
// use linux_object::error::SysResult;
use linux_object::{
    net::*,
};
use linux_object::fs::{FileLike, OpenFlags};
use smoltcp::wire::IpEndpoint;
use smoltcp::wire::IpAddress;
// use alloc::sync::Arc;
use alloc::string::String;

extern crate alloc;
#[macro_use]
extern crate log;
#[macro_use]
extern crate cfg_if;

#[macro_use]
mod logging;

#[cfg(not(feature = "libos"))]
mod lang;

mod fs;
mod handler;
mod memory;
mod platform;
mod utils;

static STARTED: AtomicBool = AtomicBool::new(false);

#[cfg(all(not(any(feature = "libos")), feature = "mock-disk"))]
static MOCK_CORE: AtomicBool = AtomicBool::new(false);

#[allow(dead_code)]
async fn test_connect() {
    println!("test connect:"); 
    let socket = TcpSocketState::new();
    let _result_flags = socket.set_flags(OpenFlags::from_bits_truncate(SocketType::SOCK_NONBLOCK as usize & ! SOCKET_TYPE_MASK));
    println!("done set_flags"); 
    let endpoint = Endpoint::Ip(IpEndpoint::new(
        IpAddress::v4(10,0,2,100),
        1234,
    ));
    println!("done ip set"); 
    let _result_connect = socket.connect(endpoint).await;
    let str = "hello world";
    let result_write = FileLike::write(&socket, str.as_bytes());
    println!("done write"); 
    let mut data = [0u8; 32];
    info!("<= {:?}", result_write);
    let _result_read = FileLike::read(&socket, &mut data).await;
    println!("{}", String::from_utf8(data.to_vec()).unwrap());
}

#[allow(dead_code)]
async fn test_bind() {
    println!("test bind:"); 
    let socket = TcpSocketState::new();
    let _result_flags = socket.set_flags(OpenFlags::from_bits_truncate(SocketType::SOCK_NONBLOCK as usize & ! SOCKET_TYPE_MASK));
    println!("done set_flags"); 
    let endpoint = Endpoint::Ip(IpEndpoint::new(
        IpAddress::Unspecified,
        1234,
    ));
    println!("done ip set"); 
    let _result_bind = socket.bind(endpoint);
    println!("done bind");
    let _result_listen = socket.listen();
    println!("done listen");
    let (re_ac, endpoint) = if let Ok((re_ac, endpoint)) = socket.accept().await {
        (re_ac,endpoint)
    } else {
        panic!("not OK");
    };
    println!("{:?}", endpoint);
    let mut data = [0u8; 32];
    let read_file = re_ac.clone();
    let _result_read = FileLike::read(& *read_file, &mut data).await;
    println!("{}", String::from_utf8(data.to_vec()).unwrap());
}

fn primary_main(config: kernel_hal::KernelConfig) {
    logging::init();
    memory::init_heap();
    kernel_hal::primary_init_early(config, &handler::ZcoreKernelHandler);
    let options = utils::boot_options();
    logging::set_max_level(&options.log_level);
    info!("Boot options: {:#?}", options);
    memory::init_frame_allocator(&kernel_hal::mem::free_pmem_regions());
    kernel_hal::primary_init();
    STARTED.store(true, Ordering::SeqCst);
    // executor::spawn(test_bind());
    cfg_if! {
        if #[cfg(all(feature = "linux", feature = "zircon"))] {
            panic!("Feature `linux` and `zircon` cannot be enabled at the same time!");
        } else if #[cfg(feature = "linux")] {
            let args = options.root_proc.split('?').map(Into::into).collect(); // parse "arg0?arg1?arg2"
            let envs = alloc::vec!["PATH=/usr/sbin:/usr/bin:/sbin:/bin".into()];
            let rootfs = fs::rootfs();
            let proc = zcore_loader::linux::run(args, envs, rootfs);
            utils::wait_for_exit(Some(proc))
        } else if #[cfg(feature = "zircon")] {
            let zbi = fs::zbi();
            let proc = zcore_loader::zircon::run_userboot(zbi, &options.cmdline);
            utils::wait_for_exit(Some(proc))
        } else {
            panic!("One of the features `linux` or `zircon` must be specified!");
        }
    }
}

#[cfg(not(any(feature = "libos", target_arch = "aarch64")))]
fn secondary_main() -> ! {
    while !STARTED.load(Ordering::SeqCst) {
        core::hint::spin_loop();
    }
    kernel_hal::secondary_init();
    info!("hart{} inited", kernel_hal::cpu::cpu_id());
    #[cfg(feature = "mock-disk")]
    {
        if MOCK_CORE
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            utils::mock_disk();
        }
    }
    utils::wait_for_exit(None)
}
