use std::{io::stdout, sync::Arc};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use parking_lot::Mutex;
use ratatui::{prelude::*, widgets::*, Terminal};

use crate::{cpu::Status, machine::Machine};

pub struct Tui {
    machine: Arc<Mutex<Machine>>,
}

impl Tui {
    pub fn new(machine: Machine) -> Self {
        Self {
            machine: Arc::new(Mutex::new(machine)),
        }
    }

    pub fn show(&mut self) -> std::io::Result<()> {
        let d = self.machine.clone();

        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        let t: std::thread::JoinHandle<()> = std::thread::spawn(move || {
            loop {
                let mut d = d.lock();
                let cpu = d.cpu.lock();
                // d.flush(&    self.mem.lock().unwrap().0[0x200..=0x5FF]);

                terminal
                    .draw(|frame: &mut Frame| {
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
                            .constraints([Constraint::Length(100)])
                            .split(main_layout[0]);

                        let right_layout = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([Constraint::Length(21), Constraint::Min(0)])
                            .split(main_layout[1]);

                        let right_layout_inner = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([Constraint::Length(21), Constraint::Min(0)])
                            .split(right_layout[0]);

                        let color_flag = |f: u8, ch: &str| {
                            if f == 1 {
                                Span::from(format!("{} ", ch).fg(Color::Green).bold())
                            } else {
                                Span::raw(format!("{} ", ch).to_string()).fg(Color::Gray)
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
                            "PC    A  X  Y    SP",
                            Style::default().fg(Color::Magenta).bold(),
                        );
                        let status_line = Line::from(vec![format!(
                            "{:04X}  {:02X} {:02X} {:02X}   {:02X}    ",
                            cpu.pc, cpu.a, cpu.x, cpu.y, cpu.sp
                        )
                        .into()]);

                        let flag_line = Line::from(vec![
                            color_flag(f[0], "N"),
                            color_flag(f[1], "V"),
                            color_flag(f[2], "-"),
                            color_flag(f[3], "B"),
                            color_flag(f[4], "D"),
                            color_flag(f[5], "I"),
                            color_flag(f[6], "Z"),
                            color_flag(f[7], "C"),
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

                        let cycles_line = Line::from(vec![
                            "Cycles: ".fg(Color::Gray),
                            format!("{}", cpu.cycles).into(),
                        ]);
                        let status_text = vec![
                            status_header,
                            status_line,
                            Line::from(""),
                            flag_line,
                            Line::from(""),
                            cycles_line,
                        ];
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
                        let trace_lines = (left_layout[0].height - 4) as usize;
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

                        let mut stack_lines = vec![];

                        // Stack (ascending order so reverse from actual memory layout)
                        for (idx, byte) in cpu.mem.mem.0[0x100..=0x1FF].iter().rev().enumerate() {
                            let addr = 0xff - idx;
                            let byte_text = format!("{:02X}", byte);
                            stack_lines.push(Line::from(vec![
                                format!("{:02X}: ", addr).dark_gray(),
                                if cpu.sp == addr as u8 {
                                    Span::styled(
                                        byte_text,
                                        Style::default().bg(Color::Yellow).fg(Color::White),
                                    )
                                } else {
                                    byte_text.into()
                                },
                            ]));
                        }

                        // Display memory as rows of 8 bytes indexed by address
                        // let stack_text = Line::from(
                        //     cpu.mem.mem.0[0x100..=0x1FF]
                        //         .iter()
                        //         .enumerate()
                        //         .map(|(idx, byte)| {
                        //             let byte_text = format!("{:02X} ", byte);
                        //             if cpu.sp == idx as u8 {
                        //                 Span::styled(
                        //                     byte_text,
                        //                     Style::default().bg(Color::Yellow).fg(Color::White),
                        //                 )
                        //             } else {
                        //                 Span::from(byte_text)
                        //             }
                        //         })
                        //         .collect::<Vec<Span<'_>>>(),
                        // );

                        let stack = Paragraph::new(stack_lines).wrap(Wrap { trim: true }).block(
                            Block::default()
                                .title("stack")
                                .padding(Padding::uniform(1))
                                .borders(Borders::ALL),
                        );

                        // Display memory as rows of 8 bytes indexed by address
                        let mem_text = Text::from(
                            cpu.mem
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

                        frame.render_widget(trace, left_layout[0]);
                        frame.render_widget(stack, right_layout_inner[0]);
                        frame.render_widget(status, right_layout_inner[1]);
                        frame.render_widget(mem, right_layout[1]);
                        frame.render_widget(command, outer_layout[1]);
                    })
                    .unwrap();

                drop(cpu);
                if event::poll(std::time::Duration::from_millis(50)).unwrap() {
                    if let Event::Key(key) = event::read().unwrap() {
                        if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q')
                        {
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
        });

        let d = self.machine.clone();
        let c = d.lock().cpu.clone();

        #[cfg(feature = "sdl")]
        {
            use crate::display::Display;
            let mut display = Display::new(c);
            display.show();
            t.join();
        }

        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }
}
