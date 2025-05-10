#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use x86_64::interrupts;
#[cfg(target_arch = "x86_64")]
pub use x86_64::init;

pub fn endless_loop() -> ! {
    interrupts::disable();

    loop {
        interrupts::wait_for_interrupts();
    }
}
