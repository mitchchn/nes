pub mod cpu;
pub mod display;
pub mod tui;
pub mod io;
pub mod debugger;
pub mod mem;
pub mod stdout;
pub mod stdin;
pub mod serial;
pub mod bus;

#[macro_use]
extern crate bitflags;

#[no_link]
extern crate rustasm6502;

// use crate::machine::Machine;

// #[wasm_bindgen]
// pub struct Emulator {
//     m: Machine,
// }

// #[wasm_bindgen]
// impl Emulator {
//     #[wasm_bindgen]
//     pub fn new() -> Self {
//         Emulator { m: Machine::new() }
//     }

//     // #[wasm_bindgen]
//     pub fn load(&mut self, rom: &[u8]) {
//         self.m.load(rom)
//     }

//     pub fn reset(&mut self) {
//         self.m.reset()
//     }

//     pub fn run(&mut self) {
//         self.m.run()
//     }
// }
