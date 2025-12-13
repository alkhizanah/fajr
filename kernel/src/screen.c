#include <stddef.h>

#include <limine.h>

#include "arch.h"
#include "screen.h"

static struct limine_framebuffer *framebuffer;

void screen_init(
    struct limine_framebuffer_response volatile *framebuffer_response) {
    if (framebuffer_response == NULL ||
        framebuffer_response->framebuffer_count < 1) {
        for (;;) {
            wait_for_interrupts();
        }
    }

    framebuffer = framebuffer_response->framebuffers[0];
}

Color *screen_pixel(size_t x, size_t y) {
    return &((Color *)framebuffer->address)[x + y * framebuffer->width];
}

size_t screen_width(void) { return framebuffer->width; }

size_t screen_height(void) { return framebuffer->height; }
