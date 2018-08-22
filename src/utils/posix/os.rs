extern crate libc;

pub fn errno() -> i32 {
    unsafe {
        *libc::__errno_location()
    }
}
