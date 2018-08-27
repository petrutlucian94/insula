pub mod defs;

pub use self::defs::*;

struct FWCfgEntry {
    len: u32,
    allow_write: bool,
    data: *const u8,
    // TODO: callbacks
}

struct FWCfgFile {
    size: u32,
    select: u16,
    reserved: u16,
    name: [char; FW_CFG_MAX_FILE_PATH as usize]
}

struct FWCfgFiles {
    count: u32,
    f: *const FWCfgFile  // TODO: we may need an actual array
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
        if ((key & FW_CFG_ENTRY_MASK as usize) >= self.max_entry()) {
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
}
