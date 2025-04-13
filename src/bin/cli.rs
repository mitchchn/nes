use std::{
    borrow::BorrowMut,
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use clap::Parser;
use nes::{debugger::Debugger, display::Display, tui::Tui};

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
    let rom = if let Some(arg) = args.file {
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
        vec![0xa9, 0x69, 0x48, 0xa9, 0x42, 0x48, 0xa9, 0xbb, 0x48]
    };

    let mut d = Debugger::new();

    // d.load(&rom, 0xC000);
    // d.load(&rom, 0xFFFF-255);
    // d.load(&rom, 0x8000);
    d.load(&rom, 0);
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
