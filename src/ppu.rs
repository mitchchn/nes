use std::sync::Arc;

use parking_lot::Mutex;

use crate::{cart::Cart, io::IO};

pub struct Ppu {
    pub cart: Option<Arc<Mutex<Cart>>>,
}

impl Ppu {
    pub fn new() -> Self {
        Self { cart: None }
    }
}

impl IO for Ppu {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            // Pattern tables
            0x0000..0x2000 => {
                if let Some(cart) = &self.cart {
                    cart.lock().read(addr)
                } else {
                    0
                }
            }
            // Nametables
            0x2000..0x3000 => 0,
            // Palette RAM
            0x3F00..0x4000 => 0,
            _ => 0,
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        todo!()
    }
}
