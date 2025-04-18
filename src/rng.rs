use crate::io::IO;
use rand::{Rng as RandRng, prelude::*};

pub struct Rng {
    value: u8,
}

impl Rng {
    pub fn new() -> Self {
        Self { value: 0 }
    }
}

impl IO for Rng {
    fn read(&mut self, _addr: u16) -> u8 {
        let mut rng = rand::rng();
        rng.random::<u8>()
    }
    fn write(&mut self, _addr: u16, _data: u8) {}
}
