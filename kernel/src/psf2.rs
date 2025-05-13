pub const PSF2_MAGIC: [u8; 4] = [0x72, 0xb5, 0x4a, 0x86];

#[derive(Debug, Clone, Copy)]
pub struct Psf2Header {
    pub magic: [u8; 4],
    pub version: u32,
    /// Size of the header in bytes, always 32
    pub header_size: u32,
    /// Flags of the font
    pub flags: u32,
    /// Amount of glyphs
    pub glyphs_count: u32,
    /// Size in bytes of each glyph
    pub glyph_size: u32,
    /// The height of each glyph
    pub glyph_width: u32,
    /// The width of each glyph
    pub glyph_height: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Psf2Font<'a> {
    pub header: Psf2Header,
    /// Data without the header
    pub data: &'a [u8],
}

impl Psf2Font<'_> {
    pub fn parse(data: &[u8]) -> Psf2Font {
        fn get_4_bytes(data: &[u8]) -> [u8; 4] {
            [data[0], data[1], data[2], data[3]]
        }

        let header = Psf2Header {
            magic: get_4_bytes(data),
            version: u32::from_le_bytes(get_4_bytes(&data[4..])),
            header_size: u32::from_le_bytes(get_4_bytes(&data[8..])),
            flags: u32::from_le_bytes(get_4_bytes(&data[12..])),
            glyphs_count: u32::from_le_bytes(get_4_bytes(&data[16..])),
            glyph_size: u32::from_le_bytes(get_4_bytes(&data[20..])),
            glyph_height: u32::from_le_bytes(get_4_bytes(&data[24..])),
            glyph_width: u32::from_le_bytes(get_4_bytes(&data[28..])),
        };

        assert_eq!(header.magic, PSF2_MAGIC);

        Psf2Font {
            header,
            data: &data[32..data.len()],
        }
    }
}
