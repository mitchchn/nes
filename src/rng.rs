use crate::io::IO;
use rand::{prelude::*, Rng as RandRng};

pub struct Rng {}

impl Rng {
    pub fn new() -> Self {
        Self {}
    }
}

impl IO for Rng {
    fn read(&mut self, _addr: u16) -> u8 {
        let mut rng = rand::rng();
        rng.random::<u8>()
    }
    fn write(&mut self, _addr: u16, _data: u8) {}
}
