use super::bus::BusDevice;

pub struct PostCodeHandler {}

impl PostCodeHandler {
    pub fn new() -> Self {
        PostCodeHandler {}
    }
}

impl BusDevice for PostCodeHandler {
    fn write(&mut self, _offset: u64, data: &[u8]) {
        println!("POST: 0x{:x}", data[0]);
    }
}
