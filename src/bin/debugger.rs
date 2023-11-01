use std::cell::RefCell;
use std::io::stdout;
use std::rc::Rc;
use std::{env, fs, io};

use crossterm::event::{Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, ExecutableCommand};
use ratatui::prelude::{Constraint, CrosstermBackend, Direction, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};
use ratatui::{Frame, Terminal};

use nes::{
    cpu::{Mode, Status, CPU6502, INSTRUCTIONS},
    io::IO,
    mem::Memory,
};

pub struct Debugger {
    pub cpu: CPU6502,
    pub mem: Rc<RefCell<Memory>>,
    pub instruction_log: Vec<String>,
    pub last_instr_addr: u16,
}

impl Debugger {
    pub fn new() -> Self {
        let mem = Rc::new(RefCell::new(Memory::new()));
        let cpu = CPU6502::new(mem.clone());

        let m = Debugger {
            cpu,
            mem,
            instruction_log: vec![],
            last_instr_addr: 0,
        };
        m
    }

    pub fn disassemble(&mut self) -> Vec<(u16, String)> {
        let mut instructions = vec![];

        let mut addr = 0;
        while addr < 0xFFFF {
            let opcode: u8 = self.read(addr);

            let instruction = INSTRUCTIONS[opcode as usize];
            let op_addr =
                if matches!(instruction.1, Mode::IMP) || matches!(instruction.1, Mode::ACC) {
                    addr
                } else {
                    addr + 1
                };

            let formatted_operand = match instruction.1 {
                Mode::IMP => "".to_string(),
                Mode::IMM => format!("#${:02X}", self.read(op_addr)),
                Mode::ACC => "A".to_string(),
                Mode::ABS => format!("${:04X}", op_addr),
                Mode::ABX => format!("${:04X},X", op_addr),
                Mode::ABY => format!("${:04X},Y", op_addr),
                Mode::ZPG => format!("${:02X}", op_addr),
                Mode::ZPX => format!("${:02X},X", op_addr),
                Mode::ZPY => format!("${:02X},Y", op_addr),
                Mode::ZIX => format!("(${:02X},X)", op_addr),
                Mode::ZIY => format!("(${:02X},Y)", op_addr),
                Mode::IND => format!("(${:04X})", op_addr),
                Mode::REL => format!("${:04X}", op_addr),
            };
            instructions.push((addr, format!("{:#?} {}", instruction.0, &formatted_operand)));
            addr = op_addr + 1;
        }
        instructions
    }

    pub fn load(&mut self, data: &[u8], offset: u16) {
        self.mem.borrow_mut().load(data, offset)
    }

    pub fn step(&mut self) {
        self.last_instr_addr = self.cpu.pc;

        self.cpu.clock();
        while !self.cpu.halted() && self.cpu.cycles_left > 0 {
            self.cpu.clock()
        }

        let decoded_instr = self.cpu.decode_instruction();
        self.instruction_log
            .push(format!("{:04X}  {}", self.cpu.pc, decoded_instr));
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.instruction_log = vec![];
        self.last_instr_addr = 0;
    }

    pub fn run(&mut self) {
        while !self.cpu.halted() {
            self.step();
        }
    }

    pub fn debug(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        loop {
            terminal.draw(|frame: &mut Frame| {
                let outer_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(10), Constraint::Max(4)])
                    .split(frame.size());

                let main_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(outer_layout[0]);

                let left_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Max(6), Constraint::Min(10)])
                    .split(main_layout[0]);

                let right_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(100)])
                    .split(main_layout[1]);

                let color_flag = |f: u8| {
                    if f == 1 {
                        format!("{} ", f).green()
                    } else {
                        Span::raw(format!("{} ", f).to_string())
                    }
                };

                let f: [u8; 8] = [
                    if self.cpu.p.contains(Status::N) { 1 } else { 0 },
                    if self.cpu.p.contains(Status::V) { 1 } else { 0 },
                    if self.cpu.p.contains(Status::U) { 1 } else { 0 },
                    if self.cpu.p.contains(Status::B) { 1 } else { 0 },
                    if self.cpu.p.contains(Status::D) { 1 } else { 0 },
                    if self.cpu.p.contains(Status::I) { 1 } else { 0 },
                    if self.cpu.p.contains(Status::Z) { 1 } else { 0 },
                    if self.cpu.p.contains(Status::C) { 1 } else { 0 },
                ];

                let status_header = Line::styled(
                    "PC    A  X  Y    SP    N V - B D I Z C",
                    Style::default().bold(),
                );
                let status_line = Line::from(vec![
                    format!(
                        "{:04X}  {:02X} {:02X} {:02X}   {:02X}    ",
                        self.cpu.pc, self.cpu.a, self.cpu.x, self.cpu.y, self.cpu.sp
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

                let trace_text: Text<'_> = Text::from(
                    self.instruction_log
                        .iter()
                        .map(Line::raw)
                        .collect::<Vec<Line<'_>>>(),
                );

                // Lines visible in trace area, subtracting 4 from height for border and padding
                let trace_lines = (left_layout[1].height - 4) as usize;
                let trace_scroll_pos = if trace_text.lines.len() > trace_lines {
                    trace_text.lines.len() - trace_lines
                } else {
                    0
                };

                let trace = Paragraph::new(trace_text)
                    .scroll((trace_scroll_pos as u16, 0))
                    .block(
                        Block::default()
                            .title("trace")
                            .padding(Padding::uniform(1))
                            .borders(Borders::ALL),
                    );

                // Display memory as rows of 8 bytes indexed by address
                let mem_text = Text::from(
                    self.mem
                        .borrow()
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
                frame.render_widget(mem, right_layout[0]);
                frame.render_widget(command, outer_layout[1]);
            })?;
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
        self.mem.borrow_mut().read(addr)
    }
    fn write(&mut self, addr: u16, data: u8) {
        self.mem.borrow_mut().write(addr, data)
    }
}

pub fn main() {
    let arg = env::args().nth(2);
    let rom = if let Some(arg) = arg {
        fs::read(arg).expect("Usage: debugger [FILENAME]")
    } else {
        vec![
            0xa9, 0x00, 0xa2, 0x08, 0x4e, 0x34, 0x12, 0x90, 0x04, 0x18, 0x6d, 0xff, 0xff, 0x6a,
            0x6e, 0x34, 0x12, 0xca, 0xd0, 0xf3, 0x8d, 0x12, 0x34, 0xad, 0x34, 0x12, 0x60,
        ]
    };

    let mut d = Debugger::new();
    // let rom = fs::read("src/nestest.nes").expect("Could not open file");

    d.load(&rom, 0);
    d.cpu.reset();
    let _ = d.debug();
}
