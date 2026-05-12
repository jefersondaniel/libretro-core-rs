use libretro::{
    ContentContract, Core, Environment, GameInfo, Runtime, SystemAvInfo, SystemInfo,
    fixed_system_av_info, silent_stereo_frames_for_video_frame,
};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const FPS_HZ: u32 = 60;
const SAMPLE_RATE_HZ: u32 = 48_000;
const FPS: f64 = FPS_HZ as f64;
const SAMPLE_RATE: f64 = SAMPLE_RATE_HZ as f64;
const CORE_VERSION: &str = "0.1.0";
const SUPPORTED_CONTENT_EXTENSIONS: &str = "bin|dat";
const BLUE_0RGB1555: u16 = 0x001f;

#[derive(Debug)]
struct SoftwareLibretroCore {
    framebuffer: Vec<u16>,
    silence: Vec<[i16; 2]>,
}

impl Default for SoftwareLibretroCore {
    fn default() -> Self {
        Self {
            framebuffer: vec![BLUE_0RGB1555; (WIDTH * HEIGHT) as usize],
            silence: silent_stereo_frames_for_video_frame(SAMPLE_RATE_HZ, FPS_HZ),
        }
    }
}

impl Core for SoftwareLibretroCore {
    fn system_info(&self) -> SystemInfo {
        let mut info = SystemInfo::new("software-libretro", CORE_VERSION);
        content_contract().apply_to_system_info(&mut info);
        info
    }

    fn av_info(&self) -> SystemAvInfo {
        fixed_system_av_info(WIDTH, HEIGHT, FPS, SAMPLE_RATE)
    }

    fn on_set_environment(&mut self, env: &mut Environment<'_>) {
        let _ = content_contract().register_environment(env);
    }

    fn load_game(&mut self, _game: Option<GameInfo<'_>>, _runtime: &mut Runtime<'_>) -> bool {
        true
    }

    fn run(&mut self, runtime: &mut Runtime<'_>) {
        runtime.poll_input();
        self.framebuffer.fill(BLUE_0RGB1555);
        let _ = runtime.video_refresh_frame_with_audio(
            &self.framebuffer,
            WIDTH,
            HEIGHT,
            WIDTH as usize * std::mem::size_of::<u16>(),
            &self.silence,
        );
    }
}

fn content_contract() -> ContentContract {
    ContentContract::new(SUPPORTED_CONTENT_EXTENSIONS).with_support_no_game(true)
}

libretro::export_core!(SoftwareLibretroCore::default());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blue_frame_uses_default_libretro_0rgb1555_format() {
        let core = SoftwareLibretroCore::default();

        assert_eq!(core.framebuffer.len(), (WIDTH * HEIGHT) as usize);
        assert!(core.framebuffer.iter().all(|pixel| *pixel == BLUE_0RGB1555));
        assert_eq!(WIDTH as usize * std::mem::size_of::<u16>(), 640);
    }

    #[test]
    fn silent_audio_batch_matches_one_frame_at_sixty_hz() {
        let core = SoftwareLibretroCore::default();

        assert_eq!(core.silence.len(), 800);
        assert!(core.silence.iter().all(|frame| *frame == [0, 0]));
    }
}
