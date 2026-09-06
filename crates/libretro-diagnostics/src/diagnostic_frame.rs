//! Shared visible diagnostic frames for libretro cores.
//!
//! This module owns generic "show something instead of black" presentation
//! helpers. Product cores provide their own diagnostic text and error taxonomy;
//! this module owns CPU software framebuffers and block-text geometry. GPU
//! resources and GL commands live in the overlay module.

const XRGB_SOFTWARE_BG_TOP: u32 = 0x00101824;
const XRGB_SOFTWARE_BG_BOTTOM: u32 = 0x00262A18;
const XRGB_SOFTWARE_TEXT: u32 = 0x00FFE85A;
const XRGB_SOFTWARE_DIM_TEXT: u32 = 0x0098D8FF;
const XRGB_SOFTWARE_BORDER: u32 = 0x00E05A2A;
const XRGB_SOFTWARE_BAR: u32 = 0x0047D16C;
const DIAGNOSTIC_MAX_CHARS: usize = 320;
const DIAGNOSTIC_MAX_VERTICES: usize = 24_000;

pub fn render_software_diagnostic_xrgb8888_frame(
    framebuffer: &mut Vec<u32>,
    width: u32,
    height: u32,
    frame_index: u64,
    header_lines: &[&str],
    message: &str,
) {
    if width == 0 || height == 0 {
        framebuffer.clear();
        return;
    }

    let width_usize = width as usize;
    let height_usize = height as usize;
    framebuffer.resize(width_usize * height_usize, 0);

    for y in 0..height_usize {
        let mix = y as u32 * 255 / height.saturating_sub(1).max(1);
        let color = lerp_xrgb(XRGB_SOFTWARE_BG_TOP, XRGB_SOFTWARE_BG_BOTTOM, mix);
        let row = y * width_usize;
        framebuffer[row..row + width_usize].fill(color);
    }

    let mut surface = SoftwareDiagnosticSurface::new(framebuffer.as_mut_slice(), width, height);
    surface.fill_rect(0, 0, width, 4, XRGB_SOFTWARE_BORDER);
    surface.fill_rect(0, height.saturating_sub(4), width, 4, XRGB_SOFTWARE_BORDER);
    surface.fill_rect(0, 0, 4, height, XRGB_SOFTWARE_BORDER);
    surface.fill_rect(width.saturating_sub(4), 0, 4, height, XRGB_SOFTWARE_BORDER);

    let bar_width = ((frame_index as u32 % width).max(1)).min(width);
    surface.fill_rect(
        8,
        height.saturating_sub(14),
        bar_width.saturating_sub(8),
        6,
        XRGB_SOFTWARE_BAR,
    );

    let lines = diagnostic_lines(
        width,
        height,
        header_lines,
        message,
        DiagnosticLineGrid {
            char_advance: 6,
            line_advance: 9,
            horizontal_padding: 24,
            vertical_padding: 32,
        },
    );
    let scale = diagnostic_pixel_scale(width, height);
    let line_advance = 9 * scale;
    let mut y = 16;
    for (index, line) in lines.iter().enumerate() {
        let color = if index < header_lines.len() {
            XRGB_SOFTWARE_TEXT
        } else {
            XRGB_SOFTWARE_DIM_TEXT
        };
        surface.draw_text(12, y, scale, line, color);
        y = y.saturating_add(line_advance);
    }
}

pub fn diagnostic_block_text_vertices(
    width: u32,
    height: u32,
    header_lines: &[&str],
    message: &str,
) -> Vec<[f32; 2]> {
    let min_dimension = width.min(height);
    let scale = (min_dimension / 160).clamp(2, 4) as f32;
    let x_origin = 12.0;
    let y_origin = 16.0;
    let char_advance = 6.0 * scale;
    let line_advance = 9.0 * scale;
    let max_columns = ((width as f32 - x_origin * 2.0) / char_advance)
        .floor()
        .max(8.0) as usize;
    let max_lines = ((height as f32 - y_origin * 2.0) / line_advance)
        .floor()
        .max(1.0) as usize;

    let mut lines = header_lines
        .iter()
        .map(|line| (*line).to_string())
        .collect::<Vec<_>>();
    lines.push(String::new());
    lines.extend(wrap_diagnostic_message(message, max_columns));

    let mut vertices = Vec::new();
    for (line_index, line) in lines.iter().take(max_lines).enumerate() {
        let y = y_origin + line_index as f32 * line_advance;
        for (column, character) in line.chars().take(max_columns).enumerate() {
            if !push_diagnostic_glyph(
                &mut vertices,
                diagnostic_character(character),
                x_origin + column as f32 * char_advance,
                y,
                scale,
            ) {
                return vertices;
            }
        }
    }

    vertices
}

