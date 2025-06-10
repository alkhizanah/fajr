use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::{
    acpi::ACPI,
    memory::align_down,
    paging::{self, MIN_PAGE_SIZE},
};

#[derive(Debug, Clone, Copy)]
pub struct IoApic(usize);

impl IoApic {
    pub const fn new(address: usize) -> IoApic {
        IoApic(address)
    }

    fn read(&self, index: u32) -> u32 {
        unsafe {
            (self.0 as *mut u32).write_volatile(index);
            ((self.0 + 0x10) as *const u32).read_volatile()
        }
    }

    fn write(&self, index: u32, value: u32) {
        unsafe {
            (self.0 as *mut u32).write_volatile(index);
            ((self.0 + 0x10) as *mut u32).write_volatile(value);
        }
    }

    pub fn id(&self) -> u32 {
        self.read(0x0)
    }

    pub fn version(&self) -> u32 {
        self.read(0x1)
    }

    pub fn arbitration_id(&self) -> u32 {
        self.read(0x2)
    }

    pub fn irq_enable(&self, ioapic_irq: u32) {
        let irq_reg = 0x10 + (2 * ioapic_irq);
        self.write(irq_reg, self.read(irq_reg) & !(1 << 16));
    }

    pub fn irq_disable(&self, ioapic_irq: u32) {
        let irq_reg = 0x10 + (2 * ioapic_irq);
        self.write(irq_reg, self.read(irq_reg) | (1 << 16));
    }

    pub fn irq_set(&self, ioapic_irq: u32, lapic_id: u32, irq_vector: u32) {
        let low_reg = 0x10 + (2 * ioapic_irq);
        let high_reg = low_reg + 1;

        let mut low = self.read(low_reg);
        let mut high = self.read(high_reg);

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

        self.write(high_reg, high);
        self.write(low_reg, low);
    }
}

lazy_static! {
    pub static ref IO_APICS: Vec<IoApic> = {
        let mut io_apics = Vec::with_capacity(1);

        let madt = ACPI.madt.as_ref().expect("no I/O APIC is available");

        for io_apic_entry in madt.io_apic_iter() {
            let io_apic_phys_addr = io_apic_entry.physical_address as usize;
            let io_apic_virt_addr = paging::offset(io_apic_phys_addr);

            paging::get_active_table()
                .map(
                    align_down(io_apic_virt_addr, MIN_PAGE_SIZE),
                    align_down(io_apic_phys_addr, MIN_PAGE_SIZE),
                )
                .set_writable(true)
                .set_write_through(true)
                .set_cachability(false);

            io_apics.push(IoApic::new(io_apic_virt_addr));
        }

        if io_apics.len() == 0 {
            panic!("no I/O APIC is available");
        }

        io_apics
    };
}
