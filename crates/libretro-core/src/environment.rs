//! Typed values for libretro environment commands and frontend state.
//!
//! `Environment` methods in `lib.rs` use these enums and newtypes to expose
//! command-specific semantics for messages, language, power, throttling,
//! fast-forward, latency, rates, rotation, and AV enable hints.

use std::ffi::{CStr, c_char, c_void};
use std::ptr;

use enumflags2::{BitFlags, bitflags};

use crate::{
    CameraInterface, CameraRequest, ControllerInfo, DiskControlInterfaceVersion, Environment,
    ExtendedGameInfo, FrameTime, InputDescriptor, InputDescriptorStorage, InputDeviceCapabilities,
    LedInterface, LocationInterface, LogLevel, MemoryMapDescriptor, MicrophoneInterface,
    MidiInterface, NetplayClientId, PerfInterface, RumbleInterface, SavestateContext,
    SensorInterface, SerializationQuirks, SoftwareFramebuffer, SoftwareFramebufferRequest,
    SubsystemInfo, SystemAvInfo, raw,
};

#[bitflags]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AvEnable {
    Video = raw::RETRO_AV_ENABLE_VIDEO,
    Audio = raw::RETRO_AV_ENABLE_AUDIO,
    FastSavestates = raw::RETRO_AV_ENABLE_FAST_SAVESTATES,
    HardDisableAudio = raw::RETRO_AV_ENABLE_HARD_DISABLE_AUDIO,
}

pub type AvEnableFlags = BitFlags<AvEnable>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum VideoRotation {
    #[default]
    Clockwise0,
    CounterClockwise90,
    CounterClockwise180,
    CounterClockwise270,
}

