#![no_std]
#![no_main]

use core::arch::asm;

use limine::BaseRevision;
use limine::request::{RequestsEndMarker, RequestsStartMarker};

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

#[unsafe(no_mangle)]
unsafe extern "C" fn entry() -> ! {
    assert!(BASE_REVISION.is_supported());

    hlt();
}

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    hlt();
}

fn hlt() -> ! {
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
