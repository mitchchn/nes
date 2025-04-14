use std::{
    borrow::BorrowMut,
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use clap::Parser;
use nes::{machine::Machine, tui::Tui};

/// 6502 CPU Emulator and Debugger
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to binary file to load into memory
    file: Option<PathBuf>,
    #[arg(long, short)]
    /// Run until halted (non-interactive)
    run: bool,
    /// Verbose (print stats)
    #[arg(long, short)]
    verbose: bool,
    /// Serial port device
    #[arg(long, short)]
    port: Option<PathBuf>,
    /// Run at the maximum possible speed
    #[arg(long, short)]
    maxspeed: bool,
    /// Start address (PC)
    #[arg(long, short)]
    start: Option<String>,
}

pub fn main() {
    let args: Args = Args::parse();
    let rom = if let Some(arg) = &args.file {
        fs::read(arg).expect("Usage: debugger [FILENAME]")
    } else {
        // vec![
        //     0xa9, 0x00, 0xa2, 0x08, 0x4e, 0x34, 0x12, 0x90, 0x04, 0x18, 0x6d, 0xff, 0xff, 0x6a,
        //     0x6e, 0x34, 0x12, 0xca, 0xd0, 0xf3, 0x8d, 0x12, 0x34, 0xad, 0x34, 0x12, 0x60,
        // ]
        // : Option<String>vec![
        //     0xa9, 0x01, 0x8d, 0x00, 0x02, 0xa9, 0x05, 0x8d, 0x01, 0x02, 0xa9, 0x08, 0x8d, 0x02, 0x02
        // ]
        // vec![0xa9, 0x01, 0xa2, 0x00, 0x9d, 0x00, 0x02, 0xe8, 0x10, 0xfa]
        // vec![0xa2, 0x00, 0xa9, 0x01, 0x9d, 0x00, 0x02, 0xe8, 0x10, 0xfa]
        // vec![
        //     0xa2, 0x00, 0xa9, 0x01, 0x9d, 0x00, 0x02, 0xa4, 0xff, 0x88, 0xd0, 0xfd, 0xe8, 0x10,
        //     0xf5,
        // ]
        // vec![0xa9, 0x69, 0x48, 0xa9, 0x42, 0x48, 0xa9, 0xbb, 0x48]

        // Snake!
        vec![
            0x20, 0x06, 0x06, 0x20, 0x38, 0x06, 0x20, 0x0d, 0x06, 0x20, 0x2a, 0x06, 0x60, 0xa9,
            0x02, 0x85, 0x02, 0xa9, 0x04, 0x85, 0x03, 0xa9, 0x11, 0x85, 0x10, 0xa9, 0x10, 0x85,
            0x12, 0xa9, 0x0f, 0x85, 0x14, 0xa9, 0x04, 0x85, 0x11, 0x85, 0x13, 0x85, 0x15, 0x60,
            0xa5, 0xfe, 0x85, 0x00, 0xa5, 0xfe, 0x29, 0x03, 0x18, 0x69, 0x02, 0x85, 0x01, 0x60,
            0x20, 0x4d, 0x06, 0x20, 0x8d, 0x06, 0x20, 0xc3, 0x06, 0x20, 0x19, 0x07, 0x20, 0x20,
            0x07, 0x20, 0x2d, 0x07, 0x4c, 0x38, 0x06, 0xa5, 0xff, 0xc9, 0x77, 0xf0, 0x0d, 0xc9,
            0x64, 0xf0, 0x14, 0xc9, 0x73, 0xf0, 0x1b, 0xc9, 0x61, 0xf0, 0x22, 0x60, 0xa9, 0x04,
            0x24, 0x02, 0xd0, 0x26, 0xa9, 0x01, 0x85, 0x02, 0x60, 0xa9, 0x08, 0x24, 0x02, 0xd0,
            0x1b, 0xa9, 0x02, 0x85, 0x02, 0x60, 0xa9, 0x01, 0x24, 0x02, 0xd0, 0x10, 0xa9, 0x04,
            0x85, 0x02, 0x60, 0xa9, 0x02, 0x24, 0x02, 0xd0, 0x05, 0xa9, 0x08, 0x85, 0x02, 0x60,
            0x60, 0x20, 0x94, 0x06, 0x20, 0xa8, 0x06, 0x60, 0xa5, 0x00, 0xc5, 0x10, 0xd0, 0x0d,
            0xa5, 0x01, 0xc5, 0x11, 0xd0, 0x07, 0xe6, 0x03, 0xe6, 0x03, 0x20, 0x2a, 0x06, 0x60,
            0xa2, 0x02, 0xb5, 0x10, 0xc5, 0x10, 0xd0, 0x06, 0xb5, 0x11, 0xc5, 0x11, 0xf0, 0x09,
            0xe8, 0xe8, 0xe4, 0x03, 0xf0, 0x06, 0x4c, 0xaa, 0x06, 0x4c, 0x35, 0x07, 0x60, 0xa6,
            0x03, 0xca, 0x8a, 0xb5, 0x10, 0x95, 0x12, 0xca, 0x10, 0xf9, 0xa5, 0x02, 0x4a, 0xb0,
            0x09, 0x4a, 0xb0, 0x19, 0x4a, 0xb0, 0x1f, 0x4a, 0xb0, 0x2f, 0xa5, 0x10, 0x38, 0xe9,
            0x20, 0x85, 0x10, 0x90, 0x01, 0x60, 0xc6, 0x11, 0xa9, 0x01, 0xc5, 0x11, 0xf0, 0x28,
            0x60, 0xe6, 0x10, 0xa9, 0x1f, 0x24, 0x10, 0xf0, 0x1f, 0x60, 0xa5, 0x10, 0x18, 0x69,
            0x20, 0x85, 0x10, 0xb0, 0x01, 0x60, 0xe6, 0x11, 0xa9, 0x06, 0xc5, 0x11, 0xf0, 0x0c,
            0x60, 0xc6, 0x10, 0xa5, 0x10, 0x29, 0x1f, 0xc9, 0x1f, 0xf0, 0x01, 0x60, 0x4c, 0x35,
            0x07, 0xa0, 0x00, 0xa5, 0xfe, 0x91, 0x00, 0x60, 0xa6, 0x03, 0xa9, 0x00, 0x81, 0x10,
            0xa2, 0x00, 0xa9, 0x01, 0x81, 0x10, 0x60, 0xa2, 0x00, 0xea, 0xea, 0xca, 0xd0, 0xfb,
            0x60,
        ]
    };

    let mut d = Machine::new();

    // d.load(&rom, 0xC000);
    // d.load(&rom, 0xFFFF-255);
    // d.load(&rom, 0x8000);
    // d.load(&rom, 0);
    if args.file.is_some() {
        d.load_cart(&rom);
    } else {
        d.load(&rom, 0x600);
    }

    d.reset();
    // d.cpu.lock().pc = 0x400;
    // d.cpu.lock().pc = 0x4000;

    if let Some(start) = args.start {
        let start = start.strip_prefix("0x").unwrap_or(&start);
        d.cpu.lock().pc = u16::from_str_radix(&start, 16).unwrap_or_default();
    }

    if args.maxspeed {
        d.max_speed = true;
    }

    if args.run {
        d.non_interactive_mode = true;

        let start: SystemTime = SystemTime::now();
        let handle = d.run();
        handle.unwrap().join();

        let end = SystemTime::now().duration_since(start).unwrap();

        if args.verbose {
            let cpu = d.cpu.lock();
            println!("\n---");
            println!("Total cycles: \t\t{}", cpu.cycles());
            println!("Total instructions: \t{}", cpu.instructions);
            println!("Halted in {}.{}s.", end.as_secs(), end.subsec_millis());
        }
    } else {
        let mut tui = Tui::new(d);
        let _ = tui.show();
    }
}
