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
    ram_memory: Rc<RefCell<RamMemory>>
}

impl PPU {
    pub fn init(ram_memory_ref: Rc<RefCell<RamMemory>>) -> PPU {
        // Configure scale
        let mut window_options: WindowOptions = WindowOptions::default();
        window_options.scale = Scale::X2;

        let mut window = Window::new(
            "GBEmulator",
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            window_options,
        ).unwrap_or_else(|e| {
            panic!("Failed creating minifb window ({})", e);
        });

        // Limit FPS to about 60FPS
        // window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        window.limit_update_rate(Some(std::time::Duration::from_micros(1660)));


        // Why render before initializing the ppu ?
        window.update_with_buffer(
            &get_empty_screen_buffer(), 
            SCREEN_WIDTH, SCREEN_HEIGHT).unwrap_or_else(|e| {
            panic!("Failed rendering window due to error ({})", e);
        });

        PPU {
            buffer: get_empty_screen_buffer(),
            window: window,
            ram_memory: ram_memory_ref
        }
    }




    // COLOR & BITMAP STUFF
    fn draw_sprite_in_buffer(&mut self, sprite: Sprite, x:u8, y:u8) {
        let sprite_bitmap: SpriteBitmap = Self::sprite_to_bitmap(sprite);
        self.draw_sprite_bitmap_in_buffer(sprite_bitmap, x.into(), y.into());

    }

