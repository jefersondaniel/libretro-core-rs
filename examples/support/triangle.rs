//! Shared example-only glow triangle renderer. No frontend or libretro callbacks
//! belong here. The owning example supplies a current context and render target.
use libretro::glow::{self, HasContext};

pub struct TriangleRenderer {
    program: glow::Program,
    buffer: glow::Buffer,
    vao: Option<glow::VertexArray>,
}
impl TriangleRenderer {
    /// # Safety
    /// The supplied GL context must be current for initialization and every later
    /// draw/destroy call. Dropping after context loss only abandons resource names.
    pub unsafe fn new(gl: &glow::Context) -> Result<Self, String> {
        unsafe {
            let (vs, fs) = shader_sources(gl.version().is_embedded, gl.version().major);
            let program = gl.create_program()?;
            for (kind, source) in [(glow::VERTEX_SHADER, vs), (glow::FRAGMENT_SHADER, fs)] {
                let shader = match gl.create_shader(kind) {
                    Ok(s) => s,
                    Err(e) => {
                        gl.delete_program(program);
                        return Err(e);
                    }
                };
                gl.shader_source(shader, &source);
                gl.compile_shader(shader);
                if !gl.get_shader_compile_status(shader) {
                    let e = gl.get_shader_info_log(shader);
                    gl.delete_shader(shader);
                    gl.delete_program(program);
                    return Err(e);
                }
                gl.attach_shader(program, shader);
                gl.delete_shader(shader);
            }
            gl.bind_attrib_location(program, 0, "a_pos");
            gl.bind_attrib_location(program, 1, "a_color");
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                let e = gl.get_program_info_log(program);
                gl.delete_program(program);
                return Err(e);
            }
            let buffer = match gl.create_buffer() {
                Ok(b) => b,
                Err(e) => {
                    gl.delete_program(program);
                    return Err(e);
                }
            };
            let vao = if gl.version().major >= 3 {
                match gl.create_vertex_array() {
                    Ok(v) => Some(v),
                    Err(e) => {
                        gl.delete_buffer(buffer);
                        gl.delete_program(program);
                        return Err(e);
                    }
                }
            } else {
                None
            };
            Ok(Self {
                program,
                buffer,
                vao,
            })
        }
    }
    /// # Safety
    /// The creating context must be current; framebuffer is a valid target owned
    /// by that context. This sets rendering state and releases bindings afterward.
    pub unsafe fn draw(
        &self,
        gl: &glow::Context,
        framebuffer: u32,
        width: i32,
        height: i32,
        vertices: &[[f32; 5]; 3],
        clear: [f32; 4],
    ) -> Result<(), String> {
        unsafe {
            gl.bind_framebuffer(
                glow::FRAMEBUFFER,
                std::num::NonZeroU32::new(framebuffer).map(glow::NativeFramebuffer),
            );
            gl.viewport(0, 0, width, height);
            gl.disable(glow::SCISSOR_TEST);
            gl.disable(glow::DEPTH_TEST);
            gl.disable(glow::STENCIL_TEST);
            gl.disable(glow::CULL_FACE);
            gl.disable(glow::BLEND);
            gl.disable(glow::SAMPLE_ALPHA_TO_COVERAGE);
            gl.disable(glow::SAMPLE_COVERAGE);
            if gl.version().major >= 3 {
                gl.disable(glow::RASTERIZER_DISCARD);
            }
            gl.color_mask(true, true, true, true);
            gl.clear_color(clear[0], clear[1], clear[2], clear[3]);
            gl.clear(glow::COLOR_BUFFER_BIT);
            if self.vao.is_some() {
                gl.bind_vertex_array(self.vao);
            }
            gl.use_program(Some(self.program));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.buffer));
            // Fixed-size geometry needs no allocation on the frame path.
            let mut bytes = [0_u8; 3 * 5 * size_of::<f32>()];
            for (value, output) in vertices.iter().flatten().zip(bytes.chunks_exact_mut(4)) {
                output.copy_from_slice(&value.to_ne_bytes());
            }
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &bytes, glow::STREAM_DRAW);
            let error = gl.get_error();
            if error != glow::NO_ERROR {
                gl.bind_buffer(glow::ARRAY_BUFFER, None);
                gl.use_program(None);
                if self.vao.is_some() {
                    gl.bind_vertex_array(None);
                }
                return Err(format!("triangle vertex upload GL error: {error:#x}"));
            }
            for (location, size, offset) in [(0, 2, 0), (1, 3, 8)] {
                gl.enable_vertex_attrib_array(location);
                gl.vertex_attrib_pointer_f32(location, size, glow::FLOAT, false, 20, offset);
                let v = gl.version();
                if (v.is_embedded && v.major >= 3)
                    || (!v.is_embedded && (v.major, v.minor) >= (3, 3))
                    || [
                        "GL_ARB_instanced_arrays",
                        "GL_EXT_instanced_arrays",
                        "GL_ANGLE_instanced_arrays",
                    ]
                    .iter()
                    .any(|e| gl.supported_extensions().contains(*e))
                {
                    gl.vertex_attrib_divisor(location, 0);
                }
            }
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
            gl.disable_vertex_attrib_array(0);
            gl.disable_vertex_attrib_array(1);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.use_program(None);
            if self.vao.is_some() {
                gl.bind_vertex_array(None);
            }
            // Keep the target bound so a text overlay can draw before presentation.
            let error = gl.get_error();
            if error != glow::NO_ERROR {
                return Err(format!("triangle GL error: {error:#x}"));
            }
            Ok(())
        }
    }
    /// # Safety
    /// Delete only while the creating context is current; after loss, drop instead.
    pub unsafe fn destroy(self, gl: &glow::Context) {
        unsafe {
            if let Some(vao) = self.vao {
                gl.delete_vertex_array(vao);
            }
            gl.delete_buffer(self.buffer);
            gl.delete_program(self.program);
        }
    }
}
fn shader_sources(embedded: bool, major: u32) -> (String, String) {
    let modern = major >= 3;
    let prefix = if embedded {
        if modern {
            "#version 300 es\nprecision mediump float;\n"
        } else {
            "#version 100\nprecision mediump float;\n"
        }
    } else if modern {
        "#version 130\n"
    } else {
        "#version 120\n"
    };
    let input = if modern { "in" } else { "attribute" };
    let output = if modern { "out" } else { "varying" };
    let fragment_input = if modern { "in" } else { "varying" };
    let declaration = if modern { "out vec4 frag_color;" } else { "" };
    let target = if modern { "frag_color" } else { "gl_FragColor" };
    (
        format!(
            "{prefix}{input} vec2 a_pos; {input} vec3 a_color; {output} vec3 v_color; void main(){{gl_Position=vec4(a_pos,0.0,1.0);v_color=a_color;}}"
        ),
        format!(
            "{prefix}{fragment_input} vec3 v_color;{declaration} void main(){{{target}=vec4(v_color,1.0);}}"
        ),
    )
}
