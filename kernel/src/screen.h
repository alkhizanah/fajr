#pragma once

#include <stdint.h>
#include <stddef.h>

#include <limine.h>

typedef struct {
    uint8_t b;
    uint8_t g;
    uint8_t r;
    uint8_t padding;
} Color;

void screen_init(struct limine_framebuffer_request);
size_t screen_width(void);
size_t screen_height(void);
Color *screen_pixel(size_t x, size_t y);
