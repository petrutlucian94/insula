extern crate byteorder;
extern crate std;

pub mod defs;

use self::byteorder::{ByteOrder, LittleEndian, BigEndian};

use std::collections::HashMap;
use std::cmp::min;
use std::mem::size_of;

use ::devices::bus::BusDevice;
pub use self::defs::*;
use ::ffi::*;

struct FWCfgEntry {
    allow_write: bool,
    data: Vec<u8>,
    // TODO: callbacks
}

// Once initialized, those entries will be read-only.
unsafe impl Send for FWCfgEntry {}
unsafe impl Send for FWCfgFilesWrapper {}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FWCfgFile {
    pub size: u32, // big-endian
    pub select: u16, // big-endian
    pub reserved: u16,
    pub name: [::std::os::raw::c_char; FW_CFG_MAX_FILE_PATH as usize],
}

#[repr(C)]
pub struct FWCfgFiles {
    pub count: u32, /* number of entries, in big-endian format */
    pub f: __IncompleteArrayField<FWCfgFile>,
}

pub struct FWCfgFilesWrapper {
    buf: Vec<u8>,
    files: *mut FWCfgFiles,
    slots: u32
}

impl FWCfgFilesWrapper {
    pub fn new(slots: u32) -> Self {
        let size = size_of::<FWCfgFiles>() +
                   size_of::<u32>() * slots as usize;
        let buf: Vec<u8> = vec![0; size];
        let files: &mut FWCfgFiles = unsafe {
            &mut *(buf.as_ptr() as *mut FWCfgFiles)
        };

        FWCfgFilesWrapper {
            buf: buf,
            files: files,
            slots: slots
        }
    }

    pub fn to_files_vec(&self) -> Vec<FWCfgFile> {
        unsafe {
            (*self.files)
                .f
                .as_slice(self.slots as usize)
        }.to_vec()
    }

    pub fn as_mut_ptr(&mut self) -> *mut FWCfgFiles {
        self.buf.as_mut_ptr() as *mut FWCfgFiles
    }
}

pub struct FWCfgState {
    file_slots: u16,
    entries: [HashMap<u32, FWCfgEntry>; 2],
    files_wrapper: FWCfgFilesWrapper,
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
            file_slots: FW_CFG_FILE_SLOTS_DFLT as u16,
            entries: [HashMap::new(), HashMap::new()],
            files_wrapper: FWCfgFilesWrapper::new(FW_CFG_FILE_SLOTS_DFLT),
            cur_entry: 0,
            cur_offset: 0,
            dma_enabled: false
        };

        obj.add_bytes(FW_CFG_SIGNATURE, b"QEMU", 4, true);
        obj.add_file("test", &[1; 5], 5);

        obj
    }

    fn max_entry(&self) -> usize {
        // todo: I think this may be removed.
        FW_CFG_FILE_FIRST as usize + self.file_slots as usize
    }

    fn get_arch(key: usize) -> usize {
        ((key as u32 & FW_CFG_ARCH_LOCAL) != 0) as usize
    }

    pub fn add_bytes(&mut self, key: u32, data: &[u8], len: u32, read_only: bool) {
        let arch = FWCfgState::get_arch(key as usize);

        let key = key & FW_CFG_ENTRY_MASK;

        assert!((key as usize) < self.max_entry());
        // assert!(self.entries[arch][key].data == NULL); /* avoid key conflict */

        let mut entry = match self.entries[arch].get_mut(&key) {
            Some(ref existing_entry) => panic!("fw cfg key already exists: {}",
                                               key),
            _ => {
                let mut data_vec = Vec::new();
                data_vec.extend_from_slice(&data[..len as usize]);

                FWCfgEntry {
                    data: data_vec,
                    allow_write: !read_only
                }
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

    pub fn add_i16(&mut self, key: u32, data: i16, read_only: bool) {
        let mut buf = [0; size_of::<i16>()];
        LittleEndian::write_i16(&mut buf, data);
        self.add_bytes(key, &buf, size_of::<i16>() as u32, read_only);
    }

    pub fn add_i64(&mut self, key: u32, data: i64, read_only: bool) {
        let mut buf = [0; size_of::<i64>()];
        LittleEndian::write_i64(&mut buf, data);
        self.add_bytes(key, &buf, size_of::<i64>() as u32, read_only);
    }

    pub fn add_file(&mut self, filename: &str, data: &[u8], len: u32) {
        // qemu sorts the files by filename, we can probably skip this.

        let fw_cfg_files = self.files_wrapper.files;
        let count = unsafe { u32::from_be((*fw_cfg_files).count) };
        let files_vec = self.files_wrapper.to_files_vec();
        let mut file_entry = files_vec[count as usize];

        file_entry.size = len.to_be();
        file_entry.select = ((FW_CFG_FILE_FIRST + count) as u16).to_be();

        self.add_bytes(FW_CFG_FILE_FIRST + count, data, len, true);
        unsafe { (*fw_cfg_files).count = (count + 1).to_be() };
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
            Some(entry) if entry.data.len() >= 0 &&
                        self.cur_offset < entry.data.len() as u32 => {
                // Fill the buffer with data from the config entry,
                // starting with the current offset.
                let entry_len = entry.data.len() as u32;
                let entry_read_len = min(
                    read_len, (entry_len - self.cur_offset) as usize);
                data[..entry_read_len].clone_from_slice(
                    &entry.data[cur_offset..cur_offset + read_len as usize]);
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

