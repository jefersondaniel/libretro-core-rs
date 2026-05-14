use libretro::{
    ContentContract, Core, Environment, GameInfo, Runtime, SystemAvInfo, SystemInfo,
    fixed_system_av_info, silent_stereo_frames_for_video_frame,
};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const FPS: u32 = 60;
const SAMPLE_RATE: u32 = 48_000;
const BLUE_0RGB1555: u16 = 0x001F;

#[derive(Default)]
struct HelloCore;

impl Core for HelloCore {
    fn system_info(&self) -> SystemInfo {
        SystemInfo::new("hello-libretro", env!("CARGO_PKG_VERSION"))
    }

    fn av_info(&self) -> SystemAvInfo {
        fixed_system_av_info(WIDTH, HEIGHT, FPS as f64, SAMPLE_RATE as f64)
    }

    fn on_set_environment(&mut self, env: &mut Environment<'_>) {
        let _ = ContentContract::new("")
            .with_support_no_game(true)
            .register_environment(env);
    }

    fn load_game(&mut self, _game: Option<GameInfo<'_>>, _rt: &mut Runtime<'_>) -> bool {
        true
    }

    fn run(&mut self, rt: &mut Runtime<'_>) {
        rt.poll_input();

        let frame = vec![BLUE_0RGB1555; (WIDTH * HEIGHT) as usize];
        let audio = silent_stereo_frames_for_video_frame(SAMPLE_RATE, FPS);

        let _ = rt.video_refresh_frame_with_audio(
            &frame,
            WIDTH,
            HEIGHT,
            WIDTH as usize * std::mem::size_of::<u16>(),
            &audio,
        );
    }
}

libretro::export_core!(HelloCore::default());
