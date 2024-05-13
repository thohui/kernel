/// https://wiki.osdev.org/RSDP
#[repr(C, packed)]
#[derive(Debug)]
pub struct Rsdp {
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_address: *const u8,
}

impl Rsdp {
    /// Attempts to parse a Rsdp
    pub unsafe fn from_addr<'a>(rsdp_addr: *const u8) -> &'a Rsdp {
        let rsdp = &*(rsdp_addr as *const Rsdp);

        // Validate rsdp signature
        // assert!(&rsdp.signature, b"RSDP PTR ", "Invalid Rsdp signature");
        if &rsdp.signature != b"RSD PTR " {
            panic!("invalid rsdp signature");
        }

        rsdp
    }
}