    fn draw_sprite_bitmap_in_buffer(&mut self, sprite: SpriteBitmap, x: usize, y: usize) {
        let mut new_y = y;
        for row in sprite.chunks(8) {
            for (index, pixel) in row.iter().enumerate() {
                let pixel_index = new_y * SCREEN_WIDTH + x + index;
                self.buffer[pixel_index] = *pixel;

                if *pixel != 16777215 {
                    trace!("Pixel at #{}", pixel_index);
                }
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

    fn get_sprite_tile(&self, tile_id: u8) -> Sprite {
        let mut tile_addr: u16;
        if self.get_ppu_config("bg_window_data_area") {
            trace!("PPU: Setting background and window sprite area to 0x8000");
            tile_addr = 0x8000;
        } else {
            trace!("PPU: Setting background and window sprite area to 0x8800");
            tile_addr = 0x8800;
        }

        tile_addr += 16 * tile_id as u16;
        trace!("PPU: Tile addr for #{} is 0x{:04X}", tile_id, tile_addr);

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




    // MEMORY STUFF
    pub fn set_addr(&mut self, addr: u16, value: u8) {
        info!("PPU: Write to ppu addr 0x{:04X} -> 0x{:02X}", addr ,value);
        let should_write_to_ram_memory: bool = true;

        if PPU_JOYPAD_INPUT_ADDR.contains(&addr) {
            trace!("PPU: 0x{:04X} is joypad input addr", addr);
        } else if PPU_SERIAL_ADDR.contains(&addr) {
            trace!("PPU: 0x{:04X} is serial addr", addr);
        } else if PPU_TIMER_DIVIDER_ADDR.contains(&addr) {
            trace!("PPU: 0x{:04X} is dimer addr", addr);
        } else if PPU_AUDIO_ADDR.contains(&addr) {
            trace!("PPU: 0x{:04X} is audio addr", addr);
        } else if PPU_WAVE_ADDR.contains(&addr) {
            trace!("PPU: 0x{:04X} is wave addr", addr);
        } else if PPU_LCD_ADDR.contains(&addr) {
            trace!("PPU: 0x{:04X} is lcd control addr", addr);
            self.lcd_control_set_handler(addr, value);
        }

        if should_write_to_ram_memory {
            self.ram_memory.borrow_mut().set_addr(addr, value);
        }
    }

    pub fn get_addr(&self, addr: u16) -> u8 {
        // This will return Option with value if we want to return a custom value (not value in ram)
        let custom_handler_result = self.lcd_control_get_handler(addr);
        if custom_handler_result.is_some() {
            return custom_handler_result.unwrap();
        }


        let ram_value = self.ram_memory.borrow().get_addr(addr);
        return ram_value;
    }

    fn lcd_control_set_handler(&mut self, addr: u16, value: u8) {
        match addr {
            PPU_ADDR_LCD_CONTROL => {
                // 7 -> Control enable bit
                if bit_check(value, PPU_LCD_CONTROL_BIT_ENABLE) {
                    if !self.get_ppu_config("is_enabled") {
                        trace!("PPU: LCD_CONTROL: Enabling lcd display");

                        // The default handler also writes to memory
                        // self.set_ppu_config("is_enabled", true);
                    }
                } else {
                    if self.get_ppu_config("is_enabled") { // Disable only if screen is enabled
                        trace!("PPU: LCD_CONTROL: Disabling lcd display");
                        self.window.update_with_buffer(
                            &get_empty_screen_buffer(), 
                            SCREEN_WIDTH, SCREEN_HEIGHT).unwrap_or_else(|e| {
                                panic!("PPU: LCD_CONTROL: Failed rendering window due to error ({})", e);
                        });
    
                        // The default handler also writes to memory
                        // self.set_ppu_config("is_enabled", false);
                    }
                }



                // 6 - Window tile map area
                if bit_check(value, PPU_LCD_CONTROL_BIT_WINDOW_TILE_MAP_AREA) {
                    trace!("PPU: LCD_CONTROL: Window tile map area is 0x9C00-0x9FFF")
                } else {
                    trace!("PPU: LCD_CONTROL: Window tile map area is 0x9800-0x9BFF")
                }

                // 5 - Window enabled
                if bit_check(value, PPU_LCD_CONTROL_BIT_WINDOW_ENABLE) {
                    trace!("PPU: LCD_CONTROL: Window enabled")
                } else {
                    trace!("PPU: LCD_CONTROL: Window disabled")
                }


                // 4 - BG And Window tile data area
                if bit_check(value, PPU_LCD_CONTROL_BIT_BG_AND_WINDOW_TILE_DATA_AREA) {
                    trace!("PPU: LCD_CONTROL: BG And Window tile data area is 0x8000-0x8FFF")
                } else {
                    trace!("PPU: LCD_CONTROL: BG And Window tile data area is 0x8800-0x97FF")
                }


                // 3 - BG Tile map area
                if bit_check(value, PPU_LCD_CONTROL_BIT_BG_TILE_MAP_AREA) {
                    trace!("PPU: LCD_CONTROL: BG Tile map area is 0x9C00-0x9FFF")
                } else {
                    trace!("PPU: LCD_CONTROL: BG Tile map area is 0x9800-0x9BFF")
                }


                // 2 - Object size
                if bit_check(value, PPU_LCD_CONTROL_BIT_OBJ_SIZE) {
                    trace!("PPU: LCD_CONTROL: Object size is 8x16")
                } else {
                    trace!("PPU: LCD_CONTROL: Object size is 8x8")
                }


                // 1 - Object enabled
                if bit_check(value, PPU_LCD_CONTROL_BIT_OBJ_ENABLE) {
                    trace!("PPU: LCD_CONTROL: Objects are enabled")
                } else {
                    trace!("PPU: LCD_CONTROL: Objects are disabled")
                }


                // 0 - BG And Window priority
                if bit_check(value, PPU_LCD_CONTROL_BIT_BG_AND_WINDOW_PRIORITY) {
                    trace!("PPU: LCD_CONTROL: BG And Window priotity ON")
                } else {
                    trace!("PPU: LCD_CONTROL: BG And Window priotity OFF")
                }
            },
            _ => warn!("PPU: lcd_control_set_handler was called with an unknown memory addr (0x{:04X})", addr)
        }
    }

    fn lcd_control_get_handler(&self, addr: u16) -> Option<u8> {
        // This indicates the screen is in VBLANK - cpu can write period
        // Because the cpu is trying to read this means the ppu is not rendering == VBLANK period
        if addr == 0xFF44 {
            return Some(0x90);
        }
        return None;
    }




    fn get_addr_and_bit(config: &str) -> (u16, u8) {
        return match config {
            "is_enabled"            => (PPU_ADDR_LCD_CONTROL, PPU_LCD_CONTROL_BIT_ENABLE),
            "window_tile_map"       => (PPU_ADDR_LCD_CONTROL, PPU_LCD_CONTROL_BIT_WINDOW_TILE_MAP_AREA),
            "bg_tile_map"           => (PPU_ADDR_LCD_CONTROL, PPU_LCD_CONTROL_BIT_BG_TILE_MAP_AREA),
            "bg_window_data_area"   => (PPU_ADDR_LCD_CONTROL, PPU_LCD_CONTROL_BIT_BG_AND_WINDOW_TILE_DATA_AREA),
            _ => panic!("PPU: Unknown config path requested")
        } 
    }

    fn get_ppu_config(&self, config: &str) -> bool {
        let (addr, bit) = Self::get_addr_and_bit(config);
        return bit_check(self.get_addr(addr), bit);
    }    

    fn set_ppu_config(&mut self, config: &str, value: bool) {
        let (addr, bit) = Self::get_addr_and_bit(config);

        let old_value: u8 = self.get_addr(addr);

        let mut mask: u8 = 0b00000001;
        mask = mask << (bit - 1);

        let new_value: u8;
        if value {
            new_value = old_value | mask;
        } else {
            mask = mask ^ 0xff;
            new_value = old_value & mask;
        }

        self.set_addr(addr, new_value);
    }

    





    pub fn render(&mut self){
        if self.get_ppu_config("is_enabled") && !PPU_DISABLE {
            if PPU_DUMP_SPRITES { // Render all frames
                trace!("PPU: Dumping sprites to screen");
                for sprite_id in 0..=0xff {
                    let sprite: Sprite = self.get_sprite_tile(sprite_id);
                    // let sprite: Sprite = self.get_sprite_tile(25); // "Copyright" sprite of the nintendo logo in the boot rom
        
                    let x_pos = (sprite_id % 32) * 8;
                    let y_pos = (sprite_id / 32) * 8;
                    
                    // let x_pos = 80;
                    // let y_pos = 80;
                    
                    self.draw_sprite_in_buffer(sprite, x_pos, y_pos)
                }
    
                trace!("PPU: Rendering frame");
                self.window.update_with_buffer(&self.buffer, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap_or_else(|e| {
                    panic!("Failed rendering window due to error ({})", e);
                });
            } else { // Actually render the image that should be displayed

                // Display background
                let bg_addr_init: u16;
                if self.get_ppu_config("bg_tile_map") {
                    bg_addr_init = 0x9C00;
                } else {
                    bg_addr_init = 0x9800;
                }
                let mut bg_addr: u16 = bg_addr_init;

                // Render sprites
                while bg_addr < (bg_addr_init + 0x0400) {
                    let map_entry_index = (bg_addr - bg_addr_init) as u8;
                    let x_pos: u8 = (map_entry_index % 32) * 8;
                    let y_pos: u8 = (map_entry_index / 32) * 8;
                    
                    let tile_index = self.get_addr(bg_addr);

                    // Ignore tile index 0 - For some reason only 1 in 4 renders actually renders the real tile
                    // The rest render tile 0 - For example
                    //        [src/ppu.rs:310] Drawing sprite id 25 in (128, 0)
                    //        [src/ppu.rs:310] Drawing sprite id 0 in (128, 0)
                    //        [src/ppu.rs:310] Drawing sprite id 0 in (128, 0)
                    //        [src/ppu.rs:310] Drawing sprite id 0 in (128, 0)


                    if tile_index == 0 {
                        bg_addr += 1;
                        continue;
                    }
                    
                    let sprite = self.get_sprite_tile(tile_index);
                    self.draw_sprite_in_buffer(sprite, x_pos, y_pos);
                    
                    trace!("Drawing sprite id {} in ({}, {})", tile_index, x_pos, y_pos);

                    bg_addr += 1;
                }

                // TODO : Check if window is enabled
                // TODO : Display window

                // Print buffer vector
                trace!("PPU: Rendering frame");
                self.window.update_with_buffer(&self.buffer, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap_or_else(|e| {
                    panic!("Failed rendering window due to error ({})", e);
                });
            } 
        } else {
            if PPU_DISABLE {
                trace!("PPU: DISABLED IN CONFIG, not rendering")
            } else {
                trace!("PPU: Window disabled, not rendering")
            }
        }
    }
}