//! Typed video geometry, timing, and audio pacing helpers.
//!
//! These wrappers keep libretro's AV information explicit at the call site:
//! dimensions are grouped as `GameGeometry`, timing as `SystemTiming`, and
//! common fixed-rate cores can use helpers instead of hand-building raw structs.

use crate::raw;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameGeometry {
    pub base_width: u32,
    pub base_height: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub aspect_ratio: f32,
}

impl GameGeometry {
    pub(crate) fn as_raw(self) -> raw::retro_game_geometry {
        raw::retro_game_geometry {
            base_width: self.base_width,
            base_height: self.base_height,
            max_width: self.max_width,
            max_height: self.max_height,
            aspect_ratio: self.aspect_ratio,
        }
    }

    #[cfg(test)]
    pub(crate) fn from_raw(raw: raw::retro_game_geometry) -> Self {
        Self {
            base_width: raw.base_width,
            base_height: raw.base_height,
            max_width: raw.max_width,
            max_height: raw.max_height,
            aspect_ratio: raw.aspect_ratio,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SystemTiming {
    pub fps: f64,
    pub sample_rate: f64,
}

impl SystemTiming {
    pub(crate) fn as_raw(self) -> raw::retro_system_timing {
        raw::retro_system_timing {
            fps: self.fps,
            sample_rate: self.sample_rate,
        }
    }

    #[cfg(test)]
    pub(crate) fn from_raw(raw: raw::retro_system_timing) -> Self {
        Self {
            fps: raw.fps,
            sample_rate: raw.sample_rate,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SystemAvInfo {
    pub geometry: GameGeometry,
    pub timing: SystemTiming,
}

impl SystemAvInfo {
    pub(crate) fn as_raw(self) -> raw::retro_system_av_info {
        raw::retro_system_av_info {
            geometry: self.geometry.as_raw(),
            timing: self.timing.as_raw(),
        }
    }

    #[cfg(test)]
    pub(crate) fn from_raw(raw: raw::retro_system_av_info) -> Self {
        Self {
            geometry: GameGeometry::from_raw(raw.geometry),
            timing: SystemTiming::from_raw(raw.timing),
        }
    }
}

pub fn game_geometry(width: u32, height: u32) -> GameGeometry {
    bounded_game_geometry(width, height, width, height)
}

pub fn bounded_game_geometry(
    width: u32,
    height: u32,
    max_width: u32,
    max_height: u32,
) -> GameGeometry {
    GameGeometry {
        base_width: width,
        base_height: height,
        max_width,
        max_height,
        aspect_ratio: width as f32 / height as f32,
    }
}

pub fn system_av_info(geometry: GameGeometry, fps: f64, sample_rate: f64) -> SystemAvInfo {
    SystemAvInfo {
        geometry,
        timing: SystemTiming { fps, sample_rate },
    }
}

pub fn fixed_system_av_info(width: u32, height: u32, fps: f64, sample_rate: f64) -> SystemAvInfo {
    system_av_info(game_geometry(width, height), fps, sample_rate)
}

pub const fn exact_audio_frames_per_video_frame(sample_rate_hz: u32, fps_hz: u32) -> usize {
    if fps_hz == 0 {
        panic!("libretro video frame rate must be non-zero");
    }
    if !sample_rate_hz.is_multiple_of(fps_hz) {
        panic!("libretro audio sample rate must divide evenly by video frame rate");
    }

    (sample_rate_hz / fps_hz) as usize
}

pub fn silent_stereo_frames(frame_count: usize) -> Vec<[i16; 2]> {
    vec![[0, 0]; frame_count]
}

pub fn silent_stereo_frames_for_video_frame(sample_rate_hz: u32, fps_hz: u32) -> Vec<[i16; 2]> {
    silent_stereo_frames(exact_audio_frames_per_video_frame(sample_rate_hz, fps_hz))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_av_info_uses_matching_base_and_max_geometry() {
        let av = fixed_system_av_info(320, 240, 60.0, 48_000.0);

        assert_eq!(av.geometry.base_width, 320);
        assert_eq!(av.geometry.base_height, 240);
        assert_eq!(av.geometry.max_width, 320);
        assert_eq!(av.geometry.max_height, 240);
        assert_eq!(av.geometry.aspect_ratio, 4.0 / 3.0);
        assert_eq!(av.timing.fps, 60.0);
        assert_eq!(av.timing.sample_rate, 48_000.0);
    }

    #[test]
    fn bounded_geometry_preserves_requested_maximums() {
        let geometry = bounded_game_geometry(320, 240, 640, 480);

        assert_eq!(geometry.base_width, 320);
        assert_eq!(geometry.base_height, 240);
        assert_eq!(geometry.max_width, 640);
        assert_eq!(geometry.max_height, 480);
    }

    #[test]
    fn silent_stereo_frames_are_zeroed() {
        let frames = silent_stereo_frames(3);

        assert_eq!(frames, vec![[0, 0], [0, 0], [0, 0]]);
    }

    #[test]
    fn exact_audio_frames_per_video_frame_matches_libretro_sixty_hz_pacing() {
        assert_eq!(exact_audio_frames_per_video_frame(48_000, 60), 800);
        assert_eq!(
            silent_stereo_frames_for_video_frame(48_000, 60),
            vec![[0, 0]; 800]
        );
    }

    #[test]
    #[should_panic(expected = "libretro audio sample rate must divide evenly")]
    fn exact_audio_frames_per_video_frame_rejects_fractional_batches() {
        let _ = exact_audio_frames_per_video_frame(44_100, 64);
    }
}
