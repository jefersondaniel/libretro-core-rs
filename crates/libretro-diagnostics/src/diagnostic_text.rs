//! Shared bitmap text diagnostics for libretro OpenGL cores.
//!
//! This module owns the embedded Elecbyte FNT parsing and CPU glyph geometry
//! used for on-screen diagnostics. GPU upload and drawing live in overlay.rs.

use std::collections::HashMap;

pub const DIAGNOSTIC_TEXT_FLOATS_PER_VERTEX: usize = 4;
pub const DIAGNOSTIC_FONT_BYTES: &[u8] = include_bytes!("../assets/f-6x8f.fnt");

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiagnosticTextLayout {
    pub x: f32,
    pub y: f32,
    pub scale: f32,
}

impl DiagnosticTextLayout {
    pub const DEFAULT: Self = Self {
        x: 12.0,
        y: 16.0,
        scale: 1.0,
    };

    pub fn new(x: f32, y: f32, scale: f32) -> Self {
        Self {
            x,
            y,
            scale: scale.max(1.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiagnosticGlyph {
    pub x: u32,
    pub width: u32,
}

#[derive(Debug)]
pub struct DiagnosticFont {
    glyphs: HashMap<char, DiagnosticGlyph>,
    pub rgba_pixels: Vec<u8>,
    #[cfg(test)]
    image_width: u32,
    #[cfg(test)]
    image_height: u32,
    pub texture_width: u32,
    pub texture_height: u32,
    pub glyph_width: u32,
    pub glyph_height: u32,
    spacing_x: i32,
    #[cfg_attr(not(test), allow(dead_code))]
    spacing_y: i32,
}

impl DiagnosticFont {
    pub fn from_fnt_v1(bytes: &[u8]) -> Result<Self, String> {
        let header = FntV1Header::parse(bytes)?;
        let pcx = checked_slice(bytes, header.pcx_offset, header.pcx_size, "PCX section")?;
        let text = checked_slice(bytes, header.text_offset, header.text_size, "text section")?;
        let text = String::from_utf8_lossy(text);
        let definition = FntTextDefinition::parse(&text)?;
        let (indexed_pixels, image_width, image_height) = parse_pcx_indexed(pcx)?;

        if definition.glyph_height == 0 || definition.glyph_width == 0 {
            return Err("diagnostic font has zero-sized glyph metrics".to_string());
        }
        if image_height < definition.glyph_height {
            return Err(format!(
                "diagnostic font PCX height {image_height} is smaller than glyph height {}",
                definition.glyph_height
            ));
        }
        let texture_width = image_width.next_power_of_two();
        let texture_height = image_height.next_power_of_two();
        // Baseline f-6x8f maps its final glyph one pixel past the PCX width;
        // the padded atlas keeps that column transparent.
        for (character, glyph) in &definition.glyphs {
            if glyph.x >= texture_width || glyph.x.saturating_add(glyph.width) > texture_width {
                return Err(format!(
                    "diagnostic font glyph {character:?} range {}..{} exceeds padded texture width {texture_width}",
                    glyph.x,
                    glyph.x + glyph.width
                ));
            }
        }
        let mut rgba_pixels = vec![0_u8; (texture_width * texture_height * 4) as usize];
        for y in 0..image_height {
            for x in 0..image_width {
                let source_index = (y * image_width + x) as usize;
                let destination_index = ((y * texture_width + x) * 4) as usize;
                let alpha = if indexed_pixels[source_index] == 0 {
                    0
                } else {
                    255
                };
                rgba_pixels[destination_index..destination_index + 4]
                    .copy_from_slice(&[255, 255, 255, alpha]);
            }
        }

        Ok(Self {
            glyphs: definition.glyphs,
            rgba_pixels,
            #[cfg(test)]
            image_width,
            #[cfg(test)]
            image_height,
            texture_width,
            texture_height,
            glyph_width: definition.glyph_width,
            glyph_height: definition.glyph_height,
            spacing_x: definition.spacing_x,
            spacing_y: definition.spacing_y,
        })
    }

    pub fn glyph(&self, value: char) -> Option<DiagnosticGlyph> {
        self.glyphs.get(&value).copied()
    }

    pub fn advance_for(&self, value: char) -> f32 {
        let width = self
            .glyph(value)
            .map(|glyph| glyph.width)
            .unwrap_or(self.glyph_width);
        (width as i32 + self.spacing_x).max(0) as f32
    }

    pub fn line_advance(&self) -> f32 {
        (self.glyph_height as i32 + self.spacing_y + 2).max(1) as f32
    }
}

#[derive(Debug)]
struct FntV1Header {
    pcx_offset: usize,
    pcx_size: usize,
    text_offset: usize,
    text_size: usize,
}

impl FntV1Header {
    fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 32 {
            return Err("FNT v1 file is too short for its header".to_string());
        }
        let signature = bytes
            .get(0..12)
            .ok_or_else(|| "FNT v1 signature is truncated".to_string())?;
        let signature_end = signature
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(signature.len());
        if &signature[..signature_end] != b"ElecbyteFnt" {
            return Err("FNT v1 signature is not ElecbyteFnt".to_string());
        }

        Ok(Self {
            pcx_offset: read_u32_le(bytes, 16, "PCX offset")? as usize,
            pcx_size: read_u32_le(bytes, 20, "PCX size")? as usize,
            text_offset: read_u32_le(bytes, 24, "text offset")? as usize,
            text_size: read_u32_le(bytes, 28, "text size")? as usize,
        })
    }
}

#[derive(Debug)]
struct FntTextDefinition {
    glyphs: HashMap<char, DiagnosticGlyph>,
    glyph_width: u32,
    glyph_height: u32,
    spacing_x: i32,
    spacing_y: i32,
}

impl FntTextDefinition {
    fn parse(text: &str) -> Result<Self, String> {
        let mut section = "";
        let mut glyph_width = None;
        let mut glyph_height = None;
        let mut spacing_x = 0;
        let mut spacing_y = 0;
        let mut glyphs = HashMap::new();

        for raw_line in text.lines() {
            let line = strip_comment(raw_line).trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = &line[1..line.len() - 1];
                continue;
            }

            if section.eq_ignore_ascii_case("Def") {
                let Some((key, value)) = line.split_once('=') else {
                    continue;
                };
                let key = key.trim();
                let value = value.trim();
                if key.eq_ignore_ascii_case("Size") {
                    let (width, height) = parse_pair_i32(value)?;
                    glyph_width = Some(width.max(0) as u32);
                    glyph_height = Some(height.max(0) as u32);
                } else if key.eq_ignore_ascii_case("Spacing") {
                    let (x, y) = parse_pair_i32(value)?;
                    spacing_x = x;
                    spacing_y = y;
                }
                continue;
            }

            if section.eq_ignore_ascii_case("Map") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 3 {
                    continue;
                }
                let Some(character) = parse_map_character(parts[0]) else {
                    continue;
                };
                let x = parts[1]
                    .parse::<u32>()
                    .map_err(|_| format!("invalid FNT glyph x offset: {}", parts[1]))?;
                let width = parts[2]
                    .parse::<u32>()
                    .map_err(|_| format!("invalid FNT glyph width: {}", parts[2]))?;
                glyphs.insert(character, DiagnosticGlyph { x, width });
            }
        }

        Ok(Self {
            glyphs,
            glyph_width: glyph_width
                .ok_or_else(|| "FNT [Def] Size width is missing".to_string())?,
            glyph_height: glyph_height
                .ok_or_else(|| "FNT [Def] Size height is missing".to_string())?,
            spacing_x,
            spacing_y,
        })
    }
}

pub fn diagnostic_text_vertices(font: &DiagnosticFont, lines: &[&str]) -> Vec<f32> {
    diagnostic_text_vertices_with_layout(font, lines, DiagnosticTextLayout::DEFAULT)
}

pub fn diagnostic_text_vertices_with_layout(
    font: &DiagnosticFont,
    lines: &[&str],
    layout: DiagnosticTextLayout,
) -> Vec<f32> {
    let glyph_count = lines
        .iter()
        .flat_map(|line| line.chars())
        .filter(|character| *character != ' ')
        .count();
    let mut vertices = Vec::with_capacity(glyph_count * 6 * DIAGNOSTIC_TEXT_FLOATS_PER_VERTEX);
    let mut y = layout.y;

    for line in lines {
        let mut x = layout.x;
        for character in line.chars() {
            if character == ' ' {
                x += font.advance_for(character) * layout.scale;
                continue;
            }

            if let Some(glyph) = font.glyph(character) {
                append_glyph_quad(&mut vertices, font, glyph, x, y, layout.scale);
            }
            x += font.advance_for(character) * layout.scale;
        }
        y += font.line_advance() * layout.scale;
    }

    vertices
}

fn append_glyph_quad(
    vertices: &mut Vec<f32>,
    font: &DiagnosticFont,
    glyph: DiagnosticGlyph,
    x: f32,
    y: f32,
    scale: f32,
) {
    let x0 = x;
    let x1 = x + glyph.width as f32 * scale;
    let y0 = y;
    let y1 = y + font.glyph_height as f32 * scale;
    let u0 = glyph.x as f32 / font.texture_width as f32;
    let u1 = (glyph.x + glyph.width) as f32 / font.texture_width as f32;
    let v0 = 0.0;
    let v1 = font.glyph_height as f32 / font.texture_height as f32;

    vertices.extend_from_slice(&[
        x0, y0, u0, v0, x1, y0, u1, v0, x1, y1, u1, v1, x0, y0, u0, v0, x1, y1, u1, v1, x0, y1, u0,
        v1,
    ]);
}

fn parse_pcx_indexed(data: &[u8]) -> Result<(Vec<u8>, u32, u32), String> {
    if data.len() < 128 {
        return Err("PCX data is too short".to_string());
    }
    if data[0] != 0x0A || data[2] != 1 || data[3] != 8 {
        return Err("diagnostic font PCX is not 8-bit RLE PCX".to_string());
    }

    let x_min = read_u16_le(data, 4, "PCX x_min")?;
    let y_min = read_u16_le(data, 6, "PCX y_min")?;
    let x_max = read_u16_le(data, 8, "PCX x_max")?;
    let y_max = read_u16_le(data, 10, "PCX y_max")?;
    if x_max < x_min || y_max < y_min {
        return Err("PCX bounds are inverted".to_string());
    }
    let width = u32::from(x_max - x_min + 1);
    let height = u32::from(y_max - y_min + 1);
    let bytes_per_line = u32::from(read_u16_le(data, 66, "PCX bytes_per_line")?);
    let required_decoded = bytes_per_line
        .checked_mul(height)
        .ok_or_else(|| "PCX decoded size overflowed".to_string())?;
    let rle_end = if data.len() >= 769 && data[data.len() - 769] == 0x0C {
        data.len() - 769
    } else {
        data.len()
    };

    let mut decoded = Vec::with_capacity(required_decoded as usize);
    let mut cursor = 128;
    while decoded.len() < required_decoded as usize && cursor < rle_end {
        let byte = data[cursor];
        cursor += 1;
        if byte >= 0xC0 {
            if cursor >= rle_end {
                return Err("PCX RLE run is missing its value byte".to_string());
            }
            let count = usize::from(byte & 0x3F);
            let value = data[cursor];
            cursor += 1;
            for _ in 0..count {
                decoded.push(value);
            }
        } else {
            decoded.push(byte);
        }
    }

    if decoded.len() < required_decoded as usize {
        return Err(format!(
            "PCX decoded {} bytes, expected at least {required_decoded}",
            decoded.len()
        ));
    }

    let mut trimmed = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        let row_start = (y * bytes_per_line) as usize;
        let row_end = row_start + width as usize;
        trimmed.extend_from_slice(&decoded[row_start..row_end]);
    }

