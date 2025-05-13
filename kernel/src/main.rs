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
pub mod requests;
pub mod screen;

use arch::{endless_loop, interrupts};
use requests::BASE_REVISION;

#[unsafe(no_mangle)]
extern "C" fn entry() -> ! {
    assert!(BASE_REVISION.is_supported());

    interrupts::disable();

    arch::init();

    memory::init();

    endless_loop();
}
