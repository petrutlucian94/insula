use std::io::{self, Write};

use super::bus::BusDevice;

pub struct QemuDebugConsole {}

impl QemuDebugConsole {
    pub fn new() -> Self {
        QemuDebugConsole {}
    }
}

impl BusDevice for QemuDebugConsole {
    fn write(&mut self, _offset: u64, data: &[u8]) {
        io::stdout().write(data).unwrap();
    }

    fn read(&mut self, offset: u64, data: &mut [u8]) {
        if data.len() == 1 && offset == 0 {
            data[0] = 0xe9;
        }

        // let data_u64 = data as &mut u64;
        // data.write_u64(data, 0xe9);
        // data.write_u64::<LittleEndian>(0xe9).unwrap();
    }
}