    Ok((trimmed, width, height))
}

fn checked_slice<'a>(
    bytes: &'a [u8],
    offset: usize,
    size: usize,
    label: &str,
) -> Result<&'a [u8], String> {
    let end = offset
        .checked_add(size)
        .ok_or_else(|| format!("{label} offset/size overflowed"))?;
    bytes
        .get(offset..end)
        .ok_or_else(|| format!("{label} points outside the FNT file"))
}

fn read_u32_le(bytes: &[u8], offset: usize, label: &str) -> Result<u32, String> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| format!("{label} is truncated"))?;
    Ok(u32::from_le_bytes(
        value.try_into().expect("slice length already checked"),
    ))
}

fn read_u16_le(bytes: &[u8], offset: usize, label: &str) -> Result<u16, String> {
    let value = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| format!("{label} is truncated"))?;
    Ok(u16::from_le_bytes(
        value.try_into().expect("slice length already checked"),
    ))
}

fn parse_pair_i32(value: &str) -> Result<(i32, i32), String> {
    let Some((left, right)) = value.split_once(',') else {
        return Err(format!("expected comma-separated pair, got {value:?}"));
    };
    let left = left
        .trim()
        .parse::<i32>()
        .map_err(|_| format!("invalid pair value: {left:?}"))?;
    let right = right
        .trim()
        .parse::<i32>()
        .map_err(|_| format!("invalid pair value: {right:?}"))?;
    Ok((left, right))
}

