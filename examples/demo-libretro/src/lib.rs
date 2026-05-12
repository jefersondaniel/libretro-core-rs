use std::mem;

use libretro::{
    ContentContract, Core, Environment, GameInfo, GlBufferTarget, GlBufferUsage, GlDrawMode,
    GlFramebufferTarget, HwContextType, JoypadButton, PixelFormat, Runtime, SystemAvInfo,
    SystemInfo, fixed_system_av_info, glsym, opengl_modern_preferred_hw_render_candidates,
};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
const FPS_HZ: u32 = 60;
const SAMPLE_RATE_HZ: u32 = 48_000;
const FPS: f64 = FPS_HZ as f64;
const SAMPLE_RATE: f64 = SAMPLE_RATE_HZ as f64;
const CORE_VERSION: &str = "0.3.0";
const SUPPORTED_CONTENT_EXTENSIONS: &str = "bin|dat";
const MOVE_SPEED: f32 = 0.025;
const TRIANGLE_HALF_WIDTH: f32 = 0.12;
const TRIANGLE_HALF_HEIGHT: f32 = 0.14;
const TRIANGLE_MIN_X: f32 = -1.0 + TRIANGLE_HALF_WIDTH;
const TRIANGLE_MAX_X: f32 = 1.0 - TRIANGLE_HALF_WIDTH;
const TRIANGLE_MIN_Y: f32 = -1.0 + TRIANGLE_HALF_HEIGHT;
const TRIANGLE_MAX_Y: f32 = 1.0 - TRIANGLE_HALF_HEIGHT;
const CLEAR_COLOR: [f32; 4] = [0.08, 0.09, 0.12, 1.0];
const TRIANGLE_GREEN: [f32; 3] = [0.20, 0.82, 0.34];
const TRIANGLE_RED: [f32; 3] = [0.90, 0.22, 0.28];
const MOVE_SOUND_WAV: &[u8] = include_bytes!("../assets/move.wav");

const OPENGL_VERTEX_SHADER_SOURCE: &str = r#"attribute vec2 a_pos;
attribute vec3 a_color;

varying vec3 v_color;

void main() {
    gl_Position = vec4(a_pos, 0.0, 1.0);
    v_color = a_color;
}
"#;

const OPENGL_FRAGMENT_SHADER_SOURCE: &str = r#"varying vec3 v_color;

void main() {
    gl_FragColor = vec4(v_color, 1.0);
}
"#;

const OPENGL_CORE_VERTEX_SHADER_SOURCE: &str = r#"#version 150
in vec2 a_pos;
in vec3 a_color;

out vec3 v_color;

void main() {
    gl_Position = vec4(a_pos, 0.0, 1.0);
    v_color = a_color;
}
"#;

const OPENGL_CORE_FRAGMENT_SHADER_SOURCE: &str = r#"#version 150
in vec3 v_color;
out vec4 frag_color;

void main() {
    frag_color = vec4(v_color, 1.0);
}
"#;

const OPENGLES2_VERTEX_SHADER_SOURCE: &str = r#"attribute vec2 a_pos;
attribute vec3 a_color;

varying vec3 v_color;

void main() {
    gl_Position = vec4(a_pos, 0.0, 1.0);
    v_color = a_color;
}
"#;

const OPENGLES2_FRAGMENT_SHADER_SOURCE: &str = r#"precision mediump float;
varying vec3 v_color;

void main() {
    gl_FragColor = vec4(v_color, 1.0);
}
"#;

const OPENGLES3_VERTEX_SHADER_SOURCE: &str = r#"#version 300 es
precision mediump float;

in vec2 a_pos;
in vec3 a_color;

out vec3 v_color;

void main() {
    gl_Position = vec4(a_pos, 0.0, 1.0);
    v_color = a_color;
}
"#;

const OPENGLES3_FRAGMENT_SHADER_SOURCE: &str = r#"#version 300 es
precision mediump float;

in vec3 v_color;
out vec4 frag_color;

void main() {
    frag_color = vec4(v_color, 1.0);
}
"#;

struct DemoLibretroCore {
    gl: Option<glsym>,
    program: u32,
    vbo: u32,
    vao: u32,
    pos_location: u32,
    color_location: u32,
    triangle_center: [f32; 2],
    content_loaded: bool,
    last_load_error: Option<String>,
    move_sound: MoveSound,
    silence: Vec<[i16; 2]>,
}

