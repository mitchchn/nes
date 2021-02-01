use wasm_bindgen::prelude::*;

mod bus;
mod cpu;
mod display;
mod io;
mod machine;

#[macro_use]
extern crate bitflags;

#[no_link]
extern crate rustasm6502;

use crate::machine::Machine;

#[wasm_bindgen]
pub struct Emulator {
    m: Machine,
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen]
    pub fn new() -> Self {
        Emulator { m: Machine::new() }
    }

    #[wasm_bindgen]
    pub fn load(&mut self, rom: &[u8], offset: u16) {
        self.m.load(rom, offset)
    }

    pub fn reset(&mut self) {
        self.m.reset()
    }

    pub fn run(&mut self) {
        self.m.run()
    }
}
