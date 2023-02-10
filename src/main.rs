use std::io::Read;
use std::fs::File;

pub mod consts;
mod rom_parser;

pub use consts::*;
use rom_parser::Rom;

fn main() {
    let rom_file: File = File::open("roms/bully.gb").expect("Failed opening rom file");

    let rom_content: Vec<u8> = rom_file.bytes().map(|value| {
        value.expect("Failed reading rom file")
    }).collect();

    let rom: Rom = Rom::create_from_bytes(rom_content);
    println!("Loaded rom {}", rom.get_title());
}