impl Default for DemoLibretroCore {
    fn default() -> Self {
        Self {
            gl: None,
            program: 0,
            vbo: 0,
            vao: 0,
            pos_location: 0,
            color_location: 0,
            triangle_center: [0.0, 0.0],
            content_loaded: false,
            last_load_error: None,
            move_sound: MoveSound::default(),
            silence: vec![[0; 2]; (SAMPLE_RATE_HZ / FPS_HZ) as usize],
        }
    }
}

impl Default for MoveSound {
    fn default() -> Self {
        Self::from_wav_bytes(MOVE_SOUND_WAV, SAMPLE_RATE_HZ)
            .unwrap_or_else(|error| panic!("failed to decode embedded move.wav: {error}"))
    }
}

impl Core for DemoLibretroCore {
    fn system_info(&self) -> SystemInfo {
        let mut info = SystemInfo::new("demo-libretro", CORE_VERSION);
        content_contract().apply_to_system_info(&mut info);
        info
    }

    fn av_info(&self) -> SystemAvInfo {
        fixed_system_av_info(WIDTH, HEIGHT, FPS, SAMPLE_RATE)
    }

    fn on_set_environment(&mut self, env: &mut Environment<'_>) {
        let _ = content_contract().register_environment(env);
    }

    fn load_game(&mut self, game: Option<GameInfo<'_>>, runtime: &mut Runtime<'_>) -> bool {
        self.triangle_center = [0.0, 0.0];
        self.move_sound.stop();
        let logger = runtime.logger();
        if let Some(path) = game.and_then(|info| info.path_lossy().map(|path| path.into_owned())) {
            logger.info(format!("demo-libretro: loaded content path {path}"));
            self.content_loaded = true;
            self.last_load_error = None;
        } else {
            logger.warn("demo-libretro: no content path supplied; booting no-game demo");
            self.content_loaded = false;
            self.last_load_error = Some("no content path supplied".to_string());
        }

        let mut env = runtime.environment();
        if !env.set_pixel_format(PixelFormat::Xrgb8888) {
            return false;
        }

        let candidates = opengl_modern_preferred_hw_render_candidates();
        env.set_hw_render_from_candidates(&candidates).is_some()
    }

    fn unload_game(&mut self) {
        self.content_loaded = false;
        self.last_load_error = None;
        self.triangle_center = [0.0, 0.0];
        self.move_sound.stop();
    }

    fn run(&mut self, runtime: &mut Runtime<'_>) {
        runtime.poll_input();

        let moved = self.apply_input(runtime);
        if moved && !self.move_sound.is_playing() {
            self.move_sound.trigger();
        }
        let audio_frames = self.move_sound.mix_into_silence(&self.silence);

        let Some(framebuffer) = runtime.current_framebuffer() else {
            let _ = runtime.video_refresh_dupe_with_audio(WIDTH, HEIGHT, &audio_frames);
            return;
        };
        let Some(gl) = self.gl.as_ref() else {
            let _ = runtime.video_refresh_dupe_with_audio(WIDTH, HEIGHT, &audio_frames);
            return;
        };

        let vertices = build_triangle_vertices(
            self.triangle_center,
            if self.content_loaded {
                TRIANGLE_GREEN
            } else {
                TRIANGLE_RED
            },
        );

        gl.bind_framebuffer(GlFramebufferTarget::Framebuffer, framebuffer);
        gl.viewport(0, 0, WIDTH as i32, HEIGHT as i32);
        gl.clear_color(
            CLEAR_COLOR[0],
            CLEAR_COLOR[1],
            CLEAR_COLOR[2],
            CLEAR_COLOR[3],
        );
        gl.clear_color_buffer();
        gl.use_program(self.program);
        gl.bind_buffer(GlBufferTarget::ArrayBuffer, self.vbo);
        gl.buffer_data(
            GlBufferTarget::ArrayBuffer,
            &vertices,
            GlBufferUsage::StaticDraw,
        );

        if self.vao != 0 {
            gl.bind_vertex_array(self.vao)
                .unwrap_or_else(|error| panic!("failed to bind vertex array: {error}"));
        } else {
            gl.enable_vertex_attrib_array(self.pos_location);
            gl.vertex_attrib_pointer_f32(
                self.pos_location,
                2,
                false,
                5 * mem::size_of::<f32>() as i32,
                0,
            );
            gl.enable_vertex_attrib_array(self.color_location);
            gl.vertex_attrib_pointer_f32(
                self.color_location,
                3,
                false,
                5 * mem::size_of::<f32>() as i32,
                2 * mem::size_of::<f32>(),
            );
        }

        gl.draw_arrays(GlDrawMode::Triangles, 0, 3);

        if self.vao != 0 {
            gl.bind_vertex_array(0)
                .unwrap_or_else(|error| panic!("failed to unbind vertex array: {error}"));
        } else {
            gl.disable_vertex_attrib_array(self.pos_location);
            gl.disable_vertex_attrib_array(self.color_location);
        }

        gl.bind_buffer(GlBufferTarget::ArrayBuffer, 0);
        gl.bind_framebuffer(GlFramebufferTarget::Framebuffer, 0);
        gl.use_program(0);

        let _ = runtime.video_refresh_hw_with_audio(WIDTH, HEIGHT, 0, &audio_frames);
    }

