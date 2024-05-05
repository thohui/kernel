#[derive(Debug)]
pub enum Bar {
    Memory32,
    Memory64,

    Io { address: u16 },
}

impl Bar {
    /// Attempts to parse a Bar.
    pub fn parse(bar: u32) -> Result<Self, ()> {
        match bar & 1 {
            0 => {
                let bar_type = bar & 0x3;
                // TODO: implement parsing of memory space bar layout.
                match bar_type {
                    0 => Ok(Bar::Memory32),
                    2 => Ok(Bar::Memory64),
                    _ => Err(()),
                }
            }
            1 => Ok(Bar::Io {
                address: (bar >> 2) as u16,
            }),
            _ => Err(()),
        }
    }
}
