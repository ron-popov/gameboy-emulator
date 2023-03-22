use crate::consts::*;
use crate::ram_memory::RamMemory;
use std::rc::Rc;
use std::cell::RefCell;
use minifb::{Window, WindowOptions, Scale};

use bmp::{Image, Pixel};

type Sprite = [u8; 16]; // Sprite as represented in VRAM
type SpriteBitmap = [u32; 64]; // Sprite as 64 (8 by 8) pixels

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


        // Why render before initializing the ppu ?
        window.update_with_buffer(
            &get_empty_screen_buffer(), 
            SCREEN_WIDTH, SCREEN_HEIGHT).unwrap_or_else(|e| {
            panic!("Failed rendering window due to error ({})", e);
        });

        PPU {
            buffer: get_empty_screen_buffer(),
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
                            &get_empty_screen_buffer(), 
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

        debug!("PPU: Getting sprite id {}", tile_id);

        let tile_addr: u16 = 0x8000 + 16 * tile_id as u16;
        
        let mut sprite: Sprite = [0 as u8; 16];
        for x in 0..16 {
            sprite[x] = self.ram_memory.borrow_mut().get_addr(tile_addr + x as u16);
        }

        return sprite;
    }

    fn sprite_to_bitmap(sprite: Sprite) -> SpriteBitmap {
        let mut sprite_bitmap: SpriteBitmap = [0x00ffffff; 64];
        let mut counter: usize = 0;
        for pair in sprite.chunks(2) {
            let first = pair[0];
            let second = pair[1];

            for bit in (0..8).rev() {
                let color_code: u8 = match [bit_check(first, bit), bit_check(second, bit)] {
                    [false, false] => 0, 
                    [true, false] => 1,
                    [false, true] => 2,
                    [true, true] => 3,
                    _ => panic!("PPU: Invalid types in sprite_to_bitmap")
                };

                sprite_bitmap[counter] = Self::get_color_by_color_pallete(color_code);

                counter += 1;
            }
        }

        return sprite_bitmap;
    }

    fn draw_sprite_in_buffer(&mut self, sprite: SpriteBitmap, x: usize, y: usize) {
        let mut new_y = y;
        for row in sprite.chunks(8) {
            for (index, pixel) in row.iter().enumerate() {
                let pixel_index = new_y * SCREEN_WIDTH + x + index;
                self.buffer[pixel_index] = *pixel;
            }
            new_y += 1;
        }
    }

    fn get_color_by_color_pallete(color_code: u8) -> u32 {
        match color_code {
            0 => { //WHITE
                return COLOR_WHITE;
            },
            1 => {
                return COLOR_LIGHT_GREY;
            },
            2 => {
                return COLOR_DARK_GREY;
            },
            3 => {
                return COLOR_BLACK;
            },
            _ => panic!("Invalid color code")
        }
    }

    // Dumps all sprites to a file
    pub fn dump_sprites(&self) {
        let mut img = Image::new(256 ,256);

        for sprite_id in 0..=255 {
            let sprite: Sprite = self.get_sprite_tile(sprite_id);
            let sprite_bitmap: SpriteBitmap = Self::sprite_to_bitmap(sprite);

            let initial_x = (sprite_id % 8) * 8;
            let initial_y = (sprite_id / 8) * 8;
            for (index, argb) in sprite_bitmap.iter().enumerate() {
                let pixel = match *argb {
                    COLOR_WHITE => Pixel::new(0xff,0xff,0xff),
                    COLOR_LIGHT_GREY => Pixel::new(0xaa,0xaa,0xaa),
                    COLOR_DARK_GREY => Pixel::new(0x55,0x55,0x55),
                    COLOR_BLACK => Pixel::new(0x00,0x00,0x00),
                    _ => panic!("PPU: Sprite Dump - Invalid color code")
                };

                let final_x = initial_x + (index as u8 % 8);
                let final_y = initial_y + (index as u8 / 8);

                img.set_pixel(final_x as u32, final_y as u32, pixel);
            }
        }

        let save_result = img.save(SPRITE_DUMP_PATH);
        match save_result {
            Ok(_) => (),
            Err(e) => {
                error!("Failed saving memory dump ({})", e);
            }
        }
    }

    pub fn render(&mut self){
        if self.is_enabled {
            // Render all frames
            // for sprite_id in 0..=255 {
                
            // }

            trace!("PPU: Rendering frame");
            self.window.update_with_buffer(&self.buffer, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap_or_else(|e| {
                panic!("Failed rendering window due to error ({})", e);
            });
        } else {
            trace!("PPU: Window disabled, not rendering")
        }

        self.dump_sprites();
    }
}