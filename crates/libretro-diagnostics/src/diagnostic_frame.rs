//! Shared visible diagnostic frames for libretro cores.
//!
//! This module owns generic "show something instead of black" presentation
//! helpers. Product cores provide their own diagnostic text and error taxonomy;
//! this crate owns the software framebuffer, block-text fallback, GL cleanup,
//! and hardware-frame legality details.

use std::cell::Cell;

use libretro::{
    Gl, GlBuffer, GlBufferTarget, GlBufferUsage, GlCapability, GlDrawMode, GlDrawRange,
    GlFramebuffer, GlFramebufferTarget, GlProgram, GlRect, GlVertexArray,
    GlVertexAttribF32Components, GlVertexAttribF32Layout, GlVertexAttribLocation, HwContextType,
    glsym,
};

use crate::diagnostic_gl::StagedDiagnosticGl;
use crate::diagnostic_text::DiagnosticTextOverlay;

const XRGB_SOFTWARE_BG_TOP: u32 = 0x00101824;
const XRGB_SOFTWARE_BG_BOTTOM: u32 = 0x00262A18;
const XRGB_SOFTWARE_TEXT: u32 = 0x00FFE85A;
const XRGB_SOFTWARE_DIM_TEXT: u32 = 0x0098D8FF;
const XRGB_SOFTWARE_BORDER: u32 = 0x00E05A2A;
const XRGB_SOFTWARE_BAR: u32 = 0x0047D16C;
const DIAGNOSTIC_MAX_CHARS: usize = 320;
const DIAGNOSTIC_MAX_VERTICES: usize = 24_000;

#[derive(Clone)]
pub enum DiagnosticGl {
    Loaded(Gl),
    Staged(StagedDiagnosticGl),
}

impl From<Gl> for DiagnosticGl {
    fn from(gl: Gl) -> Self {
        Self::Loaded(gl)
    }
}

impl From<glsym> for DiagnosticGl {
    fn from(gl: glsym) -> Self {
        Self::Loaded(Gl::from_glsym(gl))
    }
}

impl From<StagedDiagnosticGl> for DiagnosticGl {
    fn from(gl: StagedDiagnosticGl) -> Self {
        Self::Staged(gl)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DiagnosticFrameText<'a> {
    pub font_header_lines: &'a [&'a str],
    pub block_header_lines: &'a [&'a str],
    pub message: &'a str,
}

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

