extern crate byteorder;
extern crate std;

pub mod defs;

use self::byteorder::{ByteOrder, LittleEndian};

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
    entries: [Vec<FWCfgEntry>; 2],
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
        FWCfgState {
            file_slots: 0,
            entries: [Vec::new(), Vec::new()],
            files: Vec::new(),
            cur_entry: 0,
            cur_offset: 0,
            dma_enabled: false
        }
    }

    fn max_entry(&self) -> usize {
        defs::FW_CFG_FILE_FIRST as usize + self.file_slots as usize
    }

    fn get_arch(key: usize) -> usize {
        ((key as u32 & defs::FW_CFG_ARCH_LOCAL) != 0) as usize
    }

    fn add_bytes(&mut self, key: usize, data: &[u8], len: u32, read_only: bool) {
        let arch = FWCfgState::get_arch(key);

        let key = key & defs::FW_CFG_ENTRY_MASK as usize;

        assert!(key < self.max_entry());
        // assert!(self.entries[arch][key].data == NULL); /* avoid key conflict */

        self.entries[arch][key].data = data.as_ptr();
        self.entries[arch][key].len = len;
        // self.entries[arch][key].select_cb = select_cb;
        // self.entries[arch][key].write_cb = write_cb;
        // self.entries[arch][key].callback_opaque = callback_opaque;
        self.entries[arch][key].allow_write = !read_only;
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
            _ => Some(
                &self.entries[arch][
                    self.cur_entry as usize & FW_CFG_ENTRY_MASK as usize])
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
        // if data.len() == 1 && offset == 0 {
        //     data[0] = 0xe9;
        // }
        let arch = FWCfgState::get_arch(self.cur_entry as usize);
        let mut value: u64 = 0;
        let mut size = data.len();
        // TODO: clean this up, currently avoiding assigning a
        // borred object.
        let mut cur_offset = self.cur_offset;

        assert!(size > 0 && size <= 8);

        match self.get_cur_entry(arch) {
            Some(entry) if entry.len >= 0 && self.cur_offset < entry.len => {
                let entry_data = unsafe {
                    std::slice::from_raw_parts(
                        entry.data as *const _ as *const u8,
                        entry.len as usize
                    )
                };

                loop {
                    value = (value << 8) |
                        entry_data[cur_offset as usize] as u64;

                    cur_offset += 1;
                    size -= 1;
                    if size == 0 || self.cur_offset >= entry.len {
                        break;
                    }
                }
                // Fill the rest of the requested bytes with zeros.
                value <<= 8 * size;
            },
            _ => (),
        }

        self.cur_offset = cur_offset;
        // let data = std::slice::from_raw_parts(&data, 8);
        // data.write_u64::<LittleEndian>(value).unwrap();
        LittleEndian::write_u64(data, value);
    }
}

