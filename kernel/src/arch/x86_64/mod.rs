pub mod gdt;
pub mod idt;
pub mod interrupts;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
struct DescriptorTableRegister {
    size: u16,
    address: u64,
}

pub fn init() {
    interrupts::disable();
    gdt::init();
    idt::init();
}
