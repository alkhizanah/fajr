pub mod cpu;
pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod io_apic;
pub mod io_ports;
pub mod local_apic;
pub mod msr;
pub mod paging;
pub mod pic;
pub mod tss;

use cpu::Cpu;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
struct DescriptorTableRegister {
    size: u16,
    address: u64,
}

pub fn init_bsp() {
    Cpu::set(Cpu::new(0));

    gdt::load();
    tss::load();
    idt::load();
    pic::disable();
    io_apic::init();
    local_apic::init();
}

pub fn init_ap(cpu_id: u32) {
    Cpu::set(Cpu::new(cpu_id));

    gdt::load();
    tss::load();
    idt::load();
    local_apic::init();
}
