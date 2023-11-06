extern crate sdl2;

use std::cell::RefCell;
use std::io::stdout;
use std::rc::Rc;
use std::sync::atomic::AtomicU16;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{JoinHandle, Thread};
use std::time::Duration;
use std::{env, fs, io, thread};

use crossterm::event::{Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, ExecutableCommand};
use nes::cpu::Instruction;
use ratatui::prelude::{Constraint, CrosstermBackend, Direction, Layout};
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::canvas::{Canvas, Circle, Map, MapResolution, Points, Rectangle};
use ratatui::widgets::{
    Axis, Block, BorderType, Borders, Chart, Dataset, Padding, Paragraph, Wrap,
};
use ratatui::{Frame, Terminal};

use nes::{
    cpu::{Mode, Status, CPU6502, INSTRUCTIONS},
    display::Display,
    io::IO,
    mem::Memory,
};

enum CpuMessage {
    Pause,
}

pub struct DebuggerCpuState {
    /// Program counter
    pub pc: u16,
    /// Accmulator
    pub a: u8,
    /// X index
    pub x: u8,
    /// Y index
    pub y: u8,
    /// Stack pointer
    pub sp: u8,
    /// Processor status
    pub p: Status,
}

pub struct Debugger {
    pub cpu: Arc<Mutex<CPU6502>>,
    pub mem: Arc<Mutex<Memory>>,
    pub instruction_log: Vec<(u16, String)>,
    cpu_thread: Option<mpsc::Sender<CpuMessage>>,
}

