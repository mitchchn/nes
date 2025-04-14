pub mod bus;
pub mod cart;
pub mod core;
pub mod cpu;
pub mod display;
pub mod io;
pub mod machine;
pub mod mem;
pub mod ppu;
pub mod rng;
pub mod serial;
pub mod stdin;
pub mod stdout;
pub mod tui;

#[macro_use]
extern crate bitflags;

// #[no_link]
// extern crate rustasm6502;

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
