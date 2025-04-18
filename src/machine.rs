use std::{
    borrow::BorrowMut,
    cell::RefCell,
    ops::{Deref, Mul},
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime},
};

use parking_lot::Mutex;

use crate::{
    bus::Bus,
    cart::Cart,
    cpu::{CPU6502, INSTRUCTIONS, Mode},
    io::IO,
    mem::Memory,
    ppu::Ppu,
    rng::Rng,
};

pub enum CpuMessage {
    Pause,
}

#[derive(Debug)]
pub struct FrameTimer {
    pub start_time: Instant,
    pub current_cycles: u64,

    pub cycles_per_frame: u64,
    pub frame_time: Duration,

    pub last_context_switch_error: Duration,
}

impl FrameTimer {
    pub fn new(target_fps: u64, clock_speed: u64) -> Self {
        // 33,3333 cycles for 60 FPS @ 2MHZ
        let cycles_per_frame = (clock_speed / target_fps);
        // 16.666 ms (16,666 us) per frame at 60 FPS
        let ns_per_frame = 1_000_000_000 / target_fps;
        Self {
            start_time: Instant::now(),
            current_cycles: 0,
            cycles_per_frame,
            frame_time: Duration::from_nanos(ns_per_frame),
            last_context_switch_error: Duration::from_nanos(0),
        }
    }
    pub fn computed(&self) -> bool {
        self.current_cycles >= self.cycles_per_frame
    }

    pub fn time_remaining(&self) -> Duration {
        let time_elapsed_in_frame = self.start_time.elapsed();
        if time_elapsed_in_frame >= self.frame_time {
            println!("Frame took longer than 16ms!");
            Duration::from_millis(0)
        } else {
            self.frame_time - time_elapsed_in_frame
        }
    }

    pub fn sleep(&mut self) {
        let time_remaining = self.time_remaining();
        if time_remaining <= self.last_context_switch_error {
            return;
        }

        let corrected_frame_time_delay = time_remaining - self.last_context_switch_error;
        // dbg!(&delay);
        let sleep_start_time = Instant::now();
        std::thread::sleep(corrected_frame_time_delay);
        self.last_context_switch_error = sleep_start_time.elapsed() - corrected_frame_time_delay;
    }

    pub fn clock(&mut self) {
        self.current_cycles += 1;
    }

    pub fn reset(&mut self) {
        self.current_cycles = 0;
        self.start_time = Instant::now();
    }
}

static HALT: AtomicBool = AtomicBool::new(true);

pub struct Machine {
    pub cpu: Arc<Mutex<CPU6502<Bus>>>,
    pub instruction_log: Vec<(u16, String)>,
    pub breakpoints: Vec<u16>,
    pub clock_speed: Option<u64>,
    pub non_interactive_mode: bool,
    pub max_speed: bool,
}

impl Machine {
    pub fn new() -> Self {
        let mem: Memory = Memory::new();
        // let stdout = Some(Stdout::new());
        // let stdin = Some(Stdin::new());
        // let serial =
        // Some(Serial::new("/dev/tty.debug-console").expect("Could not open serial port"));
        let cart = None;
        let rng = Some(Rng::new());
        let ppu = Ppu::new();

        let bus = Bus {
            mem,
            cart,
            ppu,
            rng,
        };

        let cpu = Arc::new(Mutex::new(CPU6502::new(bus)));

        let m = Machine {
            cpu,
            instruction_log: vec![],
            breakpoints: vec![],
            clock_speed: Some(2_000_000),
            // clock_speed: Some(20_000),
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

    pub fn load_cart(&mut self, data: &[u8]) {
        self.cpu
            .lock()
            .mem
            .load_cart(Cart::new(data).expect("Could not load cartridge"));

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

        let max_speed = self.max_speed;
        let cpu = self.cpu.clone();
        let cpu_thread = thread::spawn(move || {
            let mut frame_timer = FrameTimer::new(60, clock_speed);
            // Run loop
            'running: loop {
                let mut cpu = cpu.lock();
                if HALT.load(Ordering::Relaxed) {
                    break 'running;
                }

                // Execute current instruction
                'execute: loop {
                    cpu.clock();
                    frame_timer.clock();
                    if cpu.cycles_left == 0 {
                        break 'execute;
                    }
                }

                // if &frame_timer.current_cycles > &(532 as u64) {
                // dbg!(&frame_timer);
                // dbg!(&frame_timer.time_remaining());
                // HALT.store(true, Ordering::Relaxed);
                // }

                // Check breakpoints
                if breakpoints.contains(&cpu.pc) {
                    HALT.store(true, Ordering::Relaxed);
                }

                // Instructions are executed as fast as the host is capable of running them.
                // To simulate the speed of the original hardware, we wait out the remaining length of time in the frame (interval)
                // before executing the next instruction. The interval length was calculated based on the desired clockspeed.
                if !max_speed && frame_timer.computed() {
                    frame_timer.sleep();
                    frame_timer.reset();
                }
            }
        });
        Some(cpu_thread)
    }
}

impl IO for Machine {
    fn read(&mut self, addr: u16) -> u8 {
        self.cpu.lock().mem.read(addr)
    }
    fn write(&mut self, addr: u16, data: u8) {
        self.cpu.lock().mem.write(addr, data)
    }
}
