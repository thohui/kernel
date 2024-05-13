use super::AcpiHeader;

#[repr(C)]
/// FADT
pub struct Fadt {
    /// An ACPI header containing standard ACPI table header information.
    pub header: AcpiHeader,

    /// The 32-bit physical address of the firmware control structure.
    pub firmware_ctrl: u32,

    /// The 32-bit physical address of the Differentiated System Description Table (DSDT).
    pub dsdt: u32,

    /// Reserved field.
    reserved: u8,

    /// Preferred power management profile.
    pub prefered_power_management_profile: u8,

    /// System Control Interrupt (SCI) vector.
    pub sci_interrupt: u16,

    /// System Management Interrupt (SMI) command port.
    pub smi_command_port: u32,

    /// ACPI enable value.
    pub acpi_enable: u8,

    /// ACPI disable value.
    pub acpi_disable: u8,

    /// S4BIOS request value.
    pub s4bios_req: u8,

    /// Processor state control value.
    pub pstate_control: u8,

    /// Physical address of the Power Management 1A event block.
    pub pm1a_event_block: u32,

    /// Physical address of the Power Management 1B event block.
    pub pm1b_event_block: u32,

    /// Physical address of the Power Management 1A control block.
    pub pm1a_control_block: u32,

    /// Physical address of the Power Management 1B control block.
    pub pm1b_control_block: u32,

    /// Physical address of the Power Management 2 control block.
    pub pm2_control_block: u32,

    /// Physical address of the Power Management timer block.
    pub pm_timer_block: u32,

    /// Physical address of the General Purpose Event 0 block.
    pub gpe0_block: u32,

    /// Physical address of the General Purpose Event 1 block.
    pub gpe1_block: u32,

    /// Length of the Power Management 1 event block.
    pub pm1_event_length: u8,

    /// Length of the Power Management 1 control block.
    pub pm1_control_length: u8,

    /// Length of the Power Management 2 control block.
    pub pm2_control_length: u8,

    /// Length of the Power Management timer block.
    pub pm_timer_length: u8,

    /// Length of the General Purpose Event 0 block.
    pub gpe0_length: u8,

    /// Length of the General Purpose Event 1 block.
    pub gpe1_length: u8,

    /// Base index of the General Purpose Event 1.
    pub gpe1_base: u8,

    /// C-state control value.
    pub cstate_control: u8,

    /// Worst-case latency for C2 state.
    pub worst_c2_latency: u16,

    /// Worst-case latency for C3 state.
    pub worst_c3_latency: u16,

    /// Flush size value.
    pub flush_size: u16,

    /// Flush stride value.
    pub flush_stride: u16,

    /// Duty cycle offset value.
    pub duty_offset: u8,

    /// Duty cycle width value.
    pub duty_width: u8,

    /// Day alarm value.
    pub day_alarm: u8,

    /// Month alarm value.
    pub month_alarm: u8,

    /// Century value.
    pub century: u8,

    /// Unused
    boot_architecture_flags: u16,

    /// Reserved field.
    reserved2: u8,

    /// Flags specifying capabilities and features of the ACPI hardware.
    pub flags: u32,

    /// Generic address structure representing the system reset register.
    pub reset_register: GenericAddressStructure,

    /// Reset value.
    pub reset_value: u8,

    /// Reserved field.
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
