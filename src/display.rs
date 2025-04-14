use parking_lot::Mutex;
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::PixelFormatEnum,
    rect::Rect,
    render::{Canvas, Texture},
    video::Window,
    EventPump,
};

use crate::{bus::Bus, cpu::CPU6502, io::IO};
use std::{
    iter::FromIterator,
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Duration,
};

/// Display from Easy6502
/// 32x32 pixels
///
/// https://skilldrick.github.io/easy6502/
///
/// Responds to addresses $0200 - $05ff.
pub struct Display {
    // buffer: Arc<Mutex<[u8; 32 * 32]>>,
    buffer: Arc<Mutex<dyn IO>>,
}

impl Display {
    pub fn new(buffer: Arc<Mutex<CPU6502<Bus>>>) -> Self {
        Display {
            buffer, // buffer: Arc::new(Mutex::new([0; 32 * 32])),
        }
    }

    // pub fn flush(&mut self, data: &[u8]) {
    //     let mut buffer = self.buffer.lock().unwrap();
    //     buffer.clone_from_slice(data);
    // }

    pub fn show(&mut self) {
        use sdl2::event::Event;
        use sdl2::keyboard::Keycode;
        use sdl2::pixels::Color;
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("Snake game", (32.0 * 10.0) as u32, (32.0 * 10.0) as u32)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().present_vsync().build().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
        canvas.set_scale(10.0, 10.0).unwrap();

        let creator = canvas.texture_creator();
        let mut texture = creator
            .create_texture_target(PixelFormatEnum::RGB24, 32, 32)
            .unwrap();

        let mut screen_state = [0 as u8; 32 * 3 * 32];

        'running: loop {
            let mut cpu = self.buffer.lock();

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => std::process::exit(0),
                    Event::KeyDown {
                        keycode: Some(Keycode::Up),
                        ..
                    } => {
                        cpu.write(0xff, 0x77);
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Down),
                        ..
                    } => {
                        cpu.write(0xff, 0x73);
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Left),
                        ..
                    } => {
                        cpu.write(0xff, 0x61);
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Right),
                        ..
                    } => {
                        cpu.write(0xff, 0x64);
                    }
                    _ => { /* do nothing */ }
                }

                // match event {
                //     Event::Quit { .. }
                //     | Event::KeyDown {
                //         keycode: Some(Keycode::Escape),
                //         ..
                //     } => break 'running,
                //     _ => {}
                // }
            }

            fn color(byte: u8) -> Color {
                match byte {
                    0 => sdl2::pixels::Color::BLACK,
                    1 => sdl2::pixels::Color::WHITE,
                    2 | 9 => sdl2::pixels::Color::GREY,
                    3 | 10 => sdl2::pixels::Color::RED,
                    4 | 11 => sdl2::pixels::Color::GREEN,
                    5 | 12 => sdl2::pixels::Color::BLUE,
                    6 | 13 => sdl2::pixels::Color::MAGENTA,
                    7 | 14 => sdl2::pixels::Color::YELLOW,
                    _ => sdl2::pixels::Color::CYAN,
                }
            }

            fn read_screen_state(
                cpu: &mut impl DerefMut<Target = dyn IO>,
                frame: &mut [u8; 32 * 3 * 32],
            ) -> bool {
                let mut frame_idx = 0;
                let mut update = false;
                for i in 0x0200..0x600 {
                    let color_idx = cpu.read(i as u16);
                    let (b1, b2, b3) = color(color_idx).rgb();
                    if frame[frame_idx] != b1
                        || frame[frame_idx + 1] != b2
                        || frame[frame_idx + 2] != b3
                    {
                        frame[frame_idx] = b1;
                        frame[frame_idx + 1] = b2;
                        frame[frame_idx + 2] = b3;
                        update = true;
                    }
                    frame_idx += 3;
                }
                update
            }

            if read_screen_state(&mut cpu, &mut screen_state) {
                texture.update(None, &screen_state, 32 * 3).unwrap();
                canvas.copy(&texture, None, None).unwrap();
                canvas.present();
            }

            // canvas.present();
            // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}

impl IO for Display {
    fn read(&mut self, _addr: u16) -> u8 {
        0
    }
    fn write(&mut self, addr: u16, data: u8) {
        // self.buffer.lock().unwrap()[addr as usize] = data
    }
}
