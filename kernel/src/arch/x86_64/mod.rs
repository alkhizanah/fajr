pub mod gdt;
pub mod interrupts;

pub fn init() {
    interrupts::disable();
    gdt::init();
}