impl Debugger {
    pub fn new() -> Self {
        let mem = Arc::new(Mutex::new(Memory::new()));
        let cpu = Arc::new(Mutex::new(CPU6502::new(mem.clone())));

        let m = Debugger {
            cpu,
            mem,
            instruction_log: vec![],
            cpu_thread: None,
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
        self.mem.lock().unwrap().load(data, offset);
        self.instruction_log = self.disassemble();
    }

    pub fn step(&mut self) {
        let mut cpu: std::sync::MutexGuard<'_, CPU6502> = self.cpu.lock().unwrap();

        cpu.clock();
        while !cpu.halted() && cpu.cycles_left > 0 {
            cpu.clock();
        }
    }

    pub fn reset(&mut self) {
        let mut cpu = self.cpu.lock().unwrap();

        cpu.reset();
        cpu.pc = 0x0400;
    }

    pub fn pause(&mut self) {
        // let mut cpu = self.cpu.lock().unwrap();
        if let Some(cpu_thread) = &self.cpu_thread {
            let _ = cpu_thread.send(CpuMessage::Pause);
        }
    }

    pub fn run(&mut self) {
        let cpu = self.cpu.clone();
        let (tx, rx) = mpsc::channel::<CpuMessage>();
        self.cpu_thread = Some(tx);

        thread::spawn(move || loop {
            let mut cpu = cpu.lock().unwrap();
            cpu.clock();

            // breakpoint lol
            // if let Some(inst) = cpu.instruction {
            //     if inst.0 == 0x998 {
            //         break;
            //     }
            // }

            if cpu.halted() {
                break;
            }
            drop(cpu);
            thread::sleep(Duration::new(0, 1000));
            match rx.try_recv() {
                Ok(CpuMessage::Pause) | Err(TryRecvError::Disconnected) => {
                    break;
                }
                Err(TryRecvError::Empty) => {}
            }
        });
    }

    pub fn debug(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        // let mut d = Display::new();
        // d.show();

        loop {
            let cpu = self.cpu.lock().unwrap();

            // d.flush(&self.mem.lock().unwrap().0[0x200..=0x5FF]);

            terminal.draw(|frame: &mut Frame| {
                let outer_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(10), Constraint::Max(4)])
                    .split(frame.size());

                let main_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(10), Constraint::Length(54)])
                    .split(outer_layout[0]);

                let left_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(6), Constraint::Min(10)])
                    .split(main_layout[0]);

                let right_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(21), Constraint::Min(0)])
                    .split(main_layout[1]);

                let color_flag = |f: u8| {
                    if f == 1 {
                        format!("{} ", f).green()
                    } else {
                        Span::raw(format!("{} ", f).to_string())
                    }
                };

                let f: [u8; 8] = [
                    if cpu.p.contains(Status::N) { 1 } else { 0 },
                    if cpu.p.contains(Status::V) { 1 } else { 0 },
                    if cpu.p.contains(Status::U) { 1 } else { 0 },
                    if cpu.p.contains(Status::B) { 1 } else { 0 },
                    if cpu.p.contains(Status::D) { 1 } else { 0 },
                    if cpu.p.contains(Status::I) { 1 } else { 0 },
                    if cpu.p.contains(Status::Z) { 1 } else { 0 },
                    if cpu.p.contains(Status::C) { 1 } else { 0 },
                ];

                let status_header = Line::styled(
                    "PC    A  X  Y    SP    N V - B D I Z C",
                    Style::default().bold(),
                );
                let status_line = Line::from(vec![
                    format!(
                        "{:04X}  {:02X} {:02X} {:02X}   {:02X}    ",
                        cpu.pc, cpu.a, cpu.x, cpu.y, cpu.sp
                    )
                    .into(),
                    color_flag(f[0]),
                    color_flag(f[1]),
                    color_flag(f[2]),
                    color_flag(f[3]),
                    color_flag(f[4]),
                    color_flag(f[5]),
                    color_flag(f[6]),
                    color_flag(f[7]),
                ]);

                let command = Paragraph::new(Text::from(vec![
                    Line::from("6502 CPU Emulator".light_yellow()),
                    Line::raw("reset (r)    step (space)    continue (g)    quit (q)"),
                ]))
                .block(Block::default().padding(Padding::horizontal(1)));

                let status_text = Text::from(vec![status_header, status_line]);
                let status = Paragraph::new(status_text).block(
                    Block::default()
                        .title("status")
                        .padding(Padding::uniform(1))
                        .borders(Borders::ALL),
                );

                let cpu_addr = if let Some((addr, _)) = cpu.instruction {
                    addr
                } else {
                    0
                };

                // let pivot_instr = self.instruction_log.iter().find(|(addr,_)| *addr == cpu_addr);

                let trace_text: Text<'_> = Text::from(
                    self.instruction_log
                        .iter()
                        .map(|(addr, inst)| {
                            let inst = format!("{:04X} {}", *addr, inst);

                            let instruction = cpu.instruction;

                            // if matches!(Some((*addr,)), cpu.instruction) {
                            //     Line::styled(inst, Style::default().fg(Color::Green))
                            // } else {
                            //     Line::raw(inst)
                            // }

                            match cpu.instruction {
                                Some((cpu_addr, _)) if cpu_addr == *addr => {
                                    Line::styled(inst, Style::default().fg(Color::Green))
                                }
                                _ => Line::raw(inst),
                            }
                        })
                        .collect::<Vec<Line<'_>>>(),
                );

                // Lines visible in trace area, subtracting 4 from height for border and padding
                let trace_lines = (left_layout[1].height - 4) as usize;
                // let trace_scroll_pos = if trace_text.lines.len() > trace_lines {
                //     trace_text.lines.len() - trace_lines
                // } else {
                //     0
                // };
                let trace_scroll_pos = {
                    let pos = if let Some((cpu_addr, _)) = cpu.instruction {
                        let pos = self
                            .instruction_log
                            .iter()
                            .position(|(addr, _)| cpu_addr == *addr)
                            .unwrap_or_default();

                        if pos > (trace_lines / 2) {
                            pos - (trace_lines / 2)
                        } else {
                            pos
                        }
                    } else {
                        0
                    };
                    pos
                } as u16;

                let trace = Paragraph::new(trace_text)
                    .scroll((trace_scroll_pos, 0))
                    .block(
                        Block::default()
                            .title("trace")
                            .padding(Padding::uniform(1))
                            .borders(Borders::ALL),
                    );

                let mut data: Vec<(f64, f64)> = vec![];

                for i in 0x200..=0x5FF {
                    // if self.mem.borrow().0[i] == 0 {
                    //     continue;
                    // }
                    let x_pos = (((i - 0x200) % 32) as f64 * 1.0);
                    let y_pos = 32.0 - ((((i - 0x200) as f64) / 32.0).floor() * 1.0);
                    data.push((x_pos, y_pos as f64));
                }

                let datasets: Vec<Dataset<'_>> = vec![Dataset::default()
                    .marker(Marker::Braille)
                    .style(Style::default().fg(Color::White))
                    .data(&data)];

                let display = Chart::new(datasets)
                    .block(Block::default().padding(Padding::new(4, 4, 1, 1)))
                    .x_axis(Axis::default().bounds([0.0, 32.0]))
                    .y_axis(Axis::default().bounds([0.0, 32.0]));

                // Display memory as rows of 8 bytes indexed by address
                let mem_text = Text::from(
                    self.mem
                        .lock()
                        .unwrap()
                        .0
                        .chunks(8)
                        .into_iter()
                        .enumerate()
                        .map(|(idx, chunk)| {
                            Line::from(vec![
                                format!("{:04X}: ", idx * 8).dark_gray(),
                                chunk
                                    .iter()
                                    .map(|b| format!("{:02X} ", b))
                                    .collect::<String>()
                                    .into(),
                                "  ".into(),
                                chunk
                                    .iter()
                                    .map(|b| {
                                        if *b != 0 && b.is_ascii_alphanumeric()
                                            || b.is_ascii_punctuation()
                                        {
                                            format!("{} ", *b as char)
                                        } else {
                                            ". ".to_string()
                                        }
                                    })
                                    .collect::<String>()
                                    .into(),
                            ])
                        })
                        .collect::<Vec<Line<'_>>>(),
                );

                let mem = Paragraph::new(mem_text).wrap(Wrap { trim: true }).block(
                    Block::default()
                        .title("mem")
                        .padding(Padding::uniform(1))
                        .borders(Borders::ALL),
                );

                frame.render_widget(status, left_layout[0]);
                frame.render_widget(trace, left_layout[1]);
                frame.render_widget(display, right_layout[0]);
                frame.render_widget(mem, right_layout[1]);
                frame.render_widget(command, outer_layout[1]);
            })?;

            drop(cpu);
            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        break;
                    } else if key.kind == event::KeyEventKind::Press
                        && key.code == KeyCode::Char(' ')
                    {
                        self.step();
                    } else if key.kind == event::KeyEventKind::Press
                        && key.code == KeyCode::Char('r')
                    {
                        self.reset();
                    } else if key.kind == event::KeyEventKind::Press
                        && key.code == KeyCode::Char('g')
                    {
                        self.run();
                    } else if key.kind == event::KeyEventKind::Press
                        && key.code == KeyCode::Char('p')
                    {
                        self.pause();
                    }
                }
            }
        }

        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }
}

