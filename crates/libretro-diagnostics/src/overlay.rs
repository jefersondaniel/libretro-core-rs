//! Bitmap text GPU resources. Explicit destruction requires the creating context;
//! dropping this value only abandons names, which is appropriate after context loss.
use crate::{
    DIAGNOSTIC_FONT_BYTES, DiagnosticFont, DiagnosticTextLayout,
    diagnostic_text_vertices_with_layout,
};
use libretro::glow::{self, HasContext};

/// Text overlay backed by standard glow. Construct once per context reset and
/// update lines as needed. No GL calls are made by Drop.
pub struct DiagnosticTextOverlay {
    font: DiagnosticFont,
    layout: DiagnosticTextLayout,
    program: glow::Program,
    buffer: glow::Buffer,
    texture: glow::Texture,
    vao: Option<glow::VertexArray>,
    count: i32,
    reset_divisors: bool,
    uniforms: TextUniforms,
}
struct TextUniforms {
    viewport: glow::UniformLocation,
    color: glow::UniformLocation,
    font: glow::UniformLocation,
}

impl DiagnosticTextOverlay {
    /// Creates an overlay using the embedded font.
    /// # Safety
    /// `gl` must belong to the current context. All operations on this overlay
    /// must use that same live context. The caller must not delete its resources.
    /// GLES2 instancing extensions require loaded divisor aliases, as supplied by
    /// `Runtime::create_glow_context`; a manually loaded glow context must provide them.
    pub unsafe fn new(gl: &glow::Context, lines: &[&str]) -> Result<Self, String> {
        unsafe { Self::new_with_layout(gl, lines, DiagnosticTextLayout::DEFAULT) }
    }
    /// Creates an overlay at a custom position and scale.
    /// # Safety
    /// Same context and ownership requirements as [`Self::new`].
    pub unsafe fn new_with_layout(
        gl: &glow::Context,
        lines: &[&str],
        layout: DiagnosticTextLayout,
    ) -> Result<Self, String> {
        let font = DiagnosticFont::from_fnt_v1(DIAGNOSTIC_FONT_BYTES)?;
        unsafe {
            let (vs, fs) = if gl.version().is_embedded {
                (GLES2_TEXT_VERTEX_SHADER, GLES2_TEXT_FRAGMENT_SHADER)
            } else if gl.version().major >= 3 {
                (GL130_TEXT_VERTEX_SHADER, GL130_TEXT_FRAGMENT_SHADER)
            } else {
                (GL120_TEXT_VERTEX_SHADER, GL120_TEXT_FRAGMENT_SHADER)
            };
            let program = build_program(gl, vs, fs)?;
            let uniforms = (|| {
                Ok::<_, String>(TextUniforms {
                    viewport: gl
                        .get_uniform_location(program, "u_viewport")
                        .ok_or("missing u_viewport")?,
                    color: gl
                        .get_uniform_location(program, "u_color")
                        .ok_or("missing u_color")?,
                    font: gl
                        .get_uniform_location(program, "u_font")
                        .ok_or("missing u_font")?,
                })
            })();
            let uniforms = match uniforms {
                Ok(uniforms) => uniforms,
                Err(error) => {
                    gl.delete_program(program);
                    return Err(error);
                }
            };
            let buffer = match gl.create_buffer() {
                Ok(v) => v,
                Err(e) => {
                    gl.delete_program(program);
                    return Err(e);
                }
            };
            let texture = match gl.create_texture() {
                Ok(v) => v,
                Err(e) => {
                    gl.delete_buffer(buffer);
                    gl.delete_program(program);
                    return Err(e);
                }
            };
            let vao = if gl.version().major >= 3 {
                match gl.create_vertex_array() {
                    Ok(v) => Some(v),
                    Err(e) => {
                        gl.delete_texture(texture);
                        gl.delete_buffer(buffer);
                        gl.delete_program(program);
                        return Err(e);
                    }
                }
            } else {
                None
            };
            let mut result = Self {
                font,
                layout,
                program,
                buffer,
                texture,
                vao,
                count: 0,
                reset_divisors: supports_instanced_arrays(gl),
                uniforms,
            };
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            // Slice uploads must not inherit frontend row offsets or a PBO.
            let modern = gl.version().major >= 3;
            if modern
                || (!gl.version().is_embedded && (gl.version().major, gl.version().minor) >= (2, 1))
            {
                gl.bind_buffer(glow::PIXEL_UNPACK_BUFFER, None);
            }
            if modern
                || gl.supported_extensions().contains("GL_EXT_unpack_subimage")
                || !gl.version().is_embedded
            {
                for p in [
                    glow::UNPACK_ROW_LENGTH,
                    glow::UNPACK_SKIP_PIXELS,
                    glow::UNPACK_SKIP_ROWS,
                ] {
                    gl.pixel_store_i32(p, 0);
                }
            }
            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            for p in [glow::TEXTURE_MIN_FILTER, glow::TEXTURE_MAG_FILTER] {
                gl.tex_parameter_i32(glow::TEXTURE_2D, p, glow::NEAREST as i32);
            }
            for p in [glow::TEXTURE_WRAP_S, glow::TEXTURE_WRAP_T] {
                gl.tex_parameter_i32(glow::TEXTURE_2D, p, glow::CLAMP_TO_EDGE as i32);
            }
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                result.font.texture_width as i32,
                result.font.texture_height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(&result.font.rgba_pixels)),
            );
            gl.bind_texture(glow::TEXTURE_2D, None);
            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 4);
            if let Err(e) = result.update_lines(gl, lines) {
                result.destroy(gl);
                return Err(e);
            }
            let error = gl.get_error();
            if error != glow::NO_ERROR {
                result.destroy(gl);
                return Err(format!("text initialization GL error: {error:#x}"));
            }
            Ok(result)
        }
    }
    /// Replaces the text without reuploading the font.
    /// # Safety
    /// The creating context must be current and the overlay must still be live.
    pub unsafe fn update_lines(
        &mut self,
        gl: &glow::Context,
        lines: &[&str],
    ) -> Result<(), String> {
        let vertices = diagnostic_text_vertices_with_layout(&self.font, lines, self.layout);
        let count = i32::try_from(vertices.len() / 4).map_err(|_| "too many text vertices")?;
        let bytes: Vec<u8> = vertices.iter().flat_map(|v| v.to_ne_bytes()).collect();
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.buffer));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &bytes, glow::DYNAMIC_DRAW);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            let error = gl.get_error();
            if error != glow::NO_ERROR {
                // A failed allocation need not preserve usable buffer contents.
                self.count = 0;
                return Err(format!("text vertex upload GL error: {error:#x}"));
            }
        }
        self.count = count;
        Ok(())
    }
    /// Draws onto the caller's bound framebuffer. Sets program, texture unit zero,
    /// blend and attribute state; releases its bindings afterward, without restoring
    /// previous state. The caller owns viewport and other rasterization state.
    /// # Safety
    /// The creating context must be current and all resources must be live.
    pub unsafe fn draw(
        &self,
        gl: &glow::Context,
        width: u32,
        height: u32,
        color: [f32; 4],
    ) -> Result<(), String> {
        if width == 0 || height == 0 {
            return Err("text viewport must be nonzero".into());
        }
        if self.count == 0 {
            return Ok(());
        }
        unsafe {
            if self.vao.is_some() {
                gl.bind_vertex_array(self.vao);
            }
            gl.use_program(Some(self.program));
            gl.uniform_4_f32(
                Some(&self.uniforms.viewport),
                width as f32,
                height as f32,
                0.0,
                0.0,
            );
            gl.uniform_4_f32_slice(Some(&self.uniforms.color), &color);
            gl.uniform_1_i32(Some(&self.uniforms.font), 0);
            gl.active_texture(glow::TEXTURE0);
            if (gl.version().is_embedded && gl.version().major >= 3)
                || (!gl.version().is_embedded && (gl.version().major, gl.version().minor) >= (3, 3))
            {
                gl.bind_sampler(0, None);
            }
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.buffer));
            for (location, offset) in [(0, 0), (1, 8)] {
                if self.reset_divisors {
                    gl.vertex_attrib_divisor(location, 0);
                }
                gl.enable_vertex_attrib_array(location);
                gl.vertex_attrib_pointer_f32(location, 2, glow::FLOAT, false, 16, offset);
            }
            gl.enable(glow::BLEND);
            gl.blend_equation(glow::FUNC_ADD);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.draw_arrays(glow::TRIANGLES, 0, self.count);
            gl.disable(glow::BLEND);
            gl.disable_vertex_attrib_array(0);
            gl.disable_vertex_attrib_array(1);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_texture(glow::TEXTURE_2D, None);
            gl.use_program(None);
            if self.vao.is_some() {
                gl.bind_vertex_array(None);
            }
            let error = gl.get_error();
            if error != glow::NO_ERROR {
                return Err(format!("text draw GL error: {error:#x}"));
            }
        }
        Ok(())
    }
    /// Deletes GPU resources and consumes the overlay.
    /// # Safety
    /// The creating context must be current. After context loss, drop instead.
    pub unsafe fn destroy(self, gl: &glow::Context) {
        unsafe {
            if let Some(vao) = self.vao {
                gl.delete_vertex_array(vao);
            }
            gl.delete_texture(self.texture);
            gl.delete_buffer(self.buffer);
            gl.delete_program(self.program);
        }
    }
}

