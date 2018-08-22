extern crate std;
extern crate libc;

use libkvm::mem::MemorySlot;

use ::utils::memory;

pub struct MmapMemorySlot {
    memory_size: usize,
    guest_address: u64,
    host_address: *mut libc::c_void,
    slot: u32,
    flags: u32,
}

impl MmapMemorySlot {
    pub fn new(memory_size: usize, guest_address: u64,
               slot: u32, flags: u32) -> MmapMemorySlot {
        let host_address = memory::vm_memory_alloc(memory_size);

        MmapMemorySlot {
            memory_size: memory_size,
            guest_address: guest_address,
            host_address,
            slot,
            flags,
        }
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.host_address as *mut u8, self.memory_size)
        }
    }
}

impl MemorySlot for MmapMemorySlot {
    fn slot_id(&self) -> u32 {
        self.slot
    }

    fn flags(&self) -> u32 {
        self.flags
    }

    fn memory_size(&self) -> usize {
        self.memory_size
    }

    fn guest_address(&self) -> u64 {
        self.guest_address
    }

    fn host_address(&self) -> u64 {
        self.host_address as u64
    }
}

impl Drop for MmapMemorySlot {
    fn drop(&mut self) {
        memory::anon_ram_free(self.host_address, self.memory_size);
    }
}