    fn hw_context_reset(&mut self, runtime: &mut Runtime<'_>) {
        let gl = match glsym::init(runtime) {
            Ok(gl) => gl,
            Err(error) => panic!("failed to load OpenGL symbols: {error}"),
        };
        let (vertex_shader_source, fragment_shader_source) = shader_sources_for(gl.context_type());

        let program = gl
            .build_program(vertex_shader_source, fragment_shader_source)
            .unwrap_or_else(|error| panic!("failed to build GL program: {error}"));

        let vbo = gl.gen_buffer();
        let vao = if gl.supports_vertex_arrays() {
            let vao = gl
                .gen_vertex_array()
                .unwrap_or_else(|error| panic!("failed to create vertex array: {error}"));
            gl.bind_vertex_array(vao)
                .unwrap_or_else(|error| panic!("failed to bind vertex array: {error}"));
            vao
        } else {
            0
        };

        let (pos_location, color_location) = (
            gl.get_attrib_location(program, "a_pos")
                .unwrap_or_else(|error| panic!("failed to resolve attribute location: {error}"))
                as u32,
            gl.get_attrib_location(program, "a_color")
                .unwrap_or_else(|error| panic!("failed to resolve attribute location: {error}"))
                as u32,
        );

        gl.bind_buffer(GlBufferTarget::ArrayBuffer, vbo);
        gl.buffer_data(
            GlBufferTarget::ArrayBuffer,
            &build_triangle_vertices([0.0, 0.0], TRIANGLE_RED),
            GlBufferUsage::StaticDraw,
        );

        if vao != 0 {
            // OpenGL core-profile attribute state lives in the VAO, so configure it once here.
            gl.enable_vertex_attrib_array(pos_location);
            gl.vertex_attrib_pointer_f32(
                pos_location,
                2,
                false,
                5 * mem::size_of::<f32>() as i32,
                0,
            );
            gl.enable_vertex_attrib_array(color_location);
            gl.vertex_attrib_pointer_f32(
                color_location,
                3,
                false,
                5 * mem::size_of::<f32>() as i32,
                2 * mem::size_of::<f32>(),
            );
            gl.bind_vertex_array(0)
                .unwrap_or_else(|error| panic!("failed to unbind vertex array: {error}"));
        }

        gl.bind_buffer(GlBufferTarget::ArrayBuffer, 0);

        self.gl = Some(gl);
        self.program = program;
        self.vbo = vbo;
        self.vao = vao;
        self.pos_location = pos_location;
        self.color_location = color_location;
    }

    fn hw_context_destroy(&mut self, _runtime: &mut Runtime<'_>) {
        if let Some(gl) = &self.gl {
            if self.vao != 0 {
                gl.delete_vertex_array(self.vao)
                    .unwrap_or_else(|error| panic!("failed to delete vertex array: {error}"));
            }
            if self.vbo != 0 {
                gl.delete_buffer(self.vbo);
            }
            if self.program != 0 {
                gl.delete_program(self.program);
            }
        }

        self.program = 0;
        self.vbo = 0;
        self.vao = 0;
        self.pos_location = 0;
        self.color_location = 0;
        self.gl = None;
    }
}

impl DemoLibretroCore {
    fn apply_input(&mut self, runtime: &Runtime<'_>) -> bool {
        let horizontal = axis_value(
            runtime.joypad_pressed(0, JoypadButton::Left),
            runtime.joypad_pressed(0, JoypadButton::Right),
        );
        let vertical = axis_value(
            runtime.joypad_pressed(0, JoypadButton::Down),
            runtime.joypad_pressed(0, JoypadButton::Up),
        );

        if horizontal == 0.0 && vertical == 0.0 {
            return false;
        }

        self.triangle_center[0] = (self.triangle_center[0] + horizontal * MOVE_SPEED)
            .clamp(TRIANGLE_MIN_X, TRIANGLE_MAX_X);
        self.triangle_center[1] =
            (self.triangle_center[1] + vertical * MOVE_SPEED).clamp(TRIANGLE_MIN_Y, TRIANGLE_MAX_Y);
        true
    }
}

