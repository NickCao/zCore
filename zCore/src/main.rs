#![cfg_attr(not(feature = "libos"), no_std)]
// #![deny(warnings)]
#![no_main]
#![feature(naked_functions, asm_sym, asm_const)]
#![feature(default_alloc_error_handler)]

use core::sync::atomic::{AtomicBool, Ordering};
// use linux_object::error::SysResult;
use linux_object::error::LxError;
use linux_object::net::{
    DistriComm,
};
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
async fn test_comm() {
    println!("test communication:"); 
    let comm = DistriComm::new();
    let result_connect = comm.connect().await;

    // non block
    let _result_set_flags = comm.set_nonblock();

    // block
    // let _result_set_flags = comm.set_block();
    
    println!("connect result <= {:?}", result_connect);
    if let Some(id) = comm.getid() {
        println!("my id is {}", id);
        if id & 1 == 0 {
            let mut source_id = 0;
            let mut recv_data = [0u8; 100];

            // non block poll mode
            loop {
                let recv_result = comm.recv(&mut source_id, &mut recv_data).await;
                match recv_result {
                    Ok(len) => {
                        println!("recv {} {}", source_id, String::from_utf8(recv_data[0..len].to_vec()).unwrap());
                        break;
                    }
                    Err(LxError::EAGAIN) => {
                        // poll for non block
                        // info!("poll for nonblock");
                    }
                    Err(e) => { 
                        println!("{:?}",e);
                        break;
                    }
                }
            }

            // block mode
            // if let Ok(len) = comm.recv(&mut source_id, &mut recv_data).await {
            //     println!("recv {} {}", source_id, String::from_utf8(recv_data[0..len].to_vec()).unwrap());
            // } else {
            //     println!("error");
            // }

            let send_data = "world";
            println!("send {} {}", id ^ 1, send_data);
            let result_send = comm.send(id ^ 1, send_data.as_bytes());
            warn!("send result<= {:?}", result_send);
            
        } else {
            let send_data = "hello";
            println!("send {} {}", id ^ 1, send_data);
            let result_send = comm.send(id ^ 1, send_data.as_bytes());
            warn!("send result<= {:?}", result_send);
            let mut source_id = 0;
            let mut recv_data = [0u8; 100];
            // non block poll mode
            loop {
                let recv_result = comm.recv(&mut source_id, &mut recv_data).await;
                match recv_result {
                    Ok(len) => {
                        println!("recv {} {}", source_id, String::from_utf8(recv_data[0..len].to_vec()).unwrap());
                        break;
                    }
                    Err(LxError::EAGAIN) => {
                        // poll for non block
                        // info!("poll for nonblock");
                }
                    Err(e) => { 
                        println!("{:?}",e);
                        break;
                    }
                }
            }

            // block mode
            // if let Ok(len) = comm.recv(&mut source_id, &mut recv_data).await {
            //     println!("recv {} {}", source_id, String::from_utf8(recv_data[0..len].to_vec()).unwrap());
            // } else {
            //     println!("no recv");
            // }
        }
    } else {
        println!("no id");
    }
    let disconnect_result = comm.disconnect();
    warn!("disconnect result <= {:?}", disconnect_result);
}


#[allow(dead_code)]
async fn test_connect() {
    println!("test connection:"); 
    let comm = DistriComm::new();
    let result_connect = comm.connect().await;
    println!("connect result <= {:?}", result_connect);

    // non block mode
    let _result_set_flags = comm.set_nonblock();

    // block mode
    // let _result_set_flags = comm.set_block();
    
    if let Some(id) = comm.getid() {
        println!("my id is {}", id);

        let send_data = "test message: hello".as_bytes();
        println!("send self {}", String::from_utf8(send_data.to_vec()).unwrap());

        let result_send = comm.send(id, send_data);
        println!("send result<= {:?}", result_send);
        
        let mut source_id = 0;
        let mut recv_data = [0u8; 100];
        // non block poll mode
        loop {
            let recv_result = comm.recv(&mut source_id, &mut recv_data).await;
            match recv_result {
                Ok(len) => {
                    println!("recv {} {}", source_id, String::from_utf8(recv_data[0..len].to_vec()).unwrap());
                    let mut flag: bool = len == send_data.len();
                    for i in 0..len {
                        if recv_data[i] != send_data[i] {
                            flag = false;
                        }
                    }
                    if flag {
                        println!("test connection successfully");
                    } else {
                        panic!("test connection dead");
                    }
                    break;
                }
                Err(LxError::EAGAIN) => {
                    // poll for non block
                    // info!("poll for nonblock");
                }
                Err(e) => { 
                    panic!("{:?}",e);
                }
            }
        }
        // block mode
        // if let Ok(len) = comm.recv(&mut source_id, &mut recv_data).await {
        //     println!("recv {} {}", source_id, String::from_utf8(recv_data[0..len].to_vec()).unwrap());
        //     flag = len == send_data.len();
        //     for i in 0..len {
        //         if recv_data[i] != send_data[i] {
        //             flag = false;
        //         }
        //     }
        //     if flag {
        //         println!("test connection successfully")
        //     } else {
        //         panic!("test connection dead");
        //     }
        // } else {
        //     println!("no recv");
        // }
    } else {
        panic!("no server give id");
    }
}

#[allow(dead_code)]
async fn test_bind() {
    // todo
}

fn primary_main(config: kernel_hal::KernelConfig) {
    logging::init();
    memory::init_heap();
    kernel_hal::primary_init_early(config, &handler::ZcoreKernelHandler);
    let options = utils::boot_options();
    info!("IP index: {}",options.ip_index);
    logging::set_max_level(&options.log_level);
    info!("Boot options: {:#?}", options);
    memory::init_frame_allocator(&kernel_hal::mem::free_pmem_regions());
    kernel_hal::primary_init(options.ip_index);
    STARTED.store(true, Ordering::SeqCst);
    if options.ip_index > 0 {
        // test connection
        executor::spawn(test_connect());
        // test communication
        // executor::spawn(test_comm());
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