pub fn wrap_diagnostic_message(message: &str, max_columns: usize) -> Vec<String> {
    let sanitized = message
        .chars()
        .take(DIAGNOSTIC_MAX_CHARS)
        .map(diagnostic_character)
        .collect::<String>();
    let mut lines = Vec::new();

    for raw_line in sanitized.lines() {
        let mut current = String::new();
        for word in raw_line.split_whitespace() {
            if word.len() > max_columns {
                if !current.is_empty() {
                    lines.push(std::mem::take(&mut current));
                }
                for chunk in word.as_bytes().chunks(max_columns.max(1)) {
                    lines.push(String::from_utf8_lossy(chunk).into_owned());
                }
            } else if current.is_empty() {
                current.push_str(word);
            } else if current.len() + 1 + word.len() <= max_columns {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(std::mem::take(&mut current));
                current.push_str(word);
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }

    if lines.is_empty() {
        lines.push("NO DIAGNOSTIC MESSAGE".to_string());
    }
    lines
}

fn diagnostic_lines(
    width: u32,
    height: u32,
    header_lines: &[&str],
    message: &str,
    grid: DiagnosticLineGrid,
) -> Vec<String> {
    let scale = diagnostic_pixel_scale(width, height);
    let char_advance = grid.char_advance * scale;
    let line_advance = grid.line_advance * scale;
    let max_columns =
        ((width.saturating_sub(grid.horizontal_padding)) / char_advance.max(1)).max(8) as usize;
    let max_lines =
        ((height.saturating_sub(grid.vertical_padding)) / line_advance.max(1)).max(1) as usize;

    let mut lines = header_lines
        .iter()
        .map(|line| (*line).to_string())
        .collect::<Vec<_>>();
    lines.push(String::new());
    lines.extend(wrap_diagnostic_message(message, max_columns));
    lines.truncate(max_lines);
    lines
}

fn diagnostic_pixel_scale(width: u32, height: u32) -> u32 {
    (width.min(height) / 160).clamp(2, 4)
}

fn lerp_xrgb(from: u32, to: u32, mix: u32) -> u32 {
    let mix = mix.min(255);
    let inverse = 255 - mix;
    let red = (((from >> 16) & 0xFF) * inverse + ((to >> 16) & 0xFF) * mix) / 255;
    let green = (((from >> 8) & 0xFF) * inverse + ((to >> 8) & 0xFF) * mix) / 255;
    let blue = ((from & 0xFF) * inverse + (to & 0xFF) * mix) / 255;
    (red << 16) | (green << 8) | blue
}

struct SoftwareDiagnosticSurface<'a> {
    framebuffer: &'a mut [u32],
    width: u32,
    height: u32,
}

impl<'a> SoftwareDiagnosticSurface<'a> {
    fn new(framebuffer: &'a mut [u32], width: u32, height: u32) -> Self {
        Self {
            framebuffer,
            width,
            height,
        }
    }

    fn draw_text(&mut self, x: u32, y: u32, scale: u32, text: &str, color: u32) {
        let mut cursor_x = x;
        for character in text.chars() {
            self.draw_glyph(cursor_x, y, scale, diagnostic_character(character), color);
            cursor_x = cursor_x.saturating_add(6 * scale);
        }
    }

    fn draw_glyph(&mut self, x: u32, y: u32, scale: u32, character: char, color: u32) {
        for (row_index, row) in diagnostic_glyph_rows(character).iter().enumerate() {
            for column in 0..5 {
                if row & (1 << (4 - column)) != 0 {
                    self.fill_rect(
                        x + column * scale,
                        y + row_index as u32 * scale,
                        scale,
                        scale,
                        color,
                    );
                }
            }
        }
    }

    fn fill_rect(&mut self, x: u32, y: u32, rect_width: u32, rect_height: u32, color: u32) {
        if rect_width == 0 || rect_height == 0 || x >= self.width || y >= self.height {
            return;
        }

        let max_x = x.saturating_add(rect_width).min(self.width);
        let max_y = y.saturating_add(rect_height).min(self.height);
        let width_usize = self.width as usize;
        for row in y..max_y {
            let start = row as usize * width_usize + x as usize;
            let end = row as usize * width_usize + max_x as usize;
            self.framebuffer[start..end].fill(color);
        }
    }
}

fn diagnostic_character(character: char) -> char {
    let character = character.to_ascii_uppercase();
    match character {
        'A'..='Z'
        | '0'..='9'
        | ' '
        | '-'
        | ':'
        | '.'
        | '/'
        | '_'
        | '('
        | ')'
        | ','
        | '\''
        | '='
        | '?'
        | '!' => character,
        _ => '?',
    }
}

fn push_diagnostic_glyph(
    vertices: &mut Vec<[f32; 2]>,
    character: char,
    x: f32,
    y: f32,
    scale: f32,
) -> bool {
    let rows = diagnostic_glyph_rows(character);
    for (row_index, row) in rows.iter().enumerate() {
        for column in 0..5 {
            if row & (1 << (4 - column)) != 0
                && !push_diagnostic_rect(
                    vertices,
                    x + column as f32 * scale,
                    y + row_index as f32 * scale,
                    scale,
                    scale,
                )
            {
                return false;
            }
        }
    }
    true
}

fn push_diagnostic_rect(
    vertices: &mut Vec<[f32; 2]>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> bool {
    if vertices.len() + 6 > DIAGNOSTIC_MAX_VERTICES {
        return false;
    }

    vertices.extend_from_slice(&[
        [x, y],
        [x + width, y],
        [x + width, y + height],
        [x, y],
        [x + width, y + height],
        [x, y + height],
    ]);
    true
}

fn diagnostic_glyph_rows(character: char) -> [u8; 7] {
    match character {
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        'J' => [
            0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010,
        ],
        'X' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ],
        'Y' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
        ],
        '6' => [
            0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100,
        ],
        '-' => [
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
        ':' => [
            0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000,
        ],
        '.' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100,
        ],
        '/' => [
            0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000,
        ],
        '_' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111,
        ],
        '(' => [
            0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010,
        ],
        ')' => [
            0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000,
        ],
        ',' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100, 0b01000,
        ],
        '\'' => [
            0b00100, 0b00100, 0b01000, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '=' => [
            0b00000, 0b00000, 0b11111, 0b00000, 0b11111, 0b00000, 0b00000,
        ],
        '?' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100,
        ],
        '!' => [
            0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100,
        ],
        _ => [0; 7],
    }
}

