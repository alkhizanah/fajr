#![feature(abi_x86_interrupt)]

#![no_std]
#![no_main]

#[macro_use]
pub mod console;

pub mod arch;
pub mod panic;
pub mod requests;
pub mod screen;

use arch::endless_loop;
use requests::{BASE_REVISION, STACK_SIZE_REQUEST};

#[unsafe(no_mangle)]
unsafe extern "C" fn entry() -> ! {
    assert!(BASE_REVISION.is_supported());

    STACK_SIZE_REQUEST
        .get_response()
        .expect("could not ask limine for setting stack size");

    arch::init();

    endless_loop();
}
