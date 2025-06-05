use limine::mp::Cpu;

pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod ioapic;
pub mod paging;
pub mod tss;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
struct DescriptorTableRegister {
    size: u16,
    address: u64,
}

pub fn init_bsp() {
    gdt::load();
    tss::load(0);
    idt::load();
}

pub fn init_ap(cpu: &Cpu) {
    gdt::load();
    tss::load(cpu.id);
    idt::load();
}