struct DiagnosticLineGrid {
    char_advance: u32,
    line_advance: u32,
    horizontal_padding: u32,
    vertical_padding: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    const FONT_HEADER_LINES: &[&str] = &["TEST CORE", "ERROR SCREEN"];
    #[test]
    fn software_diagnostic_frame_draws_visible_pixels() {
        let mut framebuffer = Vec::new();

        render_software_diagnostic_xrgb8888_frame(
            &mut framebuffer,
            160,
            120,
            7,
            FONT_HEADER_LINES,
            "software diagnostic",
        );

        assert_eq!(framebuffer.len(), 160 * 120);
        assert!(framebuffer.contains(&XRGB_SOFTWARE_TEXT));
        assert!(framebuffer.contains(&XRGB_SOFTWARE_DIM_TEXT));
    }

    #[test]
    fn block_diagnostic_vertices_are_bounded_and_non_empty() {
        let vertices = diagnostic_block_text_vertices(
            320,
            240,
            FONT_HEADER_LINES,
            "this diagnostic message should be wrapped but remain bounded",
        );

        assert!(!vertices.is_empty());
        assert!(vertices.len() <= DIAGNOSTIC_MAX_VERTICES);
        assert!(
            vertices
                .iter()
                .all(|[x, y]| *x >= 0.0 && *x <= 320.0 && *y >= 0.0 && *y <= 240.0)
        );
    }
}
