use core::fmt::{self, Write};

use lazy_static::lazy_static;
use spin::Mutex;

use crate::{
    psf2::Psf2Font,
    screen::{self, Color, FRAMEBUFFER},
};

pub struct Console<'a> {
    pub font: Psf2Font<'a>,
    pub background: Color,
    pub foreground: Color,
    pub width: usize,
    pub height: usize,
    pub x: usize,
    pub y: usize,
    pub padding_x: usize,
    pub padding_y: usize,
}

impl Default for Console<'_> {
    fn default() -> Self {
        let font = Psf2Font::parse(include_bytes!("fonts/default8x16.psfu"));
        let padding_x = 2;
        let padding_y = 1;

        Console {
            font,
            background: Color::BLACK,
            foreground: Color::WHITE,
            width: (FRAMEBUFFER.width() as usize / font.header.glyph_width as usize) - padding_x,
            height: (FRAMEBUFFER.height() as usize / font.header.glyph_height as usize) - padding_y,
            x: padding_x,
            y: padding_y,
            padding_x,
            padding_y,
        }
    }
}

impl Console<'_> {
    pub fn clear(&mut self) {
        screen::get_colors().fill(self.background);

        self.x = self.padding_x;
        self.y = self.padding_y;
    }

    fn write_glyph(&self, glyph_bytes: &[u8]) {
        let x = self.x * self.font.header.glyph_width as usize;
        let y = self.y * self.font.header.glyph_height as usize;

        for dx in 0..self.font.header.glyph_width as usize {
            for dy in 0..self.font.header.glyph_height as usize {
                let font_bit = self.get_glyph_bit(
                    glyph_bytes,
                    self.font.header.glyph_width as usize - 1 - dx,
                    dy,
                );

                if font_bit {
                    *screen::get_color(x + dx, y + dy) = self.foreground;
                } else {
                    *screen::get_color(x + dx, y + dy) = self.background;
                }
            }
        }
    }

    fn get_glyph_bytes(&self, index: usize) -> &[u8] {
        &self.font.data[index..index + self.font.header.glyph_height as usize]
    }

    fn get_glyph_bit(&self, glyph_bytes: &[u8], x: usize, y: usize) -> bool {
        (glyph_bytes[y] & (1 << x)) != 0
    }
}

impl Write for Console<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for ch in s.chars() {
            self.write_char(ch)?;
        }

        Ok(())
    }

    fn write_char(&mut self, ch: char) -> fmt::Result {
        if !ch.is_ascii() {
            self.write_glyph(self.get_glyph_bytes(0));
        } else if ch != '\n' {
            self.write_glyph(
                self.get_glyph_bytes(
                    (ch as usize * self.font.header.glyph_height as usize)
                        .rem_euclid(self.font.data.len()),
                ),
            );
        }

        if self.x + 1 >= self.width || ch == '\n' {
            self.x = self.padding_x;
            self.y += 1;

            if self.y >= self.height {
                let colors = screen::get_colors();
                let row_unit =
                    FRAMEBUFFER.width() as usize * self.font.header.glyph_height as usize;

                for current_row in (self.padding_y..self.height).map(|i| i * row_unit) {
                    let previous_row = current_row - row_unit;
                    let next_row = current_row + row_unit;

                    colors.copy_within(current_row..next_row, previous_row);
                }

                colors[(self.height - 1) * row_unit..].fill(self.background);

                self.y = self.height - 1;
            }
        } else {
            self.x += 1;
        }

        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::console::_print(core::format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n");
    };

    ($($arg:tt)*) => {{
        $crate::console::_print(core::format_args!($($arg)*));
        $crate::print!("\n");
    }};
}

lazy_static! {
    pub static ref CONSOLE: Mutex<Console<'static>> = Mutex::new(Console::default());
}

#[allow(static_mut_refs)]
pub fn _print(args: fmt::Arguments) {
    CONSOLE.lock().write_fmt(args).unwrap();
}
