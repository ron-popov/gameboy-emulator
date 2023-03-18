use crate::consts::*;
use crate::ram_memory::RamMemory;
use std::rc::Rc;
use std::cell::RefCell;
use minifb::{Window, WindowOptions, Scale};


type Sprite = [u8; 16];
type SpriteBitmap = [u32; 64];

pub struct PPU {
    buffer: Vec<u32>,
    window: Window,
    ram_memory: Rc<RefCell<RamMemory>>,
    is_enabled: bool
}

impl PPU {
    pub fn init(ram_memory_ref: Rc<RefCell<RamMemory>>) -> PPU {
        // Configure scale
        let mut window_options: WindowOptions = WindowOptions::default();
        window_options.scale = Scale::X4;

        let mut window = Window::new(
            "GBEmulator",
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            window_options,
        ).unwrap_or_else(|e| {
            panic!("Failed creating minifb window ({})", e);
        });

        // Limit FPS to about 60FPS
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        window.update_with_buffer(
            &get_empty_screen_buffer(SCREEN_WIDTH, SCREEN_HEIGHT), 
            SCREEN_WIDTH, SCREEN_HEIGHT).unwrap_or_else(|e| {
            panic!("Failed rendering window due to error ({})", e);
        });

        PPU {
            buffer: [0; SCREEN_HEIGHT * SCREEN_WIDTH].to_vec(),
            window: window,
            ram_memory: ram_memory_ref,
            is_enabled: false
        }
    }

    pub fn set_addr(&mut self, addr: u16, value: u8) {
        info!("PPU: Write to ppu addr 0x{:04X} -> 0x{:02X}", addr ,value);
        let should_write_to_ram_memory: bool = true;

        if PPU_JOYPAD_INPUT_ADDR.contains(&addr) {
            // TODO
        } else if PPU_SERIAL_ADDR.contains(&addr) {
            // TODO
        } else if PPU_TIMER_DIVIDER_ADDR.contains(&addr) {
            // TODO
        } else if PPU_AUDIO_ADDR.contains(&addr) {
            // TODO
        } else if PPU_WAVE_ADDR.contains(&addr) {
            // TODO
        } else if PPU_LCD_ADDR.contains(&addr) {
            self.lcd_control_set_handler(addr, value);
        }

        if should_write_to_ram_memory {
            self.ram_memory.borrow_mut().set_addr(addr, value);
        }
    }

    pub fn get_addr(&self, addr: u16) -> u8 {
        self.lcd_control_get_handler(addr);
        self.ram_memory.borrow().get_addr(addr)
    }

    fn lcd_control_set_handler(&mut self, addr: u16, value: u8) {
        match addr {
            PPU_LCD_CONTROL_ADDR => {
                // Control bit
                if bit_check(value, PPU_LCD_CONTROL_BIT_ENABLE) {
                    if !self.is_enabled {
                        trace!("PPU: Enabling lcd display");
                        self.is_enabled = true;
                    }
                } else {
                    if self.is_enabled { // Disable only if screen is enabled
                        trace!("PPU: Disabling lcd display");
                        self.window.update_with_buffer(
                            &get_empty_screen_buffer(SCREEN_WIDTH, SCREEN_HEIGHT), 
                            SCREEN_WIDTH, SCREEN_HEIGHT).unwrap_or_else(|e| {
                                panic!("PPU: Failed rendering window due to error ({})", e);
                        });
    
                        self.is_enabled = false;
                    }
                }
            },
            _ => warn!("PPU: lcd_control_set_handler was called with an invalid memory addr (0x{:04X})", addr)
        }
    }

    fn lcd_control_get_handler(&self, addr: u16) {
        if addr == 0xFF44 {
            self.ram_memory.borrow_mut().set_addr(addr, 0x90);
        }
    }

    fn get_sprite_tile(&self, tile_id: u8) -> Sprite {
        // TODO: Check sprite map addr
        // let control_reg = self.ram_memory.borrow_mut().get_addr(PPU_LCD_CONTROL_ADDR);
        // if bit_check(control_reg, PPU_LCD_CONTROL_BIT_BG_TILE_MAP_AREA);

        let tile_addr: u16 = 0x8000 + 16 * tile_id as u16;
        
        let mut sprite: Sprite = [0 as u8; 16];
        for x in 0..16 {
            sprite[x] = self.ram_memory.borrow_mut().get_addr(tile_addr + x as u16);
        }

        return sprite;
    }

    // fn sprite_to_bitmap(sprite: Sprite) -> SpriteBitmap {

    // }

    pub fn render(&mut self){
        if self.is_enabled {
            trace!("PPU: Rendering frame");
            self.window.update_with_buffer(&self.buffer, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap_or_else(|e| {
                panic!("Failed rendering window due to error ({})", e);
            });
        } else {
            trace!("PPU: Window disabled, not rendering")
        }
    }
}