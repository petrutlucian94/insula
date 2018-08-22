extern crate libc;

#[cfg(target_family = "unix")]
pub use ::utils::posix::memory::*;

#[cfg(target_family = "windows")]
pub use ::utils::win32::memory::*;

pub fn vm_memory_alloc(size: usize) -> *mut libc::c_void {
    let address = anon_ram_alloc(size);
    madvise(address, size, libc::MADV_MERGEABLE);

    address
}
