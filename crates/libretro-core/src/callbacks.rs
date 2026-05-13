use crate::raw;

#[derive(Clone, Copy, Debug)]
pub struct CoreProcAddress {
    raw: raw::retro_proc_address_t,
}

impl CoreProcAddress {
    pub const fn from_fn(function: unsafe extern "C" fn()) -> Self {
        Self {
            raw: Some(function),
        }
    }

    pub(crate) const fn as_raw(self) -> raw::retro_proc_address_t {
        self.raw
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AudioCallbackState {
    #[default]
    Inactive,
    Active,
}

impl AudioCallbackState {
    pub const fn from_active(active: bool) -> Self {
        if active { Self::Active } else { Self::Inactive }
    }

    pub const fn is_active(self) -> bool {
        matches!(self, Self::Active)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AudioBufferOccupancy(u32);

impl AudioBufferOccupancy {
    pub const fn from_percent(percent: u8) -> Option<Self> {
        if percent <= 100 {
            Some(Self(percent as u32))
        } else {
            None
        }
    }

    pub const fn from_raw_percent(percent: u32) -> Self {
        Self(percent)
    }

    pub const fn raw_percent(self) -> u32 {
        self.0
    }

    pub const fn percent(self) -> Option<u8> {
        if self.0 <= 100 {
            Some(self.0 as u8)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AudioBufferStatus {
    pub active: bool,
    pub occupancy: AudioBufferOccupancy,
    pub underrun_likely: bool,
}

impl AudioBufferStatus {
    pub const fn new(active: bool, occupancy: AudioBufferOccupancy, underrun_likely: bool) -> Self {
        Self {
            active,
            occupancy,
            underrun_likely,
        }
    }

    pub const fn from_raw(active: bool, occupancy: u32, underrun_likely: bool) -> Self {
        Self::new(
            active,
            AudioBufferOccupancy::from_raw_percent(occupancy),
            underrun_likely,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FrameTime {
    micros: i64,
}

impl FrameTime {
    pub const fn from_micros(micros: i64) -> Self {
        Self { micros }
    }

    pub const fn as_micros(self) -> i64 {
        self.micros
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AudioBufferOccupancy, AudioBufferStatus, AudioCallbackState, CoreProcAddress, FrameTime,
    };

    unsafe extern "C" fn test_proc_address() {}

    #[test]
    fn core_proc_address_wraps_type_erased_function_pointer() {
        let address = CoreProcAddress::from_fn(test_proc_address)
            .as_raw()
            .expect("function pointer should be present");
        let expected: unsafe extern "C" fn() = test_proc_address;

        assert!(std::ptr::fn_addr_eq(address, expected));
    }

    #[test]
    fn audio_callback_state_encodes_frontend_active_flag() {
        assert_eq!(
            AudioCallbackState::from_active(false),
            AudioCallbackState::Inactive
        );
        assert_eq!(
            AudioCallbackState::from_active(true),
            AudioCallbackState::Active
        );
        assert!(!AudioCallbackState::Inactive.is_active());
        assert!(AudioCallbackState::Active.is_active());
    }

    #[test]
    fn audio_buffer_occupancy_accepts_libretro_percent_range() {
        assert_eq!(
            AudioBufferOccupancy::from_percent(100)
                .expect("100 percent is valid")
                .percent(),
            Some(100)
        );
        assert_eq!(AudioBufferOccupancy::from_percent(101), None);
    }

    #[test]
    fn audio_buffer_status_preserves_out_of_range_frontend_values() {
        let status = AudioBufferStatus::from_raw(true, 150, true);

        assert_eq!(status.occupancy.raw_percent(), 150);
        assert_eq!(status.occupancy.percent(), None);
    }

    #[test]
    fn frame_time_preserves_signed_libretro_values() {
        assert_eq!(FrameTime::from_micros(16_667).as_micros(), 16_667);
        assert_eq!(FrameTime::from_micros(-1).as_micros(), -1);
    }
}
