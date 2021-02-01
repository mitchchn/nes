use crate::display::Display;
use crate::io::IO;

pub struct Bus {
    pub mem: Vec<u8>,
    pub display: Display,
}

impl Bus {
    pub fn new() -> Self {
        let b = Bus {
            mem: init_mem(),
            display: Display::new(),
        };

        b
    }
}

impl IO for Bus {
    fn read(&self, addr: u16) -> u8 {
        if addr <= std::u16::MAX {
            self.mem[addr as usize]
        } else {
            0
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if addr >= 0x200 && addr <= 0x05ff {
            self.display.write(addr - 0x200, data);
        } else if addr <= std::u16::MAX {
            self.mem[addr as usize] = data;
        }
    }
}

fn init_mem() -> Vec<u8> {
    vec![0x0; 0xFFFF + 1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let b = Bus::new();
        assert_eq!(b.mem.len(), usize::pow(2, 16));
    }
}