pub fn render_diagnostic_gl_frame(
    diagnostic_gl: impl Into<DiagnosticGl>,
    framebuffer: u32,
    width: u32,
    height: u32,
    text: DiagnosticFrameText<'_>,
) -> Result<(), String> {
    match diagnostic_gl.into() {
        DiagnosticGl::Loaded(gl) => {
            render_loaded_diagnostic_frame(gl, framebuffer, width, height, text)
        }
        DiagnosticGl::Staged(gl) => {
            render_staged_diagnostic_frame(gl, framebuffer, width, height, text)
        }
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

fn render_staged_diagnostic_frame(
    gl: StagedDiagnosticGl,
    framebuffer: u32,
    width: u32,
    height: u32,
    text: DiagnosticFrameText<'_>,
) -> Result<(), String> {
    let cleanup = StagedDiagnosticGlStateCleanup::new(gl.gl.clone());
    gl.gl.bind_framebuffer(
        GlFramebufferTarget::Framebuffer,
        GlFramebuffer::from_raw(framebuffer),
    )?;
    gl.gl.viewport(GlRect::new(0, 0, width, height))?;
    gl.gl.clear_color(0.08, 0.015, 0.015, 1.0);
    gl.gl.clear_color_depth_buffer();
    gl.gl.check_no_error("libretro staged diagnostic clear")?;

    if gl.gl.supports_shader_pipeline() {
        if gl.gl.supports_textures()
            && render_font_diagnostic_text(&gl.gl, width, height, text).is_ok()
        {
            return cleanup.finish();
        }

        let _ = render_compat_block_diagnostic_text(&gl.gl, width, height, text);
    }

    cleanup.finish()
}

fn render_loaded_diagnostic_frame(
    gl: Gl,
    framebuffer: u32,
    width: u32,
    height: u32,
    text: DiagnosticFrameText<'_>,
) -> Result<(), String> {
    let cleanup = DiagnosticGlStateCleanup::new(gl.clone());
    gl.bind_framebuffer(
        GlFramebufferTarget::Framebuffer,
        GlFramebuffer::from_raw(framebuffer),
    )?;
    gl.viewport(GlRect::new(0, 0, width, height))?;
    gl.disable(GlCapability::Blend);
    gl.disable(GlCapability::ScissorTest);
    gl.clear_color(0.08, 0.015, 0.015, 1.0);
    gl.clear_color_buffer();
    gl.check_no_error("libretro diagnostic clear")?;

    if render_font_diagnostic_text(&gl, width, height, text).is_ok() {
        return cleanup.finish();
    }

    let vertices =
        diagnostic_block_text_vertices(width, height, text.block_header_lines, text.message);
    if vertices.is_empty() {
        return cleanup.finish();
    }

    let vertex_array = if gl.supports_vertex_arrays() {
        let id = gl.gen_vertex_array()?;
        gl.bind_vertex_array(Some(id))?;
        Some(DiagnosticGlVertexArray { gl: gl.clone(), id })
    } else {
        None
    };

    let (vertex_source, fragment_source) = diagnostic_shader_sources(&gl);
    let program = DiagnosticGlProgram {
        gl: gl.clone(),
        id: gl.build_program(vertex_source, fragment_source)?,
    };
    gl.use_program(Some(program.id));

    let position_location = gl.required_attrib_location(program.id, "position")?;
    let viewport_location = gl.required_uniform_location(program.id, "u_viewport")?;
    let color_location = gl.required_uniform_location(program.id, "u_color")?;
    gl.uniform_4fv(viewport_location, &[width as f32, height as f32, 0.0, 0.0]);
    gl.uniform_4fv(color_location, &[1.0, 0.91, 0.28, 1.0]);

    let buffer_id = gl.gen_buffer()?;
    let buffer = DiagnosticGlBuffer {
        gl: gl.clone(),
        id: buffer_id,
    };
    gl.bind_buffer(GlBufferTarget::ArrayBuffer, Some(buffer.id));
    gl.buffer_data(
        GlBufferTarget::ArrayBuffer,
        &vertices,
        GlBufferUsage::StreamDraw,
    )?;
    gl.enable_vertex_attrib(position_location);
    cleanup.set_enabled_attribute(position_location);
    gl.vertex_attrib_pointer_f32(
        position_location,
        GlVertexAttribF32Layout::tightly_packed(GlVertexAttribF32Components::Two),
    );
    let vertex_count = u32::try_from(vertices.len()).map_err(|_| {
        format!(
            "diagnostic vertex count {} does not fit GL draw count",
            vertices.len()
        )
    })?;
    gl.draw_arrays(GlDrawMode::Triangles, GlDrawRange::from_start(vertex_count))?;
    gl.check_no_error("libretro diagnostic draw")?;
    gl.disable_vertex_attrib(position_location);
    cleanup.clear_enabled_attribute();
    cleanup.finish()?;
    drop(buffer);
    drop(program);
    drop(vertex_array);

    Ok(())
}

fn render_compat_block_diagnostic_text(
    gl: &Gl,
    width: u32,
    height: u32,
    text: DiagnosticFrameText<'_>,
) -> Result<(), String> {
    let vertices =
        diagnostic_ndc_block_text_vertices(width, height, text.block_header_lines, text.message);
    if vertices.is_empty() {
        return Ok(());
    }

    let (vertex_source, fragment_source) = diagnostic_compat_shader_sources(gl);
    let program = CompatDiagnosticGlProgram {
        gl: gl.clone(),
        id: gl.build_program(vertex_source, fragment_source)?,
    };
    gl.use_program(Some(program.id));

    let position_location = gl.required_attrib_location(program.id, "position")?;
    let buffer_id = gl.gen_buffer()?;
    let buffer = CompatDiagnosticGlBuffer {
        gl: gl.clone(),
        id: buffer_id,
    };

    let cleanup = CompatDiagnosticDrawCleanup::new(gl.clone());
    gl.bind_buffer(GlBufferTarget::ArrayBuffer, Some(buffer.id));
    gl.buffer_data(
        GlBufferTarget::ArrayBuffer,
        &vertices,
        GlBufferUsage::StreamDraw,
    )?;
    gl.enable_vertex_attrib(position_location);
    cleanup.set_enabled_attribute(position_location);
    gl.vertex_attrib_pointer_f32(
        position_location,
        GlVertexAttribF32Layout::tightly_packed(GlVertexAttribF32Components::Two),
    );
    let vertex_count = u32::try_from(vertices.len()).map_err(|_| {
        format!(
            "diagnostic vertex count {} does not fit GL draw count",
            vertices.len()
        )
    })?;
    gl.draw_arrays(GlDrawMode::Triangles, GlDrawRange::from_start(vertex_count))?;
    gl.check_no_error("libretro compat diagnostic draw")?;
    gl.disable_vertex_attrib(position_location);
    cleanup.clear_enabled_attribute();
    cleanup.finish()?;
    drop(buffer);
    drop(program);

    Ok(())
}

fn render_font_diagnostic_text(
    gl: &Gl,
    width: u32,
    height: u32,
    text: DiagnosticFrameText<'_>,
) -> Result<(), String> {
    let lines = diagnostic_lines_from_float_grid(
        width,
        height,
        text.font_header_lines,
        text.message,
        DiagnosticFloatLineGrid {
            char_advance: 6.0,
            line_advance: 10.0,
            x_origin: 12.0,
            y_origin: 16.0,
        },
    );
    let line_refs = lines.iter().map(String::as_str).collect::<Vec<_>>();
    let mut overlay = DiagnosticTextOverlay::new(gl, gl, &line_refs)?;
    let draw_result = overlay.draw(gl, gl, width, height, [1.0, 0.91, 0.28, 1.0]);
    overlay.destroy(gl, gl);
    draw_result
}

#[derive(Clone, Copy)]
struct DiagnosticLineGrid {
    char_advance: u32,
    line_advance: u32,
    horizontal_padding: u32,
    vertical_padding: u32,
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

#[derive(Clone, Copy)]
struct DiagnosticFloatLineGrid {
    char_advance: f32,
    line_advance: f32,
    x_origin: f32,
    y_origin: f32,
}

fn diagnostic_lines_from_float_grid(
    width: u32,
    height: u32,
    header_lines: &[&str],
    message: &str,
    grid: DiagnosticFloatLineGrid,
) -> Vec<String> {
    let max_columns = ((width as f32 - grid.x_origin * 2.0) / grid.char_advance)
        .floor()
        .max(8.0) as usize;
    let max_lines = ((height as f32 - grid.y_origin * 2.0) / grid.line_advance)
        .floor()
        .max(1.0) as usize;

    let mut lines = header_lines
        .iter()
        .map(|line| (*line).to_string())
        .collect::<Vec<_>>();
    lines.push(String::new());
    lines.extend(wrap_diagnostic_message(message, max_columns));
    lines.truncate(max_lines);
    lines
}

fn diagnostic_ndc_block_text_vertices(
    width: u32,
    height: u32,
    header_lines: &[&str],
    message: &str,
) -> Vec<[f32; 2]> {
    if width == 0 || height == 0 {
        return Vec::new();
    }

    diagnostic_block_text_vertices(width, height, header_lines, message)
        .into_iter()
        .map(|[x, y]| {
            [
                (x / width as f32) * 2.0 - 1.0,
                1.0 - (y / height as f32) * 2.0,
            ]
        })
        .collect()
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

fn diagnostic_compat_shader_sources(gl: &Gl) -> (&'static str, &'static str) {
    match gl.context_type() {
        HwContextType::OpenGl if !gl.version_info().is_gles => (
            r#"#version 120
attribute vec2 position;
void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}
"#,
            r#"#version 120
void main() {
    gl_FragColor = vec4(1.0, 0.91, 0.28, 1.0);
}
"#,
        ),
        _ => (
            r#"attribute vec2 position;
void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}
"#,
            r#"#ifdef GL_ES
precision mediump float;
#endif
void main() {
    gl_FragColor = vec4(1.0, 0.91, 0.28, 1.0);
}
"#,
        ),
    }
}

fn diagnostic_shader_sources(gl: &Gl) -> (&'static str, &'static str) {
    match gl.context_type() {
        HwContextType::OpenGlCore => (
            r#"#version 330 core
in vec2 position;
uniform vec4 u_viewport;
void main() {
    vec2 ndc = vec2((position.x / u_viewport.x) * 2.0 - 1.0,
                    1.0 - (position.y / u_viewport.y) * 2.0);
    gl_Position = vec4(ndc, 0.0, 1.0);
}
"#,
            r#"#version 330 core
uniform vec4 u_color;
out vec4 frag_color;
void main() {
    frag_color = u_color;
}
"#,
        ),
        HwContextType::OpenGl if !gl.version_info().is_gles => (
            r#"#version 120
attribute vec2 position;
uniform vec4 u_viewport;
void main() {
    vec2 ndc = vec2((position.x / u_viewport.x) * 2.0 - 1.0,
                    1.0 - (position.y / u_viewport.y) * 2.0);
    gl_Position = vec4(ndc, 0.0, 1.0);
}
"#,
            r#"#version 120
uniform vec4 u_color;
void main() {
    gl_FragColor = u_color;
}
"#,
        ),
        _ => (
            r#"attribute vec2 position;
uniform vec4 u_viewport;
void main() {
    vec2 ndc = vec2((position.x / u_viewport.x) * 2.0 - 1.0,
                    1.0 - (position.y / u_viewport.y) * 2.0);
    gl_Position = vec4(ndc, 0.0, 1.0);
}
"#,
            r#"#ifdef GL_ES
precision mediump float;
#endif
uniform vec4 u_color;
void main() {
    gl_FragColor = u_color;
}
"#,
        ),
    }
}

struct DiagnosticGlProgram {
    gl: Gl,
    id: GlProgram,
}

impl Drop for DiagnosticGlProgram {
    fn drop(&mut self) {
        self.gl.delete_program(self.id);
    }
}

struct DiagnosticGlBuffer {
    gl: Gl,
    id: GlBuffer,
}

impl Drop for DiagnosticGlBuffer {
    fn drop(&mut self) {
        self.gl.delete_buffer(self.id);
    }
}

struct DiagnosticGlVertexArray {
    gl: Gl,
    id: GlVertexArray,
}

impl Drop for DiagnosticGlVertexArray {
    fn drop(&mut self) {
        let _ = self.gl.delete_vertex_array(self.id);
    }
}

struct DiagnosticGlStateCleanup {
    gl: Gl,
    enabled_attribute: Cell<Option<GlVertexAttribLocation>>,
    active: Cell<bool>,
}

impl DiagnosticGlStateCleanup {
    fn new(gl: Gl) -> Self {
        Self {
            gl,
            enabled_attribute: Cell::new(None),
            active: Cell::new(true),
        }
    }

    fn set_enabled_attribute(&self, location: GlVertexAttribLocation) {
        self.enabled_attribute.set(Some(location));
    }

    fn clear_enabled_attribute(&self) {
        self.enabled_attribute.set(None);
    }

    fn cleanup(&self) {
        if let Some(location) = self.enabled_attribute.take() {
            self.gl.disable_vertex_attrib(location);
        }
        self.gl.unbind_buffer(GlBufferTarget::ArrayBuffer);
        self.gl.use_no_program();
        if self.gl.supports_vertex_arrays() {
            let _ = self.gl.unbind_vertex_array();
        }
        self.gl.disable(GlCapability::Blend);
        self.gl.disable(GlCapability::ScissorTest);
        self.gl.unbind_framebuffer(GlFramebufferTarget::Framebuffer);
    }

    fn finish(&self) -> Result<(), String> {
        self.cleanup();
        self.active.set(false);
        self.gl.check_no_error("libretro diagnostic cleanup")
    }
}

impl Drop for DiagnosticGlStateCleanup {
    fn drop(&mut self) {
        if self.active.get() {
            self.cleanup();
        }
    }
}

struct StagedDiagnosticGlStateCleanup {
    gl: Gl,
    active: Cell<bool>,
}

impl StagedDiagnosticGlStateCleanup {
    fn new(gl: Gl) -> Self {
        Self {
            gl,
            active: Cell::new(true),
        }
    }

    fn cleanup(&self) {
        self.gl.unbind_framebuffer(GlFramebufferTarget::Framebuffer);
    }

    fn finish(&self) -> Result<(), String> {
        self.cleanup();
        self.active.set(false);
        self.gl.check_no_error("libretro staged diagnostic cleanup")
    }
}

impl Drop for StagedDiagnosticGlStateCleanup {
    fn drop(&mut self) {
        if self.active.get() {
            self.cleanup();
        }
    }
}

struct CompatDiagnosticGlProgram {
    gl: Gl,
    id: GlProgram,
}

impl Drop for CompatDiagnosticGlProgram {
    fn drop(&mut self) {
        self.gl.delete_program(self.id);
    }
}

struct CompatDiagnosticGlBuffer {
    gl: Gl,
    id: GlBuffer,
}

impl Drop for CompatDiagnosticGlBuffer {
    fn drop(&mut self) {
        self.gl.delete_buffer(self.id);
    }
}

struct CompatDiagnosticDrawCleanup {
    gl: Gl,
    enabled_attribute: Cell<Option<GlVertexAttribLocation>>,
    active: Cell<bool>,
}

impl CompatDiagnosticDrawCleanup {
    fn new(gl: Gl) -> Self {
        Self {
            gl,
            enabled_attribute: Cell::new(None),
            active: Cell::new(true),
        }
    }

    fn set_enabled_attribute(&self, location: GlVertexAttribLocation) {
        self.enabled_attribute.set(Some(location));
    }

    fn clear_enabled_attribute(&self) {
        self.enabled_attribute.set(None);
    }

    fn cleanup(&self) {
        if let Some(location) = self.enabled_attribute.take() {
            self.gl.disable_vertex_attrib(location);
        }
        self.gl.unbind_buffer(GlBufferTarget::ArrayBuffer);
        self.gl.use_no_program();
    }

    fn finish(&self) -> Result<(), String> {
        self.cleanup();
        self.active.set(false);
        self.gl.check_no_error("libretro compat diagnostic cleanup")
    }
}

impl Drop for CompatDiagnosticDrawCleanup {
    fn drop(&mut self) {
        if self.active.get() {
            self.cleanup();
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use libretro::{
        FakeGlConfig, GlVersionInfo, HwContextType, reset_fake_gl_for_testing,
        snapshot_fake_gl_for_testing,
    };

    const FONT_HEADER_LINES: &[&str] = &["TEST CORE", "ERROR SCREEN"];
    const BLOCK_HEADER_LINES: &[&str] = &["TEST CORE", "BLOCK FALLBACK"];

    fn diagnostic_text(message: &str) -> DiagnosticFrameText<'_> {
        DiagnosticFrameText {
            font_header_lines: FONT_HEADER_LINES,
            block_header_lines: BLOCK_HEADER_LINES,
            message,
        }
    }

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
            BLOCK_HEADER_LINES,
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

    #[test]
    fn loaded_diagnostic_frame_uses_font_path_and_cleans_gl_state() {
        let _guard = crate::test_support::fake_gl_test_guard();
        reset_fake_gl_for_testing();
        let gl = glsym::fake_for_testing(FakeGlConfig {
            supports_vertex_arrays: true,
            ..FakeGlConfig::default()
        });

        render_diagnostic_gl_frame(gl, 88, 320, 240, diagnostic_text("visible diagnostic"))
            .expect("fake diagnostic frame should render");

        let snapshot = snapshot_fake_gl_for_testing();
        assert!(snapshot.draw_arrays_calls >= 1);
        assert_eq!(snapshot.texture_uploads_2d.len(), 1);
        assert_eq!(snapshot.texture_uploads_2d[0].width, 1024);
        assert_eq!(snapshot.texture_uploads_2d[0].height, 8);
        assert_eq!(snapshot.bound_vertex_array, 0);
        assert_eq!(snapshot.bound_framebuffer, 0);
        assert_eq!(snapshot.current_program, 0);
        assert_eq!(snapshot.bound_array_buffer, 0);
        assert_eq!(snapshot.bound_texture_2d, 0);
        assert_eq!(snapshot.unpack_alignment, 4);
        assert!(!snapshot.blend_enabled);
        assert!(!snapshot.scissor_enabled);
        assert!(snapshot.enabled_vertex_attribs.is_empty());
    }

    #[test]
    fn loaded_diagnostic_frame_cleans_gl_state_after_error_return() {
        let _guard = crate::test_support::fake_gl_test_guard();
        reset_fake_gl_for_testing();
        let gl = glsym::fake_for_testing(FakeGlConfig {
            next_error: Some(0x0502),
            ..FakeGlConfig::default()
        });

        let error = render_diagnostic_gl_frame(gl, 77, 320, 240, diagnostic_text("forced error"))
            .expect_err("forced GL error should make diagnostic rendering return an error");

        assert!(error.contains("GL_INVALID_OPERATION"));
        let snapshot = snapshot_fake_gl_for_testing();
        assert_eq!(snapshot.bound_framebuffer, 0);
        assert_eq!(snapshot.current_program, 0);
        assert_eq!(snapshot.bound_array_buffer, 0);
        assert!(!snapshot.blend_enabled);
        assert!(!snapshot.scissor_enabled);
        assert!(snapshot.enabled_vertex_attribs.is_empty());
    }

    #[test]
    fn block_fallback_still_renders_when_font_texture_upload_fails() {
        let _guard = crate::test_support::fake_gl_test_guard();
        reset_fake_gl_for_testing();
        let gl = glsym::fake_for_testing(FakeGlConfig {
            max_texture_size: Some(4),
            ..FakeGlConfig::default()
        });

        render_diagnostic_gl_frame(gl, 99, 320, 240, diagnostic_text("fallback diagnostic"))
            .expect("block fallback should render when font atlas is too large");

        let snapshot = snapshot_fake_gl_for_testing();
        assert!(snapshot.draw_arrays_calls >= 1);
        assert!(snapshot.texture_uploads_2d.is_empty());
        assert_eq!(snapshot.bound_framebuffer, 0);
        assert_eq!(snapshot.current_program, 0);
        assert_eq!(snapshot.bound_array_buffer, 0);
        assert!(!snapshot.blend_enabled);
        assert!(!snapshot.scissor_enabled);
        assert!(snapshot.enabled_vertex_attribs.is_empty());
    }

    #[test]
    fn desktop_core_context_uses_modern_shader_path() {
        let _guard = crate::test_support::fake_gl_test_guard();
        reset_fake_gl_for_testing();
        let gl = glsym::fake_for_testing(FakeGlConfig {
            context_type: HwContextType::OpenGlCore,
            version_info: GlVersionInfo {
                is_gles: false,
                major: Some(3),
                minor: Some(3),
            },
            supports_vertex_arrays: true,
            supports_instancing: true,
            ..FakeGlConfig::default()
        });

        render_diagnostic_gl_frame(gl, 100, 320, 240, diagnostic_text("desktop diagnostic"))
            .expect("desktop core diagnostic frame should render");

        let snapshot = snapshot_fake_gl_for_testing();
        assert!(snapshot.draw_arrays_calls >= 1);
        assert_eq!(snapshot.bound_vertex_array, 0);
        assert_eq!(snapshot.bound_framebuffer, 0);
        assert!(snapshot.enabled_vertex_attribs.is_empty());
    }
}
