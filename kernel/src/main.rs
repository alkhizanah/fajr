#![no_std]
#![no_main]

#[macro_use]
pub mod console;
pub mod requests;
pub mod screen;

use core::arch::asm;

use requests::{BASE_REVISION, FRAMEBUFFER_REQUEST, STACK_SIZE_REQUEST};

pub const STACK_SIZE: u64 = 0x100000;

#[unsafe(no_mangle)]
unsafe extern "C" fn entry() -> ! {
    assert!(BASE_REVISION.is_supported());

    STACK_SIZE_REQUEST
        .get_response()
        .expect("could not ask limine for setting stack size");

    hlt_loop();
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    // We print panic info only if screen can be initialized, otherwise that would make a
    // stack overflow, because if screen can not be initialized, it will panic, therefore
    // calling the panic handler again
    if FRAMEBUFFER_REQUEST
        .get_response()
        .is_some_and(|response| response.framebuffers().next().is_some())
    {
        println!("{}", info);
    }

    hlt_loop();
}

fn hlt_loop() -> ! {
    unsafe {
        #[cfg(target_arch = "x86_64")]
        asm!("cli");
    }

    loop {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            asm!("hlt");
        }
    }
}
