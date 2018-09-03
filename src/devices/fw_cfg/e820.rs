extern crate std;

use std::mem::size_of;

use ::ffi::defs::*;

pub const E820_NR_ENTRIES: usize = 16;

pub const E820_RAM: u32 = 1;
pub const E820_RESERVED: u32 = 2;
pub const E820_ACPI: u32 = 3;
pub const E820_NVS4: u32 = 4;
pub const E820_UNUSABLE: u32 = 5;

#[repr(C, packed)]
#[derive(Debug, Default, Copy)]
pub struct E820Entry {
    addr: __u64,
    size: __u64,
    type_: __u32,
}

impl Clone for E820Entry {
    fn clone(&self) -> Self {
        *self
    }
}


pub struct E820Table {
    pub count: usize,
    pub entries: [E820Entry; E820_NR_ENTRIES],
}

impl Default for E820Table{
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

impl E820Table {
    pub fn new() -> Self {
        E820Table {..Default::default()}
    }
}

impl E820Table {
    pub fn add_entry(&mut self, addr: u64, size: u64, type_: u32) {
        assert!(self.count < E820_NR_ENTRIES);

        let mut entry = &mut self.entries[self.count];

        entry.addr = addr;
        entry.size = size;
        entry.type_ = type_;

        self.count += 1;
    }
    pub fn to_slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.entries.as_ptr() as *const u8,
                size_of::<E820Entry>() * self.count)
        }
    }
}
