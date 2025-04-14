use std::sync::Arc;

use parking_lot::Mutex;

use crate::{cart::Cart, io::IO, mem::Memory, ppu::Ppu, rng::Rng};

const RAM_START: u16 = 0x0000;
const RAM_END: u16 = 0x4FFF;
const SERIAL_START: u16 = 0x5000;
const SERIAL_END: u16 = 0x5FFF;
// const SERIAL_START: u16 = 0x8400;
// const SERIAL_END: u16 = 0x8403;
const ROM_START: u16 = 0x4000;
const ROM_END: u16 = 0xFFFF;

// pub struct CpuBus {
//     pub bus: Rc<RefCell<Bus>>,
//     pub cpu: CPU6502,
// }

// impl CpuBus {
//     pub fn new(bus: Bus) -> Self {
//         let bus = Rc::new(RefCell::new(bus));
//         let cpu = CPU6502::new(bus);
//         Self {
//             bus,
//             cpu
//         }
//     }
// }

pub struct Bus {
    pub mem: Memory,
    pub ppu: Ppu,
    // pub stdout: Option<Stdout>,
    // pub stdin: Option<Stdin>,
    // pub serial: Option<Serial>,
    pub rng: Option<Rng>,
    pub cart: Option<Arc<Mutex<Cart>>>,
}

impl Bus {
    pub fn load_cart(&mut self, cart: Cart) {
        let cart = Arc::new(Mutex::new(cart));

        self.cart = Some(cart.clone());
        self.ppu.cart = Some(cart.clone());
    }

    pub fn load_mem(&mut self, data: &[u8], offset: u16) {
        self.mem.load(data, offset);
    }
}

impl IO for Bus {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0xFE => {
                if let Some(rng) = &mut self.rng {
                    rng.read(addr)
                } else {
                    0
                }
            }
            // SERIAL_START..=SERIAL_END => self.serial.read(addr - SERIAL_START),
            ROM_START..=ROM_END => {
                if let Some(ref cart) = self.cart {
                    cart.lock().read(addr)
                } else {
                    self.mem.read(addr)
                }
            }
            _ => self.mem.read(addr),
        }
    }
    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            // SERIAL_START..=SERIAL_END => self.serial.write(addr - SERIAL_START, data),
            _ => self.mem.write(addr, data),
        }
    }
}
