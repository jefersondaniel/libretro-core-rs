use libretro::{
    CompatGl, GlBuffer, GlBufferTarget, GlBufferUsage, GlDrawMode, GlDrawRange, GlProgram,
    GlUniformLocation, GlVertexAttribF32Components, GlVertexAttribF32Layout,
    GlVertexAttribLocation,
};

pub(crate) const FLOATS_PER_VERTEX: usize = 5;

const GLES2_VERTEX_SHADER: &str = r#"attribute vec2 a_pos;
attribute vec3 a_color;

uniform vec4 u_rotation;

varying vec3 v_color;

void main() {
    vec2 rotated = vec2(
        a_pos.x * u_rotation.x - a_pos.y * u_rotation.y,
        a_pos.x * u_rotation.y + a_pos.y * u_rotation.x
    );
    gl_Position = vec4(rotated, 0.0, 1.0);
    v_color = a_color;
}
"#;

const GLES2_FRAGMENT_SHADER: &str = r#"#ifdef GL_ES
precision mediump float;
#endif

varying vec3 v_color;

void main() {
    gl_FragColor = vec4(v_color, 1.0);
}
"#;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TriangleInitStage {
    ShaderProgram,
    AttributeOrBuffer,
}

#[derive(Debug)]
pub(crate) struct TriangleInitError {
    stage: TriangleInitStage,
    message: String,
}

impl TriangleInitError {
    fn new(stage: TriangleInitStage, message: impl Into<String>) -> Self {
        Self {
            stage,
            message: message.into(),
        }
    }

    pub(crate) fn stage(&self) -> TriangleInitStage {
        self.stage
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }
}

pub(crate) struct TriangleRenderer {
    program: Option<GlProgram>,
    vbo: Option<GlBuffer>,
    pos_location: GlVertexAttribLocation,
    color_location: GlVertexAttribLocation,
    rotation_location: GlUniformLocation,
}

impl TriangleRenderer {
    pub(crate) fn new(gl: &CompatGl) -> Result<Self, TriangleInitError> {
        let program = gl
            .build_program(GLES2_VERTEX_SHADER, GLES2_FRAGMENT_SHADER)
            .map_err(|error| {
                TriangleInitError::new(
                    TriangleInitStage::ShaderProgram,
                    format!("failed to build GLES2 shader program: {error}"),
                )
            })?;

        let pos_location = match gl.required_attrib_location(program, "a_pos") {
            Ok(location) => location,
            Err(error) => {
                gl.delete_program(program);
                return Err(TriangleInitError::new(
                    TriangleInitStage::AttributeOrBuffer,
                    error,
                ));
            }
        };
        let color_location = match gl.required_attrib_location(program, "a_color") {
            Ok(location) => location,
            Err(error) => {
                gl.delete_program(program);
                return Err(TriangleInitError::new(
                    TriangleInitStage::AttributeOrBuffer,
                    error,
                ));
            }
        };
        let rotation_location = match gl.required_uniform_location(program, "u_rotation") {
            Ok(location) => location,
            Err(error) => {
                gl.delete_program(program);
                return Err(TriangleInitError::new(
                    TriangleInitStage::AttributeOrBuffer,
                    error,
                ));
            }
        };

        let vbo = match gl.gen_buffer() {
            Ok(vbo) => vbo,
            Err(error) => {
                gl.delete_program(program);
                return Err(TriangleInitError::new(
                    TriangleInitStage::AttributeOrBuffer,
                    error,
                ));
            }
        };
        gl.bind_buffer(GlBufferTarget::ArrayBuffer, Some(vbo));
        if let Err(error) = gl.buffer_data(
            GlBufferTarget::ArrayBuffer,
            &triangle_vertices(),
            GlBufferUsage::StaticDraw,
        ) {
            gl.delete_buffer(vbo);
            gl.delete_program(program);
            return Err(TriangleInitError::new(
                TriangleInitStage::AttributeOrBuffer,
                error,
            ));
        }
        gl.unbind_buffer(GlBufferTarget::ArrayBuffer);
        if let Err(error) = gl.check_no_error("retrocompat triangle buffer setup") {
            gl.delete_buffer(vbo);
            gl.delete_program(program);
            return Err(TriangleInitError::new(
                TriangleInitStage::AttributeOrBuffer,
                error,
            ));
        }

        Ok(Self {
            program: Some(program),
            vbo: Some(vbo),
            pos_location,
            color_location,
            rotation_location,
        })
    }

    pub(crate) fn draw(&self, gl: &CompatGl, rotation_radians: f32) -> Result<(), String> {
        let Some(vbo) = self.vbo else {
            return Ok(());
        };
        let Some(program) = self.program else {
            return Ok(());
        };
        let rotation = [rotation_radians.cos(), rotation_radians.sin(), 0.0, 0.0];
        gl.use_program(Some(program));
        gl.uniform_4fv(self.rotation_location, &rotation);
        gl.bind_buffer(GlBufferTarget::ArrayBuffer, Some(vbo));
        let position_layout = GlVertexAttribF32Layout::interleaved(
            GlVertexAttribF32Components::Two,
            FLOATS_PER_VERTEX,
        )?;
        let color_layout = GlVertexAttribF32Layout::interleaved(
            GlVertexAttribF32Components::Three,
            FLOATS_PER_VERTEX,
        )?
        .with_offset_components(GlVertexAttribF32Components::Two);
        gl.enable_vertex_attrib(self.pos_location);
        gl.vertex_attrib_pointer_f32(self.pos_location, position_layout);
        gl.enable_vertex_attrib(self.color_location);
        gl.vertex_attrib_pointer_f32(self.color_location, color_layout);

        gl.draw_arrays(GlDrawMode::Triangles, GlDrawRange::from_start(3))?;

        gl.disable_vertex_attrib(self.pos_location);
        gl.disable_vertex_attrib(self.color_location);
        gl.unbind_buffer(GlBufferTarget::ArrayBuffer);
        gl.use_no_program();
        gl.check_no_error("retrocompat triangle draw")
    }

    pub(crate) fn destroy(&mut self, gl: &CompatGl) {
        if let Some(vbo) = self.vbo.take() {
            gl.delete_buffer(vbo);
        }
        if let Some(program) = self.program.take() {
            gl.delete_program(program);
        }
    }
}

fn triangle_vertices() -> [f32; 15] {
    [
        0.0, 0.62, 0.20, 0.80, 1.0, -0.62, -0.50, 1.0, 0.35, 0.20, 0.62, -0.50, 1.0, 0.90, 0.15,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_buffer_has_three_position_color_vertices() {
        let vertices = triangle_vertices();

        assert_eq!(vertices.len(), 3 * FLOATS_PER_VERTEX);
        assert_eq!(vertices[0..2], [0.0, 0.62]);
        assert_eq!(vertices[5..7], [-0.62, -0.50]);
        assert_eq!(vertices[10..12], [0.62, -0.50]);
    }

    #[test]
    fn triangle_shader_uses_gles2_era_syntax() {
        assert!(GLES2_VERTEX_SHADER.contains("attribute vec2 a_pos"));
        assert!(GLES2_VERTEX_SHADER.contains("uniform vec4 u_rotation"));
        assert!(GLES2_VERTEX_SHADER.contains("varying vec3 v_color"));
        assert!(GLES2_FRAGMENT_SHADER.contains("#ifdef GL_ES"));
        assert!(GLES2_FRAGMENT_SHADER.contains("precision mediump float"));
        assert!(GLES2_FRAGMENT_SHADER.contains("gl_FragColor"));
    }
}
