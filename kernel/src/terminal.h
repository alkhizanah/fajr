#pragma once

#include "psf2.h"
#include "screen.h"

void terminal_set_font(Psf2Font);
void terminal_set_background(Color);
void terminal_set_foreground(Color);
void terminal_putc(char);
void terminal_puts(const char *);
void terminal_clear(void);

[[gnu::format(printf, 1, 2)]]
void kprintf(const char *format, ...);
