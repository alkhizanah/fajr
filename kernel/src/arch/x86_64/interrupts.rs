use core::arch::asm;

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
