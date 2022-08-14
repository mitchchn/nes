pub trait IO {
    fn read(&mut self, addr: u16) -> u8;

    fn write(&mut self, addr: u16, data: u8);

    fn write_str(&mut self, addr: u16, str: &str) {
        for (i, c) in str.chars().enumerate() {
            self.write(addr + i as u16, c as u8);
        }
    }

    fn read_str(&mut self, addr: u16) -> String {
        use std::iter::FromIterator;
        let mut buf = vec![];
        let mut pos = addr;

        loop {
            let c = self.read(pos) as char;
            if c == '\0' {
                return String::from_iter(buf);
            }
            buf.push(c);
            pos += 1;
        }
    }
}

pub struct NullIO {}

impl IO for NullIO {
    fn read(&mut self, _addr: u16) -> u8 {
        0
    }

    fn write(&mut self, _addr: u16, _data: u8) {}
}
