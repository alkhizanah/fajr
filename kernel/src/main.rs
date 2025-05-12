#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

#[macro_use]
pub mod console;

pub mod arch;
pub mod panic;
pub mod requests;
pub mod screen;

use arch::{endless_loop, interrupts};
use requests::BASE_REVISION;

#[unsafe(no_mangle)]
extern "C" fn entry() -> ! {
    assert!(BASE_REVISION.is_supported());

    interrupts::disable();

    arch::init();

    endless_loop();
}
