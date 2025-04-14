use std::{
    ffi::{CStr, CString},
    path::{Path, PathBuf},
};

use rust_libretro::{
    contexts::{GetAvInfoContext, InitContext, RunContext},
    core::CoreOptions,
    retro_core,
    sys::{retro_game_geometry, retro_game_info, retro_system_av_info, retro_system_timing},
    types::SystemInfo,
};

use crate::{bus::Bus, cart::Cart, cpu::CPU6502, mem::Memory, ppu::Ppu};

pub struct Core {
    pixels: Vec<u8>,
    emulator: CPU6502<Bus>,
}

impl CoreOptions for Core {}
impl rust_libretro::core::Core for Core {
    fn get_info(&self) -> SystemInfo {
        SystemInfo {
            library_name: CString::new("Mitch NES").unwrap(),
            library_version: CString::new("1.0.0").unwrap(),
            valid_extensions: CString::new("").unwrap(),
            need_fullpath: false,
            block_extract: false,
        }
    }
    fn on_get_av_info(&mut self, _ctx: &mut GetAvInfoContext) -> retro_system_av_info {
        retro_system_av_info {
            geometry: retro_game_geometry {
                base_width: 256,
                base_height: 240,
                max_width: 256,
                max_height: 240,
                aspect_ratio: 1.28,
            },
            timing: retro_system_timing {
                fps: 60.0,
                sample_rate: 0.0,
            },
        }
    }
    fn on_init(&mut self, ctx: &mut InitContext) {}

    fn on_load_game(
        &mut self,
        game: Option<retro_game_info>,
        ctx: &mut rust_libretro::contexts::LoadGameContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(game) = game {
            let path = unsafe { CStr::from_ptr(game.path) }
                .to_string_lossy()
                .to_string();
            let cart = Cart::load(&PathBuf::from(path)).unwrap();
            self.emulator.mem.load_cart(cart);
            Ok(())
        } else {
            Ok(())
        }
    }

    fn on_run(&mut self, ctx: &mut RunContext, delta_us: Option<i64>) {
        let width = 256u32;
        let height = 240u32;

        let color_a = 0xFF;
        let color_b = 0x80;

        for (i, chunk) in self.pixels.chunks_exact_mut(4).enumerate() {
            let x = (i % width as usize) as f64 / width as f64;
            let y = (i / width as usize) as f64 / height as f64;

            let total = (50.0f64 * x).floor() + (37.5f64 * y).floor();
            let even = total as usize % 2 == 0;

            let color = if even { color_a } else { color_b };

            chunk.fill(color);
        }

        ctx.draw_frame(self.pixels.as_ref(), width, height, width as usize * 4);
    }
}

retro_core!(Core {
    pixels: vec![0; 256 * 240 * 4],
    emulator: {
        let bus = Bus {
            mem: Memory::new(),
            ppu: Ppu::new(),
            stdout: None,
            stdin: None,
            serial: None,
            cart: None,
            rng: None,
        };

        CPU6502::new(bus)
    }
});
