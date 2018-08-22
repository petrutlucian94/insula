extern crate libc;
extern crate std;

use super::os::errno;

pub fn anon_ram_alloc(size: usize) -> *mut libc::c_void {
    // todo: handle flags, alignment
    let addr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_NORESERVE,
            -1,
            0)
    };

    if addr == libc::MAP_FAILED {
        panic!("mmapp failed with: {}", errno());
    }
    addr
}

pub fn anon_ram_free(address: *mut libc::c_void, size: usize) {
    let result = unsafe { libc::munmap(address, size) };

    if result != 0 {
        panic!("munmap failed with: {}", errno());
    }
}

pub fn madvise(address: *mut libc::c_void, size: usize, flags: i32) {
    let result = unsafe {
        libc::madvise(address, size, flags)
    };

    if result == -1 {
        panic!("madvise failed with: {}", errno());
    }
}
