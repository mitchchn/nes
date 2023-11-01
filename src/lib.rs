use wasm_bindgen::prelude::*;

pub mod cpu;
mod display;
pub mod io;
mod machine;
mod mapper;
pub mod mem;
mod rom;
mod stdout;

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

    // #[wasm_bindgen]
    pub fn load(&mut self, rom: &[u8]) {
        self.m.load(rom)
    }

    pub fn reset(&mut self) {
        self.m.reset()
    }

    pub fn run(&mut self) {
        self.m.run()
    }
}
