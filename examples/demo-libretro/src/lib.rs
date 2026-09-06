//! Input/audio demo using a standard glow context from the frontend.
use libretro::glow::{self, HasContext};
use libretro::{
    ContentContract, Core, Environment, GameInfo, JoypadButton, PixelFormat, Runtime, SystemAvInfo,
    SystemInfo, fixed_system_av_info, opengl_modern_preferred_hw_render_candidates,
};
#[path = "../../support/triangle.rs"]
mod renderer;
use renderer::TriangleRenderer;
const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
const FPS_HZ: u32 = 60;
const SAMPLE_RATE_HZ: u32 = 48_000;
const FPS: f64 = FPS_HZ as f64;
const SAMPLE_RATE: f64 = SAMPLE_RATE_HZ as f64;

const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
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

struct DemoLibretroCore {
    gl: Option<glow::Context>,
    renderer: Option<TriangleRenderer>,
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
            renderer: None,
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
        let Some(renderer) = self.renderer.as_ref() else {
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
        // SAFETY: run has the same current context used to create these resources.
        unsafe {
            if let Err(error) = renderer.draw(
                gl,
                framebuffer,
                WIDTH as i32,
                HEIGHT as i32,
                &vertices,
                CLEAR_COLOR,
            ) {
                runtime.logger().error(error);
            }
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
        runtime.video_refresh_hw(WIDTH, HEIGHT, 0);
        runtime.audio_sample_batch(&audio_frames);
    }
    fn hw_context_reset(&mut self, runtime: &mut Runtime<'_>) {
        // A reset may replace a lost context. Abandon old names, never delete them
        // through the replacement context.
        self.renderer = None;
        self.gl = None;
        let result = runtime.create_glow_context().and_then(|gl| {
            // SAFETY: reset supplies a current context.
            let renderer = unsafe { TriangleRenderer::new(&gl) }?;
            Ok((gl, renderer))
        });
        match result {
            Ok((gl, renderer)) => {
                self.gl = Some(gl);
                self.renderer = Some(renderer);
            }
            Err(e) => {
                runtime.logger().error(e);
            }
        }
    }
    fn hw_context_destroy(&mut self, _runtime: &mut Runtime<'_>) {
        // SAFETY: the frontend makes the retiring context current during destroy.
        if let (Some(gl), Some(renderer)) = (self.gl.as_ref(), self.renderer.take()) {
            unsafe {
                renderer.destroy(gl);
            }
        }
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
