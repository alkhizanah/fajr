pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod paging;
pub mod tss;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
struct DescriptorTableRegister {
    size: u16,
    address: u64,
}

pub fn init() {
    gdt::init();
    idt::init();
}
