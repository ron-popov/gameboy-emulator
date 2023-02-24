#[macro_use] extern crate log;
extern crate simplelog;

use std::sync::Arc;
use std::{io::Read};
use std::fs::File;
use std::mem::forget;
use simplelog::*;
use clap::{Command, Arg, ArgAction};

mod ram_memory;
mod rom_parser;
mod consts;
mod param;
mod tests;
mod cpu;
mod opcodes;
mod ppu;

// use consts::*;
use rom_parser::Rom;
use ram_memory::RamMemory;
use cpu::CPU;

use crate::{ppu::PPU, consts::DMG_BOOT_ROM};

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
    .arg(Arg::new("boot_rom")
        .short('b')
        .long("boot-rom")
        .action(ArgAction::SetTrue))
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
    info!("Loading rom \"{}\"", rom.title);

    let orig_ram_memory = RamMemory::init_from_rom(&rom);
    let mut ram_memory_arc: Arc<RamMemory> = Arc::new(orig_ram_memory);

    
    let orig_ppu: PPU = PPU::init();
    let mut ppu_arc: Arc<PPU> = Arc::new(orig_ppu);
    
    let mut cpu: CPU = CPU::init_from_rom(ram_memory_arc.clone(), ppu_arc.clone());
    
    // Init boot rom
    if args.get_flag("boot_rom") {
        let ram_memory = Arc::get_mut(&mut ram_memory_arc).expect("Failed getting mut ram_memory for writing boot rom");
        for (i,x) in DMG_BOOT_ROM.iter().enumerate() {
            ram_memory.set_addr(i as u16, *x);
        }
        forget(ram_memory);
    }

    loop {
        // Only run boot rom for now
        // if cpu.get_program_counter() == 0x0100 {
            // panic!("No more boot rom");
        // }

        // Execute a single cpu instruction
        // cpu.execute_instruction();

        // Render screen (if needed)
        let ppu: &mut PPU = Arc::get_mut(&mut ppu_arc).expect("Failed getting mut ppu for rendering screen");
        ppu.render();
    }
}


// TODO: Use refcells