#![allow(unused)]

pub const FW_CFG_SIGNATURE: u32 = 0x00;
pub const FW_CFG_ID: u32 = 0x01;
pub const FW_CFG_UUID: u32 = 0x02;
pub const FW_CFG_RAM_SIZE: u32 = 0x03;
pub const FW_CFG_NOGRAPHIC: u32 = 0x04;
pub const FW_CFG_NB_CPUS: u32 = 0x05;
pub const FW_CFG_MACHINE_ID: u32 = 0x06;
pub const FW_CFG_KERNEL_ADDR: u32 = 0x07;
pub const FW_CFG_KERNEL_SIZE: u32 = 0x08;
pub const FW_CFG_KERNEL_CMDLINE: u32 = 0x09;
pub const FW_CFG_INITRD_ADDR: u32 = 0x0a;
pub const FW_CFG_INITRD_SIZE: u32 = 0x0b;
pub const FW_CFG_BOOT_DEVICE: u32 = 0x0c;
pub const FW_CFG_NUMA: u32 = 0x0d;
pub const FW_CFG_BOOT_MENU: u32 = 0x0e;
pub const FW_CFG_MAX_CPUS: u32 = 0x0f;
pub const FW_CFG_KERNEL_ENTRY: u32 = 0x10;
pub const FW_CFG_KERNEL_DATA: u32 = 0x11;
pub const FW_CFG_INITRD_DATA: u32 = 0x12;
pub const FW_CFG_CMDLINE_ADDR: u32 = 0x13;
pub const FW_CFG_CMDLINE_SIZE: u32 = 0x14;
pub const FW_CFG_CMDLINE_DATA: u32 = 0x15;
pub const FW_CFG_SETUP_ADDR: u32 = 0x16;
pub const FW_CFG_SETUP_SIZE: u32 = 0x17;
pub const FW_CFG_SETUP_DATA: u32 = 0x18;
pub const FW_CFG_FILE_DIR: u32 = 0x19;

pub const FW_CFG_FILE_FIRST: u32 = 0x20;
pub const FW_CFG_FILE_SLOTS_MIN: u32 = 0x10;

pub const FW_CFG_WRITE_CHANNEL: u32 = 0x4000;
pub const FW_CFG_ARCH_LOCAL: u32 = 0x8000;
pub const FW_CFG_ENTRY_MASK: u32 = (!(FW_CFG_WRITE_CHANNEL | FW_CFG_ARCH_LOCAL));

pub const FW_CFG_INVALID: u32 = 0xffff;

/* width in bytes of fw_cfg control register */
pub const FW_CFG_CTL_SIZE: u32 = 0x02;

pub const FW_CFG_MAX_FILE_PATH: u32 = 56;

pub const FW_CFG_FILE_SLOTS_DFLT: u32 = 0x20;
