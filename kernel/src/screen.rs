use lazy_static::lazy_static;
use limine::framebuffer::Framebuffer;

use crate::requests::FRAMEBUFFER_REQUEST;

lazy_static! {
    pub static ref FRAMEBUFFER: Framebuffer<'static> = FRAMEBUFFER_REQUEST
        .get_response()
        .expect("could not ask limine to get the framebuffers")
        .framebuffers()
        .next()
        .expect("no framebuffers are available");
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct Color {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub padding: u8,
}

impl Color {
    pub const WHITE: Color = Color::new(255, 255, 255);
    pub const BLACK: Color = Color::new(0, 0, 0);

    pub const fn new(r: u8, g: u8, b: u8) -> Color {
        Color {
            r,
            g,
            b,
            padding: 0,
        }
    }
}

pub fn get_colors() -> &'static mut [Color] {
    unsafe {
        core::slice::from_raw_parts_mut(
            FRAMEBUFFER.addr().cast::<Color>(),
            (FRAMEBUFFER.width() * FRAMEBUFFER.height()) as usize,
        )
    }
}

pub fn get_color(x: usize, y: usize) -> &'static mut Color {
    &mut get_colors()[x + y * FRAMEBUFFER.width() as usize]
}
