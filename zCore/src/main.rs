#![cfg_attr(not(feature = "libos"), no_std)]
// #![deny(warnings)]
#![no_main]
#![feature(naked_functions, asm_sym, asm_const)]
#![feature(default_alloc_error_handler)]

use core::sync::atomic::{AtomicBool, Ordering};
use linux_object::net::DistriTran;
use rcore_fs_dfs::transport::Transport;

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

async fn test_comm() {
    println!("Communication self-test:");
    let trans = DistriTran::new().await;
    println!("Node ID: {}", trans.nid());
    // test set self
    trans.set(trans.nid(), trans.nid(), b"foo").unwrap();
    if trans.nid() != 0 {
        // test set other
        trans.set(0, trans.nid(), b"bar").unwrap();
        trans.set(0, trans.nid() + 1, b"bar").unwrap();
        trans.set(0, trans.nid() + 2, b"bar").unwrap();
    }
    // test get self
    let mut buf = alloc::vec![0u8; 4096];
    let len = trans.get(trans.nid(), trans.nid(), &mut buf).unwrap();
    assert_eq!(b"foo", &buf[..len]);
    if trans.nid() != 0 {
        // test get other
        let len = trans.get(0, trans.nid(), &mut buf).unwrap();
        assert_eq!(b"bar", &buf[..len]);
    }
}

fn primary_main(config: kernel_hal::KernelConfig) {
    logging::init();
    memory::init_heap();
    kernel_hal::primary_init_early(config, &handler::ZcoreKernelHandler);
    let options = utils::boot_options();
    info!("IP index: {}", options.ip_index);
    logging::set_max_level(&options.log_level);
    info!("Boot options: {:#?}", options);
    memory::init_frame_allocator(&kernel_hal::mem::free_pmem_regions());
    kernel_hal::primary_init(options.ip_index);
    STARTED.store(true, Ordering::SeqCst);
    if options.ip_index > 0 {
        kernel_hal::thread::spawn(test_comm());
    }
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
