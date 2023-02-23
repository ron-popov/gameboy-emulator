#[macro_use] extern crate log;
extern crate simplelog;

use std::{io::Read, cell::RefCell};
use std::fs::File;
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
    let ram_memory_refcell: RefCell<RamMemory> = RefCell::new(orig_ram_memory);

    let mut ram_memory = ram_memory_refcell.borrow_mut();
    if args.get_flag("boot_rom") {
        for (i,x) in DMG_BOOT_ROM.iter().enumerate() {
            ram_memory.set_addr(i as u16, *x);
        }
    }

    use std::mem::forget;
    forget(ram_memory);

    let orig_ppu: PPU = PPU::init();
    let ppu_refcell: RefCell<PPU> = RefCell::new(orig_ppu);
    let mut cpu: CPU = CPU::init_from_rom(ram_memory_refcell, &ppu_refcell);

    loop {
        if cpu.get_program_counter() == 0x0100 {
            panic!("No more boot rom");
        }

        cpu.execute_instruction();

        match ppu_refcell.try_borrow_mut() {
            Ok(ppu) => {
                ppu.render();
            },
            Err(_) => {
                panic!("Failed borrowing ppu")
            }
        }
        // let mut temp_ppu = ppu_refcell.try_borrow_mut();
        // temp_ppu.render();
    }
}


// TODO: Use refcells