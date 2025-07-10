use core::arch::asm;

pub fn inb(port: u16) -> u8 {
    let value;

    unsafe {
        asm!("in al, dx", out("al") value, in("dx") port);
    }

    value
}

pub fn outb(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value);
    }
}
