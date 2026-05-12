use std::mem;

use libretro::{CompatGl, GlBufferTarget, GlBufferUsage, GlDrawMode};

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
    program: u32,
    vbo: u32,
    pos_location: u32,
    color_location: u32,
    rotation_location: i32,
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

        let vbo = gl.gen_buffer();
        if vbo == 0 {
            gl.delete_program(program);
            return Err(TriangleInitError::new(
                TriangleInitStage::AttributeOrBuffer,
                "triangle VBO allocation returned 0",
            ));
        }
        gl.bind_buffer(GlBufferTarget::ArrayBuffer, vbo);
        gl.buffer_data(
            GlBufferTarget::ArrayBuffer,
            &triangle_vertices(),
            GlBufferUsage::StaticDraw,
        );
        gl.bind_buffer(GlBufferTarget::ArrayBuffer, 0);
        if let Err(error) = gl.check_no_error("retrocompat triangle buffer setup") {
            gl.delete_buffer(vbo);
            gl.delete_program(program);
            return Err(TriangleInitError::new(
                TriangleInitStage::AttributeOrBuffer,
                error,
            ));
        }

        Ok(Self {
            program,
            vbo,
            pos_location,
            color_location,
            rotation_location,
        })
    }

    pub(crate) fn draw(&self, gl: &CompatGl, rotation_radians: f32) -> Result<(), String> {
        let rotation = [rotation_radians.cos(), rotation_radians.sin(), 0.0, 0.0];
        gl.use_program(self.program);
        gl.uniform_4fv(self.rotation_location, &rotation);
        gl.bind_buffer(GlBufferTarget::ArrayBuffer, self.vbo);
        gl.enable_vertex_attrib_array(self.pos_location);
        gl.vertex_attrib_pointer_f32(
            self.pos_location,
            2,
            false,
            (FLOATS_PER_VERTEX * mem::size_of::<f32>()) as i32,
            0,
        );
        gl.enable_vertex_attrib_array(self.color_location);
        gl.vertex_attrib_pointer_f32(
            self.color_location,
            3,
            false,
            (FLOATS_PER_VERTEX * mem::size_of::<f32>()) as i32,
            2 * mem::size_of::<f32>(),
        );

        gl.draw_arrays(GlDrawMode::Triangles, 0, 3);

        gl.disable_vertex_attrib_array(self.pos_location);
        gl.disable_vertex_attrib_array(self.color_location);
        gl.bind_buffer(GlBufferTarget::ArrayBuffer, 0);
        gl.use_program(0);
        gl.check_no_error("retrocompat triangle draw")
    }

    pub(crate) fn destroy(&mut self, gl: &CompatGl) {
        if self.vbo != 0 {
            gl.delete_buffer(self.vbo);
            self.vbo = 0;
        }
        if self.program != 0 {
            gl.delete_program(self.program);
            self.program = 0;
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