impl VideoRotation {
    pub const fn as_raw(self) -> u32 {
        match self {
            Self::Clockwise0 => 0,
            Self::CounterClockwise90 => 1,
            Self::CounterClockwise180 => 2,
            Self::CounterClockwise270 => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PerformanceLevel(u32);

impl PerformanceLevel {
    pub const fn new(level: u32) -> Self {
        Self(level)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ThrottleMode {
    #[default]
    None,
    FrameStepping,
    FastForward,
    SlowMotion,
    Rewinding,
    Vsync,
    Unblocked,
    Unknown(u32),
}

impl ThrottleMode {
    pub const fn from_raw(mode: u32) -> Self {
        match mode {
            raw::RETRO_THROTTLE_NONE => Self::None,
            raw::RETRO_THROTTLE_FRAME_STEPPING => Self::FrameStepping,
            raw::RETRO_THROTTLE_FAST_FORWARD => Self::FastForward,
            raw::RETRO_THROTTLE_SLOW_MOTION => Self::SlowMotion,
            raw::RETRO_THROTTLE_REWINDING => Self::Rewinding,
            raw::RETRO_THROTTLE_VSYNC => Self::Vsync,
            raw::RETRO_THROTTLE_UNBLOCKED => Self::Unblocked,
            other => Self::Unknown(other),
        }
    }

    pub const fn as_raw(self) -> u32 {
        match self {
            Self::None => raw::RETRO_THROTTLE_NONE,
            Self::FrameStepping => raw::RETRO_THROTTLE_FRAME_STEPPING,
            Self::FastForward => raw::RETRO_THROTTLE_FAST_FORWARD,
            Self::SlowMotion => raw::RETRO_THROTTLE_SLOW_MOTION,
            Self::Rewinding => raw::RETRO_THROTTLE_REWINDING,
            Self::Vsync => raw::RETRO_THROTTLE_VSYNC,
            Self::Unblocked => raw::RETRO_THROTTLE_UNBLOCKED,
            Self::Unknown(mode) => mode,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RunLoopRateHz(f32);

impl RunLoopRateHz {
    pub fn new(hz: f32) -> Option<Self> {
        (hz.is_finite() && hz > 0.0).then_some(Self(hz))
    }

    pub const fn get(self) -> f32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThrottleState {
    pub mode: ThrottleMode,
    pub rate: Option<RunLoopRateHz>,
}

impl ThrottleState {
    fn from_raw(state: raw::retro_throttle_state) -> Self {
        Self {
            mode: ThrottleMode::from_raw(state.mode),
            rate: RunLoopRateHz::new(state.rate),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FastForwardRatio(f32);

impl FastForwardRatio {
    pub const fn frontend_default() -> Self {
        Self(-1.0)
    }

    pub const fn normal_speed() -> Self {
        Self(1.0)
    }

    pub const fn unlimited() -> Self {
        Self(0.0)
    }

    pub fn multiplier(ratio: f32) -> Option<Self> {
        (ratio.is_finite() && ratio > 1.0).then_some(Self(ratio))
    }

    pub const fn get(self) -> f32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FastForwardingOverride {
    pub ratio: FastForwardRatio,
    pub fastforward: bool,
    pub notification: bool,
    pub inhibit_toggle: bool,
}

impl FastForwardingOverride {
    pub fn enable() -> Self {
        Self {
            ratio: FastForwardRatio::frontend_default(),
            fastforward: true,
            notification: true,
            inhibit_toggle: false,
        }
    }

    pub fn disable() -> Self {
        Self {
            fastforward: false,
            ..Self::enable()
        }
    }

    pub fn with_ratio(mut self, ratio: FastForwardRatio) -> Self {
        self.ratio = ratio;
        self
    }

    pub fn with_notification(mut self, notification: bool) -> Self {
        self.notification = notification;
        self
    }

    pub fn with_inhibit_toggle(mut self, inhibit_toggle: bool) -> Self {
        self.inhibit_toggle = inhibit_toggle;
        self
    }

    fn into_raw(self) -> raw::retro_fastforwarding_override {
        raw::retro_fastforwarding_override {
            ratio: self.ratio.get(),
            fastforward: self.fastforward,
            notification: self.notification,
            inhibit_toggle: self.inhibit_toggle,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PowerState {
    #[default]
    Unknown,
    Discharging,
    Charging,
    Charged,
    PluggedIn,
}

impl PowerState {
    const fn from_raw(state: raw::retro_power_state) -> Self {
        match state {
            raw::retro_power_state::Unknown => Self::Unknown,
            raw::retro_power_state::Discharging => Self::Discharging,
            raw::retro_power_state::Charging => Self::Charging,
            raw::retro_power_state::Charged => Self::Charged,
            raw::retro_power_state::PluggedIn => Self::PluggedIn,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DevicePower {
    pub state: PowerState,
    pub seconds_remaining: Option<u32>,
    pub percent: Option<u8>,
}

impl DevicePower {
    fn from_raw(power: raw::retro_device_power) -> Self {
        Self {
            state: PowerState::from_raw(power.state),
            seconds_remaining: u32::try_from(power.seconds).ok(),
            percent: u8::try_from(power.percent)
                .ok()
                .filter(|percent| *percent <= 100),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MessageTarget {
    All,
    #[default]
    Osd,
    Log,
}

impl MessageTarget {
    const fn as_raw(self) -> raw::retro_message_target {
        match self {
            Self::All => raw::retro_message_target::All,
            Self::Osd => raw::retro_message_target::Osd,
            Self::Log => raw::retro_message_target::Log,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MessageKind {
    #[default]
    Notification,
    NotificationAlt,
    Status,
    Progress,
}

impl MessageKind {
    const fn as_raw(self) -> raw::retro_message_type {
        match self {
            Self::Notification => raw::retro_message_type::Notification,
            Self::NotificationAlt => raw::retro_message_type::NotificationAlt,
            Self::Status => raw::retro_message_type::Status,
            Self::Progress => raw::retro_message_type::Progress,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MessageProgress {
    Indeterminate,
    Percent(u8),
}

impl MessageProgress {
    pub fn percent(percent: u8) -> Option<Self> {
        (percent <= 100).then_some(Self::Percent(percent))
    }

    const fn as_raw(self) -> i8 {
        match self {
            Self::Indeterminate => -1,
            Self::Percent(percent) => percent as i8,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtendedMessage {
    message: String,
    duration_millis: u32,
    priority: u32,
    level: LogLevel,
    target: MessageTarget,
    kind: MessageKind,
    progress: MessageProgress,
}

impl ExtendedMessage {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            duration_millis: 0,
            priority: 0,
            level: LogLevel::Info,
            target: MessageTarget::Osd,
            kind: MessageKind::Notification,
            progress: MessageProgress::Indeterminate,
        }
    }

    pub fn with_duration_millis(mut self, duration_millis: u32) -> Self {
        self.duration_millis = duration_millis;
        self
    }

    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    pub fn with_target(mut self, target: MessageTarget) -> Self {
        self.target = target;
        self
    }

    pub fn with_kind(mut self, kind: MessageKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn with_progress(mut self, progress: MessageProgress) -> Self {
        self.progress = progress;
        self.kind = MessageKind::Progress;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AudioLatencyMillis(u32);

impl AudioLatencyMillis {
    pub const fn new(milliseconds: u32) -> Self {
        Self(milliseconds)
    }

    pub const fn as_millis(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RefreshRateHz(f32);

impl RefreshRateHz {
    pub const fn new(hz: f32) -> Self {
        Self(hz)
    }

    pub const fn get(self) -> f32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AudioSampleRateHz(u32);

impl AudioSampleRateHz {
    pub const fn new(hz: u32) -> Self {
        Self(hz)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Language {
    #[default]
    English,
    Japanese,
    French,
    Spanish,
    German,
    Italian,
    Dutch,
    PortugueseBrazil,
    PortuguesePortugal,
    Russian,
    Korean,
    ChineseTraditional,
    ChineseSimplified,
    Esperanto,
    Polish,
    Vietnamese,
    Arabic,
    Greek,
    Turkish,
    Slovak,
    Persian,
    Hebrew,
    Asturian,
    Finnish,
    Indonesian,
    Swedish,
    Ukrainian,
    Czech,
    CatalanValencia,
    Catalan,
    BritishEnglish,
    Hungarian,
    Belarusian,
    Galician,
    Norwegian,
    Irish,
    Unknown(i32),
}

impl Language {
    pub const fn from_raw(language: i32) -> Self {
        match language {
            0 => Self::English,
            1 => Self::Japanese,
            2 => Self::French,
            3 => Self::Spanish,
            4 => Self::German,
            5 => Self::Italian,
            6 => Self::Dutch,
            7 => Self::PortugueseBrazil,
            8 => Self::PortuguesePortugal,
            9 => Self::Russian,
            10 => Self::Korean,
            11 => Self::ChineseTraditional,
            12 => Self::ChineseSimplified,
            13 => Self::Esperanto,
            14 => Self::Polish,
            15 => Self::Vietnamese,
            16 => Self::Arabic,
            17 => Self::Greek,
            18 => Self::Turkish,
            19 => Self::Slovak,
            20 => Self::Persian,
            21 => Self::Hebrew,
            22 => Self::Asturian,
            23 => Self::Finnish,
            24 => Self::Indonesian,
            25 => Self::Swedish,
            26 => Self::Ukrainian,
            27 => Self::Czech,
            28 => Self::CatalanValencia,
            29 => Self::Catalan,
            30 => Self::BritishEnglish,
            31 => Self::Hungarian,
            32 => Self::Belarusian,
            33 => Self::Galician,
            34 => Self::Norwegian,
            35 => Self::Irish,
            other => Self::Unknown(other),
        }
    }

    pub const fn as_raw(self) -> i32 {
        match self {
            Self::English => 0,
            Self::Japanese => 1,
            Self::French => 2,
            Self::Spanish => 3,
            Self::German => 4,
            Self::Italian => 5,
            Self::Dutch => 6,
            Self::PortugueseBrazil => 7,
            Self::PortuguesePortugal => 8,
            Self::Russian => 9,
            Self::Korean => 10,
            Self::ChineseTraditional => 11,
            Self::ChineseSimplified => 12,
            Self::Esperanto => 13,
            Self::Polish => 14,
            Self::Vietnamese => 15,
            Self::Arabic => 16,
            Self::Greek => 17,
            Self::Turkish => 18,
            Self::Slovak => 19,
            Self::Persian => 20,
            Self::Hebrew => 21,
            Self::Asturian => 22,
            Self::Finnish => 23,
            Self::Indonesian => 24,
            Self::Swedish => 25,
            Self::Ukrainian => 26,
            Self::Czech => 27,
            Self::CatalanValencia => 28,
            Self::Catalan => 29,
            Self::BritishEnglish => 30,
            Self::Hungarian => 31,
            Self::Belarusian => 32,
            Self::Galician => 33,
            Self::Norwegian => 34,
            Self::Irish => 35,
            Self::Unknown(language) => language,
        }
    }
}

impl Environment<'_> {
    pub fn set_rotation(&mut self, rotation: VideoRotation) -> bool {
        let mut raw_rotation = rotation.as_raw();
        self.call_env(
            raw::RETRO_ENVIRONMENT_SET_ROTATION,
            (&mut raw_rotation as *mut u32).cast::<c_void>(),
        )
    }

    pub fn overscan(&mut self) -> Option<bool> {
        let mut overscan = false;
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_OVERSCAN,
            (&mut overscan as *mut bool).cast::<c_void>(),
        ) {
            Some(overscan)
        } else {
            None
        }
    }

    pub fn can_dupe_frames(&mut self) -> Option<bool> {
        let mut can_dupe = false;
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_CAN_DUPE,
            (&mut can_dupe as *mut bool).cast::<c_void>(),
        ) {
            Some(can_dupe)
        } else {
            None
        }
    }

    pub fn shutdown(&mut self) -> bool {
        self.call_env(raw::RETRO_ENVIRONMENT_SHUTDOWN, ptr::null_mut())
    }

    pub fn set_system_av_info(&mut self, info: SystemAvInfo) -> bool {
        let mut info = info.as_raw();
        self.call_env(
            raw::RETRO_ENVIRONMENT_SET_SYSTEM_AV_INFO,
            (&mut info as *mut raw::retro_system_av_info).cast::<c_void>(),
        )
    }

    pub fn extended_game_info(&mut self) -> Option<ExtendedGameInfo<'_>> {
        self.extended_game_infos(1)
            .and_then(|mut infos| infos.pop())
    }

    pub fn extended_game_infos(&mut self, count: usize) -> Option<Vec<ExtendedGameInfo<'_>>> {
        if count == 0 {
            return Some(Vec::new());
        }

        let mut infos = ptr::null::<raw::retro_game_info_ext>();
        if !self.call_env(
            raw::RETRO_ENVIRONMENT_GET_GAME_INFO_EXT,
            (&mut infos as *mut *const raw::retro_game_info_ext).cast::<c_void>(),
        ) || infos.is_null()
        {
            return None;
        }

        // SAFETY: On success, libretro returns an array whose length is one
        // during retro_load_game or `num_info` during retro_load_game_special.
        let infos = unsafe { std::slice::from_raw_parts(infos, count) };
        Some(
            infos
                .iter()
                .map(|info| unsafe { ExtendedGameInfo::from_raw(info) })
                .collect(),
        )
    }

    pub fn set_performance_level(&mut self, level: PerformanceLevel) -> bool {
        let mut raw_level = level.get();
        self.call_env(
            raw::RETRO_ENVIRONMENT_SET_PERFORMANCE_LEVEL,
            (&mut raw_level as *mut u32).cast::<c_void>(),
        )
    }

    pub fn perf_interface(&mut self) -> Option<PerfInterface> {
        let mut callback = raw::retro_perf_callback::default();
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_PERF_INTERFACE,
            (&mut callback as *mut raw::retro_perf_callback).cast::<c_void>(),
        ) {
            Some(PerfInterface::from_raw(callback))
        } else {
            None
        }
    }

    pub fn system_directory(&mut self) -> Option<String> {
        self.frontend_string(raw::RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY)
    }

    pub fn libretro_path(&mut self) -> Option<String> {
        self.frontend_string(raw::RETRO_ENVIRONMENT_GET_LIBRETRO_PATH)
    }

    pub fn core_assets_directory(&mut self) -> Option<String> {
        self.frontend_string(raw::RETRO_ENVIRONMENT_GET_CORE_ASSETS_DIRECTORY)
    }

    pub fn content_directory(&mut self) -> Option<String> {
        self.frontend_string(raw::RETRO_ENVIRONMENT_GET_CONTENT_DIRECTORY)
    }

    pub fn save_directory(&mut self) -> Option<String> {
        self.frontend_string(raw::RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY)
    }

    pub fn username(&mut self) -> Option<String> {
        self.frontend_string(raw::RETRO_ENVIRONMENT_GET_USERNAME)
    }

    pub fn playlist_directory(&mut self) -> Option<String> {
        self.frontend_string(raw::RETRO_ENVIRONMENT_GET_PLAYLIST_DIRECTORY)
    }

    pub fn file_browser_start_directory(&mut self) -> Option<String> {
        self.frontend_string(raw::RETRO_ENVIRONMENT_GET_FILE_BROWSER_START_DIRECTORY)
    }

    pub fn language(&mut self) -> Option<Language> {
        let mut language = raw::retro_language::English as i32;
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_LANGUAGE,
            (&mut language as *mut i32).cast::<c_void>(),
        ) {
            Some(Language::from_raw(language))
        } else {
            None
        }
    }

    pub fn jit_capable(&mut self) -> Option<bool> {
        let mut capable = false;
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_JIT_CAPABLE,
            (&mut capable as *mut bool).cast::<c_void>(),
        ) {
            Some(capable)
        } else {
            None
        }
    }

    pub fn set_support_achievements(&mut self, supported: bool) -> bool {
        let mut raw_supported = supported;
        self.call_env(
            raw::RETRO_ENVIRONMENT_SET_SUPPORT_ACHIEVEMENTS,
            (&mut raw_supported as *mut bool).cast::<c_void>(),
        )
    }

    pub fn audio_video_enable(&mut self) -> Option<AvEnableFlags> {
        let mut flags = 0u32;
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_AUDIO_VIDEO_ENABLE,
            (&mut flags as *mut u32).cast::<c_void>(),
        ) {
            Some(AvEnableFlags::from_bits_truncate(flags))
        } else {
            None
        }
    }

    pub fn fastforwarding(&mut self) -> Option<bool> {
        let mut fastforwarding = false;
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_FASTFORWARDING,
            (&mut fastforwarding as *mut bool).cast::<c_void>(),
        ) {
            Some(fastforwarding)
        } else {
            None
        }
    }

    pub fn fastforwarding_override_supported(&mut self) -> bool {
        self.call_env(
            raw::RETRO_ENVIRONMENT_SET_FASTFORWARDING_OVERRIDE,
            ptr::null_mut(),
        )
    }

    pub fn set_fastforwarding_override(&mut self, override_: FastForwardingOverride) -> bool {
        let mut override_ = override_.into_raw();
        self.call_env(
            raw::RETRO_ENVIRONMENT_SET_FASTFORWARDING_OVERRIDE,
            (&mut override_ as *mut raw::retro_fastforwarding_override).cast::<c_void>(),
        )
    }

    pub fn target_refresh_rate(&mut self) -> Option<RefreshRateHz> {
        let mut hz = 0.0f32;
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_TARGET_REFRESH_RATE,
            (&mut hz as *mut f32).cast::<c_void>(),
        ) {
            Some(RefreshRateHz::new(hz))
        } else {
            None
        }
    }

    pub fn target_sample_rate(&mut self) -> Option<AudioSampleRateHz> {
        let mut hz = 0u32;
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_TARGET_SAMPLE_RATE,
            (&mut hz as *mut u32).cast::<c_void>(),
        ) {
            Some(AudioSampleRateHz::new(hz))
        } else {
            None
        }
    }

    pub fn throttle_state(&mut self) -> Option<ThrottleState> {
        let mut state = raw::retro_throttle_state::default();
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_THROTTLE_STATE,
            (&mut state as *mut raw::retro_throttle_state).cast::<c_void>(),
        ) {
            Some(ThrottleState::from_raw(state))
        } else {
            None
        }
    }

    pub fn savestate_context(&mut self) -> Option<SavestateContext> {
        let mut context = SavestateContext::Normal.as_raw();
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_SAVESTATE_CONTEXT,
            (&mut context as *mut i32).cast::<c_void>(),
        ) {
            Some(SavestateContext::from_raw(context))
        } else {
            None
        }
    }

    pub fn set_memory_maps(&mut self, descriptors: &[MemoryMapDescriptor<'_>]) -> bool {
        let Ok(num_descriptors) = u32::try_from(descriptors.len()) else {
            return false;
        };

        let mut addrspaces = Vec::with_capacity(descriptors.len());
        let mut raw_descriptors = Vec::with_capacity(descriptors.len());
        for descriptor in descriptors {
            let addrspace = descriptor.addrspace.as_ref().map(crate::sanitize_cstring);
            let addrspace_ptr = addrspace
                .as_ref()
                .map(|addrspace| addrspace.as_ptr())
                .unwrap_or(ptr::null());
            raw_descriptors.push(raw::retro_memory_descriptor {
                flags: descriptor.raw_flags(),
                ptr: descriptor
                    .ptr
                    .map(|ptr| ptr.as_ptr().cast::<c_void>())
                    .unwrap_or(ptr::null_mut()),
                offset: descriptor.offset.as_usize(),
                start: descriptor.start.as_usize(),
                select: descriptor.select.as_usize(),
                disconnect: descriptor.disconnect.as_usize(),
                len: descriptor.len.as_usize(),
                addrspace: addrspace_ptr,
            });
            addrspaces.push(addrspace);
        }

        let mut map = raw::retro_memory_map {
            descriptors: raw_descriptors.as_ptr(),
            num_descriptors,
        };
        self.call_env(
            raw::RETRO_ENVIRONMENT_SET_MEMORY_MAPS,
            (&mut map as *mut raw::retro_memory_map).cast::<c_void>(),
        )
    }

    pub fn set_controller_info(&mut self, ports: &[ControllerInfo]) -> bool {
        let mut descriptions = Vec::with_capacity(ports.len());
        let mut raw_description_groups = Vec::with_capacity(ports.len());
        for port in ports {
            if port.types.is_empty() {
                return false;
            }
            let Ok(num_types) = u32::try_from(port.types.len()) else {
                return false;
            };

            let mut port_descriptions = Vec::with_capacity(port.types.len());
            let mut raw_descriptions = Vec::with_capacity(port.types.len());
            for controller in &port.types {
                let description = crate::sanitize_cstring(&controller.description);
                raw_descriptions.push(raw::retro_controller_description {
                    desc: description.as_ptr(),
                    id: controller.device.as_raw(),
                });
                port_descriptions.push(description);
            }

            descriptions.push(port_descriptions);
            raw_description_groups.push((raw_descriptions, num_types));
        }

        let mut raw_ports = raw_description_groups
            .iter()
            .map(|(raw_descriptions, num_types)| raw::retro_controller_info {
                types: raw_descriptions.as_ptr(),
                num_types: *num_types,
            })
            .collect::<Vec<_>>();
        raw_ports.push(raw::retro_controller_info::default());

        let ok = self.call_env(
            raw::RETRO_ENVIRONMENT_SET_CONTROLLER_INFO,
            raw_ports.as_mut_ptr().cast::<c_void>(),
        );
        drop(descriptions);
        ok
    }

    pub fn set_proc_address_callback(&mut self) -> bool {
        let mut interface = raw::retro_get_proc_address_interface {
            get_proc_address: Some(crate::proc_address_trampoline),
        };
        self.call_env(
            raw::RETRO_ENVIRONMENT_SET_PROC_ADDRESS_CALLBACK,
            (&mut interface as *mut raw::retro_get_proc_address_interface).cast::<c_void>(),
        )
    }

    pub fn set_subsystem_info(&mut self, subsystems: &[SubsystemInfo]) -> bool {
        let mut storage = crate::subsystem::SubsystemInfoStorage::new(subsystems);
        let ok = self.call_env(
            raw::RETRO_ENVIRONMENT_SET_SUBSYSTEM_INFO,
            storage.raw.as_mut_ptr().cast::<c_void>(),
        );
        if ok {
            self.state.subsystem_info = Some(storage);
        }
        ok
    }

    pub fn disk_control_interface_version(&mut self) -> Option<DiskControlInterfaceVersion> {
        let mut version = 0u32;
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_DISK_CONTROL_INTERFACE_VERSION,
            (&mut version as *mut u32).cast::<c_void>(),
        ) {
            Some(DiskControlInterfaceVersion::new(version))
        } else {
            None
        }
    }

    pub fn set_disk_control_interface(&mut self) -> bool {
        let mut callback = raw::retro_disk_control_callback {
            set_eject_state: Some(crate::disk_set_eject_state_trampoline),
            get_eject_state: Some(crate::disk_get_eject_state_trampoline),
            get_image_index: Some(crate::disk_get_image_index_trampoline),
            set_image_index: Some(crate::disk_set_image_index_trampoline),
            get_num_images: Some(crate::disk_get_num_images_trampoline),
            replace_image_index: Some(crate::disk_replace_image_index_trampoline),
            add_image_index: Some(crate::disk_add_image_index_trampoline),
        };
        self.call_env(
            raw::RETRO_ENVIRONMENT_SET_DISK_CONTROL_INTERFACE,
            (&mut callback as *mut raw::retro_disk_control_callback).cast::<c_void>(),
        )
    }

    pub fn clear_disk_control_interface(&mut self) -> bool {
        self.call_env(
            raw::RETRO_ENVIRONMENT_SET_DISK_CONTROL_INTERFACE,
            ptr::null_mut(),
        )
    }

    pub fn set_disk_control_ext_interface(&mut self) -> bool {
        let mut callback = raw::retro_disk_control_ext_callback {
            set_eject_state: Some(crate::disk_set_eject_state_trampoline),
            get_eject_state: Some(crate::disk_get_eject_state_trampoline),
            get_image_index: Some(crate::disk_get_image_index_trampoline),
            set_image_index: Some(crate::disk_set_image_index_trampoline),
            get_num_images: Some(crate::disk_get_num_images_trampoline),
            replace_image_index: Some(crate::disk_replace_image_index_trampoline),
            add_image_index: Some(crate::disk_add_image_index_trampoline),
            set_initial_image: Some(crate::disk_set_initial_image_trampoline),
            get_image_path: Some(crate::disk_get_image_path_trampoline),
            get_image_label: Some(crate::disk_get_image_label_trampoline),
        };
        self.call_env(
            raw::RETRO_ENVIRONMENT_SET_DISK_CONTROL_EXT_INTERFACE,
            (&mut callback as *mut raw::retro_disk_control_ext_callback).cast::<c_void>(),
        )
    }

    pub fn clear_disk_control_ext_interface(&mut self) -> bool {
        self.call_env(
            raw::RETRO_ENVIRONMENT_SET_DISK_CONTROL_EXT_INTERFACE,
            ptr::null_mut(),
        )
    }

    pub fn set_netpacket_interface(&mut self, protocol_version: Option<&str>) -> bool {
        let mut storage = crate::netplay::NetpacketInterfaceStorage::new(protocol_version);
        let ok = self.call_env(
            raw::RETRO_ENVIRONMENT_SET_NETPACKET_INTERFACE,
            (&mut storage.raw as *mut raw::retro_netpacket_callback).cast::<c_void>(),
        );
        if ok {
            self.state.netpacket_interface = Some(storage);
        }
        ok
    }

    pub fn clear_netpacket_interface(&mut self) -> bool {
        let ok = self.call_env(
            raw::RETRO_ENVIRONMENT_SET_NETPACKET_INTERFACE,
            ptr::null_mut(),
        );
        if ok {
            self.state.netpacket_interface = None;
        }
        ok
    }

    pub fn current_software_framebuffer(
        &mut self,
        request: SoftwareFramebufferRequest,
    ) -> Option<SoftwareFramebuffer> {
        let mut framebuffer = request.into_raw();
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_CURRENT_SOFTWARE_FRAMEBUFFER,
            (&mut framebuffer as *mut raw::retro_framebuffer).cast::<c_void>(),
        ) {
            SoftwareFramebuffer::from_raw(framebuffer)
        } else {
            None
        }
    }

    pub fn set_minimum_audio_latency(&mut self, latency: Option<AudioLatencyMillis>) -> bool {
        match latency {
            Some(latency) => {
                let mut milliseconds = latency.as_millis();
                self.call_env(
                    raw::RETRO_ENVIRONMENT_SET_MINIMUM_AUDIO_LATENCY,
                    (&mut milliseconds as *mut u32).cast::<c_void>(),
                )
            }
            None => self.call_env(
                raw::RETRO_ENVIRONMENT_SET_MINIMUM_AUDIO_LATENCY,
                ptr::null_mut(),
            ),
        }
    }

    pub fn set_audio_buffer_status_callback(&mut self, enabled: bool) -> bool {
        if enabled {
            let mut callback = raw::retro_audio_buffer_status_callback {
                callback: Some(crate::audio_buffer_status_trampoline),
            };
            self.call_env(
                raw::RETRO_ENVIRONMENT_SET_AUDIO_BUFFER_STATUS_CALLBACK,
                (&mut callback as *mut raw::retro_audio_buffer_status_callback).cast::<c_void>(),
            )
        } else {
            self.call_env(
                raw::RETRO_ENVIRONMENT_SET_AUDIO_BUFFER_STATUS_CALLBACK,
                ptr::null_mut(),
            )
        }
    }

    pub fn audio_callback_available(&mut self) -> bool {
        self.call_env(raw::RETRO_ENVIRONMENT_SET_AUDIO_CALLBACK, ptr::null_mut())
    }

    pub fn set_audio_callback(&mut self) -> bool {
        let mut callback = raw::retro_audio_callback {
            callback: Some(crate::audio_callback_trampoline),
            set_state: Some(crate::audio_set_state_trampoline),
        };
        self.call_env(
            raw::RETRO_ENVIRONMENT_SET_AUDIO_CALLBACK,
            (&mut callback as *mut raw::retro_audio_callback).cast::<c_void>(),
        )
    }

    pub fn clear_audio_callback(&mut self) -> bool {
        let mut callback = raw::retro_audio_callback::default();
        self.call_env(
            raw::RETRO_ENVIRONMENT_SET_AUDIO_CALLBACK,
            (&mut callback as *mut raw::retro_audio_callback).cast::<c_void>(),
        )
    }

    pub fn set_frame_time_callback(&mut self, reference: FrameTime) -> bool {
        let mut callback = raw::retro_frame_time_callback {
            callback: Some(crate::frame_time_trampoline),
            reference: reference.as_micros(),
        };
        self.call_env(
            raw::RETRO_ENVIRONMENT_SET_FRAME_TIME_CALLBACK,
            (&mut callback as *mut raw::retro_frame_time_callback).cast::<c_void>(),
        )
    }

    pub fn clear_frame_time_callback(&mut self) -> bool {
        self.call_env(
            raw::RETRO_ENVIRONMENT_SET_FRAME_TIME_CALLBACK,
            ptr::null_mut(),
        )
    }

    pub fn message_interface_version(&mut self) -> Option<u32> {
        let mut version = 0u32;
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_MESSAGE_INTERFACE_VERSION,
            (&mut version as *mut u32).cast::<c_void>(),
        ) {
            Some(version)
        } else {
            None
        }
    }

    pub fn set_message_ext(&mut self, message: ExtendedMessage) -> bool {
        let text = crate::sanitize_cstring(&message.message);
        let mut raw_message = raw::retro_message_ext {
            msg: text.as_ptr(),
            duration: message.duration_millis,
            priority: message.priority,
            level: message.level,
            target: message.target.as_raw(),
            type_: message.kind.as_raw(),
            progress: message.progress.as_raw(),
        };
        self.call_env(
            raw::RETRO_ENVIRONMENT_SET_MESSAGE_EXT,
            (&mut raw_message as *mut raw::retro_message_ext).cast::<c_void>(),
        )
    }

    pub fn input_device_capabilities(&mut self) -> Option<InputDeviceCapabilities> {
        let mut capabilities = 0u64;
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_INPUT_DEVICE_CAPABILITIES,
            (&mut capabilities as *mut u64).cast::<c_void>(),
        ) {
            Some(InputDeviceCapabilities::from_bits_truncate(capabilities))
        } else {
            None
        }
    }

    pub fn supports_joypad_bitmasks(&mut self) -> bool {
        self.call_env(raw::RETRO_ENVIRONMENT_GET_INPUT_BITMASKS, ptr::null_mut())
    }

    pub fn set_keyboard_callback(&mut self) -> bool {
        let mut callback = raw::retro_keyboard_callback {
            callback: Some(crate::keyboard_event_trampoline),
        };
        self.call_env(
            raw::RETRO_ENVIRONMENT_SET_KEYBOARD_CALLBACK,
            (&mut callback as *mut raw::retro_keyboard_callback).cast::<c_void>(),
        )
    }

    pub fn set_input_descriptors(&mut self, descriptors: &[InputDescriptor]) -> bool {
        let mut descriptions = Vec::with_capacity(descriptors.len());
        let mut raw_descriptors = Vec::with_capacity(descriptors.len() + 1);

        for descriptor in descriptors {
            let description = crate::sanitize_cstring(&descriptor.description);
            raw_descriptors.push(raw::retro_input_descriptor {
                port: descriptor.port.as_raw(),
                device: descriptor.device.as_raw(),
                index: descriptor.index.as_raw(),
                id: descriptor.id.as_raw(),
                description: description.as_ptr(),
            });
            descriptions.push(description);
        }
        raw_descriptors.push(raw::retro_input_descriptor::default());

        let ok = self.call_env(
            raw::RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS,
            raw_descriptors.as_mut_ptr().cast::<c_void>(),
        );
        if ok {
            self.state.input_descriptors = Some(InputDescriptorStorage {
                _descriptions: descriptions,
                _raw: raw_descriptors,
            });
        }
        ok
    }

    pub fn input_max_users(&mut self) -> Option<u32> {
        let mut users = 0u32;
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_INPUT_MAX_USERS,
            (&mut users as *mut u32).cast::<c_void>(),
        ) {
            Some(users)
        } else {
            None
        }
    }

    pub fn led_interface(&mut self) -> Option<LedInterface> {
        let mut interface = raw::retro_led_interface::default();
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_LED_INTERFACE,
            (&mut interface as *mut raw::retro_led_interface).cast::<c_void>(),
        ) {
            Some(LedInterface::from_raw(interface))
        } else {
            None
        }
    }

    pub fn rumble_interface(&mut self) -> Option<RumbleInterface> {
        let mut interface = raw::retro_rumble_interface::default();
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_RUMBLE_INTERFACE,
            (&mut interface as *mut raw::retro_rumble_interface).cast::<c_void>(),
        ) {
            Some(RumbleInterface::from_raw(interface))
        } else {
            None
        }
    }

    pub fn device_power(&mut self) -> Option<DevicePower> {
        let mut power = raw::retro_device_power::default();
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_DEVICE_POWER,
            (&mut power as *mut raw::retro_device_power).cast::<c_void>(),
        ) {
            Some(DevicePower::from_raw(power))
        } else {
            None
        }
    }

    pub fn netplay_client_index(&mut self) -> Option<NetplayClientId> {
        let mut client_id = 0u32;
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_NETPLAY_CLIENT_INDEX,
            (&mut client_id as *mut u32).cast::<c_void>(),
        ) {
            u16::try_from(client_id).ok().map(NetplayClientId::new)
        } else {
            None
        }
    }

    pub fn sensor_interface(&mut self) -> Option<SensorInterface> {
        let mut interface = raw::retro_sensor_interface::default();
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_SENSOR_INTERFACE,
            (&mut interface as *mut raw::retro_sensor_interface).cast::<c_void>(),
        ) {
            Some(SensorInterface::from_raw(interface))
        } else {
            None
        }
    }

    pub fn camera_interface(&mut self, request: CameraRequest) -> Option<CameraInterface> {
        let mut interface = raw::retro_camera_callback {
            caps: request.capabilities.bits(),
            width: request.size.width,
            height: request.size.height,
            ..raw::retro_camera_callback::default()
        };
        if self.state.event_handlers.camera_raw_frame.is_some() {
            interface.frame_raw_framebuffer = Some(crate::camera_frame_raw_trampoline);
        }
        if self.state.event_handlers.camera_texture_frame.is_some() {
            interface.frame_opengl_texture = Some(crate::camera_frame_opengl_texture_trampoline);
        }
        if self.state.event_handlers.camera_initialized.is_some() {
            interface.initialized = Some(crate::camera_initialized_trampoline);
        }
        if self.state.event_handlers.camera_deinitialized.is_some() {
            interface.deinitialized = Some(crate::camera_deinitialized_trampoline);
        }
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_CAMERA_INTERFACE,
            (&mut interface as *mut raw::retro_camera_callback).cast::<c_void>(),
        ) {
            Some(CameraInterface::from_raw(interface))
        } else {
            None
        }
    }

    pub fn location_interface(&mut self) -> Option<LocationInterface> {
        let mut interface = raw::retro_location_callback::default();
        if self.state.event_handlers.location_initialized.is_some() {
            interface.initialized = Some(crate::location_initialized_trampoline);
        }
        if self.state.event_handlers.location_deinitialized.is_some() {
            interface.deinitialized = Some(crate::location_deinitialized_trampoline);
        }
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_LOCATION_INTERFACE,
            (&mut interface as *mut raw::retro_location_callback).cast::<c_void>(),
        ) {
            Some(LocationInterface::from_raw(interface))
        } else {
            None
        }
    }

    pub fn microphone_interface(&mut self) -> Option<MicrophoneInterface> {
        let mut interface = raw::retro_microphone_interface {
            interface_version: raw::RETRO_MICROPHONE_INTERFACE_VERSION,
            ..raw::retro_microphone_interface::default()
        };
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_MICROPHONE_INTERFACE,
            (&mut interface as *mut raw::retro_microphone_interface).cast::<c_void>(),
        ) {
            Some(MicrophoneInterface::from_raw(interface))
        } else {
            None
        }
    }

    pub fn midi_interface_available(&mut self) -> bool {
        self.call_env(raw::RETRO_ENVIRONMENT_GET_MIDI_INTERFACE, ptr::null_mut())
    }

    pub fn midi_interface(&mut self) -> Option<MidiInterface> {
        let mut interface = raw::retro_midi_interface::default();
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_MIDI_INTERFACE,
            (&mut interface as *mut raw::retro_midi_interface).cast::<c_void>(),
        ) {
            Some(MidiInterface::from_raw(interface))
        } else {
            None
        }
    }

    pub fn set_serialization_quirks(
        &mut self,
        quirks: SerializationQuirks,
    ) -> Option<SerializationQuirks> {
        let mut raw_quirks = quirks.bits();
        if self.call_env(
            raw::RETRO_ENVIRONMENT_SET_SERIALIZATION_QUIRKS,
            (&mut raw_quirks as *mut u64).cast::<c_void>(),
        ) {
            Some(SerializationQuirks::from_bits_truncate(raw_quirks))
        } else {
            None
        }
    }

    fn frontend_string(&mut self, command: u32) -> Option<String> {
        let mut value: *const c_char = ptr::null();
        let ok = self.call_env(command, (&mut value as *mut *const c_char).cast::<c_void>());
        if !ok || value.is_null() {
            return None;
        }

        // SAFETY: On success with a non-null pointer, the frontend returns a
        // valid NUL-terminated, frontend-owned string for this immediate read.
        Some(
            unsafe { CStr::from_ptr(value) }
                .to_string_lossy()
                .into_owned(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FastForwardRatio, Language, MessageProgress, RunLoopRateHz, ThrottleMode, VideoRotation,
    };

    #[test]
    fn known_languages_round_trip_to_libretro_ids() {
        let languages = [
            Language::English,
            Language::Japanese,
            Language::French,
            Language::Spanish,
            Language::German,
            Language::Italian,
            Language::Dutch,
            Language::PortugueseBrazil,
            Language::PortuguesePortugal,
            Language::Russian,
            Language::Korean,
            Language::ChineseTraditional,
            Language::ChineseSimplified,
            Language::Esperanto,
            Language::Polish,
            Language::Vietnamese,
            Language::Arabic,
            Language::Greek,
            Language::Turkish,
            Language::Slovak,
            Language::Persian,
            Language::Hebrew,
            Language::Asturian,
            Language::Finnish,
            Language::Indonesian,
            Language::Swedish,
            Language::Ukrainian,
            Language::Czech,
            Language::CatalanValencia,
            Language::Catalan,
            Language::BritishEnglish,
            Language::Hungarian,
            Language::Belarusian,
            Language::Galician,
            Language::Norwegian,
            Language::Irish,
        ];

        for language in languages {
            assert_eq!(Language::from_raw(language.as_raw()), language);
        }
    }

    #[test]
    fn unknown_language_preserves_original_id() {
        assert_eq!(Language::from_raw(99), Language::Unknown(99));
        assert_eq!(Language::Unknown(99).as_raw(), 99);
    }

    #[test]
    fn progress_percent_rejects_values_outside_libretro_range() {
        assert_eq!(
            MessageProgress::percent(100),
            Some(MessageProgress::Percent(100))
        );
        assert_eq!(MessageProgress::percent(101), None);
    }

    #[test]
    fn video_rotations_encode_libretro_values() {
        assert_eq!(VideoRotation::Clockwise0.as_raw(), 0);
        assert_eq!(VideoRotation::CounterClockwise90.as_raw(), 1);
        assert_eq!(VideoRotation::CounterClockwise180.as_raw(), 2);
        assert_eq!(VideoRotation::CounterClockwise270.as_raw(), 3);
    }

    #[test]
    fn throttle_modes_round_trip_to_libretro_ids() {
        let modes = [
            ThrottleMode::None,
            ThrottleMode::FrameStepping,
            ThrottleMode::FastForward,
            ThrottleMode::SlowMotion,
            ThrottleMode::Rewinding,
            ThrottleMode::Vsync,
            ThrottleMode::Unblocked,
        ];

        for mode in modes {
            assert_eq!(ThrottleMode::from_raw(mode.as_raw()), mode);
        }
    }

    #[test]
    fn throttle_rate_rejects_unknown_or_invalid_rates() {
        assert_eq!(RunLoopRateHz::new(60.0).map(RunLoopRateHz::get), Some(60.0));
        assert_eq!(RunLoopRateHz::new(0.0), None);
        assert_eq!(RunLoopRateHz::new(f32::NAN), None);
    }

    #[test]
    fn fastforward_ratio_rejects_invalid_explicit_multipliers() {
        assert_eq!(
            FastForwardRatio::multiplier(2.0).map(FastForwardRatio::get),
            Some(2.0)
        );
        assert_eq!(FastForwardRatio::multiplier(1.0), None);
        assert_eq!(FastForwardRatio::multiplier(f32::INFINITY), None);
        assert_eq!(FastForwardRatio::multiplier(f32::NAN), None);
    }
}
