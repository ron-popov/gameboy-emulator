#[macro_use] extern crate log;
extern crate simplelog;

use std::rc::Rc;
use std::cell::RefCell;
use std::{io::Read};
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

use consts::*;
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
    // .arg(Arg::new("ppu_logs_only")
    //     .long("ppu-logs-only")
    //     .action(ArgAction::SetTrue))
    // .arg(Arg::new("cpu_logs_only")
    //     .long("cpu-logs-only")
    //     .action(ArgAction::SetTrue))
    .get_matches();

    let log_level: LevelFilter = match args.get_count("verbose") {
        1 => LevelFilter::Debug,
        2 => LevelFilter::Trace,
        _ => LevelFilter::Info
    };

    let mut logger_config = ConfigBuilder::new();
    logger_config.set_time_level(LevelFilter::Off);
    logger_config.set_target_level(LevelFilter::Off);
    logger_config.set_level_color(Level::Error, None);
    logger_config.set_level_color(Level::Warn, None);
    logger_config.set_level_color(Level::Info, None);
    logger_config.set_level_color(Level::Debug, None);
    logger_config.set_level_color(Level::Trace, None);

    // if args.get_flag("ppu_logs_only") {
    //     logger_config.add_filter_allow("ppu".to_string());
    // } else if args.get_flag("cpu_logs_only") {
    //     logger_config.add_filter_allow("cpu".to_string());
    // }

    CombinedLogger::init(
        vec![
            TermLogger::new(log_level, logger_config.build(), TerminalMode::Mixed, ColorChoice::Auto)
        ]
    ).unwrap();

    // Print ascii art
    info!("{}", GBEMULATOR_ASCII_ART);

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
    let ram_memory_ref: Rc<RefCell<RamMemory>> = Rc::new(RefCell::new(orig_ram_memory));

    
    let orig_ppu: PPU = PPU::init(ram_memory_ref.clone());
    let ppu_ref: Rc<RefCell<PPU>> = Rc::new(RefCell::new(orig_ppu));
    
    let mut cpu: CPU = CPU::init_with_ram_ppu(ram_memory_ref.clone(), ppu_ref.clone(), args.get_flag("boot_rom"));
    
    // Init boot rom
    if args.get_flag("boot_rom") {
        for (i,x) in DMG_BOOT_ROM.iter().enumerate() {
            ram_memory_ref.borrow_mut().set_addr(i as u16, *x);
        }
    }

    loop {
        // Only run boot rom for now
        if args.get_flag("boot_rom") {
            if cpu.get_program_counter() == 0x0100 {
                panic!("No more boot rom");
            }
        }

        // Execute a single cpu instruction
        cpu.execute_instruction();
        cpu.execute_instruction();
        cpu.execute_instruction();
        cpu.execute_instruction();

        // Render screen (if needed)
        ppu_ref.borrow_mut().render();
    }
}

