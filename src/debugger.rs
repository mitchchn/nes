use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, SystemTime},
};

use parking_lot::Mutex;

use crate::{
    cpu::{Mode, CPU6502, INSTRUCTIONS},
    io::IO,
    mem::Memory,
    stdin::Stdin,
    stdout::Stdout,
};

pub enum CpuMessage {
    Pause,
}

pub struct Bus {
    pub mem: Memory,
    pub stdout: Stdout,
    pub stdin: Stdin,
}

impl IO for Bus {
    fn read(&mut self, addr: u16) -> u8 {
        if addr >= 0xB001 && addr < 0xB400 {
            self.stdin.read(addr - 0xB001)
        } else {
            self.mem.read(addr)
        }
    }
    fn write(&mut self, addr: u16, data: u8) {
        if addr >= 0xA000 && addr < 0xA400 {
            // println!("hi!")
            self.stdout.write(addr - 0xA000, data);
            self.stdout.flush();
        } else if addr == 0xB000 {
            self.stdin.write(0, 0);
        } else {
            self.mem.write(addr, data);
        }
    }
}

static HALT: AtomicBool = AtomicBool::new(true);

pub struct Debugger {
    pub cpu: Arc<Mutex<CPU6502>>,
    pub bus: Arc<Mutex<Bus>>,
    pub instruction_log: Vec<(u16, String)>,
    pub breakpoints: Vec<u16>,
    pub clock_speed: Option<usize>,
    pub non_interactive_mode: bool,
}

impl Debugger {
    pub fn new() -> Self {
        let mem = Memory::new();
        let stdout = Stdout::new();
        let stdin = Stdin::new();

        let bus = Arc::new(Mutex::new(Bus { mem, stdout, stdin }));

        let cpu = Arc::new(Mutex::new(CPU6502::new(bus.clone())));

        let m = Debugger {
            cpu,
            bus,
            instruction_log: vec![],
            breakpoints: vec![],
            clock_speed: None,
            non_interactive_mode: false,
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
        self.bus.lock().mem.load(data, offset);
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
        cpu.pc = 0x0400;
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

    pub fn pause(&mut self) {
        HALT.store(true, Ordering::Relaxed)
    }

    pub fn run(&mut self) -> Option<JoinHandle<()>> {
        HALT.store(false, Ordering::Relaxed);

        let cpu = self.cpu.clone();
        let breakpoints = self.breakpoints.clone();
        let ns_per_clock = if let Some(clock_speed) = self.clock_speed {
            1_000_000_000 / (clock_speed * 1_000_000)
        } else {
            0
        };
        let cycle_length: Duration = Duration::from_nanos(ns_per_clock as u64);

        let cpu_thread = thread::spawn(move || {
            // let mut cpu = CPU6502::new(mem);
            // cpu.pc = 0x400;

            'running: loop {
                if !HALT.load(Ordering::Relaxed) {
                    let mut cpu = cpu.lock();
                    cpu.clock();

                    // Run checks only after completing an instruction, otherwise keep cycling
                    // TODO: once sub-cycle emulation is implemented, continue full loop and yield thread on each cycle
                    while cpu.cycles_left > 0 {
                        if !cycle_length.is_zero() {
                            // Spinwait
                            let cycle_start = SystemTime::now();
                            while SystemTime::now()
                                .duration_since(cycle_start)
                                .unwrap_or(cycle_length)
                                < cycle_length
                            {
                                // std::hint::spin_loop();
                            }
                        }
                        cpu.clock();
                    }

                    // check breakpoints
                    if breakpoints.contains(&cpu.pc) {
                        HALT.store(true, Ordering::Relaxed);
                    }
                } else {
                    break 'running;
                }
            }
        });
        return Some(cpu_thread);
    }
}

impl IO for Debugger {
    fn read(&mut self, addr: u16) -> u8 {
        self.bus.lock().read(addr)
    }
    fn write(&mut self, addr: u16, data: u8) {
        self.bus.lock().write(addr, data)
    }
}
