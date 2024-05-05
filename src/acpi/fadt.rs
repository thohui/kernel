use super::AcpiHeader;

#[repr(C)]
/// FADT
pub struct Fadt {
    pub header: AcpiHeader,
    pub firmware_ctrl: u32,
    pub dsdt: u32,

    reserved: u8,

    pub prefered_power_management_profile: u8,
    pub sci_interrupt: u16,
    pub smi_command_port: u32,
    pub acpi_enable: u8,
    pub acpi_disable: u8,
    pub s4bios_req: u8,
    pub pstate_control: u8,
    pub pm1a_event_block: u32,
    pub pm1b_event_block: u32,
    pub pm1a_control_block: u32,
    pub pm1b_control_block: u32,
    pub pm2_control_block: u32,
    pub pm_timer_block: u32,
    pub gpe0_block: u32,
    pub gpe1_block: u32,
    pub pm1_event_length: u8,
    pub pm1_control_length: u8,
    pub pm2_control_length: u8,
    pub pm_timer_length: u8,
    pub gpe0_length: u8,
    pub gpe1_length: u8,
    pub gpe1_base: u8,
    pub cstate_control: u8,
    pub worst_c2_latency: u16,
    pub worst_c3_latency: u16,
    pub flush_size: u16,
    pub flush_stride: u16,
    pub duty_offset: u8,
    pub duty_width: u8,
    pub day_alarm: u8,
    pub month_alarm: u8,
    pub century: u8,

    /// Unused
    boot_architecture_flags: u16,

    reserved2: u8,
    pub flags: u32,
    pub reset_register: GenericAddressStructure,
    pub reset_value: u8,
    reserved_3: [u8; 3],
}

impl Fadt {
    pub unsafe fn from_addr(addr: *const ()) -> &'static Fadt {
        &*(addr as *const Fadt)
    }
}

impl core::fmt::Debug for Fadt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Fadt")
            .field("header", &self.header)
            .finish()
    }
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct GenericAddressStructure {
    pub address_space: u8,
    pub bit_width: u8,
    pub bit_offset: u8,
    pub access_size: u8,
    pub address: u64,
}