fn supports_instanced_arrays(gl: &glow::Context) -> bool {
    let version = gl.version();
    (version.is_embedded && version.major >= 3)
        || (!version.is_embedded && (version.major, version.minor) >= (3, 3))
        || [
            "GL_ARB_instanced_arrays",
            "GL_EXT_instanced_arrays",
            "GL_ANGLE_instanced_arrays",
        ]
        .iter()
        .any(|extension| gl.supported_extensions().contains(*extension))
}

unsafe fn build_program(gl: &glow::Context, vs: &str, fs: &str) -> Result<glow::Program, String> {
    unsafe {
        let p = gl.create_program()?;
        for (kind, source) in [(glow::VERTEX_SHADER, vs), (glow::FRAGMENT_SHADER, fs)] {
            let shader = match gl.create_shader(kind) {
                Ok(s) => s,
                Err(e) => {
                    gl.delete_program(p);
                    return Err(e);
                }
            };
            gl.shader_source(shader, source);
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                let e = gl.get_shader_info_log(shader);
                gl.delete_shader(shader);
                gl.delete_program(p);
                return Err(e);
            }
            gl.attach_shader(p, shader);
            gl.delete_shader(shader);
        }
        gl.bind_attrib_location(p, 0, "a_pos");
        gl.bind_attrib_location(p, 1, "a_uv");
        gl.link_program(p);
        if !gl.get_program_link_status(p) {
            let e = gl.get_program_info_log(p);
            gl.delete_program(p);
            return Err(e);
        }
        Ok(p)
    }
}
const GLES2_TEXT_VERTEX_SHADER: &str = r#"attribute vec2 a_pos;
attribute vec2 a_uv;

