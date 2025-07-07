pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod io_apic;
pub mod local_apic;
pub mod msr;
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
    io_apic::init();
    local_apic::init(0);
}

pub fn init_ap(cpu_id: u32) {
    // We assume that the bootstrap processor is always with id 0, application processors
    // should not have the same id
    assert_ne!(cpu_id, 0);

    gdt::load();
    tss::load(cpu_id);
    idt::load();
    local_apic::init(cpu_id);
}
