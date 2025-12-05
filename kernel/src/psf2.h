#pragma once

#include <stdint.h>

typedef struct {
    uint8_t  magic[4];
    uint32_t version;
    uint32_t header_size;
    uint32_t flags;
    uint32_t glyph_count;
    uint32_t glyph_size;
    uint32_t glyph_width;
    uint32_t glyph_height;
} Psf2Header;

typedef struct {
    Psf2Header header;
    const uint8_t *data;
} Psf2Font;

Psf2Font psf2_parse(const uint8_t *data);