uniform vec4 u_viewport;

varying vec2 v_uv;

void main() {
    vec2 ndc = vec2(
        (a_pos.x / u_viewport.x) * 2.0 - 1.0,
        1.0 - (a_pos.y / u_viewport.y) * 2.0
    );
    gl_Position = vec4(ndc, 0.0, 1.0);
    v_uv = a_uv;
}
"#;

const GLES2_TEXT_FRAGMENT_SHADER: &str = r#"#ifdef GL_ES
precision mediump float;
precision mediump sampler2D;
#endif

uniform sampler2D u_font;
uniform vec4 u_color;

varying vec2 v_uv;

void main() {
    float alpha = texture2D(u_font, v_uv).a;
    gl_FragColor = vec4(u_color.rgb, u_color.a * alpha);
}
"#;

const GL120_TEXT_VERTEX_SHADER: &str = r#"#version 120
attribute vec2 a_pos;
attribute vec2 a_uv;

uniform vec4 u_viewport;

varying vec2 v_uv;

void main() {
    vec2 ndc = vec2(
        (a_pos.x / u_viewport.x) * 2.0 - 1.0,
        1.0 - (a_pos.y / u_viewport.y) * 2.0
    );
    gl_Position = vec4(ndc, 0.0, 1.0);
    v_uv = a_uv;
}
"#;

const GL120_TEXT_FRAGMENT_SHADER: &str = r#"#version 120
uniform sampler2D u_font;
uniform vec4 u_color;

varying vec2 v_uv;

void main() {
    float alpha = texture2D(u_font, v_uv).a;
    gl_FragColor = vec4(u_color.rgb, u_color.a * alpha);
}
"#;

const GL130_TEXT_VERTEX_SHADER: &str = r#"#version 130
in vec2 a_pos;
in vec2 a_uv;

uniform vec4 u_viewport;

out vec2 v_uv;

void main() {
    vec2 ndc = vec2(
        (a_pos.x / u_viewport.x) * 2.0 - 1.0,
        1.0 - (a_pos.y / u_viewport.y) * 2.0
    );
    gl_Position = vec4(ndc, 0.0, 1.0);
    v_uv = a_uv;
}
"#;

const GL130_TEXT_FRAGMENT_SHADER: &str = r#"#version 130
uniform sampler2D u_font;
uniform vec4 u_color;

in vec2 v_uv;
out vec4 frag_color;

void main() {
    float alpha = texture(u_font, v_uv).a;
    frag_color = vec4(u_color.rgb, u_color.a * alpha);
}
"#;

#[cfg(test)]
#[path = "overlay_tests.rs"]
mod tests;
