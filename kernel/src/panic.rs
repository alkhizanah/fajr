use core::fmt::Write;

use crate::arch::endless_loop;
use crate::console::CONSOLE;
use crate::requests::FRAMEBUFFER_REQUEST;
use crate::screen::Color;

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    // We print panic info only if screen can be initialized, otherwise that would make a
    // stack overflow, because if screen can not be initialized, it will panic, therefore
    // calling the panic handler again
    if FRAMEBUFFER_REQUEST
        .get_response()
        .is_some_and(|response| response.framebuffers().next().is_some())
    {
        let mut console = CONSOLE.lock();

        console.background = Color::new(0, 128, 255);
        console.foreground = Color::WHITE;

        console.clear();

        if let Some(location) = info.location() {
            let _ = writeln!(
                &mut console,
                "Panic occured at {} in the kernel's source code",
                location
            );
        } else {
            let _ = writeln!(
                &mut console,
                "Panic occured but can't get the location information"
            );
        }

        let _ = writeln!(
            &mut console,
            "You can get help by going to the GitHub repository of the kernel, located at github.com/alkhizanah/khazraj",
        );

        let _ = writeln!(&mut console, "Panic message: {}", info.message());
    }

    endless_loop();
}
