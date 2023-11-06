use sdl2::rect::Rect;

use crate::io::IO;
use std::{iter::FromIterator, time::Duration, sync::{Arc, Mutex}};

/// Display from Easy6502
/// 32x32 pixels
///
/// https://skilldrick.github.io/easy6502/
///
/// Responds to addresses $0200 - $05ff.
pub struct Display {
    buffer: Arc<Mutex<[u8; 32 * 32]>>,
}

impl Display {
    pub fn new() -> Self {
        Display {
            buffer: Arc::new(Mutex::new([0; 32 * 32])),
        }
    }

    pub fn flush(&mut self, data: &[u8]) {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.clone_from_slice(data);
    }

    pub fn show(&mut self) {
        use sdl2::event::Event;
        use sdl2::keyboard::Keycode;
        use sdl2::pixels::Color;

        let buffer = self.buffer.clone();
        std::thread::spawn(move || {
            let sdl_context = sdl2::init().unwrap();
            let video_subsystem = sdl_context.video().unwrap();
            let window = video_subsystem
                .window("6502 Emulator", 512, 512)
                .position_centered()
                .build()
                .unwrap();

            let mut canvas = window.into_canvas().build().unwrap();
            canvas.set_draw_color(Color::RGB(255, 255, 255));
            canvas.clear();
            canvas.present();

            let mut event_pump = sdl_context.event_pump().unwrap();

            'running: loop {
                // i = (i + 1) % 255;
                // canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
                // canvas.clear();
                canvas.set_draw_color(Color::RGB(255, 255, 255));
                
                let buffer = buffer.lock().unwrap();
                for i in 0..buffer.len() {
                    if buffer[i] == 0 {
                        canvas.set_draw_color(Color::BLACK);
                    } else {
                        canvas.set_draw_color(Color::WHITE);

                    }
                    let x_pos = ((((i) % 32) * 1) * 16) as i32;
                    let y_pos = ((((i as f64) / 32.0).floor() as usize) * 16) as i32;

                    let pixel = Rect::new(x_pos, y_pos, 16, 16);
                    let _ = canvas.draw_rect(pixel);
                    let _ = canvas.fill_rect(pixel);
                }
                drop(buffer);   


                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                            keycode: Some(Keycode::Escape),
                            ..
                        } => break 'running,
                        _ => {}
                    }
                }

                canvas.present();
                ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
            }
        });
    }
}

impl IO for Display {
    fn read(&mut self, _addr: u16) -> u8 {
       0
    }
    fn write(&mut self, addr: u16, data: u8) {
        self.buffer.lock().unwrap()[addr as usize] = data
    }
}
