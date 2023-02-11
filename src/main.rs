#[macro_use] extern crate log;
extern crate simplelog;

use std::io::Read;
use std::fs::File;
use simplelog::*;
use clap::{Command, Arg, ArgAction};

pub mod consts;
mod rom_parser;
mod tests;

// use consts::*;
use rom_parser::Rom;

fn main() {
    let args = Command::new("gbemulator")
    .arg(Arg::new("rom_file")
        .short('f')
        .long("rom-file")
        .required(true))
    .arg(Arg::new("verbose")
        .short('v')
        .long("verbose")
        .action(ArgAction::Count))
    .get_matches();

    let log_level: LevelFilter = match args.get_count("verbose") {
        1 => LevelFilter::Debug,
        2 => LevelFilter::Trace,
        _ => LevelFilter::Info
    };

    CombinedLogger::init(
        vec![
            TermLogger::new(log_level, Config::default(), TerminalMode::Mixed, ColorChoice::Auto)
        ]
    ).unwrap();

    // Print ascii art
    info!("\n   _____ ____                       _       _             \n  / ____|  _ \\                     | |     | |            \n | |  __| |_) | ___ _ __ ___  _   _| | __ _| |_ ___  _ __ \n | | |_ |  _ < / _ \\ \'_ ` _ \\| | | | |/ _` | __/ _ \\| \'__|\n | |__| | |_) |  __/ | | | | | |_| | | (_| | || (_) | |   \n  \\_____|____/ \\___|_| |_| |_|\\__,_|_|\\__,_|\\__\\___/|_|   \n                                                          \n                                                          \n");

    // Parse rom file
    let rom_file_path: &String = args.get_one("rom_file").expect("Failed getting rom_file_path");
    debug!("Loading rom from \"{}\"", rom_file_path);
    let rom_file: File = File::open(rom_file_path).expect("Failed opening rom file");

    let rom_content: Vec<u8> = rom_file.bytes().map(|value| {
        value.expect("Failed reading rom file")
    }).collect();

    let rom: Rom = Rom::create_from_bytes(rom_content);
    debug!("Loaded rom with title {}, CGB flag is 0x{:2X}", rom.title, rom.cgb_flag);
    // trace!("Rom size is 0x{:4X}", rom.rom_size);
    // trace!("Ram size is 0x{:4X}", rom.ram_size);
}
