use core::arch::asm;

#[repr(u32)]
pub enum ModelSpecificRegister {
    ApicBase = 0x0000_001B,
    Efer = 0xC000_0080,
    Star = 0xC000_0081,
    LStar = 0xC000_0082,
    CStar = 0xC000_0083,
    SfMask = 0xC000_0084,
    GsBase = 0xC000_0101,
    KernelGsBase = 0xC000_0102,
}

impl ModelSpecificRegister {
    pub fn read(self) -> u64 {
        unsafe {
            let value_low: u32;
            let value_high: u32;

            asm!(r"
            mov ecx, {0:e}
            rdmsr
            mov {1:e}, eax
            mov {2:e}, edx
        ", in(reg) self as u32, out(reg) value_low, out(reg) value_high);

            ((value_high as u64) << 32) | (value_low as u64)
        }
    }

    pub fn write(self, value: u64) {
        let value_low = value as u32;
        let value_high = (value >> 32) as u32;

        unsafe {
            asm!(r"
            mov ecx, {0:e}
            mov eax, {1:e}
            mov edx, {2:e}
            rdmsr
        ", in(reg) self as u32, in(reg) value_low, in(reg) value_high);
        }
    }
}
