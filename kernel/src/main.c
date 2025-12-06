#include <stddef.h>

#include <limine.h>

#include "arch.h"
#include "psf2.h"
#include "screen.h"
#include "terminal.h"

[[gnu::used]] [[gnu::section(".limine_requests_start")]]
static volatile uint64_t limine_requests_start_marker[] =
    LIMINE_REQUESTS_START_MARKER;

[[gnu::used]] [[gnu::section(".limine_requests")]]
static volatile uint64_t limine_base_revision[] = LIMINE_BASE_REVISION(4);

[[gnu::used]] [[gnu::section(".limine_requests")]]
static volatile struct limine_framebuffer_request framebuffer_request = {
    .id = LIMINE_FRAMEBUFFER_REQUEST_ID, .revision = 0};

[[gnu::used]] [[gnu::section(".limine_requests_end")]]
static volatile uint64_t limine_requests_end_marker[] =
    LIMINE_REQUESTS_END_MARKER;

const uint8_t default_font_data[] = {
#embed "fonts/default8x16.psfu"
};

void _start(void) {
    if (!LIMINE_BASE_REVISION_SUPPORTED(limine_base_revision)) {
        for (;;) {
            wait_for_interrupts();
        }
    }

    screen_init(framebuffer_request);

    terminal_set_font(psf2_parse(default_font_data));

    terminal_set_foreground((Color){255, 255, 255, 0});

    arch_init_bsp();

    for (;;) {
        wait_for_interrupts();
    }
}
