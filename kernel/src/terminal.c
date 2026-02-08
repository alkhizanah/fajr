#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>

#include "psf2.h"
#include "screen.h"
#include "terminal.h"

#define NANOPRINTF_IMPLEMENTATION
#define NANOPRINTF_VISIBILITY_STATIC
#define NANOPRINTF_USE_FIELD_WIDTH_FORMAT_SPECIFIERS 1
#define NANOPRINTF_USE_PRECISION_FORMAT_SPECIFIERS 1
#define NANOPRINTF_USE_LARGE_FORMAT_SPECIFIERS 1
#define NANOPRINTF_USE_SMALL_FORMAT_SPECIFIERS 1
#define NANOPRINTF_USE_FLOAT_FORMAT_SPECIFIERS 0
#define NANOPRINTF_USE_BINARY_FORMAT_SPECIFIERS 1
#define NANOPRINTF_USE_WRITEBACK_FORMAT_SPECIFIERS 1
#include "nanoprintf.h"

typedef struct {
    Psf2Font font;
    Color background;
    Color foreground;
    size_t x;
    size_t y;
} Terminal;

static Terminal terminal;

void terminal_set_font(Psf2Font font) { terminal.font = font; }

void terminal_set_background(Color color) { terminal.background = color; }

void terminal_set_foreground(Color color) { terminal.foreground = color; }

static bool terminal_glyph_bit(const uint8_t *glyph_bytes, size_t x, size_t y) {
    return (glyph_bytes[y] & (1 << x)) != 0;
}

static void terminal_write_glyph(const uint8_t *glyph_bytes) {
    size_t x = terminal.x * terminal.font.header.glyph_width;
    size_t y = terminal.y * terminal.font.header.glyph_height;

    for (size_t dy = 0; dy < terminal.font.header.glyph_height; dy++) {
        for (size_t dx = 0; dx < terminal.font.header.glyph_width; dx++) {
            Color *pixel = screen_pixel(x + dx, y + dy);

            if (terminal_glyph_bit(glyph_bytes,
                                   terminal.font.header.glyph_width - 1 - dx,
                                   dy)) {
                *pixel = terminal.foreground;
            } else {
                *pixel = terminal.background;
            }
        }
    }
}

void terminal_clear(void) {
    terminal.x = 0;
    terminal.y = 0;

    for (size_t y = 0; y < screen_height(); y++) {
        for (size_t x = 0; x < screen_width(); x++) {
            *screen_pixel(x, y) = terminal.background;
        }
    }
}

static void terminal_lift_up(void) {
    for (size_t row = 0;
         row < screen_height() / terminal.font.header.glyph_height - 1; row++) {
        const size_t sy = row * terminal.font.header.glyph_height;

        for (size_t y = sy; y < sy + terminal.font.header.glyph_height; y++) {
            for (size_t x = 0; x < screen_width(); x++) {
                *screen_pixel(x, y) =
                    *screen_pixel(x, y + terminal.font.header.glyph_height);
            }
        }
    }

    const size_t last_row =
        screen_height() / terminal.font.header.glyph_height - 1;

    const size_t sy = last_row * terminal.font.header.glyph_height;

    for (size_t y = sy; y < sy + terminal.font.header.glyph_height; y++) {
        for (size_t x = 0; x < screen_width(); x++) {
            *screen_pixel(x, y) = terminal.background;
        }
    }

    terminal.x = 0;
    terminal.y = last_row;
}

void terminal_putc(char c) {
    if (c == '\n') {
        terminal.x = 0;

        if (++terminal.y >=
            screen_height() / terminal.font.header.glyph_height) {
            terminal_lift_up();
        }
    } else {
        terminal_write_glyph(terminal.font.data +
                             c * terminal.font.header.glyph_height);

        if (++terminal.x >= screen_width() / terminal.font.header.glyph_width) {
            terminal.x = 0;

            if (++terminal.y >=
                screen_height() / terminal.font.header.glyph_height) {
                terminal_lift_up();
            }
        }
    }
}

void terminal_puts(const char *s) {
    while (*s != 0) {
        terminal_putc(*s++);
    }
}

void terminal_npf_putc(int c, void *ctx) {
    (void)ctx;
    terminal_putc(c);
}

void kprintf(const char *format, ...) {
    va_list args;

    va_start(args, format);

    npf_vpprintf(&terminal_npf_putc, NULL, format, args);

    va_end(args);
}
