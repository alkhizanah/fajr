#[cfg(target_arch = "x86_64")]
pub mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

pub fn endless_loop() -> ! {
    interrupts::disable();

    loop {
        interrupts::wait_for_interrupts();
    }
}
