#![no_std]
#![no_main]

use core::arch::asm;

use limine::BaseRevision;
use limine::request::{RequestsEndMarker, RequestsStartMarker, StackSizeRequest};

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

pub const STACK_SIZE: u64 = 0x100000;

#[used]
#[unsafe(link_section = ".requests")]
static STACK_SIZE_REQUEST: StackSizeRequest = StackSizeRequest::new().with_size(STACK_SIZE);

#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

#[unsafe(no_mangle)]
unsafe extern "C" fn entry() -> ! {
    assert!(BASE_REVISION.is_supported());

    if STACK_SIZE_REQUEST.get_response().is_none() {
        panic!("could not ask limine for bigger stack size");
    }

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
