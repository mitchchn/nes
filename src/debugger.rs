use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime}, cell::RefCell, rc::Rc, borrow::BorrowMut, ops::Deref,
};

use parking_lot::Mutex;

use crate::{
    bus::Bus,
    cpu::{Mode, CPU6502, INSTRUCTIONS},
    display::Display,
    io::IO,
    mem::Memory,
    serial::Serial,
    stdin::Stdin,
    stdout::Stdout,
};

pub enum CpuMessage {
    Pause,
}

static HALT: AtomicBool = AtomicBool::new(true);

pub struct Debugger {
    pub cpu: Arc<Mutex<CPU6502<Bus>>>,
    pub instruction_log: Vec<(u16, String)>,
    pub breakpoints: Vec<u16>,
    pub clock_speed: Option<u64>,
    pub non_interactive_mode: bool,
    pub max_speed: bool,
}

impl Debugger {
    pub fn new() -> Self {
        let mem = Memory::new();
        let stdout = Stdout::new();
        let stdin = Stdin::new();
        let display = Display::new();
        let serial = Serial::new("/tmp/vserial0").expect("Could not open serial port");

        let bus = Bus {
            mem,
            stdout,
            stdin,
            display,
            serial,
        };

        let cpu = Arc::new(Mutex::new(CPU6502::new(bus)));

        let m = Debugger {
            cpu,
            instruction_log: vec![],
            breakpoints: vec![],
            clock_speed: Some(2_000_000),
            non_interactive_mode: false,
            max_speed: false,
        };
        m
    }

    pub fn disassemble(&mut self) -> Vec<(u16, String)> {
        let mut instructions = vec![];

        let mut addr = 0;
        while addr < 0xFFFF - 2 {
            let opcode: u8 = self.read(addr);

            let instruction = INSTRUCTIONS[opcode as usize];
            let next_instr_addr = match instruction.1 {
                Mode::ACC | Mode::IMP => addr + 1,
                Mode::ABS | Mode::ABX | Mode::ABY => addr + 3,
                _ => addr + 2,
            };

            let op8 = self.read(addr + 1);
            let op16: u16 = ((self.read(addr + 2) as u16) << 8) | (self.read(addr + 1) as u16);

            let formatted_operand = match instruction.1 {
                Mode::IMP => "".to_string(),
                Mode::ACC => "A".to_string(),
                Mode::IMM => format!("#${:02X}", op8),
                Mode::ABS => format!("${:04X}", op16),
                Mode::ABX => format!("${:04X},X", op16),
                Mode::ABY => format!("${:04X},Y", op16),
                Mode::ZPG => format!("${:02X}", op8),
                Mode::ZPX => format!("${:02X},X", op8),
                Mode::ZPY => format!("${:02X},Y", op8),
                Mode::ZIX => format!("(${:02X},X)", op8),
                Mode::ZIY => format!("(${:02X},Y)", op8),
                Mode::IND => format!("(${:04X})", op8),
                Mode::REL => format!("${:02X}", op8),
            };
            instructions.push((addr, format!("{:#?} {}", instruction.0, &formatted_operand)));
            addr = next_instr_addr
        }
        instructions
    }

    pub fn load(&mut self, data: &[u8], offset: u16) {
        self.cpu.lock().mem.mem.load(data, offset);
        self.instruction_log = self.disassemble();
    }

    pub fn step(&mut self) {
        let mut cpu = self.cpu.lock();

        cpu.clock();
        while !cpu.halted() && cpu.cycles_left > 0 {
            cpu.clock();
        }
    }

    pub fn is_halted(&self) -> bool {
        HALT.load(Ordering::Relaxed)
    }

    pub fn reset(&mut self) {
        let mut cpu = self.cpu.lock();

        cpu.reset();
        self.breakpoints = vec![
            // dec mode success
            0x3469,
            // non-dec success/dec mode start!
            // 0x336D,
            // 0x346f,
            // 0x3484,
            // 0x3479,
            // 0x3470,
            // decimal mode adc/sbc
            // 0x346F
        ]
        // cpu.pc = 0x3387;
        // cpu.pc = 0x331C;
        // cpu.pc = 0x36AD;
    }

    pub fn show(&mut self) {
        // self.bus.borrow_mut().display.show();
    }

    pub fn pause(&mut self) {
        HALT.store(true, Ordering::Relaxed)
    }

    pub fn run(&mut self) -> Option<JoinHandle<()>> {
        HALT.store(false, Ordering::Relaxed);

        let breakpoints = self.breakpoints.clone();
        let clock_speed: u64 = self.clock_speed.unwrap_or(1_000_000);

        let target_fps = 60;
        let cycles_per_interval = clock_speed / target_fps;
        let ns_per_interval: u64 = 1_000_000_000 / target_fps;
        let max_speed = self.max_speed;

        let cpu = self.cpu.clone();
        let cpu_thread = thread::spawn(move || {
            let mut cycles_since_last_interval = 0;
            let mut time_to_next_interval = Instant::now() + Duration::from_nanos(ns_per_interval);

            // Run loop
            'running: loop {
                let mut cpu = cpu.lock();
                if HALT.load(Ordering::Relaxed) {
                    break 'running;
                }

                // Execute current instruction
                'execute: loop {
                    cpu.clock();
                    cycles_since_last_interval += 1;
                    if cpu.cycles_left == 0 {
                        break 'execute;
                    }

                }

                // Check breakpoints
                if breakpoints.contains(&cpu.pc) {
                    HALT.store(true, Ordering::Relaxed);
                }

                // Instructions are executed as fast as the host is capable of running them.
                // To simulate the speed of the original hardware, we wait out the remaining length of time in the frame (interval)
                // before executing the next instruction. The interval length was calculated based on the desired clockspeed.
                if !max_speed && cycles_since_last_interval > cycles_per_interval {
                    let time_left_in_interval = time_to_next_interval - Instant::now();
                    if time_left_in_interval.as_nanos() > 0 {
                        thread::sleep(time_left_in_interval);
                    }

                    cycles_since_last_interval = 0;
                    time_to_next_interval = Instant::now() + Duration::from_nanos(ns_per_interval);
                }
            }
        });
        Some(cpu_thread)
    }
}

impl IO for Debugger {
    fn read(&mut self, addr: u16) -> u8 {
        self.cpu.lock().mem.read(addr)
    }
    fn write(&mut self, addr: u16, data: u8) {
        self.cpu.lock().mem.write(addr, data)
    }
}
