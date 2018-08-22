use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;

use memory::MmapMemorySlot;

const MAX_FW_SIZE: u64 = 16 << 20;

pub struct Firmware {
    size: u64,
    path: String,
}

impl Firmware {
    pub fn new(path: &str) -> Self {
        let mut fw = Firmware {
            size: 0,
            path: path.to_string()
        };

        let mut f = fw.open();
        fw.size = f.seek(SeekFrom::End(0)).unwrap();

        fw
    }

    fn open(&self) -> File {
        let f = File::open(&self.path).expect(&format!(
            "Cannot find firmware image \"{}\".",
            self.path
        ));
        f
    }

    pub fn load(&mut self, mem: &mut MmapMemorySlot) {
        let mut f = self.open();

        if self.size > MAX_FW_SIZE {
            panic!("The firmware image is expected to be at most
                    16 mb large. Actual size: {}B.", self.size);
        }

        f.seek(SeekFrom::Start(0)).unwrap();
        f.read(mem.as_slice_mut()).unwrap();
    }

    pub fn get_size(&self) -> u64 {
        self.size
    }
}