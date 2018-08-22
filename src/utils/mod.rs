#[cfg(target_family = "unix")]
mod posix;
#[cfg(target_family = "windows")]
mod win32;

pub mod memory;