#[derive(Clone, Debug)]
struct MoveSound {
    frames: Vec<[i16; 2]>,
    cursor: Option<usize>,
}

impl MoveSound {
    fn from_wav_bytes(bytes: &[u8], output_sample_rate: u32) -> Result<Self, String> {
        let source = parse_pcm_wav(bytes)?;
        let frames = resample_pcm_to_stereo_i16(&source, output_sample_rate);
        if frames.is_empty() {
            return Err("embedded move.wav decoded to zero frames".to_string());
        }

        Ok(Self {
            frames,
            cursor: None,
        })
    }

    fn is_playing(&self) -> bool {
        self.cursor.is_some()
    }

    fn trigger(&mut self) {
        self.cursor = Some(0);
    }

    fn stop(&mut self) {
        self.cursor = None;
    }

    fn mix_into_silence(&mut self, silence: &[[i16; 2]]) -> Vec<[i16; 2]> {
        let mut out = silence.to_vec();
        let Some(mut cursor) = self.cursor else {
            return out;
        };

        for frame in &mut out {
            if cursor >= self.frames.len() {
                self.cursor = None;
                return out;
            }

            *frame = self.frames[cursor];
            cursor += 1;
        }

        self.cursor = if cursor >= self.frames.len() {
            None
        } else {
            Some(cursor)
        };

        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PcmWav {
    sample_rate: u32,
    channels: u16,
    bits_per_sample: u16,
    data: Vec<u8>,
}

fn content_contract() -> ContentContract {
    ContentContract::new(SUPPORTED_CONTENT_EXTENSIONS)
        .with_need_fullpath(true)
        .with_block_extract(true)
}

fn parse_pcm_wav(bytes: &[u8]) -> Result<PcmWav, String> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("expected a RIFF/WAVE file".to_string());
    }

    let mut cursor = 12usize;
    let mut format = None;
    let mut data = None;

    while cursor + 8 <= bytes.len() {
        let chunk_id = &bytes[cursor..cursor + 4];
        let chunk_size = u32::from_le_bytes([
            bytes[cursor + 4],
            bytes[cursor + 5],
            bytes[cursor + 6],
            bytes[cursor + 7],
        ]) as usize;
        let body_start = cursor + 8;
        let body_end = body_start + chunk_size;
        if body_end > bytes.len() {
            return Err("WAV chunk extends past end of file".to_string());
        }

        match chunk_id {
            b"fmt " => {
                if chunk_size < 16 {
                    return Err("WAV fmt chunk is too small".to_string());
                }
                let body = &bytes[body_start..body_end];
                let audio_format = u16::from_le_bytes([body[0], body[1]]);
                let channels = u16::from_le_bytes([body[2], body[3]]);
                let sample_rate = u32::from_le_bytes([body[4], body[5], body[6], body[7]]);
                let bits_per_sample = u16::from_le_bytes([body[14], body[15]]);
                format = Some((audio_format, channels, sample_rate, bits_per_sample));
            }
            b"data" => {
                data = Some(bytes[body_start..body_end].to_vec());
            }
            _ => {}
        }

        cursor = body_end + (chunk_size & 1);
    }

    let Some((audio_format, channels, sample_rate, bits_per_sample)) = format else {
        return Err("WAV file is missing a fmt chunk".to_string());
    };
    if audio_format != 1 {
        return Err(format!(
            "only PCM WAV is supported, found format {audio_format}"
        ));
    }
    if !(channels == 1 || channels == 2) {
        return Err(format!("unsupported channel count {channels}"));
    }
    if !(bits_per_sample == 8 || bits_per_sample == 16) {
        return Err(format!("unsupported bits-per-sample {bits_per_sample}"));
    }

    let data = data.ok_or_else(|| "WAV file is missing a data chunk".to_string())?;
    Ok(PcmWav {
        sample_rate,
        channels,
        bits_per_sample,
        data,
    })
}

