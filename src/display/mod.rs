use core::{ops::DerefMut, slice::from_raw_parts_mut};

use limine::framebuffer::{self, Framebuffer};
use spin::{lock_api, Once};

use crate::sync::spinlock::SpinLock;

use self::font::FONT;

pub mod font;

pub static DISPLAY: Once<SpinLock<Display>> = Once::new();

pub fn init_display(framebuffer: Framebuffer<'static>) {
    DISPLAY.call_once(|| SpinLock::new(Display::new(framebuffer)));
}

pub struct Display<'a> {
    pub height: u64,
    pub width: u64,
    data: &'a mut [u8],
    bytes_per_pixel: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct Color([u8; 4]);

impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        // BGR.
        Color([blue, green, red, 0])
    }
}

impl AsRef<[u8; 4]> for Color {
    fn as_ref(&self) -> &[u8; 4] {
        &self.0
    }
}

impl Display<'_> {
    pub fn new<'a>(framebuffer: Framebuffer<'a>) -> Display<'a> {
        let total_space = (framebuffer.pitch() * framebuffer.height()) as usize;
        let data = unsafe { from_raw_parts_mut(framebuffer.addr(), total_space) };
        Display {
            data,
            height: framebuffer.height(),
            width: framebuffer.width(),
            bytes_per_pixel: (framebuffer.bpp() / 8) as usize,
        }
    }

    pub fn draw_pixel(&mut self, x: u64, y: u64, color: Color) {
        let pixel_index = (y * self.width + x) as usize;
        let byte_index = pixel_index * self.bytes_per_pixel;

        let pixel_slice = &mut self.data[byte_index..(byte_index + self.bytes_per_pixel)];
        pixel_slice.copy_from_slice(color.as_ref())
    }

    pub fn draw_character(&mut self, x: u64, y: u64, character: char) {
        let (fwidth, fheight) = FONT.glyph_size();
        let bytes_per_line = (fwidth + 7) / 8;

        let glyph_data = FONT.glyph(character as usize);

        if let Some(glyph_data) = glyph_data {
            for row in 0..fheight {
                let glyph_row_data = &glyph_data[(row * bytes_per_line) as usize..];

                for col in 0..fwidth {
                    let byte = glyph_row_data[col as usize / 8];
                    let bit = 7 - (col % 8);
                    let pixel = (byte >> bit) & 1;

                    let color = if pixel == 0x1 {
                        Color::new(255, 0, 0)
                    } else {
                        Color::new(0, 0, 0)
                    };
                    self.draw_pixel(x + col, y + row, color);
                }
            }
        }
    }
}
