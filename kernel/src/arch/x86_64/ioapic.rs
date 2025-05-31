use lazy_static::lazy_static;

use crate::acpi::ACPI;

#[derive(Debug, Clone, Copy)]
pub struct IoApic {
    base: usize,
}

impl IoApic {
    pub const fn new(base: usize) -> IoApic {
        IoApic { base }
    }

    pub const fn set_base(&mut self, base: usize) {
        self.base = base;
    }

    fn reg_read(&self, index: u32) -> u32 {
        unsafe {
            *(self.base as *mut u32) = index;
            return *((self.base + 0x10) as *const u32);
        }
    }

    fn reg_write(&self, index: u32, value: u32) {
        unsafe {
            *(self.base as *mut u32) = index;
            *((self.base + 0x10) as *mut u32) = value;
        }
    }

    pub fn id(&self) -> u32 {
        self.reg_read(0x0)
    }

    pub fn version(&self) -> u32 {
        self.reg_read(0x1)
    }

    pub fn arbitration_id(&self) -> u32 {
        self.reg_read(0x2)
    }

    pub fn irq_enable(&self, ioapic_irq: u32) {
        let irq_reg = 0x10 + (2 * ioapic_irq as u32);
        self.reg_write(irq_reg, self.reg_read(irq_reg) & !(1 << 16));
    }

    pub fn irq_disable(&self, ioapic_irq: u32) {
        let irq_reg = 0x10 + (2 * ioapic_irq as u32);
        self.reg_write(irq_reg, self.reg_read(irq_reg) | (1 << 16));
    }

    pub fn irq_set(&self, ioapic_irq: u32, lapic_id: u32, irq_vector: u32) {
        let low_reg = 0x10 + (2 * ioapic_irq as u32);
        let high_reg = low_reg + 1;

        let mut low = self.reg_read(low_reg);
        let mut high = self.reg_read(high_reg);

        // Enable the IRQ
        low &= !(1 << 16);
        // Use physical destination mode, not logical destination mode
        low &= !(1 << 11);
        // Set the destination mode to fixed
        low &= !0x700;
        // Set the irq vector
        low &= !0xff;
        low |= irq_vector;

        // Set the LAPIC id
        high &= !0xff000000;
        high |= lapic_id << 24;

        self.reg_write(high_reg, high);
        self.reg_write(low_reg, low);
    }
}

lazy_static! {
    pub static ref IOAPIC: IoApic = IoApic {
        base: ACPI
            .madt
            .as_ref()
            .expect("no I/O APIC is available")
            .get_ioapic_base(),
    };
}
