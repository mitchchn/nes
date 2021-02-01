use wasm_bindgen::prelude::*;

mod bus;
mod cpu;
mod io;
mod machine;
mod display;

#[macro_use]
extern crate bitflags;

use crate::machine::Machine;


#[wasm_bindgen]
pub struct Emulator {
    m: Machine,
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen]
    pub fn new() -> Self {
        Emulator{
            m: Machine::new()
        }
    }

    #[wasm_bindgen]
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