use crate::consts::*;
use minifb::{Window, WindowOptions, Scale};

pub struct PPU {
    buffer: Vec<u32>,
    window: Window,
    lcd_enabled: bool
}

impl PPU {
    pub fn init() -> PPU {
        // Configure scale
        let mut window_options: WindowOptions = WindowOptions::default();
        window_options.scale = Scale::X4;

        let mut window = Window::new(
            "GBEmulator",
            SCREEN_WIDHT,
            SCREEN_HEIGHT,
            window_options,
        ).unwrap_or_else(|e| {
            panic!("Failed creating minifb window ({})", e);
        });

        // Limit FPS to about 60FPS
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        PPU {
            buffer: [0; SCREEN_HEIGHT * SCREEN_WIDHT].to_vec(),
            window: window
        }
    }

    pub fn set_addr(&mut self, addr: u16, value: u8) {
        
    }

    

    pub fn get_addr(&self, addr: u16) -> u8 {
        0xFF
    }

    pub fn render(&mut self){
        trace!("Rendering frame");
        self.window.update_with_buffer(&self.buffer, SCREEN_WIDHT, SCREEN_HEIGHT).unwrap_or_else(|e| {
            panic!("Failed rendering window due to error ({})", e);
        });
    }
}