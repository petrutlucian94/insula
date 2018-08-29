extern crate byteorder;
extern crate std;

pub mod defs;

use self::byteorder::{ByteOrder, LittleEndian};

use std::collections::HashMap;
use std::cmp::min;

use ::devices::bus::BusDevice;
pub use self::defs::*;

struct FWCfgEntry {
    len: u32,
    allow_write: bool,
    data: *const u8,
    // TODO: callbacks
}

// Once initialized, those entries will be read-only.
unsafe impl Send for FWCfgEntry {}

struct FWCfgFile {
    size: u32,
    select: u16,
    reserved: u16,
    name: [char; FW_CFG_MAX_FILE_PATH as usize]
    // name: String
}

struct FWCfgFiles {
    count: u32,
    f: Vec<FWCfgFile>  // TODO: we may need an actual array
}

pub struct FWCfgState {
    file_slots: u16,
    entries: [HashMap<u32, FWCfgEntry>; 2],
    files: Vec<FWCfgFiles>,
    cur_entry: u16,
    cur_offset: u32,
    dma_enabled: bool,
    // dma_addr_t dma_addr;
    // AddressSpace *dma_as;
    // MemoryRegion dma_iomem;
}

impl FWCfgState {
    pub fn new()-> Self {
        let mut obj = FWCfgState {
            file_slots: 0,
            entries: [HashMap::new(), HashMap::new()],
            files: Vec::new(),
            cur_entry: 0,
            cur_offset: 0,
            dma_enabled: false
        };

        obj.add_bytes(defs::FW_CFG_SIGNATURE, b"QEMU", 4, true);

        obj
    }

    fn max_entry(&self) -> usize {
        // todo: I think this may be removed.
        defs::FW_CFG_FILE_FIRST as usize + self.file_slots as usize
    }

    fn get_arch(key: usize) -> usize {
        ((key as u32 & defs::FW_CFG_ARCH_LOCAL) != 0) as usize
    }

    fn add_bytes(&mut self, key: u32, data: &[u8], len: u32, read_only: bool) {
        let arch = FWCfgState::get_arch(key as usize);

        let key = key & defs::FW_CFG_ENTRY_MASK;

        assert!((key as usize) < self.max_entry());
        // assert!(self.entries[arch][key].data == NULL); /* avoid key conflict */

        let mut entry = match self.entries[arch].get_mut(&key) {
            Some(ref existing_entry) => panic!("fw cfg key already exists: {}",
                                               key),
            _ => FWCfgEntry {
                    len: len,
                    data: data.as_ptr(),
                    allow_write: !read_only
                }
        };

        self.entries[arch].insert(key, entry);
        // self.entries[arch][&key].data = data.as_ptr();
        // self.entries[arch][&key].len = len;
        // // self.entries[arch][key].select_cb = select_cb;
        // // self.entries[arch][key].write_cb = write_cb;
        // // self.entries[arch][key].callback_opaque = callback_opaque;
        // self.entries[arch][&key].allow_write = !read_only;
    }

    fn select(&mut self, key: usize) -> bool {
        let mut ret = false;

        self.cur_offset = 0;
        if (key & FW_CFG_ENTRY_MASK as usize) >= self.max_entry() {
            self.cur_entry = FW_CFG_INVALID as u16;
        } else {
            self.cur_entry = key as u16;
            ret = true;
            /* entry successfully selected, now run callback if present */
            // arch = FWCfgState::get_arch(key);
            // e = s->entries[arch][key & FW_CFG_ENTRY_MASK];
            // if (e->select_cb) {
            //     e->select_cb(e->callback_opaque);
            // }
        }

        ret
    }

    fn get_cur_entry(&self, arch: usize) -> Option<&FWCfgEntry> {
        match self.cur_entry as u32 {
            FW_CFG_INVALID => None,
            _ => self.entries[arch].get(
                    &(self.cur_entry as u32 & FW_CFG_ENTRY_MASK))
        }
    }
}

impl BusDevice for FWCfgState {
    fn write(&mut self, _offset: u64, data: &[u8]) {
        match data.len() {
            1 => println!("Ignoring fw cfg write."),
            2 => {
                self.select(LittleEndian::read_u16(data) as usize);
                ()
            },
            _ => println!("Invalid fw cfg write length: {}", data.len())
        };
    }

    fn read(&mut self, _offset: u64, data: &mut [u8]) {
        let arch = FWCfgState::get_arch(self.cur_entry as usize);
        let mut value: u64 = 0;
        let mut read_len = data.len();
        // TODO: clean this up, currently avoiding assigning a
        // borrowed object.
        let mut cur_offset = self.cur_offset as usize;

        assert!(read_len <= 8);

        match self.get_cur_entry(arch) {
            Some(entry) if entry.len >= 0 && self.cur_offset < entry.len => {
                let entry_data = unsafe {
                    std::slice::from_raw_parts(
                        entry.data as *const _ as *const u8,
                        entry.len as usize
                    )
                };

                // Fill the buffer with data from the config entry,
                // starting with the current offset.
                let entry_read_len = min(
                    read_len, (entry.len - self.cur_offset) as usize);
                data[..entry_read_len].clone_from_slice(
                    &entry_data[cur_offset..cur_offset + read_len as usize]);
                if entry_read_len < read_len {
                    // Fill the rest with zeros.
                    for e in &mut data[read_len..read_len] {
                        *e = 0;
                    }
                }
            },
            _ => (),
        }

        self.cur_offset = cur_offset as u32;
    }
}

