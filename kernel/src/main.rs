#![feature(abi_x86_interrupt, allocator_api)]
#![no_std]
#![no_main]

use limine::mp::Cpu;

extern crate alloc;

#[macro_use]
pub mod console;

pub mod acpi;
pub mod allocators;
pub mod arch;
pub mod memory;
pub mod mp;
pub mod paging;
pub mod panic;
pub mod psf2;
pub mod requests;
pub mod screen;

/// Initialize bootstrap processor
#[unsafe(no_mangle)]
extern "C" fn init_bsp() -> ! {
    if !requests::BASE_REVISION.is_supported() {
        panic!("limine bootloader does not support our requested base revision");
    }

    arch::interrupts::disable();

    arch::init_bsp();

    mp::boot_cpus();

    loop {
        arch::interrupts::wait_for_interrupts();
    }
}

/// Initialize appication processor (the entry point which `mp::boot_cpus` directs the application
/// processors into)
extern "C" fn init_ap(cpu: &Cpu) -> ! {
    arch::init_ap(cpu);

    loop {
        arch::interrupts::wait_for_interrupts();
    }
}