fn parse_map_character(token: &str) -> Option<char> {
    if let Some(hex) = token
        .strip_prefix("0x")
        .or_else(|| token.strip_prefix("0X"))
    {
        let value = u32::from_str_radix(hex, 16).ok()?;
        return char::from_u32(value);
    }

    token.chars().next()
}

fn strip_comment(line: &str) -> &str {
    line.split_once(';')
        .map(|(before_comment, _)| before_comment)
        .unwrap_or(line)
}

#[cfg(test)]
mod tests {
    use super::*;
    const DIAGNOSTIC_TEXT_LINES: &[&str] = &[
        "RETROCOMPAT TEXT OK",
        "FNT f-6x8f GLES2",
        "OpenGL text path",
    ];

    #[test]
    fn embedded_diagnostic_font_parses_fixed_width_fnt_v1() {
        let font = DiagnosticFont::from_fnt_v1(DIAGNOSTIC_FONT_BYTES).unwrap();

        assert_eq!(font.image_width, 563);
        assert_eq!(font.image_height, 8);
        assert_eq!(font.texture_width, 1024);
        assert_eq!(font.texture_height, 8);
        assert_eq!(font.glyph_width, 6);
        assert_eq!(font.glyph_height, 8);
        assert_eq!(font.spacing_x, 0);
        assert_eq!(font.glyph('R'), Some(DiagnosticGlyph { x: 102, width: 6 }));
        assert_eq!(font.glyph('0'), Some(DiagnosticGlyph { x: 372, width: 6 }));
        assert_eq!(font.glyph('['), Some(DiagnosticGlyph { x: 468, width: 6 }));
        assert_eq!(font.glyph(';'), Some(DiagnosticGlyph { x: 492, width: 6 }));
    }

