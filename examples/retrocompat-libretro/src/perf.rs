use std::time::Duration;

pub(crate) const PERF_REPORT_INTERVAL_FRAMES: u32 = 60;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PerfFrameSample {
    pub total: Duration,
    pub render_submit: Duration,
    pub video_refresh: Duration,
    pub audio_flush: Duration,
    pub draw_submissions: u32,
    pub uploads: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct PerfReport {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub render_submit_ms: f32,
    pub video_refresh_ms: f32,
    pub audio_ms: f32,
    pub draw_submissions: u32,
    pub uploads: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PerfAccumulator {
    frames: u32,
    total: Duration,
    render_submit: Duration,
    video_refresh: Duration,
    audio_flush: Duration,
    draw_submissions: u32,
    uploads: u32,
}

impl PerfAccumulator {
    pub(crate) fn push(&mut self, sample: PerfFrameSample) -> Option<PerfReport> {
        self.frames = self.frames.saturating_add(1);
        self.total += sample.total;
        self.render_submit += sample.render_submit;
        self.video_refresh += sample.video_refresh;
        self.audio_flush += sample.audio_flush;
        self.draw_submissions = self
            .draw_submissions
            .saturating_add(sample.draw_submissions);
        self.uploads = self.uploads.saturating_add(sample.uploads);

        if self.frames < PERF_REPORT_INTERVAL_FRAMES {
            return None;
        }

        let frames = self.frames;
        let frame_time_ms = duration_ms(average_duration(self.total, frames)) as f32;
        let report = PerfReport {
            fps: if frame_time_ms > 0.0 {
                1_000.0 / frame_time_ms
            } else {
                0.0
            },
            frame_time_ms,
            render_submit_ms: duration_ms(average_duration(self.render_submit, frames)) as f32,
            video_refresh_ms: duration_ms(average_duration(self.video_refresh, frames)) as f32,
            audio_ms: duration_ms(average_duration(self.audio_flush, frames)) as f32,
            draw_submissions: average_counter(self.draw_submissions, frames),
            uploads: average_counter(self.uploads, frames),
        };
        *self = Self::default();
        Some(report)
    }
}

pub(crate) fn format_perf_lines(stats: Option<PerfReport>) -> [String; 3] {
    match stats {
        Some(stats) => [
            format!("FPS {:.1} FRAME {:.1}ms", stats.fps, stats.frame_time_ms),
            format!(
                "TICK 0.0ms SUBMIT {:.1}ms REFRESH {:.1}ms",
                stats.render_submit_ms, stats.video_refresh_ms
            ),
            format!(
                "AUDIO {:.1}ms DRAWS {} UPLOADS {}",
                stats.audio_ms, stats.draw_submissions, stats.uploads
            ),
        ],
        None => [
            "FPS --.- FRAME --.-ms".to_string(),
            "TICK 0.0ms SUBMIT --.-ms REFRESH --.-ms".to_string(),
            "AUDIO --.-ms DRAWS --- UPLOADS --".to_string(),
        ],
    }
}

fn average_duration(total: Duration, count: u32) -> Duration {
    if count == 0 {
        Duration::ZERO
    } else {
        total / count
    }
}

fn average_counter(total: u32, count: u32) -> u32 {
    if count == 0 {
        0
    } else {
        (total + (count / 2)) / count
    }
}

fn duration_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_one_second_sixty_frame_average() {
        let mut accumulator = PerfAccumulator::default();
        let mut report = None;
        for _ in 0..PERF_REPORT_INTERVAL_FRAMES {
            report = accumulator.push(PerfFrameSample {
                total: Duration::from_micros(16_667),
                render_submit: Duration::from_micros(700),
                video_refresh: Duration::from_micros(15_800),
                audio_flush: Duration::from_micros(100),
                draw_submissions: 2,
                uploads: 0,
            });
        }

        let report = report.expect("report after interval");
        assert!((report.fps - 60.0).abs() < 0.1);
        assert!((report.render_submit_ms - 0.7).abs() < 0.01);
        assert!((report.video_refresh_ms - 15.8).abs() < 0.01);
        assert_eq!(report.draw_submissions, 2);
        assert_eq!(report.uploads, 0);
    }

    #[test]
    fn formats_three_readable_lines() {
        let lines = format_perf_lines(Some(PerfReport {
            fps: 59.9,
            frame_time_ms: 16.7,
            render_submit_ms: 0.2,
            video_refresh_ms: 15.9,
            audio_ms: 0.1,
            draw_submissions: 2,
            uploads: 0,
        }));

        assert_eq!(lines[0], "FPS 59.9 FRAME 16.7ms");
        assert_eq!(lines[1], "TICK 0.0ms SUBMIT 0.2ms REFRESH 15.9ms");
        assert_eq!(lines[2], "AUDIO 0.1ms DRAWS 2 UPLOADS 0");
    }
}
