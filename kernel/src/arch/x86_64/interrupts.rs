use core::arch::asm;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct InterruptStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u16,
    reserved1: [u8; 6],
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u16,
    reserved2: [u8; 6],
}

pub fn disable() {
    unsafe {
        asm!("cli");
    }
}

pub fn enable() {
    unsafe {
        asm!("sti");
    }
}

pub fn wait_for_interrupts() {
    unsafe {
        asm!("hlt");
    }
}
