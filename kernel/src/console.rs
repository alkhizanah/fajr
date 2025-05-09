use core::fmt::{self, Write};

use lazy_static::lazy_static;
use spin::Mutex;

use crate::screen::{self, Color, FRAMEBUFFER};

#[derive(Clone, Copy)]
pub struct Font<'a> {
    data: &'a [u8],
    width: usize,
    height: usize,
}

impl Default for Font<'_> {
    fn default() -> Self {
        Font {
            data: include_bytes!("fonts/default8x16.bitmap"),
            width: 8,
            height: 16,
        }
    }
}

pub struct Console<'a> {
    font: Font<'a>,
    background: Color,
    foreground: Color,
    width: usize,
    height: usize,
    x: usize,
    y: usize,
}

impl Default for Console<'_> {
    fn default() -> Self {
        let font = Font::default();

        Console {
            font,
            background: Color::BLACK,
            foreground: Color::WHITE,
            width: FRAMEBUFFER.width() as usize / font.width,
            height: FRAMEBUFFER.height() as usize / font.height,
            x: 0,
            y: 0,
        }
    }
}

impl Console<'_> {
    pub fn clear(&mut self) {
        screen::get_colors().fill(self.background);

        self.x = 0;
        self.y = 0;
    }

    fn write_font_bytes(&self, font_bytes: &[u8]) {
        let x = self.x * self.font.width;
        let y = self.y * self.font.height;

        for dx in 0..self.font.width {
            for dy in 0..self.font.height {
                let font_bit = self.get_font_bit(font_bytes, self.font.width - 1 - dx, dy);

                if font_bit {
                    *screen::get_color(x + dx, y + dy) = self.foreground;
                } else {
                    *screen::get_color(x + dx, y + dy) = self.background;
                }
            }
        }
    }

    fn get_font_bytes(&self, location: usize) -> &[u8] {
        &self.font.data[location..location + self.font.height]
    }

    fn get_font_bit(&self, font_bytes: &[u8], x: usize, y: usize) -> bool {
        (font_bytes[y] & (1 << x)) != 0
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
            self.write_font_bytes(self.get_font_bytes(self.font.data.len() - 2 * 16));
        } else if ch != '\n' {
            self.write_font_bytes(
                self.get_font_bytes(
                    (ch as usize * self.font.height).rem_euclid(self.font.data.len()),
                ),
            );
        }

        if self.x + 1 >= self.width || ch == '\n' {
            self.x = 0;
            self.y += 1;

            if self.y >= self.height {
                let colors = screen::get_colors();
                let row_unit = FRAMEBUFFER.width() as usize * self.font.height;

                for current_row in (1..self.height).map(|i| i * row_unit) {
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
        $crate::console::print!("\n");
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