    #[test]
    fn diagnostic_text_vertices_emit_two_triangles_per_visible_glyph() {
        let font = DiagnosticFont::from_fnt_v1(DIAGNOSTIC_FONT_BYTES).unwrap();
        let vertices = diagnostic_text_vertices(&font, DIAGNOSTIC_TEXT_LINES);
        let visible_glyphs = DIAGNOSTIC_TEXT_LINES
            .iter()
            .flat_map(|line| line.chars())
            .filter(|character| *character != ' ')
            .count();

        assert_eq!(
            vertices.len(),
            visible_glyphs * 6 * DIAGNOSTIC_TEXT_FLOATS_PER_VERTEX
        );
        assert_eq!(vertices[0..4], [12.0, 16.0, 102.0 / 1024.0, 0.0]);
    }

    #[test]
    fn diagnostic_text_layout_scales_and_repositions_vertices() {
        let font = DiagnosticFont::from_fnt_v1(DIAGNOSTIC_FONT_BYTES).unwrap();
        let vertices = diagnostic_text_vertices_with_layout(
            &font,
            &["R"],
            DiagnosticTextLayout::new(20.0, 30.0, 2.0),
        );

        assert_eq!(vertices[0..4], [20.0, 30.0, 102.0 / 1024.0, 0.0]);
        assert_eq!(vertices[4], 32.0);
        assert_eq!(vertices[9], 46.0);
    }

    #[test]
    fn diagnostic_font_alpha_texture_preserves_glyphs_and_padding() {
        let font = DiagnosticFont::from_fnt_v1(DIAGNOSTIC_FONT_BYTES).unwrap();
        let glyph = font.glyph('R').unwrap();
        let mut opaque_pixels = 0;

        for y in 0..font.glyph_height {
            for x in glyph.x..glyph.x + glyph.width {
                let alpha_index = ((y * font.texture_width + x) * 4 + 3) as usize;
                if font.rgba_pixels[alpha_index] == 255 {
                    opaque_pixels += 1;
                }
            }
        }

        assert!(opaque_pixels > 0);

        for y in 0..font.image_height {
            let alpha_index = ((y * font.texture_width + font.image_width) * 4 + 3) as usize;
            assert_eq!(font.rgba_pixels[alpha_index], 0);
        }
    }
}
