// Copyright (C) 2018, Cloudbase Solutions Srl
//
// Licensed under LGPL version 2 or any later version.

extern crate clap;
extern crate libc;
extern crate libkvm;

mod accel;
mod args;
mod cpu;
mod devices;
mod ffi;
mod firmware;
mod memory;
mod utils;

use std::sync::{Arc, Mutex};

use args::parse_args;
use cpu::exits::VcpuExit;
use devices::bus::Bus;
use devices::{qdbg, fw_cfg, post_code};
use devices::fw_cfg::defs::*;
use memory::MmapMemorySlot;


fn main() {
    let args = parse_args();
    check_architecture();

    let fw_path = args.value_of("firmware").unwrap();
    let mut fw = firmware::Firmware::new(fw_path);

    let mut accelerator = accel::new();

    let mem_size = args.value_of("memory_mb")
                       .unwrap().parse::<usize>().unwrap() << 20;

    let fw_size = fw.get_size() as usize;

    let mem = MmapMemorySlot::new(mem_size, 0, 0, 0);
    let mut bios_mem = MmapMemorySlot::new(fw_size, 0xffe00000, 1, 2);

    accelerator.memory_region_add(&mem);
    accelerator.memory_region_add(&bios_mem);

    accelerator.init_vcpu();

    fw.load(&mut bios_mem);

    let mut io_bus = Bus::new();
    let mut mmio_bus = Bus::new();

    let qdbg_dev = qdbg::QemuDebugConsole::new();
    io_bus.insert(
        Arc::new(Mutex::new(qdbg_dev)),
        0x402,
        1,
        false).unwrap();

    let post_handler = post_code::PostCodeHandler::new();
    io_bus.insert(
        Arc::new(Mutex::new(post_handler)),
        0x80,
        1,
        false).unwrap();

    let mut fw_cfg_dev = fw_cfg::FWCfgDev::new();
    fw_cfg_dev.add_i16(FW_CFG_NB_CPUS, 1);
    fw_cfg_dev.add_i16(FW_CFG_MAX_CPUS, 1);
    fw_cfg_dev.add_i64(FW_CFG_RAM_SIZE, mem_size as i64);

    io_bus.insert(
        Arc::new(Mutex::new(fw_cfg_dev)),
        0x510,
        8,
        false).unwrap();

    loop {
        let vm_exit = accelerator.vcpu_run(0);
        // todo: handle the exits and move this somewhere else.
        match vm_exit {
            VcpuExit::IoIn(port, data) => io_bus.read(port.into(), data),
            VcpuExit::IoOut(port, data) => io_bus.write(port.into(), data),
            VcpuExit::MmioRead(addr, data) => mmio_bus.read(addr, data),
            VcpuExit::MmioWrite(addr, data) => mmio_bus.write(addr, data),
            VcpuExit::Hlt => { println!("vcpu halt."); break } ,
            VcpuExit::Shutdown => { println!("vcpu shutdown exit."); break },
            _ => panic!("Unsupported exit.")
        };
    }
}

fn check_architecture() {
    #[cfg(not(target_arch = "x86_64"))]
    {
        panic!("Unsupported architecture");
    }
}
