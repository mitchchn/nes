pub trait IO {
    fn read(&self, _addr: u16) -> u8 {
        0
    }
    fn write(&mut self, _addr: u16, _data: u8) {}
}

pub struct NullIO {}

impl IO for NullIO {}
