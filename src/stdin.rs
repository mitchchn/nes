use crate::io::IO;

/// Simple stdin device for 6502
pub struct Stdin {
    buffer: [u8; 4096],
    handle: std::io::Stdin,
}

impl Stdin {
    pub fn new() -> Self {
        Self {
            handle: std::io::stdin(),
            buffer: [0; 4096],
        }
    }
}

impl IO for Stdin {
    fn read(&mut self, addr: u16) -> u8 {
        self.buffer[addr as usize]
    }
    fn write(&mut self, _addr: u16, _data: u8) {
        let mut buf = "".to_string(); 
        let _ = self.handle.read_line(&mut buf);

        let bytes = buf.as_bytes();
        for i in 0..bytes.len() {
            self.buffer[i] = bytes[i];
        }
    }
}
