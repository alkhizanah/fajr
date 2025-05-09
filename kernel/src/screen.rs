use core::mem::MaybeUninit;

use limine::{framebuffer::Framebuffer, request::FramebufferRequest};

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

static mut FRAMEBUFFER: MaybeUninit<Framebuffer> = MaybeUninit::uninit();

pub static mut IS_INITIALIZED: bool = false;

#[derive(Clone, Copy)]
#[repr(packed)]
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

#[inline]
#[allow(static_mut_refs)]
pub fn get_framebuffer() -> &'static Framebuffer<'static> {
    if !did_init() {
        panic!("usage of `screen::get_framebuffer` while framebuffer is not initialized");
    }

    unsafe { FRAMEBUFFER.assume_init_ref() }
}

pub fn get_colors() -> &'static mut [Color] {
    let framebuffer = get_framebuffer();

    unsafe {
        core::slice::from_raw_parts_mut(
            framebuffer.addr().cast::<Color>(),
            (framebuffer.width() * framebuffer.height()) as usize,
        )
    }
}

pub fn get_color(x: usize, y: usize) -> &'static mut Color {
    &mut get_colors()[x + y * get_framebuffer().width() as usize]
}

pub fn did_init() -> bool {
    unsafe { IS_INITIALIZED }
}

pub fn init() {
    unsafe {
        FRAMEBUFFER = MaybeUninit::new(
            FRAMEBUFFER_REQUEST
                .get_response()
                .expect("could not ask limine to get the framebuffers")
                .framebuffers()
                .next()
                .expect("no framebuffers are available"),
        );

        IS_INITIALIZED = true;
    }
}