fn resample_pcm_to_stereo_i16(source: &PcmWav, output_sample_rate: u32) -> Vec<[i16; 2]> {
    let bytes_per_sample = usize::from(source.bits_per_sample / 8);
    let frame_bytes = bytes_per_sample * usize::from(source.channels);
    if frame_bytes == 0 || source.data.len() < frame_bytes {
        return Vec::new();
    }

    let source_frame_count = source.data.len() / frame_bytes;
    let output_frame_count = ((source_frame_count as u64 * output_sample_rate as u64)
        / u64::from(source.sample_rate))
    .max(1) as usize;
    let mut out = Vec::with_capacity(output_frame_count);

    for out_index in 0..output_frame_count {
        let source_index = ((out_index as u64 * u64::from(source.sample_rate))
            / u64::from(output_sample_rate))
        .min((source_frame_count - 1) as u64) as usize;
        let frame_offset = source_index * frame_bytes;

        let frame = match (source.channels, source.bits_per_sample) {
            (1, 8) => {
                let sample = u8_to_i16(source.data[frame_offset]);
                [sample, sample]
            }
            (2, 8) => [
                u8_to_i16(source.data[frame_offset]),
                u8_to_i16(source.data[frame_offset + 1]),
            ],
            (1, 16) => {
                let sample =
                    i16::from_le_bytes([source.data[frame_offset], source.data[frame_offset + 1]]);
                [sample, sample]
            }
            (2, 16) => [
                i16::from_le_bytes([source.data[frame_offset], source.data[frame_offset + 1]]),
                i16::from_le_bytes([source.data[frame_offset + 2], source.data[frame_offset + 3]]),
            ],
            _ => unreachable!("validated unsupported PCM format"),
        };

        out.push(frame);
    }

    out
}

fn u8_to_i16(sample: u8) -> i16 {
    (i16::from(sample) - 128) << 8
}

fn axis_value(negative_pressed: bool, positive_pressed: bool) -> f32 {
    match (negative_pressed, positive_pressed) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    }
}

fn build_triangle_vertices(center: [f32; 2], color: [f32; 3]) -> [[f32; 5]; 3] {
    [
        [
            center[0],
            center[1] + TRIANGLE_HALF_HEIGHT,
            color[0],
            color[1],
            color[2],
        ],
        [
            center[0] - TRIANGLE_HALF_WIDTH,
            center[1] - TRIANGLE_HALF_HEIGHT,
            color[0],
            color[1],
            color[2],
        ],
        [
            center[0] + TRIANGLE_HALF_WIDTH,
            center[1] - TRIANGLE_HALF_HEIGHT,
            color[0],
            color[1],
            color[2],
        ],
    ]
}

fn shader_sources_for(context_type: HwContextType) -> (&'static str, &'static str) {
    match context_type {
        HwContextType::OpenGlCore => (
            OPENGL_CORE_VERTEX_SHADER_SOURCE,
            OPENGL_CORE_FRAGMENT_SHADER_SOURCE,
        ),
        HwContextType::OpenGl => (OPENGL_VERTEX_SHADER_SOURCE, OPENGL_FRAGMENT_SHADER_SOURCE),
        HwContextType::OpenGlEs2 => (
            OPENGLES2_VERTEX_SHADER_SOURCE,
            OPENGLES2_FRAGMENT_SHADER_SOURCE,
        ),
        HwContextType::OpenGlEs3 => (
            OPENGLES3_VERTEX_SHADER_SOURCE,
            OPENGLES3_FRAGMENT_SHADER_SOURCE,
        ),
        other => panic!("unsupported GL context negotiated for demo-libretro: {other:?}"),
    }
}

libretro::export_core!(DemoLibretroCore::default());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_value_ignores_opposing_inputs() {
        assert_eq!(axis_value(false, false), 0.0);
        assert_eq!(axis_value(true, false), -1.0);
        assert_eq!(axis_value(false, true), 1.0);
        assert_eq!(axis_value(true, true), 0.0);
    }

    #[test]
    fn triangle_vertices_are_offset_from_center() {
        let vertices = build_triangle_vertices([0.25, -0.5], TRIANGLE_GREEN);

        assert_eq!(vertices[0][0], 0.25);
        assert_eq!(vertices[0][1], -0.5 + TRIANGLE_HALF_HEIGHT);
        assert_eq!(vertices[1][0], 0.25 - TRIANGLE_HALF_WIDTH);
        assert_eq!(vertices[2][0], 0.25 + TRIANGLE_HALF_WIDTH);
    }

    #[test]
    fn embedded_move_wav_decodes_and_resamples() {
        let sound = MoveSound::from_wav_bytes(MOVE_SOUND_WAV, SAMPLE_RATE_HZ).unwrap();
        assert!(!sound.frames.is_empty());
    }

    #[test]
    fn mono_u8_pcm_is_expanded_to_stereo() {
        let source = PcmWav {
            sample_rate: 16_000,
            channels: 1,
            bits_per_sample: 8,
            data: vec![0, 128, 255],
        };

        let frames = resample_pcm_to_stereo_i16(&source, 16_000);
        assert_eq!(frames[0], [-32768, -32768]);
        assert_eq!(frames[1], [0, 0]);
        assert_eq!(frames[2], [32512, 32512]);
    }
}
