use crate::io::IO;

pub struct Memory([u8; 0xFFFF + 1]);

impl Memory {
    pub fn new() -> Self {
        Self([0; 0xFFFF + 1])
    }

    pub fn load(&mut self, data: &[u8], offset: u16) {
        for (i, byte) in data.iter().enumerate() {
            self.0[i + offset as usize] = *byte;
        }
    }
}

impl IO for Memory {
    fn read(&mut self, addr: u16) -> u8 {
        self.0[addr as usize]
    }
    fn write(&mut self, addr: u16, data: u8) {
        self.0[addr as usize] = data;
    }
}
