use std::io::stdout;

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*, Terminal};

use crate::{cpu::Status, debugger::Debugger};

pub struct Tui {
    debugger: Debugger,
}

impl Tui {
    pub fn new(debugger: Debugger) -> Self {
        Self { debugger }
    }

    pub fn show(&mut self) -> std::io::Result<()> {
        let mut d = &mut self.debugger;

        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        // let mut d = Display::new();
        // d.show();

        loop {
            let cpu = d.cpu.lock();

            // d.flush(&self.mem.lock().unwrap().0[0x200..=0x5FF]);

            terminal.draw(|frame: &mut Frame| {
                let outer_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(10), Constraint::Max(4)])
                    .split(frame.size());

                let main_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(10), Constraint::Length(58)])
                    .split(outer_layout[0]);

                let left_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(10), Constraint::Min(10)])
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
                    format!("      Cycles: {}", cpu.cycles).into(),
                ]);

                let command = Paragraph::new(Text::from(vec![
                    Line::from("6502 CPU Emulator".light_yellow()),
                    Line::from(vec![
                        "[r]".bold(),
                        "eset   ".dim(),
                        "[n]".bold(),
                        "ext   ".dim(),
                        if !d.is_halted() {
                            "[s]".bold()
                        } else {
                            "[g]".bold()
                        },
                        if !d.is_halted() {
                            "top   ".dim()
                        } else {
                            "o   ".dim()
                        },
                        "[q]".bold(),
                        "uit".dim(),
                    ]),
                ]))
                .block(Block::default().padding(Padding::horizontal(1)));

                let status_text = Text::from(vec![status_header, status_line]);
                let status = Paragraph::new(status_text).block(
                    Block::default()
                        .title("status")
                        .padding(Padding::uniform(1))
                        .borders(Borders::ALL),
                );

                // let pivot_instr = self.instruction_log.iter().find(|(addr,_)| *addr == cpu_addr);

                let trace_text: Text<'_> = Text::from(
                    d.instruction_log
                        .iter()
                        .map(|(addr, inst)| {
                            let inst = format!("{:04X} {}", *addr, inst);

                            // if matches!(Some((*addr,)), cpu.instruction) {
                            //     Line::styled(inst, Style::default().fg(Color::Green))
                            // } else {
                            //     Line::raw(inst)
                            // }

                            if cpu.pc == *addr {
                                Line::styled(inst, Style::default().fg(Color::Green))
                            } else {
                                Line::raw(inst)
                            }

                            // match cpu.instruction {
                            //     Some((cpu_addr, _)) if cpu_addr == *addr => {
                            //         Line::styled(inst, Style::default().fg(Color::Green))
                            //     }
                            //     _ => Line::raw(inst),
                            // }
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
                    let pos = d
                        .instruction_log
                        .iter()
                        .position(|(addr, _)| cpu.pc == *addr)
                        .unwrap_or_default();

                    if pos > (trace_lines / 2) {
                        pos - (trace_lines / 2)
                    } else {
                        pos
                    }
                } as u16;

                let trace = Paragraph::new(trace_text)
                    .scroll((trace_scroll_pos, 0))
                    .block(
                        Block::default()
                            .title("trace")
                            .padding(Padding::uniform(1))
                            .borders(Borders::ALL),
                    );

                // Display memory as rows of 8 bytes indexed by address
                let stack_text = Line::from(
                    d.bus.lock().mem.0[0x100..=0x1FF]
                        .iter()
                        .enumerate()
                        .map(|(idx, byte)| {
                            let byte_text = format!("{:02X} ", byte);
                            if cpu.sp == idx as u8 {
                                Span::styled(
                                    byte_text,
                                    Style::default().bg(Color::Yellow).fg(Color::White),
                                )
                            } else {
                                Span::from(byte_text)
                            }
                        })
                        .collect::<Vec<Span<'_>>>(),
                );

                let stack = Paragraph::new(stack_text).wrap(Wrap { trim: true }).block(
                    Block::default()
                        .title("stack")
                        .padding(Padding::uniform(1))
                        .borders(Borders::ALL),
                );

                // Display memory as rows of 8 bytes indexed by address
                let mem_text = Text::from(
                    d.bus
                        .lock()
                        .mem
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
                frame.render_widget(stack, right_layout[0]);
                frame.render_widget(mem, right_layout[1]);
                frame.render_widget(command, outer_layout[1]);
            })?;

            drop(cpu);
            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        break;
                    } else if key.kind == event::KeyEventKind::Press
                        && key.code == KeyCode::Char('n')
                    {
                        d.step();
                    } else if key.kind == event::KeyEventKind::Press
                        && key.code == KeyCode::Char('r')
                    {
                        d.reset();
                    } else if key.kind == event::KeyEventKind::Press
                        && key.code == KeyCode::Char('g')
                    {
                        d.run();
                    } else if key.kind == event::KeyEventKind::Press
                        && key.code == KeyCode::Char('s')
                    {
                        d.pause();
                    }
                }
            }
        }

        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }
}
