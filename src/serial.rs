use crate::io::IO;
use std::{io::Write, ops::Deref, time::Instant};
use serialport::{self, SerialPort, TTYPort};

const ACIA_DATA: u16 = 1;
const ACIA_STATUS: u16 = 0;
const ACIA_COMMAND: u16 = 2;
const ACIA_CONTROL: u16 = 3;

bitflags! {
    pub struct Status: u8 {
        const TX_EMPTY = 1 << 1;
        const RX_FULL = 1 << 0;
    }
}

/// Simple ACIA serial device for 6502
pub struct Serial {
    port: Box<dyn SerialPort>,
    status: Status,
}

impl Serial {
    pub fn new(path: &str) -> Result<Self, serialport::Error> {
        let mut port = serialport::new(path, 19_200).open_native()?;
        port.set_exclusive(false).expect("Could not set exclusive to false");

        Ok(Self {
            port: Box::new(port),
            status: Status::empty(),
        })
    }
}

impl IO for Serial {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            ACIA_STATUS => {
                let rx_full = self.port.bytes_to_read().map(|b| b > 0).unwrap_or_default();
                let tx_empty = self.port.bytes_to_write().map(|b| b == 0).unwrap_or_default();
                
                self.status.set(Status::RX_FULL, rx_full);
                self.status.set(Status::TX_EMPTY, tx_empty);

                self.status.bits()
            }
            ACIA_DATA => {
                let mut buf = [0];
                self.port.read(&mut buf).unwrap_or_default();
                buf[0]
            }
            _ => {
                0
            }
        }
    }
    fn write(&mut self, addr: u16, data: u8) {
        let buf: [u8; 1] = [data];
        match addr {
            ACIA_DATA => {
                self.port.write(&buf).expect("Could not write to serial port");

            }
            _ => {

            }
        }
    }
}

