use core::mem;

use lazy_static::lazy_static;

static FONT_DATA: &[u8] = include_bytes!("../.././res/font.psf");

lazy_static! {
    pub static ref FONT: PSFFont<'static> = {
        let font: PSFFont = PSFFont::parse(FONT_DATA);
        font
    };
}

#[repr(C, packed)]
pub struct PSFHeader {
    magic: [u8; 2],
    mode: u8,
    char_size: u8,
}

pub struct PSFFont<'a> {
    header: &'a PSFHeader,
    data: &'a [u8],
}

impl<'a> PSFFont<'a> {
    pub fn parse(data: &'a [u8]) -> PSFFont<'a> {
        if data.len() < mem::size_of::<PSFHeader>() {
            panic!("invalid data")
        }

        let header = unsafe { &*(data.as_ptr() as *const PSFHeader) };

        if header.magic != [0x36, 0x04] {
            panic!("invalid magic bytes");
        }

        let last_glyph_pos = mem::size_of::<PSFHeader>() + header.char_size as usize * 256;
        if data.len() < last_glyph_pos {
            panic!("invalid data")
        }

        PSFFont { data, header }
    }

    pub fn glyph_size(&self) -> (u64, u64) {
        (8, self.header.char_size as u64)
    }

    pub fn glyph(&self, index: usize) -> Option<&[u8]> {
        if index >= 256 {
            return None;
        }

        let length = self.header.char_size as usize;
        let offset = mem::size_of::<PSFHeader>() + index * length;
        Some(&self.data[offset..(offset + length)])
    }
}
