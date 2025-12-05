#include <stdint.h>
#include <string.h>

#include "psf2.h"

static inline uint32_t u32_load_le(const uint8_t *p) {
    return ((uint32_t)p[0]) | ((uint32_t)p[1] << 8) | ((uint32_t)p[2] << 16) |
           ((uint32_t)p[3] << 24);
}

Psf2Font psf2_parse(const uint8_t *data) {
    Psf2Header header;

    header.magic[0] = data[0];
    header.magic[1] = data[1];
    header.magic[2] = data[2];
    header.magic[3] = data[3];

    header.version = u32_load_le(data + 4);
    header.header_size = u32_load_le(data + 8);
    header.flags = u32_load_le(data + 12);
    header.glyph_count = u32_load_le(data + 16);
    header.glyph_size = u32_load_le(data + 20);
    header.glyph_height = u32_load_le(data + 24);
    header.glyph_width = u32_load_le(data + 28);

    return (Psf2Font){.header = header, .data = data + 32};
}
