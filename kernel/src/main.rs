#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

extern crate alloc;

#[macro_use]
pub mod console;

pub mod arch;
pub mod memory;
pub mod paging;
pub mod panic;
pub mod psf2;
pub mod requests;
pub mod screen;

#[unsafe(no_mangle)]
extern "C" fn entry() -> ! {
    if !requests::BASE_REVISION.is_supported() {
        panic!("limine bootloader does not support our requested base revision");
    }

    arch::interrupts::disable();

    arch::init();

    memory::init();

    arch::endless_loop();
}