impl IO for Debugger {
    fn read(&mut self, addr: u16) -> u8 {
        self.mem.lock().unwrap().read(addr)
    }
    fn write(&mut self, addr: u16, data: u8) {
        self.mem.lock().unwrap().write(addr, data)
    }
}

pub fn main() {
    let arg = env::args().nth(2);
    let rom = if let Some(arg) = arg {
        fs::read(arg).expect("Usage: debugger [FILENAME]")
    } else {
        // vec![
        //     0xa9, 0x00, 0xa2, 0x08, 0x4e, 0x34, 0x12, 0x90, 0x04, 0x18, 0x6d, 0xff, 0xff, 0x6a,
        //     0x6e, 0x34, 0x12, 0xca, 0xd0, 0xf3, 0x8d, 0x12, 0x34, 0xad, 0x34, 0x12, 0x60,
        // ]
        // vec![
        //     0xa9, 0x01, 0x8d, 0x00, 0x02, 0xa9, 0x05, 0x8d, 0x01, 0x02, 0xa9, 0x08, 0x8d, 0x02, 0x02
        // ]
        // vec![0xa9, 0x01, 0xa2, 0x00, 0x9d, 0x00, 0x02, 0xe8, 0x10, 0xfa]
        // vec![0xa2, 0x00, 0xa9, 0x01, 0x9d, 0x00, 0x02, 0xe8, 0x10, 0xfa]
        vec![
            0xa2, 0x00, 0xa9, 0x01, 0x9d, 0x00, 0x02, 0xa4, 0xff, 0x88, 0xd0, 0xfd, 0xe8, 0x10,
            0xf5,
        ]
    };

    let mut d = Debugger::new();
    // let rom = fs::read("src/nestest.nes").expect("Could not open file");

    d.load(&rom, 0);
    d.reset();
    let _ = d.debug();
}
