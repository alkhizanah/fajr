use crate::arch::endless_loop;
use crate::requests::FRAMEBUFFER_REQUEST;

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

    endless_loop();
}
