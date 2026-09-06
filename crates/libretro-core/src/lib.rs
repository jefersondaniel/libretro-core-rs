//! Rust-first libretro core callbacks and frontend services.
//!
//! Implement [`Core`] and export it with [`export_core!`]. For hardware rendering,
//! negotiate a context during [`Core::load_game`], then call
//! `Runtime::create_glow_context` during [`Core::hw_context_reset`].
//! The optional `glow` feature (enabled by default) re-exports standard glow;
//! it does not wrap GL commands or own the frontend's context.
//! See the book's OpenGL tutorial for a complete lifecycle example.

mod av;
mod callbacks;
mod camera;
mod content;
mod disk;
mod environment;
#[cfg(all(feature = "glow", not(target_arch = "wasm32")))]
mod glow_context;
#[cfg(feature = "glow")]
pub use glow;
mod hw_render;
mod input;
mod memory;
mod microphone;
mod midi;
mod netplay;
mod options;
mod perf;
mod raw;
mod sensors;
mod subsystem;
mod vfs;

use std::any::Any;
use std::borrow::Cow;
use std::ffi::{CStr, CString, c_char, c_void};
use std::mem;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;
use std::sync::{Mutex, OnceLock};

pub use av::{
    GameGeometry, SystemAvInfo, SystemTiming, bounded_game_geometry,
    exact_audio_frames_per_video_frame, fixed_system_av_info, game_geometry, silent_stereo_frames,
    silent_stereo_frames_for_video_frame, system_av_info,
};
pub use callbacks::{
    AudioBufferOccupancy, AudioBufferStatus, AudioCallbackState, CoreProcAddress, FrameTime,
};
pub use camera::{
    CameraCapabilities, CameraCapability, CameraFrameSize, CameraInterface, CameraRawFrame,
    CameraRequest, CameraTextureFrame, CameraTextureId, CameraTextureTarget,
};
pub use content::ContentContract;
pub use disk::{DiskControlInterfaceVersion, DiskIndex, DiskTrayState};
pub use environment::{
    AudioLatencyMillis, AudioSampleRateHz, AvEnable, AvEnableFlags, DevicePower, ExtendedMessage,
    FastForwardRatio, FastForwardingOverride, Language, MessageKind, MessageProgress,
    MessageTarget, PerformanceLevel, PowerState, RefreshRateHz, RunLoopRateHz, ThrottleMode,
    ThrottleState, VideoRotation,
};

pub use hw_render::{
    HwRenderContextNegotiationInterface, HwRenderContextNegotiationInterfaceType,
    HwRenderInterface, HwRenderInterfaceType, OPENGL_COMPATIBILITY_HW_RENDER_LABEL,
    OPENGL_MODERN_PREFERRED_HW_RENDER_LABEL, opengl_compatibility_hw_render_candidates,
    opengl_modern_preferred_hw_render_candidates,
};
pub use input::{
    AnalogAxis, AnalogStick, ControllerDescription, ControllerDevice, ControllerDeviceSubclass,
    ControllerInfo, InputDescriptor, InputDescriptorId, InputDescriptorIndex,
    InputDeviceCapabilities, InputDeviceCapability, InputPort, JoypadButton, JoypadButtonSet,
    KeyboardCharacter, KeyboardEvent, KeyboardKey, KeyboardModifier, KeyboardModifiers, LedIndex,
    LedInterface, LedState, LightgunAxis, LightgunButton, MouseAxis, MouseButton, MouseWheel,
    PointerAxis, PointerIndex, RumbleEffect, RumbleInterface, RumbleStrength,
};
pub use memory::{
    CoreMemory, EmulatedAddress, ExtendedGameInfo, FramebufferMemoryAccess,
    FramebufferMemoryAccessFlags, FramebufferMemoryType, FramebufferMemoryTypes,
    MemoryDescriptorAlignment, MemoryDescriptorFlag, MemoryDescriptorFlags,
    MemoryDescriptorMinAccessSize, MemoryMapDescriptor, MemoryMapLen, MemoryMapMask,
    MemoryMapOffset, MemoryRegion, SavestateContext, SerializationQuirk, SerializationQuirks,
    SoftwareFramebuffer, SoftwareFramebufferRequest,
};
pub use microphone::{
    Microphone, MicrophoneInterface, MicrophoneParams, MicrophoneRateHz, MicrophoneReadError,
};
pub use midi::{MidiDeltaMicros, MidiInterface};
pub use netplay::{
    Netpacket, NetpacketDelivery, NetpacketFlags, NetpacketSession, NetpacketTarget,
    NetplayClientId,
};
pub use options::{
    CoreOptionCategory, CoreOptionDefinition, CoreOptionDisplay, CoreOptionValue, CoreOptions,
    CoreOptionsBuildError, CoreOptionsVersion, VariableDefinition,
};
pub use perf::{CpuFeature, CpuFeatures, PerfCounter, PerfInterface, PerfTick, PerfTimeMicros};
pub use raw::{
    RETRO_API_VERSION, RETRO_DEVICE_ANALOG, RETRO_DEVICE_ID_ANALOG_X, RETRO_DEVICE_ID_ANALOG_Y,
    RETRO_DEVICE_ID_JOYPAD_A, RETRO_DEVICE_ID_JOYPAD_B, RETRO_DEVICE_ID_JOYPAD_DOWN,
    RETRO_DEVICE_ID_JOYPAD_L, RETRO_DEVICE_ID_JOYPAD_L2, RETRO_DEVICE_ID_JOYPAD_L3,
    RETRO_DEVICE_ID_JOYPAD_LEFT, RETRO_DEVICE_ID_JOYPAD_MASK, RETRO_DEVICE_ID_JOYPAD_R,
    RETRO_DEVICE_ID_JOYPAD_R2, RETRO_DEVICE_ID_JOYPAD_R3, RETRO_DEVICE_ID_JOYPAD_RIGHT,
    RETRO_DEVICE_ID_JOYPAD_SELECT, RETRO_DEVICE_ID_JOYPAD_START, RETRO_DEVICE_ID_JOYPAD_UP,
    RETRO_DEVICE_ID_JOYPAD_X, RETRO_DEVICE_ID_JOYPAD_Y, RETRO_DEVICE_ID_LIGHTGUN_IS_OFFSCREEN,
    RETRO_DEVICE_ID_LIGHTGUN_SCREEN_X, RETRO_DEVICE_ID_LIGHTGUN_SCREEN_Y,
    RETRO_DEVICE_ID_LIGHTGUN_TRIGGER, RETRO_DEVICE_ID_MOUSE_LEFT, RETRO_DEVICE_ID_MOUSE_WHEELUP,
    RETRO_DEVICE_ID_MOUSE_X, RETRO_DEVICE_ID_POINTER_PRESSED, RETRO_DEVICE_ID_POINTER_Y,
    RETRO_DEVICE_INDEX_ANALOG_BUTTON, RETRO_DEVICE_INDEX_ANALOG_LEFT, RETRO_DEVICE_JOYPAD,
    RETRO_DEVICE_KEYBOARD, RETRO_DEVICE_LIGHTGUN, RETRO_DEVICE_MOUSE, RETRO_DEVICE_NONE,
    RETRO_DEVICE_POINTER, RETRO_ENVIRONMENT_GET_CURRENT_SOFTWARE_FRAMEBUFFER,
    RETRO_ENVIRONMENT_GET_GAME_INFO_EXT, RETRO_ENVIRONMENT_GET_LOG_INTERFACE,
    RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER, RETRO_ENVIRONMENT_GET_VARIABLE,
    RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE, RETRO_ENVIRONMENT_SET_CONTENT_INFO_OVERRIDE,
    RETRO_ENVIRONMENT_SET_CONTROLLER_INFO, RETRO_ENVIRONMENT_SET_FASTFORWARDING_OVERRIDE,
    RETRO_ENVIRONMENT_SET_GEOMETRY, RETRO_ENVIRONMENT_SET_HW_RENDER,
    RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS, RETRO_ENVIRONMENT_SET_KEYBOARD_CALLBACK,
    RETRO_ENVIRONMENT_SET_MESSAGE, RETRO_ENVIRONMENT_SET_PIXEL_FORMAT,
    RETRO_ENVIRONMENT_SET_PROC_ADDRESS_CALLBACK, RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME,
    RETRO_ENVIRONMENT_SET_VARIABLES, RETRO_HW_FRAME_BUFFER_VALID, RETRO_MEMORY_ROM,
    RETRO_MEMORY_RTC, RETRO_MEMORY_SAVE_RAM, RETRO_MEMORY_SYSTEM_RAM, RETRO_MEMORY_VIDEO_RAM,
    RETRO_REGION_NTSC, RETRO_REGION_PAL,
    retro_audio_buffer_status_callback as RawAudioBufferStatusCallback,
    retro_audio_callback as RawAudioCallback, retro_audio_sample_batch_t, retro_audio_sample_t,
    retro_environment_t, retro_frame_time_callback as RawFrameTimeCallback,
    retro_game_info as RawGameInfo, retro_hw_context_type as HwContextType,
    retro_hw_render_callback as RawHwRenderCallback, retro_input_descriptor as RawInputDescriptor,
    retro_input_poll_t, retro_input_state_t, retro_keyboard_callback as RawKeyboardCallback,
    retro_log_callback as RawLogCallback, retro_log_level as LogLevel, retro_message as RawMessage,
    retro_pixel_format as PixelFormat,
    retro_system_content_info_override as RawContentInfoOverride,
    retro_system_info as RawSystemInfo, retro_variable as RawVariable, retro_video_refresh_t,
};
pub use sensors::{
    LocationInterface, LocationIntervalMeters, LocationIntervalMillis, LocationPosition, Sensor,
    SensorAction, SensorInput, SensorInterface, SensorRateHz,
};
pub use subsystem::{
    SubsystemId, SubsystemInfo, SubsystemMemoryInfo, SubsystemMemoryType, SubsystemRomInfo,
};
pub use vfs::{
    VfsDirectory, VfsFile, VfsFileAccess, VfsFileAccessFlags, VfsFileAccessHint,
    VfsFileAccessHints, VfsInterface, VfsInterfaceVersion, VfsMetadata, VfsSeekPosition,
    VfsStatFlag, VfsStatFlags,
};

type CoreFactory = fn() -> CoreBundle;

static FACTORY: OnceLock<CoreFactory> = OnceLock::new();
static STATE: OnceLock<Mutex<CoreState>> = OnceLock::new();

/// Static metadata returned to the frontend by `Core::system_info`.
///
/// Prefer applying a `ContentContract` to this value so `valid_extensions`,
/// `need_fullpath`, and `block_extract` stay consistent with the environment
/// registration done in `Core::on_set_environment`.
#[derive(Clone, Debug)]
pub struct SystemInfo {
    pub library_name: String,
    pub library_version: String,
    pub valid_extensions: Option<String>,
    pub need_fullpath: bool,
    pub block_extract: bool,
}

impl SystemInfo {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            library_name: name.into(),
            library_version: version.into(),
            valid_extensions: None,
            need_fullpath: false,
            block_extract: false,
        }
    }
}

/// Per-extension content override registered with the frontend.
///
/// Most cores should create these through `ContentContract` instead of building
/// overrides directly.
#[derive(Clone, Debug)]
pub struct ContentInfoOverride {
    pub extensions: String,
    pub need_fullpath: bool,
    pub persistent_data: bool,
}

impl ContentInfoOverride {
    pub fn new(extensions: impl Into<String>) -> Self {
        Self {
            extensions: extensions.into(),
            need_fullpath: false,
            persistent_data: false,
        }
    }
}

/// Frontend logger with stderr fallback.
///
/// `Environment::logger` and `Runtime::logger` return this wrapper so core code
/// can log without touching the raw `retro_log_callback` table.
#[derive(Clone, Copy, Debug, Default)]
pub struct Logger {
    callback: Option<raw::retro_log_printf_t>,
}

impl Logger {
    pub fn debug(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Debug, message);
    }

    pub fn info(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Info, message);
    }

    pub fn warn(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Warn, message);
    }

    pub fn error(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Error, message);
    }

    fn log(&self, level: LogLevel, message: impl AsRef<str>) {
        let message = message.as_ref();
        if let Some(callback) = self.callback.flatten() {
            let message = sanitize_cstring(message);
            static FORMAT: &[u8] = b"%s\n\0";
            // SAFETY: The callback comes from the frontend and the format string is static.
            unsafe { callback(level, FORMAT.as_ptr().cast::<c_char>(), message.as_ptr()) };
        } else {
            eprintln!("{message}");
        }
    }
}

/// Hardware-rendering context request sent to the frontend.
///
/// Use the constructors such as `HwRenderConfig::opengl_core` or the candidate
/// helpers in `hw_render` instead of filling raw context IDs manually.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HwRenderConfig {
    pub context_type: HwContextType,
    pub depth: bool,
    pub stencil: bool,
    pub bottom_left_origin: bool,
    pub version_major: u32,
    pub version_minor: u32,
    pub cache_context: bool,
    pub debug_context: bool,
}

impl HwRenderConfig {
    pub fn new(context_type: HwContextType) -> Self {
        Self {
            context_type,
            ..Self::default()
        }
    }

    pub fn opengl() -> Self {
        Self::new(HwContextType::OpenGl)
    }

    pub fn opengl_core(version_major: u32, version_minor: u32) -> Self {
        Self::new(HwContextType::OpenGlCore).with_version(version_major, version_minor)
    }

    pub fn opengles2() -> Self {
        Self::new(HwContextType::OpenGlEs2)
    }

    pub fn opengles3() -> Self {
        Self::new(HwContextType::OpenGlEs3)
    }

    pub fn opengles_version(version_major: u32, version_minor: u32) -> Self {
        Self::new(HwContextType::OpenGlEsVersion).with_version(version_major, version_minor)
    }

    pub fn with_depth(mut self, depth: bool) -> Self {
        self.depth = depth;
        self
    }

    pub fn with_stencil(mut self, stencil: bool) -> Self {
        self.stencil = stencil;
        self
    }

    pub fn with_bottom_left_origin(mut self, bottom_left_origin: bool) -> Self {
        self.bottom_left_origin = bottom_left_origin;
        self
    }

    pub fn with_version(mut self, version_major: u32, version_minor: u32) -> Self {
        self.version_major = version_major;
        self.version_minor = version_minor;
        self
    }

    pub fn with_cache_context(mut self, cache_context: bool) -> Self {
        self.cache_context = cache_context;
        self
    }

    pub fn with_debug_context(mut self, debug_context: bool) -> Self {
        self.debug_context = debug_context;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreferredHwRender {
    pub context_type: HwContextType,
    pub supports_non_preferred_context: bool,
}

impl HwContextType {
    pub fn is_opengl_family(self) -> bool {
        matches!(
            self,
            Self::OpenGl
                | Self::OpenGlCore
                | Self::OpenGlEs2
                | Self::OpenGlEs3
                | Self::OpenGlEsVersion
        )
    }
}

fn describe_hw_render_config(config: HwRenderConfig) -> String {
    if config.version_major == 0 && config.version_minor == 0 {
        format!("{:?}", config.context_type)
    } else {
        format!(
            "{:?} {}.{}",
            config.context_type, config.version_major, config.version_minor
        )
    }
}

fn is_opengl_es_family(context_type: HwContextType) -> bool {
    matches!(
        context_type,
        HwContextType::OpenGlEs2 | HwContextType::OpenGlEs3 | HwContextType::OpenGlEsVersion
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    Ntsc,
    Pal,
}

impl Region {
    fn as_raw(self) -> u32 {
        match self {
            Self::Ntsc => RETRO_REGION_NTSC,
            Self::Pal => RETRO_REGION_PAL,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GameInfo<'a> {
    pub path: Option<&'a CStr>,
    pub data: Option<&'a [u8]>,
    pub meta: Option<&'a CStr>,
}

impl<'a> GameInfo<'a> {
    pub fn path_lossy(&self) -> Option<Cow<'a, str>> {
        self.path.map(CStr::to_string_lossy)
    }

    pub fn meta_lossy(&self) -> Option<Cow<'a, str>> {
        self.meta.map(CStr::to_string_lossy)
    }

    unsafe fn from_raw(raw: *const RawGameInfo) -> Option<Self> {
        if raw.is_null() {
            return None;
        }

        // SAFETY: The caller guarantees `raw` is valid for the duration of the call.
        let raw = unsafe { &*raw };
        let path = if raw.path.is_null() {
            None
        } else {
            // SAFETY: `path` follows the libretro ABI contract.
            Some(unsafe { CStr::from_ptr(raw.path) })
        };
        let data = if raw.data.is_null() {
            None
        } else {
            // SAFETY: `data` and `size` follow the libretro ABI contract.
            Some(unsafe { std::slice::from_raw_parts(raw.data.cast::<u8>(), raw.size) })
        };
        let meta = if raw.meta.is_null() {
            None
        } else {
            // SAFETY: `meta` follows the libretro ABI contract.
            Some(unsafe { CStr::from_ptr(raw.meta) })
        };

        Some(Self { path, data, meta })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CheatIndex(u32);

impl CheatIndex {
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

impl From<u32> for CheatIndex {
    fn from(index: u32) -> Self {
        Self::new(index)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CheatCode<'a> {
    raw: &'a CStr,
}

impl<'a> CheatCode<'a> {
    fn from_c_str(raw: &'a CStr) -> Self {
        Self { raw }
    }

    pub fn as_c_str(self) -> &'a CStr {
        self.raw
    }

    pub fn to_str(self) -> Result<&'a str, std::str::Utf8Error> {
        self.raw.to_str()
    }

    pub fn to_string_lossy(self) -> Cow<'a, str> {
        self.raw.to_string_lossy()
    }
}

type KeyboardEventHandler = Box<dyn Fn(&mut dyn Core, KeyboardEvent) + Send + Sync>;
type AudioCallbackHandler = Box<dyn Fn(&mut dyn Core) + Send + Sync>;
type AudioStateHandler = Box<dyn Fn(&mut dyn Core, AudioCallbackState) + Send + Sync>;
type AudioBufferStatusHandler = Box<dyn Fn(&mut dyn Core, AudioBufferStatus) + Send + Sync>;
type FrameTimeHandler = Box<dyn Fn(&mut dyn Core, FrameTime) + Send + Sync>;
type LocationLifecycleHandler = Box<dyn Fn(&mut dyn Core) + Send + Sync>;
type CameraRawFrameHandler = Box<dyn Fn(&mut dyn Core, CameraRawFrame<'_>) + Send + Sync>;
type CameraTextureFrameHandler = Box<dyn Fn(&mut dyn Core, CameraTextureFrame) + Send + Sync>;

type ListenerId = usize;

struct EventListener<T> {
    id: ListenerId,
    callback: T,
}

impl<T> EventListener<T> {
    fn new(id: ListenerId, callback: T) -> Self {
        Self { id, callback }
    }
}

fn add_listener<T>(listeners: &mut Vec<EventListener<T>>, id: ListenerId, callback: T) {
    if listeners.iter().any(|listener| listener.id == id) {
        return;
    }
    listeners.push(EventListener::new(id, callback));
}

fn remove_listener<T>(listeners: &mut Vec<EventListener<T>>, id: ListenerId) {
    listeners.retain(|listener| listener.id != id);
}

#[derive(Default)]
struct CoreEventHandlers {
    keyboard_event: Vec<EventListener<KeyboardEventHandler>>,
    audio_callback: Vec<EventListener<AudioCallbackHandler>>,
    audio_callback_state_changed: Vec<EventListener<AudioStateHandler>>,
    audio_buffer_status: Vec<EventListener<AudioBufferStatusHandler>>,
    frame_time: Option<(FrameTime, FrameTimeHandler)>,
    location_initialized: Vec<EventListener<LocationLifecycleHandler>>,
    location_deinitialized: Vec<EventListener<LocationLifecycleHandler>>,
    camera_initialized: Vec<EventListener<LocationLifecycleHandler>>,
    camera_deinitialized: Vec<EventListener<LocationLifecycleHandler>>,
    camera_raw_frame: Vec<EventListener<CameraRawFrameHandler>>,
    camera_texture_frame: Vec<EventListener<CameraTextureFrameHandler>>,
}

impl CoreEventHandlers {
    fn has_keyboard_event(&self) -> bool {
        !self.keyboard_event.is_empty()
    }

    fn has_audio_callback(&self) -> bool {
        !self.audio_callback.is_empty() || !self.audio_callback_state_changed.is_empty()
    }

    fn has_audio_buffer_status(&self) -> bool {
        !self.audio_buffer_status.is_empty()
    }

    fn frame_time_reference(&self) -> Option<FrameTime> {
        self.frame_time.as_ref().map(|(reference, _)| *reference)
    }

    fn dispatch_keyboard_event(&self, core: &mut dyn Core, event: KeyboardEvent) {
        for listener in &self.keyboard_event {
            (listener.callback)(core, event);
        }
    }

    fn dispatch_audio_callback(&self, core: &mut dyn Core) {
        for listener in &self.audio_callback {
            (listener.callback)(core);
        }
    }

    fn dispatch_audio_callback_state_changed(
        &self,
        core: &mut dyn Core,
        state: AudioCallbackState,
    ) {
        for listener in &self.audio_callback_state_changed {
            (listener.callback)(core, state);
        }
    }

    fn dispatch_audio_buffer_status(&self, core: &mut dyn Core, status: AudioBufferStatus) {
        for listener in &self.audio_buffer_status {
            (listener.callback)(core, status);
        }
    }

    fn dispatch_frame_time(&self, core: &mut dyn Core, time: FrameTime) {
        if let Some((_, callback)) = &self.frame_time {
            callback(core, time);
        }
    }

    fn dispatch_location_initialized(&self, core: &mut dyn Core) {
        for listener in &self.location_initialized {
            (listener.callback)(core);
        }
    }

    fn dispatch_location_deinitialized(&self, core: &mut dyn Core) {
        for listener in &self.location_deinitialized {
            (listener.callback)(core);
        }
    }

    fn dispatch_camera_initialized(&self, core: &mut dyn Core) {
        for listener in &self.camera_initialized {
            (listener.callback)(core);
        }
    }

    fn dispatch_camera_deinitialized(&self, core: &mut dyn Core) {
        for listener in &self.camera_deinitialized {
            (listener.callback)(core);
        }
    }

    fn dispatch_camera_raw_frame(&self, core: &mut dyn Core, frame: CameraRawFrame<'_>) {
        for listener in &self.camera_raw_frame {
            (listener.callback)(core, frame);
        }
    }

    fn dispatch_camera_texture_frame(&self, core: &mut dyn Core, frame: CameraTextureFrame) {
        for listener in &self.camera_texture_frame {
            (listener.callback)(core, frame);
        }
    }
}

/// Event-listener registration for frontend-to-core notifications.
///
/// Register listeners from `Core::configure_events`. The wrapper installs the
/// matching low-level libretro callbacks during environment setup, avoiding
/// call-order bugs where a core defines a callback but forgets to enable the raw
/// callback separately. Listeners are dispatched in registration order.
/// Registering the same callback function more than once for the same event is
/// a no-op, matching DOM `addEventListener` behavior. Use the matching
/// `remove_*_listener` method with the same callback function to remove a
/// listener during configuration. Callback-shaped frontend hooks with one
/// active registration, such as frame timing, use explicit `set_*_callback` and
/// `clear_*_callback` methods instead.
pub struct CoreEventConfig<C: Core> {
    handlers: CoreEventHandlers,
    _core: std::marker::PhantomData<fn() -> C>,
}

impl<C: Core> Default for CoreEventConfig<C> {
    fn default() -> Self {
        Self {
            handlers: CoreEventHandlers::default(),
            _core: std::marker::PhantomData,
        }
    }
}

impl<C: Core> CoreEventConfig<C> {
    pub fn add_keyboard_event_listener(
        &mut self,
        listener: fn(&mut C, KeyboardEvent),
    ) -> &mut Self {
        add_listener(
            &mut self.handlers.keyboard_event,
            listener as ListenerId,
            Box::new(move |core, event| {
                let core = (core as &mut dyn Any)
                    .downcast_mut::<C>()
                    .expect("registered keyboard event listener received the wrong core type");
                listener(core, event);
            }),
        );
        self
    }

    pub fn remove_keyboard_event_listener(
        &mut self,
        listener: fn(&mut C, KeyboardEvent),
    ) -> &mut Self {
        remove_listener(&mut self.handlers.keyboard_event, listener as ListenerId);
        self
    }

    pub fn add_audio_callback_listener(&mut self, listener: fn(&mut C)) -> &mut Self {
        add_listener(
            &mut self.handlers.audio_callback,
            listener as ListenerId,
            Box::new(move |core| {
                let core = (core as &mut dyn Any)
                    .downcast_mut::<C>()
                    .expect("registered audio callback listener received the wrong core type");
                listener(core);
            }),
        );
        self
    }

    pub fn remove_audio_callback_listener(&mut self, listener: fn(&mut C)) -> &mut Self {
        remove_listener(&mut self.handlers.audio_callback, listener as ListenerId);
        self
    }

    pub fn add_audio_callback_state_changed_listener(
        &mut self,
        listener: fn(&mut C, AudioCallbackState),
    ) -> &mut Self {
        add_listener(
            &mut self.handlers.audio_callback_state_changed,
            listener as ListenerId,
            Box::new(move |core, state| {
                let core = (core as &mut dyn Any)
                    .downcast_mut::<C>()
                    .expect("registered audio state listener received the wrong core type");
                listener(core, state);
            }),
        );
        self
    }

    pub fn remove_audio_callback_state_changed_listener(
        &mut self,
        listener: fn(&mut C, AudioCallbackState),
    ) -> &mut Self {
        remove_listener(
            &mut self.handlers.audio_callback_state_changed,
            listener as ListenerId,
        );
        self
    }

    pub fn add_audio_buffer_status_listener(
        &mut self,
        listener: fn(&mut C, AudioBufferStatus),
    ) -> &mut Self {
        add_listener(
            &mut self.handlers.audio_buffer_status,
            listener as ListenerId,
            Box::new(move |core, status| {
                let core = (core as &mut dyn Any)
                    .downcast_mut::<C>()
                    .expect("registered audio buffer status listener received the wrong core type");
                listener(core, status);
            }),
        );
        self
    }

    pub fn remove_audio_buffer_status_listener(
        &mut self,
        listener: fn(&mut C, AudioBufferStatus),
    ) -> &mut Self {
        remove_listener(
            &mut self.handlers.audio_buffer_status,
            listener as ListenerId,
        );
        self
    }

    pub fn set_frame_time_callback(
        &mut self,
        reference: FrameTime,
        callback: fn(&mut C, FrameTime),
    ) -> &mut Self {
        self.handlers.frame_time = Some((
            reference,
            Box::new(move |core, time| {
                let core = (core as &mut dyn Any)
                    .downcast_mut::<C>()
                    .expect("registered frame-time callback received the wrong core type");
                callback(core, time);
            }),
        ));
        self
    }

    pub fn clear_frame_time_callback(&mut self) -> &mut Self {
        self.handlers.frame_time = None;
        self
    }

    pub fn add_location_initialized_listener(&mut self, listener: fn(&mut C)) -> &mut Self {
        add_listener(
            &mut self.handlers.location_initialized,
            listener as ListenerId,
            Box::new(move |core| {
                let core = (core as &mut dyn Any).downcast_mut::<C>().expect(
                    "registered location initialized listener received the wrong core type",
                );
                listener(core);
            }),
        );
        self
    }

    pub fn remove_location_initialized_listener(&mut self, listener: fn(&mut C)) -> &mut Self {
        remove_listener(
            &mut self.handlers.location_initialized,
            listener as ListenerId,
        );
        self
    }

    pub fn add_location_deinitialized_listener(&mut self, listener: fn(&mut C)) -> &mut Self {
        add_listener(
            &mut self.handlers.location_deinitialized,
            listener as ListenerId,
            Box::new(move |core| {
                let core = (core as &mut dyn Any).downcast_mut::<C>().expect(
                    "registered location deinitialized listener received the wrong core type",
                );
                listener(core);
            }),
        );
        self
    }

    pub fn remove_location_deinitialized_listener(&mut self, listener: fn(&mut C)) -> &mut Self {
        remove_listener(
            &mut self.handlers.location_deinitialized,
            listener as ListenerId,
        );
        self
    }

    pub fn add_camera_initialized_listener(&mut self, listener: fn(&mut C)) -> &mut Self {
        add_listener(
            &mut self.handlers.camera_initialized,
            listener as ListenerId,
            Box::new(move |core| {
                let core = (core as &mut dyn Any)
                    .downcast_mut::<C>()
                    .expect("registered camera initialized listener received the wrong core type");
                listener(core);
            }),
        );
        self
    }

    pub fn remove_camera_initialized_listener(&mut self, listener: fn(&mut C)) -> &mut Self {
        remove_listener(
            &mut self.handlers.camera_initialized,
            listener as ListenerId,
        );
        self
    }

    pub fn add_camera_deinitialized_listener(&mut self, listener: fn(&mut C)) -> &mut Self {
        add_listener(
            &mut self.handlers.camera_deinitialized,
            listener as ListenerId,
            Box::new(move |core| {
                let core = (core as &mut dyn Any).downcast_mut::<C>().expect(
                    "registered camera deinitialized listener received the wrong core type",
                );
                listener(core);
            }),
        );
        self
    }

    pub fn remove_camera_deinitialized_listener(&mut self, listener: fn(&mut C)) -> &mut Self {
        remove_listener(
            &mut self.handlers.camera_deinitialized,
            listener as ListenerId,
        );
        self
    }

    pub fn add_camera_raw_frame_listener(
        &mut self,
        listener: fn(&mut C, CameraRawFrame<'_>),
    ) -> &mut Self {
        add_listener(
            &mut self.handlers.camera_raw_frame,
            listener as ListenerId,
            Box::new(move |core, frame| {
                let core = (core as &mut dyn Any)
                    .downcast_mut::<C>()
                    .expect("registered camera raw-frame listener received the wrong core type");
                listener(core, frame);
            }),
        );
        self
    }

    pub fn remove_camera_raw_frame_listener(
        &mut self,
        listener: fn(&mut C, CameraRawFrame<'_>),
    ) -> &mut Self {
        remove_listener(&mut self.handlers.camera_raw_frame, listener as ListenerId);
        self
    }

    pub fn add_camera_texture_frame_listener(
        &mut self,
        listener: fn(&mut C, CameraTextureFrame),
    ) -> &mut Self {
        add_listener(
            &mut self.handlers.camera_texture_frame,
            listener as ListenerId,
            Box::new(move |core, frame| {
                let core = (core as &mut dyn Any).downcast_mut::<C>().expect(
                    "registered camera texture-frame listener received the wrong core type",
                );
                listener(core, frame);
            }),
        );
        self
    }

    pub fn remove_camera_texture_frame_listener(
        &mut self,
        listener: fn(&mut C, CameraTextureFrame),
    ) -> &mut Self {
        remove_listener(
            &mut self.handlers.camera_texture_frame,
            listener as ListenerId,
        );
        self
    }

    fn into_handlers(self) -> CoreEventHandlers {
        self.handlers
    }
}

#[doc(hidden)]
pub struct CoreBundle {
    core: Box<dyn Core>,
    event_handlers: CoreEventHandlers,
}

#[doc(hidden)]
pub fn create_core<C: Core>(mut core: C) -> CoreBundle {
    let mut events = CoreEventConfig::<C>::default();
    core.configure_events(&mut events);
    CoreBundle {
        core: Box::new(core),
        event_handlers: events.into_handlers(),
    }
}

/// Trait implemented by a Rust libretro core.
///
/// Required methods describe metadata, AV timing, and per-frame execution.
/// Optional methods cover setup, content loading, savestates, disk control,
/// hardware-render lifecycle, netpacket callbacks, and other libretro surfaces.
/// Export an implementation with `export_core!`.
pub trait Core: Any + Send + 'static {
    fn system_info(&self) -> SystemInfo;
    fn av_info(&self) -> SystemAvInfo;
    fn run(&mut self, runtime: &mut Runtime<'_>);

    fn configure_events(&mut self, _events: &mut CoreEventConfig<Self>)
    where
        Self: Sized,
    {
    }
    fn on_set_environment(&mut self, _env: &mut Environment<'_>) {}
    fn init(&mut self, _env: &mut Environment<'_>) {}
    fn deinit(&mut self) {}
    fn set_controller_port_device(&mut self, _port: InputPort, _device: ControllerDevice) {}
    fn reset(&mut self) {}
    fn load_game(&mut self, _game: Option<GameInfo<'_>>, _runtime: &mut Runtime<'_>) -> bool {
        true
    }
    fn load_game_special(
        &mut self,
        _subsystem: SubsystemId,
        _games: &[GameInfo<'_>],
        _runtime: &mut Runtime<'_>,
    ) -> bool {
        false
    }
    fn unload_game(&mut self) {}
    fn serialize_size(&self) -> usize {
        0
    }
    fn serialize(&self, _data: &mut [u8]) -> bool {
        false
    }
    fn unserialize(&mut self, _data: &[u8]) -> bool {
        false
    }
    fn cheat_reset(&mut self) {}
    fn cheat_set(&mut self, _index: CheatIndex, _enabled: bool, _code: Option<CheatCode<'_>>) {}
    fn region(&self) -> Region {
        Region::Ntsc
    }
    fn memory_region(&mut self, _region: MemoryRegion) -> Option<CoreMemory<'_>> {
        None
    }
    fn proc_address(&mut self, _symbol: &CStr) -> Option<CoreProcAddress> {
        None
    }
    fn disk_set_tray_state(&mut self, _state: DiskTrayState) -> bool {
        false
    }
    fn disk_tray_state(&mut self) -> DiskTrayState {
        DiskTrayState::Closed
    }
    fn disk_image_index(&mut self) -> DiskIndex {
        DiskIndex::new(0)
    }
    fn disk_set_image_index(&mut self, _index: DiskIndex) -> bool {
        false
    }
    fn disk_image_count(&mut self) -> u32 {
        0
    }
    fn disk_replace_image_index(&mut self, _index: DiskIndex, _game: Option<GameInfo<'_>>) -> bool {
        false
    }
    fn disk_add_image_index(&mut self) -> bool {
        false
    }
    fn disk_set_initial_image(&mut self, _index: DiskIndex, _path: &CStr) -> bool {
        false
    }
    fn disk_image_path(&mut self, _index: DiskIndex) -> Option<String> {
        None
    }
    fn disk_image_label(&mut self, _index: DiskIndex) -> Option<String> {
        None
    }
    fn netpacket_start(&mut self, _session: NetpacketSession) {}
    fn netpacket_receive(&mut self, _packet: Netpacket<'_>) {}
    fn netpacket_stop(&mut self) {}
    fn netpacket_poll(&mut self) {}
    fn netpacket_connected(&mut self, _client_id: NetplayClientId) -> bool {
        true
    }
    fn netpacket_disconnected(&mut self, _client_id: NetplayClientId) {}
    fn core_options_update_display(&mut self, _env: &mut Environment<'_>) -> bool {
        false
    }
    fn hw_context_reset(&mut self, _runtime: &mut Runtime<'_>) {}
    fn hw_context_destroy(&mut self, _runtime: &mut Runtime<'_>) {}
}

/// Typed wrapper around libretro environment commands.
///
/// Use this during `Core::on_set_environment`, `Core::init`, and through
/// `Runtime::environment` for runtime-safe commands. Methods retain backing
/// storage when libretro allows the frontend to keep pointers after a call.
pub struct Environment<'a> {
    state: &'a mut CoreState,
}

impl<'a> Environment<'a> {
    pub fn logger(&mut self) -> Logger {
        if self.state.log_callback.is_none() {
            let mut callback = RawLogCallback::default();
            let ok = self.call_env(
                RETRO_ENVIRONMENT_GET_LOG_INTERFACE,
                (&mut callback as *mut RawLogCallback).cast::<c_void>(),
            );
            if ok {
                self.state.log_callback = Some(callback);
            }
        }

        Logger {
            callback: self.state.log_callback.map(|callback| callback.log),
        }
    }

    pub fn set_support_no_game(&mut self, enabled: bool) -> bool {
        self.call_env(
            RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME,
            &enabled as *const bool as *mut c_void,
        )
    }

    pub fn set_message(&mut self, message: impl AsRef<str>, frames: u32) -> bool {
        let message = sanitize_cstring(message.as_ref());
        let mut raw = raw::retro_message {
            msg: message.as_ptr(),
            frames,
        };
        self.call_env(
            RETRO_ENVIRONMENT_SET_MESSAGE,
            (&mut raw as *mut raw::retro_message).cast::<c_void>(),
        )
    }

    pub fn set_variables(&mut self, variables: &[VariableDefinition]) -> bool {
        let mut storage = options::CoreOptionsStorage::variables(variables);

        let ok = self.call_env(
            RETRO_ENVIRONMENT_SET_VARIABLES,
            storage.variables_ptr().cast::<c_void>(),
        );
        if ok {
            self.state.variables = Some(storage);
        }
        ok
    }

    pub fn core_options_version(&mut self) -> CoreOptionsVersion {
        let mut version = 0u32;
        if self.call_env(
            raw::RETRO_ENVIRONMENT_GET_CORE_OPTIONS_VERSION,
            (&mut version as *mut u32).cast::<c_void>(),
        ) {
            CoreOptionsVersion::new(version)
        } else {
            CoreOptionsVersion::LEGACY_VARIABLES
        }
    }

    pub fn set_core_options(
        &mut self,
        options: &CoreOptions,
    ) -> Result<bool, CoreOptionsBuildError> {
        let version = self.core_options_version();
        if version.supports_v2() {
            self.set_core_options_v2(options)
        } else if version.supports_v1() {
            self.set_core_options_v1(&options.definitions)
        } else {
            self.set_core_options_legacy(options)
        }
    }

    pub fn set_core_options_legacy(
        &mut self,
        options: &CoreOptions,
    ) -> Result<bool, CoreOptionsBuildError> {
        let mut storage = options::CoreOptionsStorage::legacy_from_options(options)?;
        let ok = self.call_env(
            RETRO_ENVIRONMENT_SET_VARIABLES,
            storage.variables_ptr().cast::<c_void>(),
        );
        if ok {
            self.state.variables = Some(storage);
        }
        Ok(ok)
    }

    pub fn set_core_options_v1(
        &mut self,
        definitions: &[CoreOptionDefinition],
    ) -> Result<bool, CoreOptionsBuildError> {
        let mut storage = options::CoreOptionsStorage::v1(definitions)?;
        let ok = self.call_env(
            raw::RETRO_ENVIRONMENT_SET_CORE_OPTIONS,
            storage.v1_definitions_ptr().cast::<c_void>(),
        );
        if ok {
            self.state.variables = Some(storage);
        }
        Ok(ok)
    }

    pub fn set_core_options_v1_intl(
        &mut self,
        us: &[CoreOptionDefinition],
        local: Option<&[CoreOptionDefinition]>,
    ) -> Result<bool, CoreOptionsBuildError> {
        let mut storage = options::CoreOptionsStorage::v1_intl(us, local)?;
        let mut raw = storage.v1_intl_raw();
        let ok = self.call_env(
            raw::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_INTL,
            (&mut raw as *mut raw::retro_core_options_intl).cast::<c_void>(),
        );
        if ok {
            self.state.variables = Some(storage);
        }
        Ok(ok)
    }

    pub fn set_core_options_v2(
        &mut self,
        options: &CoreOptions,
    ) -> Result<bool, CoreOptionsBuildError> {
        let mut storage = options::CoreOptionsStorage::v2(options)?;
        let mut raw = storage.v2_raw();
        let ok = self.call_env(
            raw::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2,
            (&mut raw as *mut raw::retro_core_options_v2).cast::<c_void>(),
        );
        if ok {
            self.state.variables = Some(storage);
        }
        Ok(ok)
    }

    pub fn set_core_options_v2_intl(
        &mut self,
        us: &CoreOptions,
        local: Option<&CoreOptions>,
    ) -> Result<bool, CoreOptionsBuildError> {
        let mut storage = options::CoreOptionsStorage::v2_intl(us, local)?;
        let mut us = storage.v2_raw();
        let mut local = storage.local_v2_raw();
        let mut raw = raw::retro_core_options_v2_intl {
            us: &mut us,
            local: local.as_mut().map_or(ptr::null_mut(), |local| {
                local as *mut raw::retro_core_options_v2
            }),
        };
        let ok = self.call_env(
            raw::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL,
            (&mut raw as *mut raw::retro_core_options_v2_intl).cast::<c_void>(),
        );
        if ok {
            self.state.variables = Some(storage);
        }
        Ok(ok)
    }

    pub fn set_core_option_display(&mut self, display: CoreOptionDisplay) -> bool {
        let key = sanitize_cstring(display.key);
        let mut raw = raw::retro_core_option_display {
            key: key.as_ptr(),
            visible: display.visible,
        };
        self.call_env(
            raw::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_DISPLAY,
            (&mut raw as *mut raw::retro_core_option_display).cast::<c_void>(),
        )
    }

    pub fn set_core_options_update_display_callback(&mut self) -> bool {
        let mut raw = raw::retro_core_options_update_display_callback {
            callback: Some(core_options_update_display_trampoline),
        };
        self.call_env(
            raw::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_UPDATE_DISPLAY_CALLBACK,
            (&mut raw as *mut raw::retro_core_options_update_display_callback).cast::<c_void>(),
        )
    }

    pub fn set_variable(&mut self, key: &str, value: Option<&str>) -> bool {
        let key = sanitize_cstring(key);
        let value = value.map(sanitize_cstring);
        let mut raw = RawVariable {
            key: key.as_ptr(),
            value: value.as_ref().map_or(ptr::null(), |value| value.as_ptr()),
        };
        self.call_env(
            raw::RETRO_ENVIRONMENT_SET_VARIABLE,
            (&mut raw as *mut RawVariable).cast::<c_void>(),
        )
    }

    pub fn vfs_interface(&mut self, version: VfsInterfaceVersion) -> Option<VfsInterface> {
        let mut info = raw::retro_vfs_interface_info {
            required_interface_version: version.get(),
            iface: ptr::null_mut(),
        };
        let ok = self.call_env(
            raw::RETRO_ENVIRONMENT_GET_VFS_INTERFACE,
            (&mut info as *mut raw::retro_vfs_interface_info).cast::<c_void>(),
        );
        if ok && !info.iface.is_null() {
            // SAFETY: On success, the frontend populated `iface` with a valid
            // interface table. Copying the function pointers avoids borrowing
            // frontend-owned storage through the public wrapper.
            Some(VfsInterface::new(
                VfsInterfaceVersion::new(info.required_interface_version),
                unsafe { *info.iface },
            ))
        } else {
            None
        }
    }

    pub fn set_content_info_overrides(&mut self, overrides: &[ContentInfoOverride]) -> bool {
        let mut extensions = Vec::with_capacity(overrides.len());
        let mut raw = Vec::with_capacity(overrides.len() + 1);

        for override_info in overrides {
            let extensions_cstring = sanitize_cstring(&override_info.extensions);
            raw.push(RawContentInfoOverride {
                extensions: extensions_cstring.as_ptr(),
                need_fullpath: override_info.need_fullpath,
                persistent_data: override_info.persistent_data,
            });
            extensions.push(extensions_cstring);
        }
        raw.push(RawContentInfoOverride::default());

        let ok = self.call_env(
            RETRO_ENVIRONMENT_SET_CONTENT_INFO_OVERRIDE,
            raw.as_mut_ptr().cast::<c_void>(),
        );
        if ok {
            self.state.content_info_overrides = Some(ContentInfoOverrideStorage {
                _extensions: extensions,
                _raw: raw,
            });
        }
        ok
    }

    pub fn set_pixel_format(&mut self, format: PixelFormat) -> bool {
        let mut format = format;
        self.call_env(
            RETRO_ENVIRONMENT_SET_PIXEL_FORMAT,
            (&mut format as *mut PixelFormat).cast::<c_void>(),
        )
    }

    pub fn set_geometry(&mut self, geometry: GameGeometry) -> bool {
        let mut geometry = geometry.as_raw();
        self.call_env(
            RETRO_ENVIRONMENT_SET_GEOMETRY,
            (&mut geometry as *mut raw::retro_game_geometry).cast::<c_void>(),
        )
    }

    pub fn set_hw_render(&mut self, config: HwRenderConfig) -> bool {
        let mut callback = RawHwRenderCallback {
            context_type: config.context_type,
            context_reset: Some(hw_context_reset_trampoline),
            get_current_framebuffer: None,
            get_proc_address: None,
            depth: config.depth,
            stencil: config.stencil,
            bottom_left_origin: config.bottom_left_origin,
            version_major: config.version_major,
            version_minor: config.version_minor,
            cache_context: config.cache_context,
            context_destroy: Some(hw_context_destroy_trampoline),
            debug_context: config.debug_context,
        };

        let ok = self.call_env(
            RETRO_ENVIRONMENT_SET_HW_RENDER,
            (&mut callback as *mut RawHwRenderCallback).cast::<c_void>(),
        );
        if ok {
            self.state.hw_render = Some(callback);
        }
        ok
    }

    pub fn set_hw_shared_context(&mut self) -> bool {
        self.call_env(
            raw::RETRO_ENVIRONMENT_SET_HW_SHARED_CONTEXT,
            ptr::null_mut(),
        )
    }

    pub fn hw_render_interface(&mut self) -> Option<HwRenderInterface<'_>> {
        let mut interface = ptr::null::<raw::retro_hw_render_interface>();
        let ok = self.call_env(
            raw::RETRO_ENVIRONMENT_GET_HW_RENDER_INTERFACE,
            (&mut interface as *mut *const raw::retro_hw_render_interface).cast::<c_void>(),
        );
        if ok && !interface.is_null() {
            // SAFETY: On success, the frontend stored a valid frontend-owned
            // base interface pointer for the active hardware API.
            Some(HwRenderInterface::from_raw(unsafe { &*interface }))
        } else {
            None
        }
    }

    pub fn hw_render_context_negotiation_interface_support(
        &mut self,
        interface_type: HwRenderContextNegotiationInterfaceType,
    ) -> Option<u32> {
        let mut interface = raw::retro_hw_render_context_negotiation_interface {
            interface_type: interface_type.as_raw(),
            interface_version: 0,
        };
        self.call_env(
            raw::RETRO_ENVIRONMENT_GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT,
            (&mut interface as *mut raw::retro_hw_render_context_negotiation_interface)
                .cast::<c_void>(),
        )
        .then_some(interface.interface_version)
    }

    pub fn set_hw_render_context_negotiation_interface(
        &mut self,
        interface: HwRenderContextNegotiationInterface,
    ) -> bool {
        self.state.hw_render_context_negotiation = Some(interface.as_raw());
        let stored = {
            let stored = self
                .state
                .hw_render_context_negotiation
                .as_mut()
                .expect("just stored HW render context negotiation interface");
            stored as *mut raw::retro_hw_render_context_negotiation_interface
        };
        let ok = self.call_env(
            raw::RETRO_ENVIRONMENT_SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE,
            stored.cast::<c_void>(),
        );
        if !ok {
            self.state.hw_render_context_negotiation = None;
        }
        ok
    }

    pub fn preferred_hw_render(&mut self) -> Option<PreferredHwRender> {
        let mut context_type = HwContextType::None;
        let supports_non_preferred_context = self.call_env(
            RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER,
            (&mut context_type as *mut HwContextType).cast::<c_void>(),
        );
        if context_type == HwContextType::None {
            return None;
        }

        Some(PreferredHwRender {
            context_type,
            supports_non_preferred_context,
        })
    }

    pub fn set_hw_render_from_candidates(
        &mut self,
        candidates: &[HwRenderConfig],
    ) -> Option<HwRenderConfig> {
        let logger = self.logger();
        let preferred = self.preferred_hw_render();
        let preferred_context_type = preferred.map(|render| render.context_type);
        let mut preferred_candidate_rejected = None;

        match preferred {
            Some(PreferredHwRender {
                context_type,
                supports_non_preferred_context,
            }) => logger.info(format!(
                "libretro wrapper: frontend preferred hw render {:?} (non-preferred allowed = {})",
                context_type, supports_non_preferred_context
            )),
            None => logger.info(
                "libretro wrapper: frontend did not report a preferred hw render; probing configured candidates",
            ),
        }

        if let Some(preferred) = preferred_context_type
            && let Some(config) = candidates
                .iter()
                .copied()
                .find(|config| config.context_type == preferred)
        {
            logger.info(format!(
                "libretro wrapper: attempting preferred hw render candidate {}",
                describe_hw_render_config(config)
            ));
            if self.set_hw_render(config) {
                logger.info(format!(
                    "libretro wrapper: frontend accepted preferred hw render candidate {}",
                    describe_hw_render_config(config)
                ));
                return Some(config);
            }
            preferred_candidate_rejected = Some(config);
            logger.warn(format!(
                "libretro wrapper: frontend rejected preferred hw render candidate {}",
                describe_hw_render_config(config)
            ));
        }

        let allow_gles_family_recovery = matches!(
            (
                preferred,
                preferred_candidate_rejected.map(|config| config.context_type),
            ),
            (
                Some(PreferredHwRender {
                    context_type: HwContextType::OpenGl,
                    supports_non_preferred_context: false,
                }),
                Some(HwContextType::OpenGl),
            )
        );

        if matches!(
            preferred,
            Some(PreferredHwRender {
                supports_non_preferred_context: false,
                ..
            })
        ) && !allow_gles_family_recovery
        {
            logger.warn(
                "libretro wrapper: frontend rejected the preferred hw render candidate and disallowed non-preferred fallbacks",
            );
            return None;
        }

        if allow_gles_family_recovery {
            logger.warn(
                "libretro wrapper: frontend rejected preferred generic OpenGl; probing OpenGL ES family fallbacks for compatibility with GLES-only frontends",
            );

            for config in candidates.iter().copied().filter(|config| {
                is_opengl_es_family(config.context_type)
                    && Some(config.context_type) != preferred_context_type
            }) {
                logger.info(format!(
                    "libretro wrapper: attempting OpenGL ES family recovery candidate {}",
                    describe_hw_render_config(config)
                ));
                if self.set_hw_render(config) {
                    logger.info(format!(
                        "libretro wrapper: frontend accepted OpenGL ES family recovery candidate {}",
                        describe_hw_render_config(config)
                    ));
                    return Some(config);
                }
                logger.warn(format!(
                    "libretro wrapper: frontend rejected OpenGL ES family recovery candidate {}",
                    describe_hw_render_config(config)
                ));
            }

            logger.warn(
                "libretro wrapper: frontend rejected preferred generic OpenGl and every OpenGL ES family recovery candidate",
            );
            return None;
        }

        for config in candidates.iter().copied() {
            if Some(config.context_type) == preferred_context_type {
                continue;
            }

            logger.info(format!(
                "libretro wrapper: attempting fallback hw render candidate {}",
                describe_hw_render_config(config)
            ));
            if self.set_hw_render(config) {
                logger.info(format!(
                    "libretro wrapper: frontend accepted fallback hw render candidate {}",
                    describe_hw_render_config(config)
                ));
                return Some(config);
            }
            logger.warn(format!(
                "libretro wrapper: frontend rejected fallback hw render candidate {}",
                describe_hw_render_config(config)
            ));
        }

        logger.warn("libretro wrapper: frontend rejected every configured hw render candidate");
        None
    }

    pub fn get_variable(&mut self, key: &str) -> Option<String> {
        let key = sanitize_cstring(key);
        let mut variable = RawVariable {
            key: key.as_ptr(),
            value: ptr::null(),
        };

        let ok = self.call_env(
            RETRO_ENVIRONMENT_GET_VARIABLE,
            (&mut variable as *mut RawVariable).cast::<c_void>(),
        );
        if !ok || variable.value.is_null() {
            return None;
        }

        // SAFETY: Frontend returns a valid NUL-terminated value on success.
        let value = unsafe { CStr::from_ptr(variable.value) };
        Some(value.to_string_lossy().into_owned())
    }

    pub fn variables_updated(&mut self) -> bool {
        let mut updated = false;
        self.call_env(
            RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE,
            (&mut updated as *mut bool).cast::<c_void>(),
        ) && updated
    }

    pub(crate) fn call_env(&mut self, command: u32, data: *mut c_void) -> bool {
        let Some(callback) = self.state.callbacks.environment else {
            return false;
        };
        // SAFETY: `callback` comes from the frontend via the libretro ABI.
        unsafe { callback(command, data) }
    }
}

/// Per-frame access to frontend callbacks and services.
///
/// `Runtime` is passed to `Core::run`, `Core::load_game`, and hardware context
/// hooks. It owns typed helpers for input polling, video/audio submission,
/// frontend messages, hardware framebuffers, memory maps, and service queries.
pub struct Runtime<'a> {
    state: &'a mut CoreState,
}

impl<'a> Runtime<'a> {
    pub fn environment(&mut self) -> Environment<'_> {
        Environment { state: self.state }
    }

    pub fn logger(&mut self) -> Logger {
        self.environment().logger()
    }

    pub fn poll_input(&self) {
        if let Some(callback) = self.state.callbacks.input_poll {
            // SAFETY: `callback` comes from the frontend via the libretro ABI.
            unsafe { callback() };
        }
    }

    fn input_state_raw(
        &self,
        port: InputPort,
        device: ControllerDevice,
        index: u32,
        id: u32,
    ) -> i16 {
        let Some(callback) = self.state.callbacks.input_state else {
            return 0;
        };
        // SAFETY: `callback` comes from the frontend via the libretro ABI.
        unsafe { callback(port.as_raw(), device.as_raw(), index, id) }
    }

    pub fn joypad_pressed(&self, port: impl Into<InputPort>, button: JoypadButton) -> bool {
        self.input_state_raw(port.into(), ControllerDevice::Joypad, 0, button.as_raw()) != 0
    }

    pub fn joypad_buttons(&self, port: impl Into<InputPort>) -> JoypadButtonSet {
        JoypadButtonSet::from_raw_bits(self.input_state_raw(
            port.into(),
            ControllerDevice::Joypad,
            0,
            input::joypad_mask_query_id(),
        ) as u16)
    }

    pub fn analog_axis(
        &self,
        port: impl Into<InputPort>,
        stick: AnalogStick,
        axis: AnalogAxis,
    ) -> i16 {
        self.input_state_raw(
            port.into(),
            ControllerDevice::Analog,
            stick.as_raw(),
            axis.as_raw(),
        )
    }

    pub fn analog_button(&self, port: impl Into<InputPort>, button: JoypadButton) -> i16 {
        self.input_state_raw(
            port.into(),
            ControllerDevice::Analog,
            input::analog_button_index(),
            button.as_raw(),
        )
    }

    pub fn mouse_axis(&self, port: impl Into<InputPort>, axis: MouseAxis) -> i16 {
        self.input_state_raw(port.into(), ControllerDevice::Mouse, 0, axis.as_raw())
    }

    pub fn mouse_button_pressed(&self, port: impl Into<InputPort>, button: MouseButton) -> bool {
        self.input_state_raw(port.into(), ControllerDevice::Mouse, 0, button.as_raw()) != 0
    }

    pub fn mouse_wheel_moved(&self, port: impl Into<InputPort>, direction: MouseWheel) -> bool {
        self.input_state_raw(port.into(), ControllerDevice::Mouse, 0, direction.as_raw()) != 0
    }

    pub fn pointer_axis(
        &self,
        port: impl Into<InputPort>,
        index: impl Into<PointerIndex>,
        axis: PointerAxis,
    ) -> i16 {
        self.input_state_raw(
            port.into(),
            ControllerDevice::Pointer,
            index.into().as_raw(),
            axis.as_raw(),
        )
    }

    pub fn pointer_pressed(
        &self,
        port: impl Into<InputPort>,
        index: impl Into<PointerIndex>,
    ) -> bool {
        self.input_state_raw(
            port.into(),
            ControllerDevice::Pointer,
            index.into().as_raw(),
            input::pointer_pressed_id(),
        ) != 0
    }

    pub fn pointer_count(&self, port: impl Into<InputPort>) -> i16 {
        self.input_state_raw(
            port.into(),
            ControllerDevice::Pointer,
            0,
            input::pointer_count_id(),
        )
    }

    pub fn pointer_is_offscreen(
        &self,
        port: impl Into<InputPort>,
        index: impl Into<PointerIndex>,
    ) -> bool {
        self.input_state_raw(
            port.into(),
            ControllerDevice::Pointer,
            index.into().as_raw(),
            input::pointer_is_offscreen_id(),
        ) != 0
    }

    pub fn lightgun_axis(&self, port: impl Into<InputPort>, axis: LightgunAxis) -> i16 {
        self.input_state_raw(port.into(), ControllerDevice::Lightgun, 0, axis.as_raw())
    }

    pub fn lightgun_button_pressed(
        &self,
        port: impl Into<InputPort>,
        button: LightgunButton,
    ) -> bool {
        self.input_state_raw(port.into(), ControllerDevice::Lightgun, 0, button.as_raw()) != 0
    }

    pub fn lightgun_is_offscreen(&self, port: impl Into<InputPort>) -> bool {
        self.input_state_raw(
            port.into(),
            ControllerDevice::Lightgun,
            0,
            input::lightgun_is_offscreen_id(),
        ) != 0
    }

    fn video_refresh_raw(&self, data: *const c_void, width: u32, height: u32, pitch: usize) {
        if let Some(callback) = self.state.callbacks.video_refresh {
            // SAFETY: `callback` comes from the frontend via the libretro ABI.
            unsafe { callback(data, width, height, pitch) };
        }
    }

    pub fn video_refresh_frame<T>(
        &self,
        pixels: &[T],
        width: u32,
        height: u32,
        pitch: usize,
    ) -> bool {
        if let Err(error) = validate_software_frame_buffer::<T>(pixels, width, height, pitch) {
            self.cached_logger().error(format!(
                "libretro wrapper: refusing invalid software video frame: {error}"
            ));
            return false;
        }
        self.video_refresh_raw(pixels.as_ptr().cast::<c_void>(), width, height, pitch);
        true
    }

    pub fn video_refresh_frame_with_audio<T>(
        &self,
        pixels: &[T],
        width: u32,
        height: u32,
        pitch: usize,
        audio_frames: &[[i16; 2]],
    ) -> usize {
        let _ = self.video_refresh_frame(pixels, width, height, pitch);
        self.audio_sample_batch(audio_frames)
    }

    pub fn video_refresh_hw(&self, width: u32, height: u32, pitch: usize) {
        self.video_refresh_raw(RETRO_HW_FRAME_BUFFER_VALID, width, height, pitch);
    }

    pub fn video_refresh_hw_with_audio(
        &self,
        width: u32,
        height: u32,
        pitch: usize,
        audio_frames: &[[i16; 2]],
    ) -> usize {
        self.video_refresh_hw(width, height, pitch);
        self.audio_sample_batch(audio_frames)
    }

    pub fn video_refresh_software_framebuffer(&self, framebuffer: SoftwareFramebuffer) {
        let (data, width, height, pitch) = framebuffer.video_refresh_args();
        self.video_refresh_raw(data, width, height, pitch);
    }

    pub fn video_refresh_software_framebuffer_with_audio(
        &self,
        framebuffer: SoftwareFramebuffer,
        audio_frames: &[[i16; 2]],
    ) -> usize {
        self.video_refresh_software_framebuffer(framebuffer);
        self.audio_sample_batch(audio_frames)
    }

    pub fn video_refresh_dupe(&self, width: u32, height: u32) {
        self.video_refresh_raw(std::ptr::null(), width, height, 0);
    }

    pub fn video_refresh_dupe_with_audio(
        &self,
        width: u32,
        height: u32,
        audio_frames: &[[i16; 2]],
    ) -> usize {
        self.video_refresh_dupe(width, height);
        self.audio_sample_batch(audio_frames)
    }

    pub fn set_geometry(&mut self, geometry: GameGeometry) -> bool {
        self.environment().set_geometry(geometry)
    }

    pub fn set_message(&mut self, message: impl AsRef<str>, frames: u32) -> bool {
        self.environment().set_message(message, frames)
    }

    pub fn set_message_ext(&mut self, message: ExtendedMessage) -> bool {
        self.environment().set_message_ext(message)
    }

    pub fn audio_sample(&self, left: i16, right: i16) {
        if let Some(callback) = self.state.callbacks.audio_sample {
            // SAFETY: `callback` comes from the frontend via the libretro ABI.
            unsafe { callback(left, right) };
        }
    }

    pub fn audio_sample_batch(&self, frames: &[[i16; 2]]) -> usize {
        let Some(callback) = self.state.callbacks.audio_sample_batch else {
            return 0;
        };
        // SAFETY: `callback` comes from the frontend via the libretro ABI.
        unsafe { callback(frames.as_ptr().cast::<i16>(), frames.len()) }
    }

    /// Returns the current frontend render target, including the default framebuffer (zero).
    ///
    /// Query this each frame while the hardware context is active. `None` means
    /// the callback is unavailable or its result does not fit an OpenGL name.
    pub fn current_framebuffer(&self) -> Option<u32> {
        let callback = self.state.hw_render?.get_current_framebuffer?;
        // SAFETY: Frontend provided the callback through `SET_HW_RENDER`.
        u32::try_from(unsafe { callback() }).ok()
    }

    pub fn hw_context_type(&self) -> Option<HwContextType> {
        Some(self.state.hw_render?.context_type)
    }

    fn get_proc_address(&self, symbol: &str) -> Result<raw::retro_proc_address_t, String> {
        let symbol = sanitize_cstring(symbol);
        let hw_render = self
            .state
            .hw_render
            .ok_or_else(|| "hardware render callbacks are not available".to_string())?;
        let callback = hw_render
            .get_proc_address
            .ok_or_else(|| "get_proc_address callback is not available".to_string())?;
        // SAFETY: Frontend provided the callback through `SET_HW_RENDER`.
        Ok(unsafe { callback(symbol.as_ptr()) })
    }

    pub fn hw_proc_address(&self, symbol: &str) -> Result<*const c_void, String> {
        if let Some(symbol_address) = self.get_proc_address(symbol)? {
            return Ok(symbol_address as *const () as *const c_void);
        }

        if let Some(symbol_address) = fallback_global_proc_address(symbol) {
            return Ok(symbol_address);
        }

        Err(format!(
            "missing GL symbol {symbol:?} from frontend proc lookup and process global symbols"
        ))
    }

    fn cached_logger(&self) -> Logger {
        Logger {
            callback: self.state.log_callback.map(|callback| callback.log),
        }
    }
}

fn validate_software_frame_buffer<T>(
    pixels: &[T],
    width: u32,
    height: u32,
    pitch: usize,
) -> Result<(), String> {
    if width == 0 || height == 0 {
        return Err(format!(
            "software frame dimensions must be non-zero, got {width}x{height}"
        ));
    }

    let pixel_size = mem::size_of::<T>();
    if pixel_size == 0 {
        return Err("software frame pixel type must not be zero-sized".to_string());
    }

    let row_bytes = (width as usize)
        .checked_mul(pixel_size)
        .ok_or_else(|| format!("software frame row byte size overflowed for width {width}"))?;
    if pitch < row_bytes {
        return Err(format!(
            "software frame pitch {pitch} is smaller than row byte size {row_bytes}"
        ));
    }

    let required_bytes = pitch
        .checked_mul(height as usize)
        .ok_or_else(|| format!("software frame byte size overflowed for height {height}"))?;
    let available_bytes = mem::size_of_val(pixels);
    if available_bytes < required_bytes {
        return Err(format!(
            "software frame buffer has {available_bytes} bytes but {required_bytes} bytes are required"
        ));
    }

    Ok(())
}

#[cfg(unix)]
fn fallback_global_proc_address(symbol: &str) -> Option<*const c_void> {
    let symbol = sanitize_cstring(symbol);
    // Some old EGL stacks only expose core GLES2 entry points through the
    // process symbol table even when RetroArch's proc callback delegates to
    // eglGetProcAddress. This keeps the frontend-owned context contract while
    // avoiding a direct dependency on a platform GL library.
    let pointer = unsafe { dlsym(std::ptr::null_mut(), symbol.as_ptr()) };
    if pointer.is_null() {
        None
    } else {
        Some(pointer.cast_const())
    }
}

#[cfg(not(unix))]
fn fallback_global_proc_address(_symbol: &str) -> Option<*const c_void> {
    None
}

#[cfg(unix)]
#[cfg_attr(target_os = "linux", link(name = "dl"))]
unsafe extern "C" {
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

#[doc(hidden)]
// These are safe Rust wrappers around libretro's C ABI entrypoints. The exported
// `extern "C"` functions generated by `export_core!` keep the ABI safe to call
// from the frontend while the raw pointer validation stays centralized here.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub mod __private {
    use super::*;

    pub type RawSystemAvInfo = raw::retro_system_av_info;

    pub fn set_factory(factory: CoreFactory) {
        if FACTORY.get().is_none() {
            let _ = FACTORY.set(factory);
        }
    }

    pub fn retro_set_environment(cb: raw::retro_environment_t) {
        with_state(|state| {
            state.callbacks.environment = cb;
            catch_state_callback(state, "retro_set_environment", (), |state| {
                state.with_core(|core, state| {
                    {
                        let mut env = Environment { state };
                        core.on_set_environment(&mut env);
                    }
                    let has_keyboard_event = state.event_handlers.has_keyboard_event();
                    let has_audio_callback = state.event_handlers.has_audio_callback();
                    let has_audio_buffer_status = state.event_handlers.has_audio_buffer_status();
                    let frame_time_reference = state.event_handlers.frame_time_reference();
                    let mut env = Environment { state };
                    if has_keyboard_event {
                        let _ = env.set_keyboard_callback();
                    }
                    if has_audio_callback {
                        let _ = env.set_audio_callback();
                    }
                    if has_audio_buffer_status {
                        let _ = env.set_audio_buffer_status_callback(true);
                    }
                    if let Some(reference) = frame_time_reference {
                        let _ = env.set_frame_time_callback(reference);
                    }
                });
            });
        });
    }

    pub fn retro_set_video_refresh(cb: raw::retro_video_refresh_t) {
        with_state(|state| state.callbacks.video_refresh = cb);
    }

    pub fn retro_set_audio_sample(cb: raw::retro_audio_sample_t) {
        with_state(|state| state.callbacks.audio_sample = cb);
    }

    pub fn retro_set_audio_sample_batch(cb: raw::retro_audio_sample_batch_t) {
        with_state(|state| state.callbacks.audio_sample_batch = cb);
    }

    pub fn retro_set_input_poll(cb: raw::retro_input_poll_t) {
        with_state(|state| state.callbacks.input_poll = cb);
    }

    pub fn retro_set_input_state(cb: raw::retro_input_state_t) {
        with_state(|state| state.callbacks.input_state = cb);
    }

    pub fn retro_init() {
        with_state(|state| {
            catch_state_callback(state, "retro_init", (), |state| {
                state.with_core(|core, state| {
                    let mut env = Environment { state };
                    core.init(&mut env);
                });
            });
        });
    }

    pub fn retro_deinit() {
        with_state(|state| {
            catch_state_callback(state, "retro_deinit", (), |state| {
                if let Some(core) = state.core.as_mut() {
                    core.deinit();
                }
            });
            state.reset_frontend_state();
            state.core = None;
        });
    }

    pub fn retro_api_version() -> u32 {
        RETRO_API_VERSION
    }

    pub fn retro_get_system_info(info: *mut RawSystemInfo) {
        if info.is_null() {
            return;
        }

        // Keep the frontend-facing out-param initialized even if the core callback fails.
        unsafe { *info = RawSystemInfo::default() };
        with_state(|state| {
            catch_state_callback(state, "retro_get_system_info", (), |state| {
                if state.system_info.is_none() {
                    let system_info = state.with_core(|core, _| core.system_info());
                    // libretro requires these pointers to remain valid until
                    // retro_deinit(); RetroArch may keep fields from an earlier
                    // call while making later retro_get_system_info() calls.
                    state.system_info = Some(OwnedSystemInfo::new(system_info));
                }
                if let Some(storage) = &state.system_info {
                    // SAFETY: `info` is provided by the frontend.
                    unsafe {
                        *info = RawSystemInfo {
                            library_name: storage.library_name.as_ptr(),
                            library_version: storage.library_version.as_ptr(),
                            valid_extensions: storage
                                .valid_extensions
                                .as_ref()
                                .map_or(ptr::null(), |value| value.as_ptr()),
                            need_fullpath: storage.need_fullpath,
                            block_extract: storage.block_extract,
                        };
                    }
                }
            });
        });
    }

    pub fn retro_get_system_av_info(info: *mut raw::retro_system_av_info) {
        if info.is_null() {
            return;
        }

        // Keep the frontend-facing out-param initialized even if the core callback fails.
        unsafe { *info = raw::retro_system_av_info::default() };
        with_state(|state| {
            catch_state_callback(state, "retro_get_system_av_info", (), |state| {
                let av = state.with_core(|core, _| core.av_info());
                // SAFETY: `info` is provided by the frontend.
                unsafe { *info = av.as_raw() };
            });
        });
    }

    pub fn retro_set_controller_port_device(port: u32, device: u32) {
        with_state(|state| {
            catch_state_callback(state, "retro_set_controller_port_device", (), |state| {
                state.with_core(|core, _| {
                    core.set_controller_port_device(
                        InputPort::from(port),
                        ControllerDevice::from_raw(device),
                    );
                });
            });
        });
    }

    pub fn retro_reset() {
        with_state(|state| {
            catch_state_callback(state, "retro_reset", (), |state| {
                state.with_core(|core, _| core.reset());
            });
        });
    }

    pub fn retro_run() {
        with_state(|state| {
            catch_state_callback(state, "retro_run", (), |state| {
                state.with_core(|core, state| {
                    let mut runtime = Runtime { state };
                    core.run(&mut runtime);
                });
            });
        });
    }

    pub fn retro_serialize_size() -> usize {
        with_state(|state| {
            catch_state_callback(state, "retro_serialize_size", 0, |state| {
                state.with_core(|core, _| core.serialize_size())
            })
        })
    }

    pub fn retro_serialize(data: *mut c_void, len: usize) -> bool {
        if data.is_null() {
            return false;
        }

        with_state(|state| {
            // SAFETY: Caller provided a writable buffer of `len` bytes.
            let buffer = unsafe { std::slice::from_raw_parts_mut(data.cast::<u8>(), len) };
            catch_state_callback(state, "retro_serialize", false, |state| {
                state.with_core(|core, _| core.serialize(buffer))
            })
        })
    }

    pub fn retro_unserialize(data: *const c_void, len: usize) -> bool {
        if data.is_null() {
            return false;
        }

        with_state(|state| {
            // SAFETY: Caller provided a readable buffer of `len` bytes.
            let buffer = unsafe { std::slice::from_raw_parts(data.cast::<u8>(), len) };
            catch_state_callback(state, "retro_unserialize", false, |state| {
                state.with_core(|core, _| core.unserialize(buffer))
            })
        })
    }

    pub fn retro_cheat_reset() {
        with_state(|state| {
            catch_state_callback(state, "retro_cheat_reset", (), |state| {
                state.with_core(|core, _| core.cheat_reset());
            });
        });
    }

    pub fn retro_cheat_set(index: u32, enabled: bool, code: *const c_char) {
        with_state(|state| {
            let code = if code.is_null() {
                None
            } else {
                // SAFETY: The frontend provides a valid C string.
                Some(CheatCode::from_c_str(unsafe { CStr::from_ptr(code) }))
            };
            catch_state_callback(state, "retro_cheat_set", (), |state| {
                state.with_core(|core, _| core.cheat_set(CheatIndex::from(index), enabled, code));
            });
        });
    }

    pub fn retro_load_game(game: *const RawGameInfo) -> bool {
        with_state(|state| {
            // SAFETY: `game` follows the libretro ABI for the duration of the call.
            let game = unsafe { GameInfo::from_raw(game) };
            catch_state_callback(state, "retro_load_game", false, |state| {
                state.with_core(|core, state| {
                    let mut runtime = Runtime { state };
                    core.load_game(game, &mut runtime)
                })
            })
        })
    }

    pub fn retro_load_game_special(
        game_type: u32,
        info: *const RawGameInfo,
        num_info: usize,
    ) -> bool {
        with_state(|state| {
            let games = if info.is_null() || num_info == 0 {
                Vec::new()
            } else {
                // SAFETY: The frontend provides a valid array for the duration of the call.
                let raw = unsafe { std::slice::from_raw_parts(info, num_info) };
                raw.iter()
                    .filter_map(|entry| unsafe { GameInfo::from_raw(entry) })
                    .collect::<Vec<_>>()
            };
            catch_state_callback(state, "retro_load_game_special", false, |state| {
                state.with_core(|core, state| {
                    let mut runtime = Runtime { state };
                    core.load_game_special(SubsystemId::new(game_type), &games, &mut runtime)
                })
            })
        })
    }

    pub fn retro_unload_game() {
        with_state(|state| {
            catch_state_callback(state, "retro_unload_game", (), |state| {
                state.with_core(|core, _| core.unload_game());
            });
        });
    }

    pub fn retro_get_region() -> u32 {
        with_state(|state| {
            catch_state_callback(state, "retro_get_region", Region::Ntsc.as_raw(), |state| {
                state.with_core(|core, _| core.region().as_raw())
            })
        })
    }

    pub fn retro_get_memory_data(id: u32) -> *mut c_void {
        with_state(|state| {
            catch_state_callback(state, "retro_get_memory_data", ptr::null_mut(), |state| {
                state.with_core(|core, _| {
                    core.memory_region(MemoryRegion::from_raw(id))
                        .map_or(ptr::null_mut(), |mut memory| memory.as_mut_ptr())
                })
            })
        })
    }

    pub fn retro_get_memory_size(id: u32) -> usize {
        with_state(|state| {
            catch_state_callback(state, "retro_get_memory_size", 0, |state| {
                state.with_core(|core, _| {
                    core.memory_region(MemoryRegion::from_raw(id))
                        .map_or(0, |memory| memory.len())
                })
            })
        })
    }
}

/// Export a `Core` implementation as the required libretro `retro_*` symbols.
///
/// The macro keeps ABI exports uniform and routes callbacks through the crate's
/// typed panic-catching boundaries.
#[macro_export]
macro_rules! export_core {
    ($factory:expr) => {
        #[doc(hidden)]
        fn __libretro_create_core() -> $crate::CoreBundle {
            $crate::create_core($factory)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_set_environment(cb: $crate::retro_environment_t) {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_set_environment(cb);
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_set_video_refresh(cb: $crate::retro_video_refresh_t) {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_set_video_refresh(cb);
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_set_audio_sample(cb: $crate::retro_audio_sample_t) {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_set_audio_sample(cb);
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_set_audio_sample_batch(cb: $crate::retro_audio_sample_batch_t) {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_set_audio_sample_batch(cb);
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_set_input_poll(cb: $crate::retro_input_poll_t) {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_set_input_poll(cb);
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_set_input_state(cb: $crate::retro_input_state_t) {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_set_input_state(cb);
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_init() {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_init();
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_deinit() {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_deinit();
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_api_version() -> u32 {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_api_version()
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_get_system_info(info: *mut $crate::RawSystemInfo) {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_get_system_info(info);
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_get_system_av_info(info: *mut $crate::__private::RawSystemAvInfo) {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_get_system_av_info(info);
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_set_controller_port_device(port: u32, device: u32) {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_set_controller_port_device(port, device);
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_reset() {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_reset();
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_run() {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_run();
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_serialize_size() -> usize {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_serialize_size()
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_serialize(data: *mut std::ffi::c_void, len: usize) -> bool {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_serialize(data, len)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_unserialize(data: *const std::ffi::c_void, len: usize) -> bool {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_unserialize(data, len)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_cheat_reset() {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_cheat_reset();
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_cheat_set(
            index: u32,
            enabled: bool,
            code: *const std::ffi::c_char,
        ) {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_cheat_set(index, enabled, code);
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_load_game(game: *const $crate::RawGameInfo) -> bool {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_load_game(game)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_load_game_special(
            game_type: u32,
            info: *const $crate::RawGameInfo,
            num_info: usize,
        ) -> bool {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_load_game_special(game_type, info, num_info)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_unload_game() {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_unload_game();
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_get_region() -> u32 {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_get_region()
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_get_memory_data(id: u32) -> *mut std::ffi::c_void {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_get_memory_data(id)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn retro_get_memory_size(id: u32) -> usize {
            $crate::__private::set_factory(__libretro_create_core);
            $crate::__private::retro_get_memory_size(id)
        }
    };
}

#[derive(Default)]
struct CoreCallbacks {
    environment: raw::retro_environment_t,
    video_refresh: raw::retro_video_refresh_t,
    audio_sample: raw::retro_audio_sample_t,
    audio_sample_batch: raw::retro_audio_sample_batch_t,
    input_poll: raw::retro_input_poll_t,
    input_state: raw::retro_input_state_t,
}

struct ContentInfoOverrideStorage {
    _extensions: Vec<CString>,
    _raw: Vec<RawContentInfoOverride>,
}

pub(crate) struct InputDescriptorStorage {
    pub(crate) _descriptions: Vec<CString>,
    pub(crate) _raw: Vec<RawInputDescriptor>,
}

struct OwnedSystemInfo {
    library_name: CString,
    library_version: CString,
    valid_extensions: Option<CString>,
    need_fullpath: bool,
    block_extract: bool,
}

impl OwnedSystemInfo {
    fn new(info: SystemInfo) -> Self {
        let library_name = sanitize_cstring(info.library_name);
        let library_version = sanitize_cstring(info.library_version);
        let valid_extensions = info.valid_extensions.map(sanitize_cstring);

        Self {
            library_name,
            library_version,
            valid_extensions,
            need_fullpath: info.need_fullpath,
            block_extract: info.block_extract,
        }
    }
}

#[derive(Default)]
struct CoreState {
    core: Option<Box<dyn Core>>,
    event_handlers: CoreEventHandlers,
    callbacks: CoreCallbacks,
    system_info: Option<OwnedSystemInfo>,
    variables: Option<options::CoreOptionsStorage>,
    content_info_overrides: Option<ContentInfoOverrideStorage>,
    input_descriptors: Option<InputDescriptorStorage>,
    subsystem_info: Option<subsystem::SubsystemInfoStorage>,
    netpacket_interface: Option<netplay::NetpacketInterfaceStorage>,
    log_callback: Option<RawLogCallback>,
    hw_render: Option<RawHwRenderCallback>,
    creating_glow_context_allowed: bool,
    hw_render_context_negotiation: Option<raw::retro_hw_render_context_negotiation_interface>,
}

impl CoreState {
    fn with_core<T>(&mut self, f: impl FnOnce(&mut dyn Core, &mut CoreState) -> T) -> T {
        let core = self.core.take().unwrap_or_else(|| {
            let factory = *FACTORY
                .get()
                .expect("libretro core factory was not registered");
            let bundle = factory();
            self.event_handlers = bundle.event_handlers;
            bundle.core
        });
        let mut restore_guard = CoreRestoreGuard::new(self, core);
        let result = f(restore_guard.core_mut(), self);
        self.core = Some(restore_guard.into_core());
        result
    }

    fn reset_frontend_state(&mut self) {
        self.callbacks = CoreCallbacks::default();
        self.system_info = None;
        self.variables = None;
        self.content_info_overrides = None;
        self.input_descriptors = None;
        self.subsystem_info = None;
        self.netpacket_interface = None;
        self.log_callback = None;
        self.hw_render = None;
        self.creating_glow_context_allowed = false;
        self.hw_render_context_negotiation = None;
    }
}

unsafe impl Send for CoreState {}

struct CoreRestoreGuard {
    state: *mut CoreState,
    core: Option<Box<dyn Core>>,
    armed: bool,
}

impl CoreRestoreGuard {
    fn new(state: &mut CoreState, core: Box<dyn Core>) -> Self {
        Self {
            state,
            core: Some(core),
            armed: true,
        }
    }

    fn core_mut(&mut self) -> &mut dyn Core {
        self.core
            .as_deref_mut()
            .expect("libretro core restore guard always owns a core")
    }

    fn into_core(mut self) -> Box<dyn Core> {
        self.armed = false;
        self.core
            .take()
            .expect("libretro core restore guard always owns a core")
    }
}

impl Drop for CoreRestoreGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }

        if let Some(core) = self.core.take() {
            // SAFETY: The guard is created from the active `CoreState` stack
            // borrow and is dropped before that state can go out of scope.
            let state = unsafe { &mut *self.state };
            if state.core.is_none() {
                state.core = Some(core);
            }
        }
    }
}

pub(crate) fn sanitize_cstring(value: impl AsRef<str>) -> CString {
    let bytes = value.as_ref().as_bytes();
    if bytes.contains(&0) {
        let bytes = bytes
            .iter()
            .copied()
            .filter(|byte| *byte != 0)
            .collect::<Vec<_>>();
        // SAFETY: The filter above removes every interior NUL byte.
        unsafe { CString::from_vec_unchecked(bytes) }
    } else {
        // SAFETY: The contains check proves this byte vector has no interior NUL.
        unsafe { CString::from_vec_unchecked(bytes.to_vec()) }
    }
}

fn with_state<T>(f: impl FnOnce(&mut CoreState) -> T) -> T {
    let state = STATE.get_or_init(|| Mutex::new(CoreState::default()));
    let mut guard = state.lock().unwrap_or_else(|poisoned| {
        eprintln!("libretro wrapper: recovering from poisoned state mutex after callback panic");
        poisoned.into_inner()
    });
    f(&mut guard)
}

fn panic_payload_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_string()
    }
}

fn log_callback_panic(state: &CoreState, callback_name: &str, payload: Box<dyn Any + Send>) {
    Logger {
        callback: state.log_callback.map(|callback| callback.log),
    }
    .error(format!(
        "libretro wrapper: panic escaped core {callback_name} callback: {}",
        panic_payload_message(&*payload)
    ));
}

fn catch_state_callback<T>(
    state: &mut CoreState,
    callback_name: &'static str,
    fallback: T,
    f: impl FnOnce(&mut CoreState) -> T,
) -> T {
    match catch_unwind(AssertUnwindSafe(|| f(state))) {
        Ok(value) => value,
        Err(payload) => {
            log_callback_panic(state, callback_name, payload);
            fallback
        }
    }
}

unsafe extern "C" fn hw_context_reset_trampoline() {
    with_state(dispatch_hw_context_reset);
}

fn dispatch_hw_context_reset(state: &mut CoreState) {
    state.creating_glow_context_allowed = true;
    catch_state_callback(state, "hw_context_reset", (), |state| {
        state.with_core(|core, state| {
            let mut runtime = Runtime { state };
            core.hw_context_reset(&mut runtime);
        });
    });
    // catch_state_callback contains core panics, so permission is cleared on
    // both success and failure before another callback can obtain Runtime.
    state.creating_glow_context_allowed = false;
}

unsafe extern "C" fn hw_context_destroy_trampoline() {
    with_state(|state| {
        catch_state_callback(state, "hw_context_destroy", (), |state| {
            state.with_core(|core, state| {
                let mut runtime = Runtime { state };
                core.hw_context_destroy(&mut runtime);
            });
        });
    });
}

unsafe extern "C" fn core_options_update_display_trampoline() -> bool {
    with_state(|state| {
        catch_state_callback(state, "core_options_update_display", false, |state| {
            state.with_core(|core, state| {
                let mut env = Environment { state };
                core.core_options_update_display(&mut env)
            })
        })
    })
}

pub(crate) unsafe extern "C" fn audio_buffer_status_trampoline(
    active: bool,
    occupancy: u32,
    underrun_likely: bool,
) {
    with_state(|state| {
        catch_state_callback(state, "audio_buffer_status", (), |state| {
            let status = AudioBufferStatus::from_raw(active, occupancy, underrun_likely);
            state.with_core(|core, state| {
                state
                    .event_handlers
                    .dispatch_audio_buffer_status(core, status);
            });
        });
    });
}

pub(crate) unsafe extern "C" fn audio_callback_trampoline() {
    with_state(|state| {
        catch_state_callback(state, "audio_callback", (), |state| {
            state.with_core(|core, state| {
                state.event_handlers.dispatch_audio_callback(core);
            });
        });
    });
}

pub(crate) unsafe extern "C" fn audio_set_state_trampoline(enabled: bool) {
    with_state(|state| {
        catch_state_callback(state, "audio_callback_state_changed", (), |state| {
            let enabled = AudioCallbackState::from_active(enabled);
            state.with_core(|core, state| {
                state
                    .event_handlers
                    .dispatch_audio_callback_state_changed(core, enabled);
            });
        });
    });
}

pub(crate) unsafe extern "C" fn frame_time_trampoline(usec: raw::retro_usec_t) {
    with_state(|state| {
        catch_state_callback(state, "frame_time", (), |state| {
            let time = FrameTime::from_micros(usec);
            state.with_core(|core, state| {
                state.event_handlers.dispatch_frame_time(core, time);
            });
        });
    });
}

pub(crate) unsafe extern "C" fn keyboard_event_trampoline(
    down: bool,
    keycode: u32,
    character: u32,
    key_modifiers: u16,
) {
    with_state(|state| {
        catch_state_callback(state, "keyboard_event", (), |state| {
            let event = KeyboardEvent::from_raw(down, keycode, character, key_modifiers);
            state.with_core(|core, state| {
                state.event_handlers.dispatch_keyboard_event(core, event);
            });
        });
    });
}

pub(crate) unsafe extern "C" fn proc_address_trampoline(
    symbol: *const c_char,
) -> raw::retro_proc_address_t {
    if symbol.is_null() {
        return None;
    }

    with_state(|state| {
        catch_state_callback(state, "proc_address", None, |state| {
            // SAFETY: The frontend provides a non-null NUL-terminated symbol
            // name for the immediate duration of this callback.
            let symbol = unsafe { CStr::from_ptr(symbol) };
            state.with_core(|core, _| core.proc_address(symbol).and_then(CoreProcAddress::as_raw))
        })
    })
}

pub(crate) unsafe extern "C" fn location_initialized_trampoline() {
    with_state(|state| {
        catch_state_callback(state, "location_initialized", (), |state| {
            state.with_core(|core, state| {
                state.event_handlers.dispatch_location_initialized(core);
            });
        });
    });
}

pub(crate) unsafe extern "C" fn location_deinitialized_trampoline() {
    with_state(|state| {
        catch_state_callback(state, "location_deinitialized", (), |state| {
            state.with_core(|core, state| {
                state.event_handlers.dispatch_location_deinitialized(core);
            });
        });
    });
}

pub(crate) unsafe extern "C" fn camera_initialized_trampoline() {
    with_state(|state| {
        catch_state_callback(state, "camera_initialized", (), |state| {
            state.with_core(|core, state| {
                state.event_handlers.dispatch_camera_initialized(core);
            });
        });
    });
}

pub(crate) unsafe extern "C" fn camera_deinitialized_trampoline() {
    with_state(|state| {
        catch_state_callback(state, "camera_deinitialized", (), |state| {
            state.with_core(|core, state| {
                state.event_handlers.dispatch_camera_deinitialized(core);
            });
        });
    });
}

pub(crate) unsafe extern "C" fn camera_frame_raw_trampoline(
    buffer: *const u32,
    width: u32,
    height: u32,
    pitch: usize,
) {
    let Some(frame) = (unsafe { CameraRawFrame::from_raw(buffer, width, height, pitch) }) else {
        return;
    };
    with_state(|state| {
        catch_state_callback(state, "camera_frame_raw", (), |state| {
            state.with_core(|core, state| {
                state.event_handlers.dispatch_camera_raw_frame(core, frame);
            });
        });
    });
}

pub(crate) unsafe extern "C" fn camera_frame_opengl_texture_trampoline(
    texture_id: u32,
    texture_target: u32,
    affine: *const f32,
) {
    if affine.is_null() {
        return;
    }
    let mut transform = [0.0f32; 9];
    transform.copy_from_slice(unsafe { std::slice::from_raw_parts(affine, 9) });
    let frame = CameraTextureFrame {
        texture_id: CameraTextureId::new(texture_id),
        texture_target: CameraTextureTarget::new(texture_target),
        affine: transform,
    };
    with_state(|state| {
        catch_state_callback(state, "camera_frame_texture", (), |state| {
            state.with_core(|core, state| {
                state
                    .event_handlers
                    .dispatch_camera_texture_frame(core, frame);
            });
        });
    });
}

pub(crate) unsafe extern "C" fn disk_set_eject_state_trampoline(ejected: bool) -> bool {
    with_state(|state| {
        catch_state_callback(state, "disk_set_tray_state", false, |state| {
            state
                .with_core(|core, _| core.disk_set_tray_state(DiskTrayState::from_ejected(ejected)))
        })
    })
}

pub(crate) unsafe extern "C" fn disk_get_eject_state_trampoline() -> bool {
    with_state(|state| {
        catch_state_callback(state, "disk_tray_state", false, |state| {
            state.with_core(|core, _| core.disk_tray_state().is_ejected())
        })
    })
}

pub(crate) unsafe extern "C" fn disk_get_image_index_trampoline() -> u32 {
    with_state(|state| {
        catch_state_callback(state, "disk_image_index", 0, |state| {
            state.with_core(|core, _| core.disk_image_index().as_raw())
        })
    })
}

pub(crate) unsafe extern "C" fn disk_set_image_index_trampoline(index: u32) -> bool {
    with_state(|state| {
        catch_state_callback(state, "disk_set_image_index", false, |state| {
            state.with_core(|core, _| core.disk_set_image_index(DiskIndex::new(index)))
        })
    })
}

pub(crate) unsafe extern "C" fn disk_get_num_images_trampoline() -> u32 {
    with_state(|state| {
        catch_state_callback(state, "disk_image_count", 0, |state| {
            state.with_core(|core, _| core.disk_image_count())
        })
    })
}

pub(crate) unsafe extern "C" fn disk_replace_image_index_trampoline(
    index: u32,
    info: *const RawGameInfo,
) -> bool {
    with_state(|state| {
        catch_state_callback(state, "disk_replace_image_index", false, |state| {
            let game = unsafe { GameInfo::from_raw(info) };
            state.with_core(|core, _| core.disk_replace_image_index(DiskIndex::new(index), game))
        })
    })
}

pub(crate) unsafe extern "C" fn disk_add_image_index_trampoline() -> bool {
    with_state(|state| {
        catch_state_callback(state, "disk_add_image_index", false, |state| {
            state.with_core(|core, _| core.disk_add_image_index())
        })
    })
}

pub(crate) unsafe extern "C" fn disk_set_initial_image_trampoline(
    index: u32,
    path: *const c_char,
) -> bool {
    if path.is_null() {
        return false;
    }

    with_state(|state| {
        catch_state_callback(state, "disk_set_initial_image", false, |state| {
            let path = unsafe { CStr::from_ptr(path) };
            state.with_core(|core, _| core.disk_set_initial_image(DiskIndex::new(index), path))
        })
    })
}

pub(crate) unsafe extern "C" fn disk_get_image_path_trampoline(
    index: u32,
    out: *mut c_char,
    len: usize,
) -> bool {
    with_state(|state| {
        catch_state_callback(state, "disk_image_path", false, |state| {
            let value = state.with_core(|core, _| core.disk_image_path(DiskIndex::new(index)));
            disk::write_frontend_string(value, out, len)
        })
    })
}

pub(crate) unsafe extern "C" fn disk_get_image_label_trampoline(
    index: u32,
    out: *mut c_char,
    len: usize,
) -> bool {
    with_state(|state| {
        catch_state_callback(state, "disk_image_label", false, |state| {
            let value = state.with_core(|core, _| core.disk_image_label(DiskIndex::new(index)));
            disk::write_frontend_string(value, out, len)
        })
    })
}

pub(crate) unsafe extern "C" fn netpacket_start_trampoline(
    client_id: u16,
    send_fn: raw::retro_netpacket_send_t,
    poll_receive_fn: raw::retro_netpacket_poll_receive_t,
) {
    let Some(session) =
        NetpacketSession::new(NetplayClientId::new(client_id), send_fn, poll_receive_fn)
    else {
        return;
    };

    with_state(|state| {
        catch_state_callback(state, "netpacket_start", (), |state| {
            state.with_core(|core, _| core.netpacket_start(session));
        });
    });
}

pub(crate) unsafe extern "C" fn netpacket_receive_trampoline(
    buf: *const c_void,
    len: usize,
    client_id: u16,
) {
    if buf.is_null() && len != 0 {
        return;
    }

    with_state(|state| {
        catch_state_callback(state, "netpacket_receive", (), |state| {
            let data = if len == 0 {
                &[]
            } else {
                unsafe { std::slice::from_raw_parts(buf.cast::<u8>(), len) }
            };
            state.with_core(|core, _| {
                core.netpacket_receive(Netpacket {
                    client_id: NetplayClientId::new(client_id),
                    data,
                });
            });
        });
    });
}

pub(crate) unsafe extern "C" fn netpacket_stop_trampoline() {
    with_state(|state| {
        catch_state_callback(state, "netpacket_stop", (), |state| {
            state.with_core(|core, _| core.netpacket_stop());
        });
    });
}

pub(crate) unsafe extern "C" fn netpacket_poll_trampoline() {
    with_state(|state| {
        catch_state_callback(state, "netpacket_poll", (), |state| {
            state.with_core(|core, _| core.netpacket_poll());
        });
    });
}

pub(crate) unsafe extern "C" fn netpacket_connected_trampoline(client_id: u16) -> bool {
    with_state(|state| {
        catch_state_callback(state, "netpacket_connected", false, |state| {
            state.with_core(|core, _| core.netpacket_connected(NetplayClientId::new(client_id)))
        })
    })
}

pub(crate) unsafe extern "C" fn netpacket_disconnected_trampoline(client_id: u16) {
    with_state(|state| {
        catch_state_callback(state, "netpacket_disconnected", (), |state| {
            state.with_core(|core, _| {
                core.netpacket_disconnected(NetplayClientId::new(client_id));
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::MutexGuard;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CapturedContentOverride {
        extensions: String,
        need_fullpath: bool,
        persistent_data: bool,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CapturedMessage {
        message: String,
        frames: u32,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CapturedExtendedMessage {
        message: String,
        duration: u32,
        priority: u32,
        level: LogLevel,
        target: raw::retro_message_target,
        kind: raw::retro_message_type,
        progress: i8,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CapturedVideoRefresh {
        data_kind: CapturedVideoDataKind,
        data_addr: usize,
        width: u32,
        height: u32,
        pitch: usize,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct CapturedInputQuery {
        port: u32,
        device: u32,
        index: u32,
        id: u32,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CapturedInputDescriptor {
        port: u32,
        device: u32,
        index: u32,
        id: u32,
        description: String,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CapturedControllerDescription {
        description: String,
        id: u32,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CapturedCoreOptionValue {
        value: String,
        label: Option<String>,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CapturedCoreOptionDefinition {
        key: String,
        description: String,
        description_categorized: Option<String>,
        info: Option<String>,
        info_categorized: Option<String>,
        category_key: Option<String>,
        values: Vec<CapturedCoreOptionValue>,
        default_value: String,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CapturedCoreOptionCategory {
        key: String,
        description: String,
        info: Option<String>,
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    struct CapturedCoreOptionsV2 {
        categories: Vec<CapturedCoreOptionCategory>,
        definitions: Vec<CapturedCoreOptionDefinition>,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CapturedCoreOptionDisplay {
        key: String,
        visible: bool,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CapturedVariable {
        key: String,
        value: Option<String>,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CapturedVfsOpen {
        path: String,
        mode: u32,
        hints: u32,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CapturedVfsRename {
        old_path: String,
        new_path: String,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CapturedMemoryDescriptor {
        flags: u64,
        ptr_is_null: bool,
        offset: usize,
        start: usize,
        select: usize,
        disconnect: usize,
        len: usize,
        addrspace: Option<String>,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CapturedSubsystemMemory {
        extension: String,
        memory_type: u32,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CapturedSubsystemRom {
        description: String,
        valid_extensions: String,
        need_fullpath: bool,
        block_extract: bool,
        required: bool,
        memory: Vec<CapturedSubsystemMemory>,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CapturedSubsystem {
        description: String,
        identifier: String,
        id: u32,
        roms: Vec<CapturedSubsystemRom>,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum CapturedVideoDataKind {
        Software,
        Hardware,
        Dupe,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum LocationLifecycleEvent {
        Initialized,
        Deinitialized,
    }

    #[derive(Clone, Debug, PartialEq)]
    enum CameraEvent {
        Initialized,
        Deinitialized,
        Raw {
            width: u32,
            height: u32,
            pitch: usize,
            pixels: Vec<u32>,
        },
        Texture {
            texture_id: u32,
            texture_target: u32,
            affine: [f32; 9],
        },
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum DiskControlEvent {
        SetTray(DiskTrayState),
        SetImage(DiskIndex),
        ReplaceImage(DiskIndex, bool),
        AddImage,
        SetInitialImage(DiskIndex, String),
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum NetpacketEvent {
        Start(NetplayClientId, bool),
        Receive(NetplayClientId, Vec<u8>),
        Stop,
        Poll,
        Connected(NetplayClientId),
        Disconnected(NetplayClientId),
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CapturedNetpacketSend {
        flags: i32,
        data: Vec<u8>,
        client_id: u16,
    }

    #[derive(Clone, Copy, Debug)]
    struct CapturedNetpacketCallback {
        start: raw::retro_netpacket_start_t,
        receive: raw::retro_netpacket_receive_t,
        stop: raw::retro_netpacket_stop_t,
        poll: raw::retro_netpacket_poll_t,
        connected: raw::retro_netpacket_connected_t,
        disconnected: raw::retro_netpacket_disconnected_t,
        protocol_version: usize,
    }

    impl CapturedNetpacketCallback {
        fn from_raw(callback: raw::retro_netpacket_callback) -> Self {
            Self {
                start: callback.start,
                receive: callback.receive,
                stop: callback.stop,
                poll: callback.poll,
                connected: callback.connected,
                disconnected: callback.disconnected,
                protocol_version: callback.protocol_version as usize,
            }
        }
    }

    static CAPTURED_CONTENT_OVERRIDES: OnceLock<Mutex<Vec<CapturedContentOverride>>> =
        OnceLock::new();
    static CAPTURED_SUPPORT_NO_GAME: OnceLock<Mutex<Vec<bool>>> = OnceLock::new();
    static CAPTURED_MESSAGES: OnceLock<Mutex<Vec<CapturedMessage>>> = OnceLock::new();
    static CAPTURED_EXTENDED_MESSAGES: OnceLock<Mutex<Vec<CapturedExtendedMessage>>> =
        OnceLock::new();
    static CAPTURED_VIDEO_REFRESHES: OnceLock<Mutex<Vec<CapturedVideoRefresh>>> = OnceLock::new();
    static CAPTURED_INPUT_QUERIES: OnceLock<Mutex<Vec<CapturedInputQuery>>> = OnceLock::new();
    static CAPTURED_INPUT_DESCRIPTORS: OnceLock<Mutex<Vec<CapturedInputDescriptor>>> =
        OnceLock::new();
    static CAPTURED_CONTROLLER_INFO: OnceLock<Mutex<Vec<Vec<CapturedControllerDescription>>>> =
        OnceLock::new();
    static CAPTURED_CORE_OPTIONS_VERSION: OnceLock<Mutex<Option<u32>>> = OnceLock::new();
    static CAPTURED_CORE_OPTIONS_V2: OnceLock<Mutex<Option<CapturedCoreOptionsV2>>> =
        OnceLock::new();
    static CAPTURED_CORE_OPTIONS_V1: OnceLock<Mutex<Vec<CapturedCoreOptionDefinition>>> =
        OnceLock::new();
    static CAPTURED_CORE_OPTION_DISPLAYS: OnceLock<Mutex<Vec<CapturedCoreOptionDisplay>>> =
        OnceLock::new();
    static CAPTURED_VARIABLES: OnceLock<Mutex<Vec<CapturedVariable>>> = OnceLock::new();
    static CAPTURED_CORE_OPTIONS_UPDATE_DISPLAY_CALLBACK: OnceLock<
        Mutex<Option<raw::retro_core_options_update_display_callback>>,
    > = OnceLock::new();
    static CAPTURED_VFS_INTERFACE_REQUESTS: OnceLock<Mutex<Vec<u32>>> = OnceLock::new();
    static CAPTURED_VFS_OPENS: OnceLock<Mutex<Vec<CapturedVfsOpen>>> = OnceLock::new();
    static CAPTURED_VFS_CLOSES: OnceLock<Mutex<u32>> = OnceLock::new();
    static CAPTURED_VFS_DIR_CLOSES: OnceLock<Mutex<u32>> = OnceLock::new();
    static CAPTURED_VFS_WRITES: OnceLock<Mutex<Vec<Vec<u8>>>> = OnceLock::new();
    static CAPTURED_VFS_REMOVES: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    static CAPTURED_VFS_RENAMES: OnceLock<Mutex<Vec<CapturedVfsRename>>> = OnceLock::new();
    static CAPTURED_VFS_MKDIRS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    static CAPTURED_VFS_READDIRS: OnceLock<Mutex<u32>> = OnceLock::new();
    static CAPTURED_MEMORY_DESCRIPTORS: OnceLock<Mutex<Vec<CapturedMemoryDescriptor>>> =
        OnceLock::new();
    static CAPTURED_SUBSYSTEM_INFO: OnceLock<Mutex<Vec<CapturedSubsystem>>> = OnceLock::new();
    static SOFTWARE_FRAMEBUFFER_PIXELS: OnceLock<Mutex<Vec<u32>>> = OnceLock::new();
    static CAPTURED_LED_STATES: OnceLock<Mutex<Vec<(i32, i32)>>> = OnceLock::new();
    static CAPTURED_RUMBLE_STATES: OnceLock<Mutex<Vec<(u32, raw::retro_rumble_effect, u16)>>> =
        OnceLock::new();
    static CAPTURED_SENSOR_STATES: OnceLock<Mutex<Vec<(u32, raw::retro_sensor_action, u32)>>> =
        OnceLock::new();
    static CAPTURED_LOCATION_INTERVALS: OnceLock<Mutex<Vec<(u32, u32)>>> = OnceLock::new();
    static CAPTURED_LOCATION_STARTS: OnceLock<Mutex<u32>> = OnceLock::new();
    static CAPTURED_LOCATION_STOPS: OnceLock<Mutex<u32>> = OnceLock::new();
    static CAPTURED_LOCATION_CALLBACK: OnceLock<Mutex<Option<raw::retro_location_callback>>> =
        OnceLock::new();
    static CAPTURED_CAMERA_CALLBACK: OnceLock<Mutex<Option<raw::retro_camera_callback>>> =
        OnceLock::new();
    static CAPTURED_CAMERA_STARTS: OnceLock<Mutex<u32>> = OnceLock::new();
    static CAPTURED_CAMERA_STOPS: OnceLock<Mutex<u32>> = OnceLock::new();
    static CAPTURED_DISK_CONTROL_CALLBACK: OnceLock<
        Mutex<Option<raw::retro_disk_control_callback>>,
    > = OnceLock::new();
    static CAPTURED_DISK_CONTROL_EXT_CALLBACK: OnceLock<
        Mutex<Option<raw::retro_disk_control_ext_callback>>,
    > = OnceLock::new();
    static CAPTURED_NETPACKET_CALLBACK: OnceLock<Mutex<Option<CapturedNetpacketCallback>>> =
        OnceLock::new();
    static CAPTURED_NETPACKET_SENDS: OnceLock<Mutex<Vec<CapturedNetpacketSend>>> = OnceLock::new();
    static CAPTURED_NETPACKET_POLLS: OnceLock<Mutex<u32>> = OnceLock::new();
    static CAPTURED_MIC_OPEN_PARAMS: OnceLock<Mutex<Vec<Option<u32>>>> = OnceLock::new();
    static CAPTURED_MIC_STATES: OnceLock<Mutex<Vec<bool>>> = OnceLock::new();
    static CAPTURED_MIC_CLOSES: OnceLock<Mutex<u32>> = OnceLock::new();
    static CAPTURED_MIDI_WRITES: OnceLock<Mutex<Vec<(u8, u32)>>> = OnceLock::new();
    static CAPTURED_MIDI_FLUSHES: OnceLock<Mutex<u32>> = OnceLock::new();
    static CAPTURED_MIDI_PROBES: OnceLock<Mutex<u32>> = OnceLock::new();
    static CAPTURED_KEYBOARD_CALLBACK: OnceLock<Mutex<Option<RawKeyboardCallback>>> =
        OnceLock::new();
    static CAPTURED_AUDIO_LATENCIES: OnceLock<Mutex<Vec<Option<u32>>>> = OnceLock::new();
    static CAPTURED_AUDIO_BUFFER_STATUS_CALLBACK: OnceLock<
        Mutex<Option<RawAudioBufferStatusCallback>>,
    > = OnceLock::new();
    static CAPTURED_AUDIO_CALLBACK: OnceLock<Mutex<Option<RawAudioCallback>>> = OnceLock::new();
    static CAPTURED_AUDIO_CALLBACK_PROBES: OnceLock<Mutex<u32>> = OnceLock::new();
    static CAPTURED_FRAME_TIME_CALLBACK: OnceLock<Mutex<Option<RawFrameTimeCallback>>> =
        OnceLock::new();
    static CAPTURED_PROC_ADDRESS_INTERFACE: OnceLock<
        Mutex<Option<raw::retro_get_proc_address_interface>>,
    > = OnceLock::new();
    static CAPTURED_FASTFORWARDING_OVERRIDES: OnceLock<
        Mutex<Vec<Option<raw::retro_fastforwarding_override>>>,
    > = OnceLock::new();
    static CAPTURED_ACHIEVEMENT_SUPPORT: OnceLock<Mutex<Vec<bool>>> = OnceLock::new();
    static CAPTURED_PERFORMANCE_LEVELS: OnceLock<Mutex<Vec<u32>>> = OnceLock::new();
    static CAPTURED_PERF_LOGS: OnceLock<Mutex<u32>> = OnceLock::new();
    static CAPTURED_PERF_REGISTERED_IDENTS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    static CAPTURED_ROTATIONS: OnceLock<Mutex<Vec<u32>>> = OnceLock::new();
    static CAPTURED_SYSTEM_AV_INFOS: OnceLock<Mutex<Vec<SystemAvInfo>>> = OnceLock::new();
    static CAPTURED_SHUTDOWNS: OnceLock<Mutex<u32>> = OnceLock::new();
    static CAPTURED_HW_SHARED_CONTEXTS: OnceLock<Mutex<u32>> = OnceLock::new();
    static CAPTURED_SERIALIZATION_QUIRKS: OnceLock<Mutex<Vec<u64>>> = OnceLock::new();
    static CAPTURED_HW_RENDER_STATE: OnceLock<Mutex<CapturedHwRenderState>> = OnceLock::new();
    static CAPTURED_GEOMETRIES: OnceLock<Mutex<Vec<GameGeometry>>> = OnceLock::new();
    static CAPTURED_LIFECYCLE_COUNTS: OnceLock<Mutex<LifecycleCallCounts>> = OnceLock::new();
    static EXTENDED_GAME_INFO_PTR: OnceLock<usize> = OnceLock::new();
    static TEST_SERIAL_GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    static EXTENDED_GAME_CONTENT: &[u8] = b"ROM";
    static FRONTEND_HW_RENDER_INTERFACE: raw::retro_hw_render_interface =
        raw::retro_hw_render_interface {
            interface_type: raw::retro_hw_render_interface_type::Vulkan as i32,
            interface_version: 1,
        };
    static FRONTEND_VFS_INTERFACE: raw::retro_vfs_interface = raw::retro_vfs_interface {
        get_path: Some(capture_vfs_get_path),
        open: Some(capture_vfs_open),
        close: Some(capture_vfs_close),
        size: Some(capture_vfs_size),
        tell: Some(capture_vfs_tell),
        seek: Some(capture_vfs_seek),
        read: Some(capture_vfs_read),
        write: Some(capture_vfs_write),
        flush: Some(capture_vfs_flush),
        remove: Some(capture_vfs_remove),
        rename: Some(capture_vfs_rename),
        truncate: Some(capture_vfs_truncate),
        stat: Some(capture_vfs_stat),
        mkdir: Some(capture_vfs_mkdir),
        opendir: Some(capture_vfs_opendir),
        readdir: Some(capture_vfs_readdir),
        dirent_get_name: Some(capture_vfs_dirent_get_name),
        dirent_is_dir: Some(capture_vfs_dirent_is_dir),
        closedir: Some(capture_vfs_closedir),
    };

    #[derive(Clone, Copy, Debug, Default)]
    struct CapturedHwRenderState {
        preferred_context_type: HwContextType,
        supports_non_preferred_context: bool,
        context_negotiation_support_version: Option<u32>,
        accept_contexts: [Option<HwContextType>; 4],
        accept_any_context: bool,
        attempts: [Option<HwContextType>; 4],
        attempt_count: usize,
        last_callback: Option<RawHwRenderCallback>,
        last_context_negotiation: Option<HwRenderContextNegotiationInterface>,
        inject_runtime_callbacks: bool,
    }

    impl CapturedHwRenderState {
        fn reset(&mut self) {
            *self = Self::default();
        }

        fn set_accept_contexts(&mut self, contexts: &[HwContextType]) {
            self.accept_contexts = [None; 4];
            for (slot, context) in self
                .accept_contexts
                .iter_mut()
                .zip(contexts.iter().copied())
            {
                *slot = Some(context);
            }
        }

        fn accepts(&self, context_type: HwContextType) -> bool {
            self.accept_any_context || self.accept_contexts.contains(&Some(context_type))
        }

        fn record_attempt(&mut self, context_type: HwContextType) {
            if let Some(slot) = self.attempts.get_mut(self.attempt_count) {
                *slot = Some(context_type);
            }
            self.attempt_count = self.attempt_count.saturating_add(1);
        }

        fn attempted_contexts(&self) -> Vec<HwContextType> {
            self.attempts.iter().flatten().copied().collect()
        }
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    struct LifecycleCallCounts {
        resets: usize,
        destroys: usize,
    }

    #[derive(Default)]
    struct LifecycleRecordingCore;

    impl Core for LifecycleRecordingCore {
        fn system_info(&self) -> SystemInfo {
            SystemInfo::new("test-core", "0.0.0")
        }

        fn av_info(&self) -> SystemAvInfo {
            SystemAvInfo::default()
        }

        fn run(&mut self, _runtime: &mut Runtime<'_>) {}

        fn hw_context_reset(&mut self, _runtime: &mut Runtime<'_>) {
            lifecycle_call_counts()
                .lock()
                .expect("lifecycle count mutex poisoned")
                .resets += 1;
        }

        fn hw_context_destroy(&mut self, _runtime: &mut Runtime<'_>) {
            lifecycle_call_counts()
                .lock()
                .expect("lifecycle count mutex poisoned")
                .destroys += 1;
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum PanicAt {
        SystemInfo,
        Init,
        LoadGame,
    }

    struct PanickingCore {
        panic_at: PanicAt,
    }

    impl PanickingCore {
        fn new(panic_at: PanicAt) -> Self {
            Self { panic_at }
        }
    }

    impl Core for PanickingCore {
        fn system_info(&self) -> SystemInfo {
            if self.panic_at == PanicAt::SystemInfo {
                panic!("intentional system info panic");
            }
            SystemInfo::new("panic-test-core", "0.0.0")
        }

        fn av_info(&self) -> SystemAvInfo {
            SystemAvInfo::default()
        }

        fn run(&mut self, _runtime: &mut Runtime<'_>) {}

        fn init(&mut self, _env: &mut Environment<'_>) {
            if self.panic_at == PanicAt::Init {
                panic!("intentional init panic");
            }
        }

        fn load_game(&mut self, _game: Option<GameInfo<'_>>, _runtime: &mut Runtime<'_>) -> bool {
            if self.panic_at == PanicAt::LoadGame {
                panic!("intentional load game panic");
            }
            true
        }
    }

    struct ChangingSystemInfoCore {
        calls: Arc<AtomicUsize>,
    }

    impl Core for ChangingSystemInfoCore {
        fn system_info(&self) -> SystemInfo {
            let call = self.calls.fetch_add(1, Ordering::SeqCst);
            let mut info = if call == 0 {
                SystemInfo::new("cached-test-core", "first")
            } else {
                SystemInfo::new("cached-test-core-mutated", "second")
            };
            info.valid_extensions = Some(if call == 0 {
                "first".to_string()
            } else {
                "second".to_string()
            });
            info
        }

        fn av_info(&self) -> SystemAvInfo {
            SystemAvInfo::default()
        }

        fn run(&mut self, _runtime: &mut Runtime<'_>) {}
    }

    struct RunPanicThenResetCore {
        reset_calls: Arc<AtomicUsize>,
    }

    impl Core for RunPanicThenResetCore {
        fn system_info(&self) -> SystemInfo {
            SystemInfo::new("panic-preserve-core", "0.0.0")
        }

        fn av_info(&self) -> SystemAvInfo {
            SystemAvInfo::default()
        }

        fn run(&mut self, _runtime: &mut Runtime<'_>) {
            panic!("intentional run panic");
        }

        fn reset(&mut self) {
            self.reset_calls.fetch_add(1, Ordering::SeqCst);
        }
    }

    struct MemoryRecordingCore {
        calls: Arc<Mutex<Vec<MemoryRegion>>>,
        save_ram: [u8; 4],
    }

    impl MemoryRecordingCore {
        fn new(calls: Arc<Mutex<Vec<MemoryRegion>>>) -> Self {
            Self {
                calls,
                save_ram: [1, 2, 3, 4],
            }
        }
    }

    struct ControllerDeviceRecordingCore {
        calls: Arc<Mutex<Vec<(InputPort, ControllerDevice)>>>,
    }

    struct CheatRecordingCore {
        calls: Arc<Mutex<Vec<(CheatIndex, bool, Option<String>)>>>,
    }

    struct KeyboardRecordingCore {
        calls: Arc<Mutex<Vec<KeyboardEvent>>>,
    }

    struct ConfiguredEventCore {
        keyboard_calls: Arc<Mutex<Vec<KeyboardEvent>>>,
    }

    struct MultiKeyboardListenerCore {
        calls: Arc<Mutex<Vec<&'static str>>>,
    }

    struct AudioBufferStatusRecordingCore {
        calls: Arc<Mutex<Vec<AudioBufferStatus>>>,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum AudioCallbackEvent {
        Request,
        State(AudioCallbackState),
    }

    struct AudioCallbackRecordingCore {
        calls: Arc<Mutex<Vec<AudioCallbackEvent>>>,
    }

    struct FrameTimeRecordingCore {
        calls: Arc<Mutex<Vec<FrameTime>>>,
    }

    struct FrameTimeReplacementCore {
        calls: Arc<Mutex<Vec<&'static str>>>,
    }

    struct FrameTimeClearedCore;

    struct ProcAddressRecordingCore {
        calls: Arc<Mutex<Vec<String>>>,
    }

    struct LocationRecordingCore {
        calls: Arc<Mutex<Vec<LocationLifecycleEvent>>>,
    }

    struct CameraRecordingCore {
        calls: Arc<Mutex<Vec<CameraEvent>>>,
    }

    struct DiskControlRecordingCore {
        calls: Arc<Mutex<Vec<DiskControlEvent>>>,
    }

    struct NetpacketRecordingCore {
        calls: Arc<Mutex<Vec<NetpacketEvent>>>,
    }

    struct CoreOptionsDisplayRecordingCore {
        calls: Arc<Mutex<Vec<&'static str>>>,
    }

    unsafe extern "C" fn test_extension_proc() {}

    impl Core for AudioCallbackRecordingCore {
        fn system_info(&self) -> SystemInfo {
            SystemInfo::new("audio-callback-test-core", "0.0.0")
        }

        fn av_info(&self) -> SystemAvInfo {
            SystemAvInfo::default()
        }

        fn run(&mut self, _runtime: &mut Runtime<'_>) {}

        fn configure_events(&mut self, events: &mut CoreEventConfig<Self>) {
            events
                .add_audio_callback_listener(Self::audio_callback)
                .add_audio_callback_state_changed_listener(Self::audio_callback_state_changed);
        }
    }

    impl AudioCallbackRecordingCore {
        fn audio_callback(&mut self) {
            self.calls
                .lock()
                .expect("audio callback calls mutex poisoned")
                .push(AudioCallbackEvent::Request);
        }

        fn audio_callback_state_changed(&mut self, state: AudioCallbackState) {
            self.calls
                .lock()
                .expect("audio callback calls mutex poisoned")
                .push(AudioCallbackEvent::State(state));
        }
    }

    impl Core for KeyboardRecordingCore {
        fn system_info(&self) -> SystemInfo {
            SystemInfo::new("keyboard-test-core", "0.0.0")
        }

        fn av_info(&self) -> SystemAvInfo {
            SystemAvInfo::default()
        }

        fn run(&mut self, _runtime: &mut Runtime<'_>) {}

        fn configure_events(&mut self, events: &mut CoreEventConfig<Self>) {
            events.add_keyboard_event_listener(Self::keyboard_event);
        }
    }

    impl KeyboardRecordingCore {
        fn keyboard_event(&mut self, event: KeyboardEvent) {
            self.calls
                .lock()
                .expect("keyboard event calls mutex poisoned")
                .push(event);
        }
    }

    impl Core for ConfiguredEventCore {
        fn system_info(&self) -> SystemInfo {
            SystemInfo::new("auto-event-test-core", "0.0.0")
        }

        fn av_info(&self) -> SystemAvInfo {
            SystemAvInfo::default()
        }

        fn run(&mut self, _runtime: &mut Runtime<'_>) {}

        fn configure_events(&mut self, events: &mut CoreEventConfig<Self>) {
            events
                .add_keyboard_event_listener(Self::keyboard_event)
                .add_audio_callback_listener(Self::audio_callback)
                .add_audio_callback_state_changed_listener(Self::audio_callback_state_changed)
                .add_audio_buffer_status_listener(Self::audio_buffer_status)
                .set_frame_time_callback(FrameTime::from_micros(16_667), Self::frame_time);
        }
    }

    impl ConfiguredEventCore {
        fn keyboard_event(&mut self, event: KeyboardEvent) {
            self.keyboard_calls
                .lock()
                .expect("keyboard event calls mutex poisoned")
                .push(event);
        }

        fn audio_callback(&mut self) {}

        fn audio_callback_state_changed(&mut self, _state: AudioCallbackState) {}

        fn audio_buffer_status(&mut self, _status: AudioBufferStatus) {}

        fn frame_time(&mut self, _time: FrameTime) {}
    }

    impl Core for MultiKeyboardListenerCore {
        fn system_info(&self) -> SystemInfo {
            SystemInfo::new("multi-keyboard-listener-test-core", "0.0.0")
        }

        fn av_info(&self) -> SystemAvInfo {
            SystemAvInfo::default()
        }

        fn run(&mut self, _runtime: &mut Runtime<'_>) {}

        fn configure_events(&mut self, events: &mut CoreEventConfig<Self>) {
            events
                .add_keyboard_event_listener(Self::first_keyboard_event)
                .add_keyboard_event_listener(Self::second_keyboard_event)
                .add_keyboard_event_listener(Self::first_keyboard_event)
                .remove_keyboard_event_listener(Self::second_keyboard_event)
                .add_keyboard_event_listener(Self::third_keyboard_event)
                .remove_keyboard_event_listener(Self::second_keyboard_event);
        }
    }

    impl MultiKeyboardListenerCore {
        fn first_keyboard_event(&mut self, _event: KeyboardEvent) {
            self.calls
                .lock()
                .expect("multi keyboard listener calls mutex poisoned")
                .push("first");
        }

        fn second_keyboard_event(&mut self, _event: KeyboardEvent) {
            self.calls
                .lock()
                .expect("multi keyboard listener calls mutex poisoned")
                .push("second");
        }

        fn third_keyboard_event(&mut self, _event: KeyboardEvent) {
            self.calls
                .lock()
                .expect("multi keyboard listener calls mutex poisoned")
                .push("third");
        }
    }

    impl Core for CoreOptionsDisplayRecordingCore {
        fn system_info(&self) -> SystemInfo {
            SystemInfo::new("core-options-display-test-core", "0.0.0")
        }

        fn av_info(&self) -> SystemAvInfo {
            SystemAvInfo::default()
        }

        fn run(&mut self, _runtime: &mut Runtime<'_>) {}

        fn core_options_update_display(&mut self, env: &mut Environment<'_>) -> bool {
            self.calls
                .lock()
                .expect("core options display calls mutex poisoned")
                .push("update");
            env.set_core_option_display(CoreOptionDisplay::new("demo_extra", false))
        }
    }

    impl Core for AudioBufferStatusRecordingCore {
        fn system_info(&self) -> SystemInfo {
            SystemInfo::new("audio-buffer-status-test-core", "0.0.0")
        }

        fn av_info(&self) -> SystemAvInfo {
            SystemAvInfo::default()
        }

        fn run(&mut self, _runtime: &mut Runtime<'_>) {}

        fn configure_events(&mut self, events: &mut CoreEventConfig<Self>) {
            events.add_audio_buffer_status_listener(Self::audio_buffer_status);
        }
    }

    impl AudioBufferStatusRecordingCore {
        fn audio_buffer_status(&mut self, status: AudioBufferStatus) {
            self.calls
                .lock()
                .expect("audio buffer status calls mutex poisoned")
                .push(status);
        }
    }

    impl Core for FrameTimeRecordingCore {
        fn system_info(&self) -> SystemInfo {
            SystemInfo::new("frame-time-test-core", "0.0.0")
        }

        fn av_info(&self) -> SystemAvInfo {
            SystemAvInfo::default()
        }

        fn run(&mut self, _runtime: &mut Runtime<'_>) {}

        fn configure_events(&mut self, events: &mut CoreEventConfig<Self>) {
            events.set_frame_time_callback(FrameTime::from_micros(16_667), Self::frame_time);
        }
    }

    impl FrameTimeRecordingCore {
        fn frame_time(&mut self, time: FrameTime) {
            self.calls
                .lock()
                .expect("frame time calls mutex poisoned")
                .push(time);
        }
    }

    impl Core for FrameTimeReplacementCore {
        fn system_info(&self) -> SystemInfo {
            SystemInfo::new("frame-time-replacement-test-core", "0.0.0")
        }

        fn av_info(&self) -> SystemAvInfo {
            SystemAvInfo::default()
        }

        fn run(&mut self, _runtime: &mut Runtime<'_>) {}

        fn configure_events(&mut self, events: &mut CoreEventConfig<Self>) {
            events
                .set_frame_time_callback(FrameTime::from_micros(1_000), Self::first_frame_time)
                .set_frame_time_callback(FrameTime::from_micros(2_000), Self::second_frame_time);
        }
    }

    impl FrameTimeReplacementCore {
        fn first_frame_time(&mut self, _time: FrameTime) {
            self.calls
                .lock()
                .expect("frame time replacement calls mutex poisoned")
                .push("first");
        }

        fn second_frame_time(&mut self, _time: FrameTime) {
            self.calls
                .lock()
                .expect("frame time replacement calls mutex poisoned")
                .push("second");
        }
    }

    impl Core for FrameTimeClearedCore {
        fn system_info(&self) -> SystemInfo {
            SystemInfo::new("frame-time-cleared-test-core", "0.0.0")
        }

        fn av_info(&self) -> SystemAvInfo {
            SystemAvInfo::default()
        }

        fn run(&mut self, _runtime: &mut Runtime<'_>) {}

        fn configure_events(&mut self, events: &mut CoreEventConfig<Self>) {
            events
                .set_frame_time_callback(FrameTime::from_micros(16_667), Self::frame_time)
                .clear_frame_time_callback();
        }
    }

    impl FrameTimeClearedCore {
        fn frame_time(&mut self, _time: FrameTime) {}
    }

    impl Core for ProcAddressRecordingCore {
        fn system_info(&self) -> SystemInfo {
            SystemInfo::new("proc-address-test-core", "0.0.0")
        }

        fn av_info(&self) -> SystemAvInfo {
            SystemAvInfo::default()
        }

        fn run(&mut self, _runtime: &mut Runtime<'_>) {}

        fn proc_address(&mut self, symbol: &CStr) -> Option<CoreProcAddress> {
            let symbol = symbol.to_string_lossy().into_owned();
            self.calls
                .lock()
                .expect("proc address calls mutex poisoned")
                .push(symbol.clone());
            (symbol == "test_extension_proc")
                .then_some(CoreProcAddress::from_fn(test_extension_proc))
        }
    }

    impl Core for LocationRecordingCore {
        fn system_info(&self) -> SystemInfo {
            SystemInfo::new("location-test-core", "0.0.0")
        }

        fn av_info(&self) -> SystemAvInfo {
            SystemAvInfo::default()
        }

        fn run(&mut self, _runtime: &mut Runtime<'_>) {}

        fn configure_events(&mut self, events: &mut CoreEventConfig<Self>) {
            events
                .add_location_initialized_listener(Self::location_initialized)
                .add_location_deinitialized_listener(Self::location_deinitialized);
        }
    }

    impl LocationRecordingCore {
        fn location_initialized(&mut self) {
            self.calls
                .lock()
                .expect("location lifecycle calls mutex poisoned")
                .push(LocationLifecycleEvent::Initialized);
        }

        fn location_deinitialized(&mut self) {
            self.calls
                .lock()
                .expect("location lifecycle calls mutex poisoned")
                .push(LocationLifecycleEvent::Deinitialized);
        }
    }

    impl Core for CameraRecordingCore {
        fn system_info(&self) -> SystemInfo {
            SystemInfo::new("camera-test-core", "0.0.0")
        }

        fn av_info(&self) -> SystemAvInfo {
            SystemAvInfo::default()
        }

        fn run(&mut self, _runtime: &mut Runtime<'_>) {}

        fn configure_events(&mut self, events: &mut CoreEventConfig<Self>) {
            events
                .add_camera_initialized_listener(Self::camera_initialized)
                .add_camera_deinitialized_listener(Self::camera_deinitialized)
                .add_camera_raw_frame_listener(Self::camera_raw_frame)
                .add_camera_texture_frame_listener(Self::camera_texture_frame);
        }
    }

    impl CameraRecordingCore {
        fn camera_initialized(&mut self) {
            self.calls
                .lock()
                .expect("camera calls mutex poisoned")
                .push(CameraEvent::Initialized);
        }

        fn camera_deinitialized(&mut self) {
            self.calls
                .lock()
                .expect("camera calls mutex poisoned")
                .push(CameraEvent::Deinitialized);
        }

        fn camera_raw_frame(&mut self, frame: CameraRawFrame<'_>) {
            self.calls
                .lock()
                .expect("camera calls mutex poisoned")
                .push(CameraEvent::Raw {
                    width: frame.width,
                    height: frame.height,
                    pitch: frame.pitch_bytes,
                    pixels: frame.pixels.to_vec(),
                });
        }

        fn camera_texture_frame(&mut self, frame: CameraTextureFrame) {
            self.calls
                .lock()
                .expect("camera calls mutex poisoned")
                .push(CameraEvent::Texture {
                    texture_id: frame.texture_id.get(),
                    texture_target: frame.texture_target.get(),
                    affine: frame.affine,
                });
        }
    }

    impl Core for DiskControlRecordingCore {
        fn system_info(&self) -> SystemInfo {
            SystemInfo::new("disk-test-core", "0.0.0")
        }

        fn av_info(&self) -> SystemAvInfo {
            SystemAvInfo::default()
        }

        fn run(&mut self, _runtime: &mut Runtime<'_>) {}

        fn disk_set_tray_state(&mut self, state: DiskTrayState) -> bool {
            self.calls
                .lock()
                .expect("disk control calls mutex poisoned")
                .push(DiskControlEvent::SetTray(state));
            true
        }

        fn disk_tray_state(&mut self) -> DiskTrayState {
            DiskTrayState::Ejected
        }

        fn disk_image_index(&mut self) -> DiskIndex {
            DiskIndex::new(2)
        }

        fn disk_set_image_index(&mut self, index: DiskIndex) -> bool {
            self.calls
                .lock()
                .expect("disk control calls mutex poisoned")
                .push(DiskControlEvent::SetImage(index));
            true
        }

        fn disk_image_count(&mut self) -> u32 {
            4
        }

        fn disk_replace_image_index(
            &mut self,
            index: DiskIndex,
            game: Option<GameInfo<'_>>,
        ) -> bool {
            self.calls
                .lock()
                .expect("disk control calls mutex poisoned")
                .push(DiskControlEvent::ReplaceImage(index, game.is_some()));
            true
        }

        fn disk_add_image_index(&mut self) -> bool {
            self.calls
                .lock()
                .expect("disk control calls mutex poisoned")
                .push(DiskControlEvent::AddImage);
            true
        }

        fn disk_set_initial_image(&mut self, index: DiskIndex, path: &CStr) -> bool {
            self.calls
                .lock()
                .expect("disk control calls mutex poisoned")
                .push(DiskControlEvent::SetInitialImage(
                    index,
                    path.to_string_lossy().into_owned(),
                ));
            true
        }

        fn disk_image_path(&mut self, index: DiskIndex) -> Option<String> {
            (index == DiskIndex::new(2)).then(|| "/games/disc\0two.cue".to_string())
        }

        fn disk_image_label(&mut self, index: DiskIndex) -> Option<String> {
            (index == DiskIndex::new(2)).then(|| "Disc Two".to_string())
        }
    }

    impl Core for NetpacketRecordingCore {
        fn system_info(&self) -> SystemInfo {
            SystemInfo::new("netpacket-test-core", "0.0.0")
        }

        fn av_info(&self) -> SystemAvInfo {
            SystemAvInfo::default()
        }

        fn run(&mut self, _runtime: &mut Runtime<'_>) {}

        fn netpacket_start(&mut self, session: NetpacketSession) {
            self.calls
                .lock()
                .expect("netpacket calls mutex poisoned")
                .push(NetpacketEvent::Start(
                    session.client_id(),
                    session.can_poll_receive(),
                ));
            session.send(
                NetpacketTarget::Broadcast,
                NetpacketFlags::reliable(),
                b"hello",
            );
            session.flush(NetpacketTarget::Client(session.client_id()));
            assert!(session.poll_receive());
        }

        fn netpacket_receive(&mut self, packet: Netpacket<'_>) {
            self.calls
                .lock()
                .expect("netpacket calls mutex poisoned")
                .push(NetpacketEvent::Receive(
                    packet.client_id,
                    packet.data.to_vec(),
                ));
        }

        fn netpacket_stop(&mut self) {
            self.calls
                .lock()
                .expect("netpacket calls mutex poisoned")
                .push(NetpacketEvent::Stop);
        }

        fn netpacket_poll(&mut self) {
            self.calls
                .lock()
                .expect("netpacket calls mutex poisoned")
                .push(NetpacketEvent::Poll);
        }

        fn netpacket_connected(&mut self, client_id: NetplayClientId) -> bool {
            self.calls
                .lock()
                .expect("netpacket calls mutex poisoned")
                .push(NetpacketEvent::Connected(client_id));
            client_id != NetplayClientId::new(9)
        }

        fn netpacket_disconnected(&mut self, client_id: NetplayClientId) {
            self.calls
                .lock()
                .expect("netpacket calls mutex poisoned")
                .push(NetpacketEvent::Disconnected(client_id));
        }
    }

    impl Core for ControllerDeviceRecordingCore {
        fn system_info(&self) -> SystemInfo {
            SystemInfo::new("controller-device-test-core", "0.0.0")
        }

        fn av_info(&self) -> SystemAvInfo {
            SystemAvInfo::default()
        }

        fn run(&mut self, _runtime: &mut Runtime<'_>) {}

        fn set_controller_port_device(&mut self, port: InputPort, device: ControllerDevice) {
            self.calls
                .lock()
                .expect("controller device calls mutex poisoned")
                .push((port, device));
        }
    }

    impl Core for CheatRecordingCore {
        fn system_info(&self) -> SystemInfo {
            SystemInfo::new("cheat-test-core", "0.0.0")
        }

        fn av_info(&self) -> SystemAvInfo {
            SystemAvInfo::default()
        }

        fn run(&mut self, _runtime: &mut Runtime<'_>) {}

        fn cheat_set(&mut self, index: CheatIndex, enabled: bool, code: Option<CheatCode<'_>>) {
            self.calls
                .lock()
                .expect("cheat calls mutex poisoned")
                .push((
                    index,
                    enabled,
                    code.map(|code| code.to_string_lossy().into_owned()),
                ));
        }
    }

    impl Core for MemoryRecordingCore {
        fn system_info(&self) -> SystemInfo {
            SystemInfo::new("memory-test-core", "0.0.0")
        }

        fn av_info(&self) -> SystemAvInfo {
            SystemAvInfo::default()
        }

        fn run(&mut self, _runtime: &mut Runtime<'_>) {}

        fn memory_region(&mut self, region: MemoryRegion) -> Option<CoreMemory<'_>> {
            self.calls
                .lock()
                .expect("memory calls mutex poisoned")
                .push(region);
            match region {
                MemoryRegion::SaveRam => Some(CoreMemory::read_write(&mut self.save_ram)),
                _ => None,
            }
        }
    }

    fn captured_hw_render_state() -> &'static Mutex<CapturedHwRenderState> {
        CAPTURED_HW_RENDER_STATE.get_or_init(|| Mutex::new(CapturedHwRenderState::default()))
    }

    fn lifecycle_call_counts() -> &'static Mutex<LifecycleCallCounts> {
        CAPTURED_LIFECYCLE_COUNTS.get_or_init(|| Mutex::new(LifecycleCallCounts::default()))
    }

    fn captured_geometries() -> &'static Mutex<Vec<GameGeometry>> {
        CAPTURED_GEOMETRIES.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_messages() -> &'static Mutex<Vec<CapturedMessage>> {
        CAPTURED_MESSAGES.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_extended_messages() -> &'static Mutex<Vec<CapturedExtendedMessage>> {
        CAPTURED_EXTENDED_MESSAGES.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_video_refreshes() -> &'static Mutex<Vec<CapturedVideoRefresh>> {
        CAPTURED_VIDEO_REFRESHES.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_input_queries() -> &'static Mutex<Vec<CapturedInputQuery>> {
        CAPTURED_INPUT_QUERIES.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_input_descriptors() -> &'static Mutex<Vec<CapturedInputDescriptor>> {
        CAPTURED_INPUT_DESCRIPTORS.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_controller_info() -> &'static Mutex<Vec<Vec<CapturedControllerDescription>>> {
        CAPTURED_CONTROLLER_INFO.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_core_options_version() -> &'static Mutex<Option<u32>> {
        CAPTURED_CORE_OPTIONS_VERSION.get_or_init(|| Mutex::new(Some(2)))
    }

    fn captured_core_options_v2() -> &'static Mutex<Option<CapturedCoreOptionsV2>> {
        CAPTURED_CORE_OPTIONS_V2.get_or_init(|| Mutex::new(None))
    }

    fn captured_core_options_v1() -> &'static Mutex<Vec<CapturedCoreOptionDefinition>> {
        CAPTURED_CORE_OPTIONS_V1.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_core_option_displays() -> &'static Mutex<Vec<CapturedCoreOptionDisplay>> {
        CAPTURED_CORE_OPTION_DISPLAYS.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_variables() -> &'static Mutex<Vec<CapturedVariable>> {
        CAPTURED_VARIABLES.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_core_options_update_display_callback()
    -> &'static Mutex<Option<raw::retro_core_options_update_display_callback>> {
        CAPTURED_CORE_OPTIONS_UPDATE_DISPLAY_CALLBACK.get_or_init(|| Mutex::new(None))
    }

    fn captured_vfs_interface_requests() -> &'static Mutex<Vec<u32>> {
        CAPTURED_VFS_INTERFACE_REQUESTS.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_vfs_opens() -> &'static Mutex<Vec<CapturedVfsOpen>> {
        CAPTURED_VFS_OPENS.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_vfs_closes() -> &'static Mutex<u32> {
        CAPTURED_VFS_CLOSES.get_or_init(|| Mutex::new(0))
    }

    fn captured_vfs_dir_closes() -> &'static Mutex<u32> {
        CAPTURED_VFS_DIR_CLOSES.get_or_init(|| Mutex::new(0))
    }

    fn captured_vfs_writes() -> &'static Mutex<Vec<Vec<u8>>> {
        CAPTURED_VFS_WRITES.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_vfs_removes() -> &'static Mutex<Vec<String>> {
        CAPTURED_VFS_REMOVES.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_vfs_renames() -> &'static Mutex<Vec<CapturedVfsRename>> {
        CAPTURED_VFS_RENAMES.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_vfs_mkdirs() -> &'static Mutex<Vec<String>> {
        CAPTURED_VFS_MKDIRS.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_vfs_readdirs() -> &'static Mutex<u32> {
        CAPTURED_VFS_READDIRS.get_or_init(|| Mutex::new(0))
    }

    fn captured_memory_descriptors() -> &'static Mutex<Vec<CapturedMemoryDescriptor>> {
        CAPTURED_MEMORY_DESCRIPTORS.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_subsystem_info() -> &'static Mutex<Vec<CapturedSubsystem>> {
        CAPTURED_SUBSYSTEM_INFO.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn software_framebuffer_pixels() -> &'static Mutex<Vec<u32>> {
        SOFTWARE_FRAMEBUFFER_PIXELS.get_or_init(|| Mutex::new(vec![0; 4 * 2]))
    }

    fn extended_game_info_ptr() -> *const raw::retro_game_info_ext {
        *EXTENDED_GAME_INFO_PTR.get_or_init(|| {
            Box::leak(Box::new([
                raw::retro_game_info_ext {
                    full_path: c"/games/test.sfc".as_ptr(),
                    archive_path: ptr::null(),
                    archive_file: ptr::null(),
                    dir: c"/games".as_ptr(),
                    name: c"test".as_ptr(),
                    ext: c"sfc".as_ptr(),
                    meta: c"plain".as_ptr(),
                    data: EXTENDED_GAME_CONTENT.as_ptr().cast::<c_void>(),
                    size: EXTENDED_GAME_CONTENT.len(),
                    file_in_archive: false,
                    persistent_data: true,
                },
                raw::retro_game_info_ext {
                    full_path: ptr::null(),
                    archive_path: c"/games/archive.zip".as_ptr(),
                    archive_file: c"inside.bin".as_ptr(),
                    dir: c"/games".as_ptr(),
                    name: c"archive".as_ptr(),
                    ext: c"bin".as_ptr(),
                    meta: ptr::null(),
                    data: ptr::null(),
                    size: 0,
                    file_in_archive: true,
                    persistent_data: false,
                },
            ])) as *const [raw::retro_game_info_ext; 2] as usize
        }) as *const raw::retro_game_info_ext
    }

    fn captured_led_states() -> &'static Mutex<Vec<(i32, i32)>> {
        CAPTURED_LED_STATES.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_rumble_states() -> &'static Mutex<Vec<(u32, raw::retro_rumble_effect, u16)>> {
        CAPTURED_RUMBLE_STATES.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_sensor_states() -> &'static Mutex<Vec<(u32, raw::retro_sensor_action, u32)>> {
        CAPTURED_SENSOR_STATES.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_location_intervals() -> &'static Mutex<Vec<(u32, u32)>> {
        CAPTURED_LOCATION_INTERVALS.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_location_starts() -> &'static Mutex<u32> {
        CAPTURED_LOCATION_STARTS.get_or_init(|| Mutex::new(0))
    }

    fn captured_location_stops() -> &'static Mutex<u32> {
        CAPTURED_LOCATION_STOPS.get_or_init(|| Mutex::new(0))
    }

    fn captured_location_callback() -> &'static Mutex<Option<raw::retro_location_callback>> {
        CAPTURED_LOCATION_CALLBACK.get_or_init(|| Mutex::new(None))
    }

    fn captured_camera_callback() -> &'static Mutex<Option<raw::retro_camera_callback>> {
        CAPTURED_CAMERA_CALLBACK.get_or_init(|| Mutex::new(None))
    }

    fn captured_camera_starts() -> &'static Mutex<u32> {
        CAPTURED_CAMERA_STARTS.get_or_init(|| Mutex::new(0))
    }

    fn captured_camera_stops() -> &'static Mutex<u32> {
        CAPTURED_CAMERA_STOPS.get_or_init(|| Mutex::new(0))
    }

    fn captured_disk_control_callback() -> &'static Mutex<Option<raw::retro_disk_control_callback>>
    {
        CAPTURED_DISK_CONTROL_CALLBACK.get_or_init(|| Mutex::new(None))
    }

    fn captured_disk_control_ext_callback()
    -> &'static Mutex<Option<raw::retro_disk_control_ext_callback>> {
        CAPTURED_DISK_CONTROL_EXT_CALLBACK.get_or_init(|| Mutex::new(None))
    }

    fn captured_netpacket_callback() -> &'static Mutex<Option<CapturedNetpacketCallback>> {
        CAPTURED_NETPACKET_CALLBACK.get_or_init(|| Mutex::new(None))
    }

    fn captured_netpacket_sends() -> &'static Mutex<Vec<CapturedNetpacketSend>> {
        CAPTURED_NETPACKET_SENDS.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_netpacket_polls() -> &'static Mutex<u32> {
        CAPTURED_NETPACKET_POLLS.get_or_init(|| Mutex::new(0))
    }

    fn captured_mic_open_params() -> &'static Mutex<Vec<Option<u32>>> {
        CAPTURED_MIC_OPEN_PARAMS.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_mic_states() -> &'static Mutex<Vec<bool>> {
        CAPTURED_MIC_STATES.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_mic_closes() -> &'static Mutex<u32> {
        CAPTURED_MIC_CLOSES.get_or_init(|| Mutex::new(0))
    }

    fn captured_midi_writes() -> &'static Mutex<Vec<(u8, u32)>> {
        CAPTURED_MIDI_WRITES.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_midi_flushes() -> &'static Mutex<u32> {
        CAPTURED_MIDI_FLUSHES.get_or_init(|| Mutex::new(0))
    }

    fn captured_midi_probes() -> &'static Mutex<u32> {
        CAPTURED_MIDI_PROBES.get_or_init(|| Mutex::new(0))
    }

    fn captured_keyboard_callback() -> &'static Mutex<Option<RawKeyboardCallback>> {
        CAPTURED_KEYBOARD_CALLBACK.get_or_init(|| Mutex::new(None))
    }

    fn captured_audio_latencies() -> &'static Mutex<Vec<Option<u32>>> {
        CAPTURED_AUDIO_LATENCIES.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_audio_buffer_status_callback()
    -> &'static Mutex<Option<RawAudioBufferStatusCallback>> {
        CAPTURED_AUDIO_BUFFER_STATUS_CALLBACK.get_or_init(|| Mutex::new(None))
    }

    fn captured_audio_callback() -> &'static Mutex<Option<RawAudioCallback>> {
        CAPTURED_AUDIO_CALLBACK.get_or_init(|| Mutex::new(None))
    }

    fn captured_audio_callback_probes() -> &'static Mutex<u32> {
        CAPTURED_AUDIO_CALLBACK_PROBES.get_or_init(|| Mutex::new(0))
    }

    fn captured_frame_time_callback() -> &'static Mutex<Option<RawFrameTimeCallback>> {
        CAPTURED_FRAME_TIME_CALLBACK.get_or_init(|| Mutex::new(None))
    }

    fn captured_proc_address_interface()
    -> &'static Mutex<Option<raw::retro_get_proc_address_interface>> {
        CAPTURED_PROC_ADDRESS_INTERFACE.get_or_init(|| Mutex::new(None))
    }

    fn captured_fastforwarding_overrides()
    -> &'static Mutex<Vec<Option<raw::retro_fastforwarding_override>>> {
        CAPTURED_FASTFORWARDING_OVERRIDES.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_achievement_support() -> &'static Mutex<Vec<bool>> {
        CAPTURED_ACHIEVEMENT_SUPPORT.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_performance_levels() -> &'static Mutex<Vec<u32>> {
        CAPTURED_PERFORMANCE_LEVELS.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_perf_logs() -> &'static Mutex<u32> {
        CAPTURED_PERF_LOGS.get_or_init(|| Mutex::new(0))
    }

    fn captured_perf_registered_idents() -> &'static Mutex<Vec<String>> {
        CAPTURED_PERF_REGISTERED_IDENTS.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_rotations() -> &'static Mutex<Vec<u32>> {
        CAPTURED_ROTATIONS.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_system_av_infos() -> &'static Mutex<Vec<SystemAvInfo>> {
        CAPTURED_SYSTEM_AV_INFOS.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_shutdowns() -> &'static Mutex<u32> {
        CAPTURED_SHUTDOWNS.get_or_init(|| Mutex::new(0))
    }

    fn captured_hw_shared_contexts() -> &'static Mutex<u32> {
        CAPTURED_HW_SHARED_CONTEXTS.get_or_init(|| Mutex::new(0))
    }

    fn captured_serialization_quirks() -> &'static Mutex<Vec<u64>> {
        CAPTURED_SERIALIZATION_QUIRKS.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn captured_support_no_game() -> &'static Mutex<Vec<bool>> {
        CAPTURED_SUPPORT_NO_GAME.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn reset_captured_messages() {
        captured_messages()
            .lock()
            .expect("message capture mutex poisoned")
            .clear();
    }

    fn reset_captured_extended_messages() {
        captured_extended_messages()
            .lock()
            .expect("extended message capture mutex poisoned")
            .clear();
    }

    fn reset_captured_video_refreshes() {
        captured_video_refreshes()
            .lock()
            .expect("video refresh capture mutex poisoned")
            .clear();
    }

    fn reset_captured_input_queries() {
        captured_input_queries()
            .lock()
            .expect("input query capture mutex poisoned")
            .clear();
    }

    fn reset_captured_input_descriptors() {
        captured_input_descriptors()
            .lock()
            .expect("input descriptor capture mutex poisoned")
            .clear();
    }

    fn reset_captured_controller_info() {
        captured_controller_info()
            .lock()
            .expect("controller info capture mutex poisoned")
            .clear();
    }

    fn reset_captured_core_options() {
        *captured_core_options_version()
            .lock()
            .expect("core options version capture mutex poisoned") = Some(2);
        *captured_core_options_v2()
            .lock()
            .expect("core options v2 capture mutex poisoned") = None;
        captured_core_options_v1()
            .lock()
            .expect("core options v1 capture mutex poisoned")
            .clear();
        captured_core_option_displays()
            .lock()
            .expect("core option display capture mutex poisoned")
            .clear();
        captured_variables()
            .lock()
            .expect("variable capture mutex poisoned")
            .clear();
        *captured_core_options_update_display_callback()
            .lock()
            .expect("core options update display callback capture mutex poisoned") = None;
    }

    fn reset_captured_vfs_interface() {
        captured_vfs_interface_requests()
            .lock()
            .expect("VFS request capture mutex poisoned")
            .clear();
        captured_vfs_opens()
            .lock()
            .expect("VFS open capture mutex poisoned")
            .clear();
        *captured_vfs_closes()
            .lock()
            .expect("VFS close capture mutex poisoned") = 0;
        *captured_vfs_dir_closes()
            .lock()
            .expect("VFS dir close capture mutex poisoned") = 0;
        captured_vfs_writes()
            .lock()
            .expect("VFS write capture mutex poisoned")
            .clear();
        captured_vfs_removes()
            .lock()
            .expect("VFS remove capture mutex poisoned")
            .clear();
        captured_vfs_renames()
            .lock()
            .expect("VFS rename capture mutex poisoned")
            .clear();
        captured_vfs_mkdirs()
            .lock()
            .expect("VFS mkdir capture mutex poisoned")
            .clear();
        *captured_vfs_readdirs()
            .lock()
            .expect("VFS readdir capture mutex poisoned") = 0;
    }

    fn reset_captured_memory_descriptors() {
        captured_memory_descriptors()
            .lock()
            .expect("memory descriptor capture mutex poisoned")
            .clear();
    }

    fn reset_captured_subsystem_info() {
        captured_subsystem_info()
            .lock()
            .expect("subsystem info capture mutex poisoned")
            .clear();
    }

    fn reset_captured_fastforwarding_overrides() {
        captured_fastforwarding_overrides()
            .lock()
            .expect("fastforwarding override capture mutex poisoned")
            .clear();
    }

    fn reset_captured_proc_address_interface() {
        *captured_proc_address_interface()
            .lock()
            .expect("proc address interface capture mutex poisoned") = None;
    }

    fn reset_software_framebuffer_pixels() {
        software_framebuffer_pixels()
            .lock()
            .expect("software framebuffer pixels mutex poisoned")
            .fill(0);
    }

    fn reset_captured_led_states() {
        captured_led_states()
            .lock()
            .expect("LED state capture mutex poisoned")
            .clear();
    }

    fn reset_captured_rumble_states() {
        captured_rumble_states()
            .lock()
            .expect("rumble state capture mutex poisoned")
            .clear();
    }

    fn reset_captured_sensor_states() {
        captured_sensor_states()
            .lock()
            .expect("sensor state capture mutex poisoned")
            .clear();
    }

    fn reset_captured_location_interface() {
        captured_location_intervals()
            .lock()
            .expect("location interval capture mutex poisoned")
            .clear();
        *captured_location_starts()
            .lock()
            .expect("location start capture mutex poisoned") = 0;
        *captured_location_stops()
            .lock()
            .expect("location stop capture mutex poisoned") = 0;
        *captured_location_callback()
            .lock()
            .expect("location callback capture mutex poisoned") = None;
    }

    fn reset_captured_camera_interface() {
        *captured_camera_callback()
            .lock()
            .expect("camera callback capture mutex poisoned") = None;
        *captured_camera_starts()
            .lock()
            .expect("camera start capture mutex poisoned") = 0;
        *captured_camera_stops()
            .lock()
            .expect("camera stop capture mutex poisoned") = 0;
    }

    fn reset_captured_disk_control_callbacks() {
        *captured_disk_control_callback()
            .lock()
            .expect("disk control callback capture mutex poisoned") = None;
        *captured_disk_control_ext_callback()
            .lock()
            .expect("disk control ext callback capture mutex poisoned") = None;
    }

    fn reset_captured_netpacket_interface() {
        *captured_netpacket_callback()
            .lock()
            .expect("netpacket callback capture mutex poisoned") = None;
        captured_netpacket_sends()
            .lock()
            .expect("netpacket sends mutex poisoned")
            .clear();
        *captured_netpacket_polls()
            .lock()
            .expect("netpacket polls mutex poisoned") = 0;
    }

    fn reset_captured_microphone_interface() {
        captured_mic_open_params()
            .lock()
            .expect("microphone open params mutex poisoned")
            .clear();
        captured_mic_states()
            .lock()
            .expect("microphone states mutex poisoned")
            .clear();
        *captured_mic_closes()
            .lock()
            .expect("microphone closes mutex poisoned") = 0;
    }

    fn reset_captured_midi_interface() {
        captured_midi_writes()
            .lock()
            .expect("MIDI write capture mutex poisoned")
            .clear();
        *captured_midi_flushes()
            .lock()
            .expect("MIDI flush capture mutex poisoned") = 0;
        *captured_midi_probes()
            .lock()
            .expect("MIDI probe capture mutex poisoned") = 0;
    }

    fn reset_captured_keyboard_callback() {
        *captured_keyboard_callback()
            .lock()
            .expect("keyboard callback capture mutex poisoned") = None;
    }

    fn reset_captured_audio_latencies() {
        captured_audio_latencies()
            .lock()
            .expect("audio latency capture mutex poisoned")
            .clear();
    }

    fn reset_captured_audio_buffer_status_callback() {
        *captured_audio_buffer_status_callback()
            .lock()
            .expect("audio buffer status callback capture mutex poisoned") = None;
    }

    fn reset_captured_audio_callback() {
        *captured_audio_callback()
            .lock()
            .expect("audio callback capture mutex poisoned") = None;
        *captured_audio_callback_probes()
            .lock()
            .expect("audio callback probe mutex poisoned") = 0;
    }

    fn reset_captured_frame_time_callback() {
        *captured_frame_time_callback()
            .lock()
            .expect("frame time callback capture mutex poisoned") = None;
    }

    fn reset_captured_achievement_support() {
        captured_achievement_support()
            .lock()
            .expect("achievement support capture mutex poisoned")
            .clear();
    }

    fn reset_captured_performance_levels() {
        captured_performance_levels()
            .lock()
            .expect("performance level capture mutex poisoned")
            .clear();
    }

    fn reset_captured_perf_interface() {
        *captured_perf_logs()
            .lock()
            .expect("perf log capture mutex poisoned") = 0;
        captured_perf_registered_idents()
            .lock()
            .expect("perf registered idents mutex poisoned")
            .clear();
    }

    fn reset_captured_rotations() {
        captured_rotations()
            .lock()
            .expect("rotation capture mutex poisoned")
            .clear();
    }

    fn reset_captured_system_av_infos() {
        captured_system_av_infos()
            .lock()
            .expect("system av info capture mutex poisoned")
            .clear();
    }

    fn reset_captured_shutdowns() {
        *captured_shutdowns()
            .lock()
            .expect("shutdown capture mutex poisoned") = 0;
    }

    fn reset_captured_hw_shared_contexts() {
        *captured_hw_shared_contexts()
            .lock()
            .expect("HW shared context capture mutex poisoned") = 0;
    }

    fn reset_captured_serialization_quirks() {
        captured_serialization_quirks()
            .lock()
            .expect("serialization quirk capture mutex poisoned")
            .clear();
    }

    fn reset_captured_support_no_game() {
        captured_support_no_game()
            .lock()
            .expect("support-no-game capture mutex poisoned")
            .clear();
    }

    fn reset_captured_geometries() {
        captured_geometries()
            .lock()
            .expect("geometry capture mutex poisoned")
            .clear();
    }

    fn reset_lifecycle_call_counts() {
        *lifecycle_call_counts()
            .lock()
            .expect("lifecycle count mutex poisoned") = LifecycleCallCounts::default();
    }

    fn snapshot_lifecycle_call_counts() -> LifecycleCallCounts {
        *lifecycle_call_counts()
            .lock()
            .expect("lifecycle count mutex poisoned")
    }

    fn serial_test_guard() -> MutexGuard<'static, ()> {
        TEST_SERIAL_GUARD
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("test serial guard mutex poisoned")
    }

    fn install_global_test_core(core: impl Core) {
        with_state(|state| {
            state.reset_frontend_state();
            let bundle = create_core(core);
            state.event_handlers = bundle.event_handlers;
            state.core = Some(bundle.core);
        });
    }

    fn clear_global_test_core() {
        with_state(|state| {
            state.reset_frontend_state();
            state.core = None;
        });
    }

    unsafe extern "C" fn capture_input_state(port: u32, device: u32, index: u32, id: u32) -> i16 {
        captured_input_queries()
            .lock()
            .expect("input query capture mutex poisoned")
            .push(CapturedInputQuery {
                port,
                device,
                index,
                id,
            });

        match (device, index, id) {
            (RETRO_DEVICE_JOYPAD, 0, RETRO_DEVICE_ID_JOYPAD_MASK) => {
                ((1u16 << RETRO_DEVICE_ID_JOYPAD_A) | (1u16 << RETRO_DEVICE_ID_JOYPAD_B)) as i16
            }
            (RETRO_DEVICE_JOYPAD, 0, RETRO_DEVICE_ID_JOYPAD_A) => 1,
            (RETRO_DEVICE_ANALOG, RETRO_DEVICE_INDEX_ANALOG_LEFT, RETRO_DEVICE_ID_ANALOG_X) => -123,
            (RETRO_DEVICE_ANALOG, RETRO_DEVICE_INDEX_ANALOG_BUTTON, RETRO_DEVICE_ID_JOYPAD_R2) => {
                123
            }
            (RETRO_DEVICE_MOUSE, 0, RETRO_DEVICE_ID_MOUSE_X) => 7,
            (RETRO_DEVICE_MOUSE, 0, RETRO_DEVICE_ID_MOUSE_LEFT) => 1,
            (RETRO_DEVICE_MOUSE, 0, RETRO_DEVICE_ID_MOUSE_WHEELUP) => 1,
            (RETRO_DEVICE_POINTER, 1, RETRO_DEVICE_ID_POINTER_Y) => -77,
            (RETRO_DEVICE_POINTER, 1, RETRO_DEVICE_ID_POINTER_PRESSED) => 1,
            (RETRO_DEVICE_POINTER, 0, raw::RETRO_DEVICE_ID_POINTER_COUNT) => 2,
            (RETRO_DEVICE_POINTER, 1, raw::RETRO_DEVICE_ID_POINTER_IS_OFFSCREEN) => 1,
            (RETRO_DEVICE_LIGHTGUN, 0, RETRO_DEVICE_ID_LIGHTGUN_SCREEN_X) => 99,
            (RETRO_DEVICE_LIGHTGUN, 0, RETRO_DEVICE_ID_LIGHTGUN_TRIGGER) => 1,
            (RETRO_DEVICE_LIGHTGUN, 0, RETRO_DEVICE_ID_LIGHTGUN_IS_OFFSCREEN) => 1,
            _ => 0,
        }
    }

    unsafe extern "C" fn capture_led_state(led: i32, state: i32) {
        captured_led_states()
            .lock()
            .expect("LED state capture mutex poisoned")
            .push((led, state));
    }

    unsafe extern "C" fn capture_rumble_state(
        port: u32,
        effect: raw::retro_rumble_effect,
        strength: u16,
    ) -> bool {
        captured_rumble_states()
            .lock()
            .expect("rumble state capture mutex poisoned")
            .push((port, effect, strength));
        strength != 0
    }

    unsafe extern "C" fn capture_sensor_state(
        port: u32,
        action: raw::retro_sensor_action,
        rate: u32,
    ) -> bool {
        captured_sensor_states()
            .lock()
            .expect("sensor state capture mutex poisoned")
            .push((port, action, rate));
        action != raw::retro_sensor_action::IlluminanceEnable
    }

    unsafe extern "C" fn capture_sensor_input(port: u32, id: u32) -> f32 {
        port as f32 + id as f32 / 10.0
    }

    unsafe extern "C" fn capture_location_start() -> bool {
        *captured_location_starts()
            .lock()
            .expect("location start capture mutex poisoned") += 1;
        true
    }

    unsafe extern "C" fn capture_location_stop() {
        *captured_location_stops()
            .lock()
            .expect("location stop capture mutex poisoned") += 1;
    }

    unsafe extern "C" fn capture_location_get_position(
        lat: *mut f64,
        lon: *mut f64,
        horiz_accuracy: *mut f64,
        vert_accuracy: *mut f64,
    ) -> bool {
        let Some(lat) = (unsafe { lat.as_mut() }) else {
            return false;
        };
        let Some(lon) = (unsafe { lon.as_mut() }) else {
            return false;
        };
        let Some(horiz_accuracy) = (unsafe { horiz_accuracy.as_mut() }) else {
            return false;
        };
        let Some(vert_accuracy) = (unsafe { vert_accuracy.as_mut() }) else {
            return false;
        };
        *lat = 12.5;
        *lon = -45.25;
        *horiz_accuracy = 3.0;
        *vert_accuracy = 8.0;
        true
    }

    unsafe extern "C" fn capture_location_set_interval(interval_ms: u32, interval_distance: u32) {
        captured_location_intervals()
            .lock()
            .expect("location interval capture mutex poisoned")
            .push((interval_ms, interval_distance));
    }

    unsafe extern "C" fn capture_camera_start() -> bool {
        *captured_camera_starts()
            .lock()
            .expect("camera start capture mutex poisoned") += 1;
        true
    }

    unsafe extern "C" fn capture_camera_stop() {
        *captured_camera_stops()
            .lock()
            .expect("camera stop capture mutex poisoned") += 1;
    }

    unsafe extern "C" fn capture_netpacket_send(
        flags: i32,
        buf: *const c_void,
        len: usize,
        client_id: u16,
    ) {
        let data = if buf.is_null() || len == 0 {
            Vec::new()
        } else {
            unsafe { std::slice::from_raw_parts(buf.cast::<u8>(), len) }.to_vec()
        };
        captured_netpacket_sends()
            .lock()
            .expect("netpacket sends mutex poisoned")
            .push(CapturedNetpacketSend {
                flags,
                data,
                client_id,
            });
    }

    unsafe extern "C" fn capture_netpacket_poll_receive() {
        *captured_netpacket_polls()
            .lock()
            .expect("netpacket polls mutex poisoned") += 1;
    }

    unsafe extern "C" fn capture_open_mic(
        params: *const raw::retro_microphone_params,
    ) -> *mut raw::retro_microphone {
        let rate = if params.is_null() {
            None
        } else {
            Some(unsafe { (*params).rate })
        };
        captured_mic_open_params()
            .lock()
            .expect("microphone open params mutex poisoned")
            .push(rate);
        static MIC_HANDLE: u8 = 0;
        (&MIC_HANDLE as *const u8).cast_mut().cast()
    }

    unsafe extern "C" fn capture_close_mic(_microphone: *mut raw::retro_microphone) {
        *captured_mic_closes()
            .lock()
            .expect("microphone closes mutex poisoned") += 1;
    }

    unsafe extern "C" fn capture_get_mic_params(
        microphone: *const raw::retro_microphone,
        params: *mut raw::retro_microphone_params,
    ) -> bool {
        let Some(params) = (unsafe { params.as_mut() }) else {
            return false;
        };
        if microphone.is_null() {
            return false;
        }
        params.rate = 22_050;
        true
    }

    unsafe extern "C" fn capture_set_mic_state(
        microphone: *mut raw::retro_microphone,
        state: bool,
    ) -> bool {
        if microphone.is_null() {
            return false;
        }
        captured_mic_states()
            .lock()
            .expect("microphone states mutex poisoned")
            .push(state);
        true
    }

    unsafe extern "C" fn capture_get_mic_state(microphone: *const raw::retro_microphone) -> bool {
        !microphone.is_null()
    }

    unsafe extern "C" fn capture_read_mic(
        microphone: *mut raw::retro_microphone,
        samples: *mut i16,
        num_samples: usize,
    ) -> i32 {
        if microphone.is_null() || samples.is_null() {
            return -1;
        }
        let samples = unsafe { std::slice::from_raw_parts_mut(samples, num_samples) };
        for (index, sample) in samples.iter_mut().enumerate() {
            *sample = i16::try_from(index + 1).expect("test sample index fits i16");
        }
        i32::try_from(num_samples).expect("test sample count fits i32")
    }

    unsafe extern "C" fn capture_midi_input_enabled() -> bool {
        true
    }

    unsafe extern "C" fn capture_midi_output_enabled() -> bool {
        true
    }

    unsafe extern "C" fn capture_midi_read(byte: *mut u8) -> bool {
        let Some(byte) = (unsafe { byte.as_mut() }) else {
            return false;
        };
        *byte = 0x90;
        true
    }

    unsafe extern "C" fn capture_midi_write(byte: u8, delta_time: u32) -> bool {
        captured_midi_writes()
            .lock()
            .expect("MIDI write capture mutex poisoned")
            .push((byte, delta_time));
        byte != 0
    }

    unsafe extern "C" fn capture_midi_flush() -> bool {
        *captured_midi_flushes()
            .lock()
            .expect("MIDI flush capture mutex poisoned") += 1;
        true
    }

    unsafe extern "C" fn capture_perf_time_usec() -> raw::retro_time_t {
        123_456
    }

    unsafe extern "C" fn capture_perf_counter() -> raw::retro_perf_tick_t {
        9_001
    }

    unsafe extern "C" fn capture_perf_cpu_features() -> u64 {
        raw::RETRO_SIMD_SSE2 | raw::RETRO_SIMD_NEON
    }

    unsafe extern "C" fn capture_perf_register(counter: *mut raw::retro_perf_counter) {
        let Some(counter) = (unsafe { counter.as_mut() }) else {
            return;
        };
        let ident = if counter.ident.is_null() {
            String::new()
        } else {
            unsafe { CStr::from_ptr(counter.ident) }
                .to_string_lossy()
                .into_owned()
        };
        captured_perf_registered_idents()
            .lock()
            .expect("perf registered idents mutex poisoned")
            .push(ident);
        counter.registered = true;
    }

    unsafe extern "C" fn capture_perf_start(counter: *mut raw::retro_perf_counter) {
        let Some(counter) = (unsafe { counter.as_mut() }) else {
            return;
        };
        counter.start = 9_001;
        counter.call_cnt += 1;
    }

    unsafe extern "C" fn capture_perf_stop(counter: *mut raw::retro_perf_counter) {
        let Some(counter) = (unsafe { counter.as_mut() }) else {
            return;
        };
        counter.total += 377;
    }

    unsafe extern "C" fn capture_perf_log() {
        *captured_perf_logs()
            .lock()
            .expect("perf log capture mutex poisoned") += 1;
    }

    unsafe fn capture_required_cstr(value: *const c_char) -> String {
        assert!(!value.is_null());
        unsafe { CStr::from_ptr(value) }
            .to_string_lossy()
            .into_owned()
    }

    unsafe fn capture_optional_cstr(value: *const c_char) -> Option<String> {
        (!value.is_null()).then(|| unsafe { capture_required_cstr(value) })
    }

    unsafe fn capture_core_option_values(
        values: &[raw::retro_core_option_value; raw::RETRO_NUM_CORE_OPTION_VALUES_MAX],
    ) -> Vec<CapturedCoreOptionValue> {
        values
            .iter()
            .take_while(|value| !value.value.is_null())
            .map(|value| CapturedCoreOptionValue {
                value: unsafe { capture_required_cstr(value.value) },
                label: unsafe { capture_optional_cstr(value.label) },
            })
            .collect()
    }

    unsafe fn capture_v1_definitions(
        mut current: *const raw::retro_core_option_definition,
    ) -> Vec<CapturedCoreOptionDefinition> {
        let mut definitions = Vec::new();
        while !current.is_null() {
            let definition = unsafe { &*current };
            if definition.key.is_null() {
                break;
            }
            definitions.push(CapturedCoreOptionDefinition {
                key: unsafe { capture_required_cstr(definition.key) },
                description: unsafe { capture_required_cstr(definition.desc) },
                description_categorized: None,
                info: unsafe { capture_optional_cstr(definition.info) },
                info_categorized: None,
                category_key: None,
                values: unsafe { capture_core_option_values(&definition.values) },
                default_value: unsafe { capture_required_cstr(definition.default_value) },
            });
            current = unsafe { current.add(1) };
        }
        definitions
    }

    unsafe fn capture_v2_options(
        options: *const raw::retro_core_options_v2,
    ) -> CapturedCoreOptionsV2 {
        let options = unsafe { &*options };
        let mut categories = Vec::new();
        let mut category = options.categories;
        while !category.is_null() {
            let raw_category = unsafe { &*category };
            if raw_category.key.is_null() {
                break;
            }
            categories.push(CapturedCoreOptionCategory {
                key: unsafe { capture_required_cstr(raw_category.key) },
                description: unsafe { capture_required_cstr(raw_category.desc) },
                info: unsafe { capture_optional_cstr(raw_category.info) },
            });
            category = unsafe { category.add(1) };
        }

        let mut definitions = Vec::new();
        let mut definition = options.definitions;
        while !definition.is_null() {
            let raw_definition = unsafe { &*definition };
            if raw_definition.key.is_null() {
                break;
            }
            definitions.push(CapturedCoreOptionDefinition {
                key: unsafe { capture_required_cstr(raw_definition.key) },
                description: unsafe { capture_required_cstr(raw_definition.desc) },
                description_categorized: unsafe {
                    capture_optional_cstr(raw_definition.desc_categorized)
                },
                info: unsafe { capture_optional_cstr(raw_definition.info) },
                info_categorized: unsafe { capture_optional_cstr(raw_definition.info_categorized) },
                category_key: unsafe { capture_optional_cstr(raw_definition.category_key) },
                values: unsafe { capture_core_option_values(&raw_definition.values) },
                default_value: unsafe { capture_required_cstr(raw_definition.default_value) },
            });
            definition = unsafe { definition.add(1) };
        }

        CapturedCoreOptionsV2 {
            categories,
            definitions,
        }
    }

    unsafe extern "C" fn core_options_env(command: u32, data: *mut c_void) -> bool {
        match command {
            raw::RETRO_ENVIRONMENT_GET_CORE_OPTIONS_VERSION => {
                let Some(version) = *captured_core_options_version()
                    .lock()
                    .expect("core options version capture mutex poisoned")
                else {
                    return false;
                };
                unsafe { *data.cast::<u32>() = version };
                true
            }
            raw::RETRO_ENVIRONMENT_SET_VARIABLES => {
                *captured_variables()
                    .lock()
                    .expect("variable capture mutex poisoned") =
                    unsafe { capture_variables(data.cast::<RawVariable>()) };
                true
            }
            raw::RETRO_ENVIRONMENT_SET_CORE_OPTIONS => {
                *captured_core_options_v1()
                    .lock()
                    .expect("core options v1 capture mutex poisoned") =
                    unsafe { capture_v1_definitions(data.cast()) };
                true
            }
            raw::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_INTL => {
                let raw = unsafe { &*data.cast::<raw::retro_core_options_intl>() };
                *captured_core_options_v1()
                    .lock()
                    .expect("core options v1 capture mutex poisoned") =
                    unsafe { capture_v1_definitions(raw.us) };
                true
            }
            raw::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2 => {
                *captured_core_options_v2()
                    .lock()
                    .expect("core options v2 capture mutex poisoned") =
                    Some(unsafe { capture_v2_options(data.cast()) });
                true
            }
            raw::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL => {
                let raw = unsafe { &*data.cast::<raw::retro_core_options_v2_intl>() };
                *captured_core_options_v2()
                    .lock()
                    .expect("core options v2 capture mutex poisoned") =
                    Some(unsafe { capture_v2_options(raw.us) });
                true
            }
            raw::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_DISPLAY => {
                let raw = unsafe { &*data.cast::<raw::retro_core_option_display>() };
                captured_core_option_displays()
                    .lock()
                    .expect("core option display capture mutex poisoned")
                    .push(CapturedCoreOptionDisplay {
                        key: unsafe { capture_required_cstr(raw.key) },
                        visible: raw.visible,
                    });
                true
            }
            raw::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_UPDATE_DISPLAY_CALLBACK => {
                *captured_core_options_update_display_callback()
                    .lock()
                    .expect("core options update display callback capture mutex poisoned") =
                    Some(unsafe {
                        *data.cast::<raw::retro_core_options_update_display_callback>()
                    });
                true
            }
            raw::RETRO_ENVIRONMENT_SET_VARIABLE => {
                let raw = unsafe { &*data.cast::<RawVariable>() };
                captured_variables()
                    .lock()
                    .expect("variable capture mutex poisoned")
                    .push(CapturedVariable {
                        key: unsafe { capture_required_cstr(raw.key) },
                        value: unsafe { capture_optional_cstr(raw.value) },
                    });
                true
            }
            _ => false,
        }
    }

    unsafe fn capture_variables(mut current: *const RawVariable) -> Vec<CapturedVariable> {
        let mut variables = Vec::new();
        while !current.is_null() {
            let variable = unsafe { &*current };
            if variable.key.is_null() {
                break;
            }
            variables.push(CapturedVariable {
                key: unsafe { capture_required_cstr(variable.key) },
                value: unsafe { capture_optional_cstr(variable.value) },
            });
            current = unsafe { current.add(1) };
        }
        variables
    }

    unsafe extern "C" fn vfs_env(command: u32, data: *mut c_void) -> bool {
        if command != raw::RETRO_ENVIRONMENT_GET_VFS_INTERFACE {
            return false;
        }
        let info = unsafe { data.cast::<raw::retro_vfs_interface_info>().as_mut() }
            .expect("VFS interface info must be non-null");
        captured_vfs_interface_requests()
            .lock()
            .expect("VFS request capture mutex poisoned")
            .push(info.required_interface_version);
        if info.required_interface_version > 3 {
            return false;
        }
        info.required_interface_version = 3;
        info.iface = (&FRONTEND_VFS_INTERFACE as *const raw::retro_vfs_interface).cast_mut();
        true
    }

    fn fake_vfs_file_handle() -> *mut raw::retro_vfs_file_handle {
        std::ptr::dangling_mut::<raw::retro_vfs_file_handle>()
    }

    fn fake_vfs_dir_handle() -> *mut raw::retro_vfs_dir_handle {
        std::ptr::dangling_mut::<raw::retro_vfs_dir_handle>()
    }

    unsafe extern "C" fn capture_vfs_get_path(
        _stream: *mut raw::retro_vfs_file_handle,
    ) -> *const c_char {
        c"/tmp/test.bin".as_ptr()
    }

    unsafe extern "C" fn capture_vfs_open(
        path: *const c_char,
        mode: u32,
        hints: u32,
    ) -> *mut raw::retro_vfs_file_handle {
        captured_vfs_opens()
            .lock()
            .expect("VFS open capture mutex poisoned")
            .push(CapturedVfsOpen {
                path: unsafe { capture_required_cstr(path) },
                mode,
                hints,
            });
        fake_vfs_file_handle()
    }

    unsafe extern "C" fn capture_vfs_close(_stream: *mut raw::retro_vfs_file_handle) -> i32 {
        *captured_vfs_closes()
            .lock()
            .expect("VFS close capture mutex poisoned") += 1;
        0
    }

    unsafe extern "C" fn capture_vfs_size(_stream: *mut raw::retro_vfs_file_handle) -> i64 {
        8
    }

    unsafe extern "C" fn capture_vfs_truncate(
        _stream: *mut raw::retro_vfs_file_handle,
        length: i64,
    ) -> i64 {
        if length == 4 { 0 } else { -1 }
    }

    unsafe extern "C" fn capture_vfs_tell(_stream: *mut raw::retro_vfs_file_handle) -> i64 {
        3
    }

    unsafe extern "C" fn capture_vfs_seek(
        _stream: *mut raw::retro_vfs_file_handle,
        offset: i64,
        seek_position: i32,
    ) -> i64 {
        assert_eq!(offset, -2);
        assert_eq!(seek_position, raw::RETRO_VFS_SEEK_POSITION_END);
        6
    }

    unsafe extern "C" fn capture_vfs_read(
        _stream: *mut raw::retro_vfs_file_handle,
        out: *mut c_void,
        len: u64,
    ) -> i64 {
        let bytes = b"abc";
        assert!(len >= bytes.len() as u64);
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), out.cast::<u8>(), bytes.len()) };
        bytes.len() as i64
    }

    unsafe extern "C" fn capture_vfs_write(
        _stream: *mut raw::retro_vfs_file_handle,
        data: *const c_void,
        len: u64,
    ) -> i64 {
        let bytes = unsafe { std::slice::from_raw_parts(data.cast::<u8>(), len as usize) };
        captured_vfs_writes()
            .lock()
            .expect("VFS write capture mutex poisoned")
            .push(bytes.to_vec());
        len as i64
    }

    unsafe extern "C" fn capture_vfs_flush(_stream: *mut raw::retro_vfs_file_handle) -> i32 {
        0
    }

    unsafe extern "C" fn capture_vfs_remove(path: *const c_char) -> i32 {
        captured_vfs_removes()
            .lock()
            .expect("VFS remove capture mutex poisoned")
            .push(unsafe { capture_required_cstr(path) });
        0
    }

    unsafe extern "C" fn capture_vfs_rename(
        old_path: *const c_char,
        new_path: *const c_char,
    ) -> i32 {
        captured_vfs_renames()
            .lock()
            .expect("VFS rename capture mutex poisoned")
            .push(CapturedVfsRename {
                old_path: unsafe { capture_required_cstr(old_path) },
                new_path: unsafe { capture_required_cstr(new_path) },
            });
        0
    }

    unsafe extern "C" fn capture_vfs_stat(path: *const c_char, size: *mut i32) -> i32 {
        assert_eq!(unsafe { capture_required_cstr(path) }, "/tmp/test.bin");
        unsafe { *size = 8 };
        (raw::RETRO_VFS_STAT_IS_VALID | raw::RETRO_VFS_STAT_IS_DIRECTORY) as i32
    }

    unsafe extern "C" fn capture_vfs_mkdir(path: *const c_char) -> i32 {
        captured_vfs_mkdirs()
            .lock()
            .expect("VFS mkdir capture mutex poisoned")
            .push(unsafe { capture_required_cstr(path) });
        0
    }

    unsafe extern "C" fn capture_vfs_opendir(
        path: *const c_char,
        include_hidden: bool,
    ) -> *mut raw::retro_vfs_dir_handle {
        assert_eq!(unsafe { capture_required_cstr(path) }, "/tmp");
        assert!(include_hidden);
        fake_vfs_dir_handle()
    }

    unsafe extern "C" fn capture_vfs_readdir(_dirstream: *mut raw::retro_vfs_dir_handle) -> bool {
        let mut calls = captured_vfs_readdirs()
            .lock()
            .expect("VFS readdir capture mutex poisoned");
        *calls += 1;
        *calls == 1
    }

    unsafe extern "C" fn capture_vfs_dirent_get_name(
        _dirstream: *mut raw::retro_vfs_dir_handle,
    ) -> *const c_char {
        c"entry.bin".as_ptr()
    }

    unsafe extern "C" fn capture_vfs_dirent_is_dir(
        _dirstream: *mut raw::retro_vfs_dir_handle,
    ) -> bool {
        false
    }

    unsafe extern "C" fn capture_vfs_closedir(_dirstream: *mut raw::retro_vfs_dir_handle) -> i32 {
        *captured_vfs_dir_closes()
            .lock()
            .expect("VFS dir close capture mutex poisoned") += 1;
        0
    }

    unsafe extern "C" fn frontend_services_env(command: u32, data: *mut c_void) -> bool {
        match command {
            raw::RETRO_ENVIRONMENT_SET_ROTATION => {
                let rotation =
                    unsafe { data.cast::<u32>().as_ref() }.expect("rotation data must be non-null");
                captured_rotations()
                    .lock()
                    .expect("rotation capture mutex poisoned")
                    .push(*rotation);
                true
            }
            raw::RETRO_ENVIRONMENT_GET_OVERSCAN => {
                unsafe { *data.cast::<bool>() = false };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_CAN_DUPE => {
                unsafe { *data.cast::<bool>() = true };
                true
            }
            raw::RETRO_ENVIRONMENT_SHUTDOWN => {
                *captured_shutdowns()
                    .lock()
                    .expect("shutdown capture mutex poisoned") += 1;
                true
            }
            raw::RETRO_ENVIRONMENT_SET_HW_SHARED_CONTEXT => {
                assert!(data.is_null());
                *captured_hw_shared_contexts()
                    .lock()
                    .expect("HW shared context capture mutex poisoned") += 1;
                true
            }
            raw::RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY => {
                unsafe { *data.cast::<*const c_char>() = c"/system".as_ptr() };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_LIBRETRO_PATH => {
                unsafe { *data.cast::<*const c_char>() = c"/cores/test_libretro.so".as_ptr() };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_CORE_ASSETS_DIRECTORY => {
                unsafe { *data.cast::<*const c_char>() = c"/assets".as_ptr() };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY => {
                unsafe { *data.cast::<*const c_char>() = c"/saves".as_ptr() };
                true
            }
            raw::RETRO_ENVIRONMENT_SET_SYSTEM_AV_INFO => {
                let info = unsafe { data.cast::<raw::retro_system_av_info>().as_ref() }
                    .expect("system av info data must be non-null");
                captured_system_av_infos()
                    .lock()
                    .expect("system av info capture mutex poisoned")
                    .push(SystemAvInfo::from_raw(*info));
                true
            }
            raw::RETRO_ENVIRONMENT_GET_GAME_INFO_EXT => {
                unsafe {
                    *data.cast::<*const raw::retro_game_info_ext>() = extended_game_info_ptr()
                };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_USERNAME => {
                unsafe { *data.cast::<*const c_char>() = c"player".as_ptr() };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_PLAYLIST_DIRECTORY => {
                unsafe { *data.cast::<*const c_char>() = c"/playlists".as_ptr() };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_FILE_BROWSER_START_DIRECTORY => {
                unsafe { *data.cast::<*const c_char>() = c"/browser".as_ptr() };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_LANGUAGE => {
                unsafe { *data.cast::<i32>() = Language::PortugueseBrazil.as_raw() };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_JIT_CAPABLE => {
                unsafe { *data.cast::<bool>() = true };
                true
            }
            raw::RETRO_ENVIRONMENT_SET_SUPPORT_ACHIEVEMENTS => {
                let supported = unsafe { data.cast::<bool>().as_ref() }
                    .expect("achievement support data must be non-null");
                captured_achievement_support()
                    .lock()
                    .expect("achievement support capture mutex poisoned")
                    .push(*supported);
                true
            }
            raw::RETRO_ENVIRONMENT_SET_PERFORMANCE_LEVEL => {
                let level = unsafe { data.cast::<u32>().as_ref() }
                    .expect("performance level data must be non-null");
                captured_performance_levels()
                    .lock()
                    .expect("performance level capture mutex poisoned")
                    .push(*level);
                true
            }
            raw::RETRO_ENVIRONMENT_GET_PERF_INTERFACE => {
                unsafe {
                    *data.cast::<raw::retro_perf_callback>() = raw::retro_perf_callback {
                        get_time_usec: Some(capture_perf_time_usec),
                        get_cpu_features: Some(capture_perf_cpu_features),
                        get_perf_counter: Some(capture_perf_counter),
                        perf_register: Some(capture_perf_register),
                        perf_start: Some(capture_perf_start),
                        perf_stop: Some(capture_perf_stop),
                        perf_log: Some(capture_perf_log),
                    }
                };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_DEVICE_POWER => {
                unsafe {
                    *data.cast::<raw::retro_device_power>() = raw::retro_device_power {
                        state: raw::retro_power_state::Discharging,
                        seconds: 3600,
                        percent: 72,
                    }
                };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_NETPLAY_CLIENT_INDEX => {
                unsafe { *data.cast::<u32>() = 2 };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_DISK_CONTROL_INTERFACE_VERSION => {
                unsafe { *data.cast::<u32>() = 1 };
                true
            }
            raw::RETRO_ENVIRONMENT_SET_SERIALIZATION_QUIRKS => {
                let quirks = unsafe { data.cast::<u64>().as_mut() }
                    .expect("serialization quirks data must be non-null");
                captured_serialization_quirks()
                    .lock()
                    .expect("serialization quirk capture mutex poisoned")
                    .push(*quirks);
                *quirks &= raw::RETRO_SERIALIZATION_QUIRK_MUST_INITIALIZE
                    | raw::RETRO_SERIALIZATION_QUIRK_PLATFORM_DEPENDENT;
                true
            }
            raw::RETRO_ENVIRONMENT_SET_MEMORY_MAPS => {
                let map = unsafe { data.cast::<raw::retro_memory_map>().as_ref() }
                    .expect("memory map data must be non-null");
                let descriptors = unsafe {
                    std::slice::from_raw_parts(map.descriptors, map.num_descriptors as usize)
                };
                let captured = descriptors
                    .iter()
                    .map(|descriptor| CapturedMemoryDescriptor {
                        flags: descriptor.flags,
                        ptr_is_null: descriptor.ptr.is_null(),
                        offset: descriptor.offset,
                        start: descriptor.start,
                        select: descriptor.select,
                        disconnect: descriptor.disconnect,
                        len: descriptor.len,
                        addrspace: if descriptor.addrspace.is_null() {
                            None
                        } else {
                            Some(
                                unsafe { CStr::from_ptr(descriptor.addrspace) }
                                    .to_string_lossy()
                                    .into_owned(),
                            )
                        },
                    })
                    .collect::<Vec<_>>();
                *captured_memory_descriptors()
                    .lock()
                    .expect("memory descriptor capture mutex poisoned") = captured;
                true
            }
            raw::RETRO_ENVIRONMENT_SET_SUBSYSTEM_INFO => {
                let mut captured = Vec::new();
                let mut current = data.cast::<raw::retro_subsystem_info>();
                loop {
                    let subsystem = unsafe { *current };
                    if subsystem.desc.is_null()
                        && subsystem.ident.is_null()
                        && subsystem.roms.is_null()
                        && subsystem.num_roms == 0
                        && subsystem.id == 0
                    {
                        break;
                    }
                    let roms = if subsystem.roms.is_null() {
                        Vec::new()
                    } else {
                        unsafe {
                            std::slice::from_raw_parts(subsystem.roms, subsystem.num_roms as usize)
                        }
                        .to_vec()
                    };
                    captured.push(CapturedSubsystem {
                        description: unsafe { CStr::from_ptr(subsystem.desc) }
                            .to_string_lossy()
                            .into_owned(),
                        identifier: unsafe { CStr::from_ptr(subsystem.ident) }
                            .to_string_lossy()
                            .into_owned(),
                        id: subsystem.id,
                        roms: roms
                            .iter()
                            .map(|rom| {
                                let memory = if rom.memory.is_null() {
                                    Vec::new()
                                } else {
                                    unsafe {
                                        std::slice::from_raw_parts(
                                            rom.memory,
                                            rom.num_memory as usize,
                                        )
                                    }
                                    .iter()
                                    .map(|memory| CapturedSubsystemMemory {
                                        extension: unsafe { CStr::from_ptr(memory.extension) }
                                            .to_string_lossy()
                                            .into_owned(),
                                        memory_type: memory.memory_type,
                                    })
                                    .collect()
                                };
                                CapturedSubsystemRom {
                                    description: unsafe { CStr::from_ptr(rom.desc) }
                                        .to_string_lossy()
                                        .into_owned(),
                                    valid_extensions: unsafe {
                                        CStr::from_ptr(rom.valid_extensions)
                                    }
                                    .to_string_lossy()
                                    .into_owned(),
                                    need_fullpath: rom.need_fullpath,
                                    block_extract: rom.block_extract,
                                    required: rom.required,
                                    memory,
                                }
                            })
                            .collect(),
                    });
                    current = unsafe { current.add(1) };
                }
                *captured_subsystem_info()
                    .lock()
                    .expect("subsystem info capture mutex poisoned") = captured;
                true
            }
            raw::RETRO_ENVIRONMENT_SET_DISK_CONTROL_INTERFACE => {
                *captured_disk_control_callback()
                    .lock()
                    .expect("disk control callback capture mutex poisoned") = if data.is_null() {
                    None
                } else {
                    Some(unsafe { *data.cast::<raw::retro_disk_control_callback>() })
                };
                true
            }
            raw::RETRO_ENVIRONMENT_SET_DISK_CONTROL_EXT_INTERFACE => {
                *captured_disk_control_ext_callback()
                    .lock()
                    .expect("disk control ext callback capture mutex poisoned") = if data.is_null()
                {
                    None
                } else {
                    Some(unsafe { *data.cast::<raw::retro_disk_control_ext_callback>() })
                };
                true
            }
            raw::RETRO_ENVIRONMENT_SET_NETPACKET_INTERFACE => {
                *captured_netpacket_callback()
                    .lock()
                    .expect("netpacket callback capture mutex poisoned") = if data.is_null() {
                    None
                } else {
                    Some(CapturedNetpacketCallback::from_raw(unsafe {
                        *data.cast::<raw::retro_netpacket_callback>()
                    }))
                };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_MICROPHONE_INTERFACE => {
                let interface = unsafe { data.cast::<raw::retro_microphone_interface>().as_mut() }
                    .expect("microphone interface data must be non-null");
                assert_eq!(
                    interface.interface_version,
                    raw::RETRO_MICROPHONE_INTERFACE_VERSION
                );
                *interface = raw::retro_microphone_interface {
                    interface_version: raw::RETRO_MICROPHONE_INTERFACE_VERSION,
                    open_mic: Some(capture_open_mic),
                    close_mic: Some(capture_close_mic),
                    get_params: Some(capture_get_mic_params),
                    set_mic_state: Some(capture_set_mic_state),
                    get_mic_state: Some(capture_get_mic_state),
                    read_mic: Some(capture_read_mic),
                };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_CURRENT_SOFTWARE_FRAMEBUFFER => {
                let framebuffer = unsafe { data.cast::<raw::retro_framebuffer>().as_mut() }
                    .expect("software framebuffer data must be non-null");
                assert_eq!(framebuffer.width, 4);
                assert_eq!(framebuffer.height, 2);
                assert_eq!(
                    framebuffer.access_flags,
                    raw::RETRO_MEMORY_ACCESS_WRITE | raw::RETRO_MEMORY_ACCESS_READ
                );

                let mut pixels = software_framebuffer_pixels()
                    .lock()
                    .expect("software framebuffer pixels mutex poisoned");
                framebuffer.data = pixels.as_mut_ptr().cast::<c_void>();
                framebuffer.pitch = 4 * mem::size_of::<u32>();
                framebuffer.format = PixelFormat::Xrgb8888;
                framebuffer.memory_flags = raw::RETRO_MEMORY_TYPE_CACHED;
                true
            }
            raw::RETRO_ENVIRONMENT_GET_INPUT_DEVICE_CAPABILITIES => {
                unsafe {
                    *data.cast::<u64>() = (1u64 << raw::RETRO_DEVICE_JOYPAD)
                        | (1u64 << raw::RETRO_DEVICE_ANALOG)
                        | (1u64 << raw::RETRO_DEVICE_POINTER)
                };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_INPUT_BITMASKS => true,
            raw::RETRO_ENVIRONMENT_GET_INPUT_MAX_USERS => {
                unsafe { *data.cast::<u32>() = 4 };
                true
            }
            raw::RETRO_ENVIRONMENT_SET_CONTROLLER_INFO => {
                let mut ports = Vec::new();
                let mut current = data.cast::<raw::retro_controller_info>();
                loop {
                    let port = unsafe { *current };
                    if port.types.is_null() && port.num_types == 0 {
                        break;
                    }
                    let types =
                        unsafe { std::slice::from_raw_parts(port.types, port.num_types as usize) };
                    ports.push(
                        types
                            .iter()
                            .map(|description| CapturedControllerDescription {
                                description: unsafe { CStr::from_ptr(description.desc) }
                                    .to_string_lossy()
                                    .into_owned(),
                                id: description.id,
                            })
                            .collect::<Vec<_>>(),
                    );
                    current = unsafe { current.add(1) };
                }
                *captured_controller_info()
                    .lock()
                    .expect("controller info capture mutex poisoned") = ports;
                true
            }
            raw::RETRO_ENVIRONMENT_SET_PROC_ADDRESS_CALLBACK => {
                *captured_proc_address_interface()
                    .lock()
                    .expect("proc address interface capture mutex poisoned") =
                    Some(unsafe { *data.cast::<raw::retro_get_proc_address_interface>() });
                true
            }
            raw::RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS => {
                let mut descriptors = Vec::new();
                let mut current = data.cast::<RawInputDescriptor>();
                loop {
                    let descriptor = unsafe { *current };
                    if descriptor.description.is_null() {
                        break;
                    }
                    descriptors.push(CapturedInputDescriptor {
                        port: descriptor.port,
                        device: descriptor.device,
                        index: descriptor.index,
                        id: descriptor.id,
                        description: unsafe { CStr::from_ptr(descriptor.description) }
                            .to_string_lossy()
                            .into_owned(),
                    });
                    current = unsafe { current.add(1) };
                }
                *captured_input_descriptors()
                    .lock()
                    .expect("input descriptor capture mutex poisoned") = descriptors;
                true
            }
            raw::RETRO_ENVIRONMENT_SET_KEYBOARD_CALLBACK => {
                *captured_keyboard_callback()
                    .lock()
                    .expect("keyboard callback capture mutex poisoned") =
                    Some(unsafe { *data.cast::<RawKeyboardCallback>() });
                true
            }
            raw::RETRO_ENVIRONMENT_GET_LED_INTERFACE => {
                unsafe {
                    *data.cast::<raw::retro_led_interface>() = raw::retro_led_interface {
                        set_led_state: Some(capture_led_state),
                    }
                };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_RUMBLE_INTERFACE => {
                unsafe {
                    *data.cast::<raw::retro_rumble_interface>() = raw::retro_rumble_interface {
                        set_rumble_state: Some(capture_rumble_state),
                    }
                };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_SENSOR_INTERFACE => {
                unsafe {
                    *data.cast::<raw::retro_sensor_interface>() = raw::retro_sensor_interface {
                        set_sensor_state: Some(capture_sensor_state),
                        get_sensor_input: Some(capture_sensor_input),
                    }
                };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_CAMERA_INTERFACE => {
                let callback = unsafe { data.cast::<raw::retro_camera_callback>().as_mut() }
                    .expect("camera callback data must be non-null");
                assert_eq!(
                    callback.caps,
                    (CameraCapabilities::from(CameraCapability::RawFramebuffer)
                        | CameraCapability::OpenGlTexture)
                        .bits()
                );
                assert_eq!(callback.width, 320);
                assert_eq!(callback.height, 240);
                assert!(callback.frame_raw_framebuffer.is_some());
                assert!(callback.frame_opengl_texture.is_some());
                assert!(callback.initialized.is_some());
                assert!(callback.deinitialized.is_some());
                callback.start = Some(capture_camera_start);
                callback.stop = Some(capture_camera_stop);
                callback.caps = CameraCapabilities::from(CameraCapability::RawFramebuffer).bits();
                callback.width = 160;
                callback.height = 120;
                *captured_camera_callback()
                    .lock()
                    .expect("camera callback capture mutex poisoned") = Some(*callback);
                true
            }
            raw::RETRO_ENVIRONMENT_GET_LOCATION_INTERFACE => {
                let callback = unsafe { data.cast::<raw::retro_location_callback>().as_mut() }
                    .expect("location callback data must be non-null");
                let initialized = callback.initialized;
                let deinitialized = callback.deinitialized;
                *callback = raw::retro_location_callback {
                    start: Some(capture_location_start),
                    stop: Some(capture_location_stop),
                    get_position: Some(capture_location_get_position),
                    set_interval: Some(capture_location_set_interval),
                    initialized,
                    deinitialized,
                };
                *captured_location_callback()
                    .lock()
                    .expect("location callback capture mutex poisoned") = Some(*callback);
                true
            }
            raw::RETRO_ENVIRONMENT_GET_MIDI_INTERFACE => {
                if data.is_null() {
                    *captured_midi_probes()
                        .lock()
                        .expect("MIDI probe capture mutex poisoned") += 1;
                } else {
                    unsafe {
                        *data.cast::<raw::retro_midi_interface>() = raw::retro_midi_interface {
                            input_enabled: Some(capture_midi_input_enabled),
                            output_enabled: Some(capture_midi_output_enabled),
                            read: Some(capture_midi_read),
                            write: Some(capture_midi_write),
                            flush: Some(capture_midi_flush),
                        }
                    };
                }
                true
            }
            raw::RETRO_ENVIRONMENT_GET_AUDIO_VIDEO_ENABLE => {
                unsafe {
                    *data.cast::<u32>() =
                        raw::RETRO_AV_ENABLE_VIDEO | raw::RETRO_AV_ENABLE_HARD_DISABLE_AUDIO
                };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_FASTFORWARDING => {
                unsafe { *data.cast::<bool>() = true };
                true
            }
            raw::RETRO_ENVIRONMENT_SET_FASTFORWARDING_OVERRIDE => {
                let captured = if data.is_null() {
                    None
                } else {
                    Some(unsafe { *data.cast::<raw::retro_fastforwarding_override>() })
                };
                captured_fastforwarding_overrides()
                    .lock()
                    .expect("fastforwarding override capture mutex poisoned")
                    .push(captured);
                true
            }
            raw::RETRO_ENVIRONMENT_GET_TARGET_REFRESH_RATE => {
                unsafe { *data.cast::<f32>() = 59.94 };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_TARGET_SAMPLE_RATE => {
                unsafe { *data.cast::<u32>() = 48_000 };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_THROTTLE_STATE => {
                unsafe {
                    *data.cast::<raw::retro_throttle_state>() = raw::retro_throttle_state {
                        mode: raw::RETRO_THROTTLE_FAST_FORWARD,
                        rate: 120.0,
                    }
                };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_SAVESTATE_CONTEXT => {
                unsafe {
                    *data.cast::<i32>() = raw::retro_savestate_context::RollbackNetplay as i32
                };
                true
            }
            raw::RETRO_ENVIRONMENT_SET_MINIMUM_AUDIO_LATENCY => {
                let captured = if data.is_null() {
                    None
                } else {
                    Some(unsafe { *data.cast::<u32>() })
                };
                captured_audio_latencies()
                    .lock()
                    .expect("audio latency capture mutex poisoned")
                    .push(captured);
                true
            }
            raw::RETRO_ENVIRONMENT_SET_AUDIO_CALLBACK => {
                if data.is_null() {
                    *captured_audio_callback_probes()
                        .lock()
                        .expect("audio callback probe mutex poisoned") += 1;
                } else {
                    *captured_audio_callback()
                        .lock()
                        .expect("audio callback capture mutex poisoned") =
                        Some(unsafe { *data.cast::<RawAudioCallback>() });
                }
                true
            }
            raw::RETRO_ENVIRONMENT_SET_AUDIO_BUFFER_STATUS_CALLBACK => {
                let callback = if data.is_null() {
                    None
                } else {
                    Some(unsafe { *data.cast::<RawAudioBufferStatusCallback>() })
                };
                *captured_audio_buffer_status_callback()
                    .lock()
                    .expect("audio buffer status callback capture mutex poisoned") = callback;
                true
            }
            raw::RETRO_ENVIRONMENT_SET_FRAME_TIME_CALLBACK => {
                let callback = if data.is_null() {
                    None
                } else {
                    Some(unsafe { *data.cast::<RawFrameTimeCallback>() })
                };
                *captured_frame_time_callback()
                    .lock()
                    .expect("frame time callback capture mutex poisoned") = callback;
                true
            }
            _ => false,
        }
    }

    unsafe extern "C" fn null_frontend_string_env(command: u32, data: *mut c_void) -> bool {
        if command != raw::RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY {
            return false;
        }
        unsafe { *data.cast::<*const c_char>() = ptr::null() };
        true
    }

    unsafe extern "C" fn capture_content_info_overrides(command: u32, data: *mut c_void) -> bool {
        if command != RETRO_ENVIRONMENT_SET_CONTENT_INFO_OVERRIDE {
            return false;
        }

        let storage = CAPTURED_CONTENT_OVERRIDES.get_or_init(|| Mutex::new(Vec::new()));
        let mut captured = storage.lock().expect("capture mutex poisoned");
        captured.clear();

        let mut cursor = data.cast::<RawContentInfoOverride>();
        loop {
            let item = unsafe { *cursor };
            if item.extensions.is_null() {
                break;
            }
            let extensions = unsafe { CStr::from_ptr(item.extensions) }
                .to_string_lossy()
                .into_owned();
            captured.push(CapturedContentOverride {
                extensions,
                need_fullpath: item.need_fullpath,
                persistent_data: item.persistent_data,
            });
            cursor = unsafe { cursor.add(1) };
        }

        true
    }

    unsafe extern "C" fn capture_content_contract_env(command: u32, data: *mut c_void) -> bool {
        match command {
            RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME => {
                captured_support_no_game()
                    .lock()
                    .expect("support-no-game capture mutex poisoned")
                    .push(unsafe { *data.cast::<bool>() });
                true
            }
            RETRO_ENVIRONMENT_SET_CONTENT_INFO_OVERRIDE => unsafe {
                capture_content_info_overrides(command, data)
            },
            _ => false,
        }
    }

    unsafe extern "C" fn capture_content_contract_env_rejects_support_no_game(
        command: u32,
        data: *mut c_void,
    ) -> bool {
        match command {
            RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME => {
                captured_support_no_game()
                    .lock()
                    .expect("support-no-game capture mutex poisoned")
                    .push(unsafe { *data.cast::<bool>() });
                false
            }
            RETRO_ENVIRONMENT_SET_CONTENT_INFO_OVERRIDE => unsafe {
                capture_content_info_overrides(command, data)
            },
            _ => false,
        }
    }

    unsafe extern "C" fn capture_message_env(command: u32, data: *mut c_void) -> bool {
        match command {
            RETRO_ENVIRONMENT_SET_MESSAGE => {
                let message = unsafe { *data.cast::<raw::retro_message>() };
                let message_text = if message.msg.is_null() {
                    String::new()
                } else {
                    unsafe { CStr::from_ptr(message.msg) }
                        .to_string_lossy()
                        .into_owned()
                };
                captured_messages()
                    .lock()
                    .expect("message capture mutex poisoned")
                    .push(CapturedMessage {
                        message: message_text,
                        frames: message.frames,
                    });
                true
            }
            raw::RETRO_ENVIRONMENT_GET_MESSAGE_INTERFACE_VERSION => {
                unsafe { *data.cast::<u32>() = 1 };
                true
            }
            raw::RETRO_ENVIRONMENT_SET_MESSAGE_EXT => {
                let message = unsafe { *data.cast::<raw::retro_message_ext>() };
                let message_text = if message.msg.is_null() {
                    String::new()
                } else {
                    unsafe { CStr::from_ptr(message.msg) }
                        .to_string_lossy()
                        .into_owned()
                };
                captured_extended_messages()
                    .lock()
                    .expect("extended message capture mutex poisoned")
                    .push(CapturedExtendedMessage {
                        message: message_text,
                        duration: message.duration,
                        priority: message.priority,
                        level: message.level,
                        target: message.target,
                        kind: message.type_,
                        progress: message.progress,
                    });
                true
            }
            _ => false,
        }
    }

    unsafe extern "C" fn capture_video_refresh(
        data: *const c_void,
        width: u32,
        height: u32,
        pitch: usize,
    ) {
        let data_kind = if data.is_null() {
            CapturedVideoDataKind::Dupe
        } else if data == RETRO_HW_FRAME_BUFFER_VALID {
            CapturedVideoDataKind::Hardware
        } else {
            CapturedVideoDataKind::Software
        };

        captured_video_refreshes()
            .lock()
            .expect("video refresh capture mutex poisoned")
            .push(CapturedVideoRefresh {
                data_kind,
                data_addr: data as usize,
                width,
                height,
                pitch,
            });
    }

    unsafe extern "C" fn fake_current_framebuffer() -> usize {
        99
    }

    unsafe extern "C" fn fake_zero_current_framebuffer() -> usize {
        0
    }

    unsafe extern "C" fn fake_gl_proc() {}

    unsafe extern "C" fn fake_get_proc_address(_sym: *const c_char) -> raw::retro_proc_address_t {
        Some(fake_gl_proc)
    }

    unsafe extern "C" fn missing_get_proc_address(
        _sym: *const c_char,
    ) -> raw::retro_proc_address_t {
        None
    }

    unsafe extern "C" fn capture_hw_render_env(command: u32, data: *mut c_void) -> bool {
        let mut captured = captured_hw_render_state()
            .lock()
            .expect("hw render capture mutex poisoned");

        match command {
            RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER => {
                let out = data.cast::<HwContextType>();
                unsafe { *out = captured.preferred_context_type };
                captured.supports_non_preferred_context
            }
            RETRO_ENVIRONMENT_SET_HW_RENDER => {
                let callback = unsafe { &mut *data.cast::<RawHwRenderCallback>() };
                captured.record_attempt(callback.context_type);
                if !captured.accepts(callback.context_type) {
                    return false;
                }
                if captured.inject_runtime_callbacks {
                    callback.get_current_framebuffer = Some(fake_current_framebuffer);
                    callback.get_proc_address = Some(fake_get_proc_address);
                }
                captured.last_callback = Some(*callback);
                true
            }
            raw::RETRO_ENVIRONMENT_GET_HW_RENDER_INTERFACE => {
                unsafe {
                    *data.cast::<*const raw::retro_hw_render_interface>() =
                        &FRONTEND_HW_RENDER_INTERFACE
                };
                true
            }
            raw::RETRO_ENVIRONMENT_GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT => {
                let Some(version) = captured.context_negotiation_support_version else {
                    return false;
                };
                let interface = unsafe {
                    data.cast::<raw::retro_hw_render_context_negotiation_interface>()
                        .as_mut()
                }
                .expect("HW render context negotiation support data must be non-null");
                assert_eq!(
                    interface.interface_type,
                    raw::retro_hw_render_context_negotiation_interface_type::Vulkan as i32
                );
                interface.interface_version = version;
                true
            }
            raw::RETRO_ENVIRONMENT_SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE => {
                let interface =
                    unsafe { *data.cast::<raw::retro_hw_render_context_negotiation_interface>() };
                captured.last_context_negotiation =
                    Some(HwRenderContextNegotiationInterface::from_raw(interface));
                true
            }
            _ => false,
        }
    }

    unsafe extern "C" fn capture_geometry_env(command: u32, data: *mut c_void) -> bool {
        if command != RETRO_ENVIRONMENT_SET_GEOMETRY {
            return false;
        }

        captured_geometries()
            .lock()
            .expect("geometry capture mutex poisoned")
            .push(GameGeometry::from_raw(unsafe {
                *data.cast::<raw::retro_game_geometry>()
            }));
        true
    }

    #[test]
    fn content_info_overrides_are_forwarded_to_frontend() {
        let _guard = serial_test_guard();
        let mut state = CoreState::default();
        state.callbacks.environment = Some(capture_content_info_overrides);

        let mut env = Environment { state: &mut state };
        let ok = env.set_content_info_overrides(&[ContentInfoOverride {
            extensions: "bin".to_string(),
            need_fullpath: true,
            persistent_data: false,
        }]);

        assert!(ok);
        assert!(env.state.content_info_overrides.is_some());

        let captured = CAPTURED_CONTENT_OVERRIDES
            .get()
            .expect("content overrides were not captured")
            .lock()
            .expect("capture mutex poisoned")
            .clone();
        assert_eq!(
            captured,
            vec![CapturedContentOverride {
                extensions: "bin".to_string(),
                need_fullpath: true,
                persistent_data: false,
            }]
        );
    }

    #[test]
    fn content_contract_does_not_send_false_support_no_game_command() {
        let _guard = serial_test_guard();
        reset_captured_support_no_game();
        let mut state = CoreState::default();
        state.callbacks.environment = Some(capture_content_contract_env_rejects_support_no_game);

        let mut env = Environment { state: &mut state };
        let ok = ContentContract::new("bin")
            .with_need_fullpath(true)
            .register_environment(&mut env);

        assert!(ok);
        assert!(
            captured_support_no_game()
                .lock()
                .expect("support-no-game capture mutex poisoned")
                .is_empty()
        );
        assert!(env.state.content_info_overrides.is_some());
    }

    #[test]
    fn content_contract_sends_true_support_no_game_when_requested() {
        let _guard = serial_test_guard();
        reset_captured_support_no_game();
        let mut state = CoreState::default();
        state.callbacks.environment = Some(capture_content_contract_env);

        let mut env = Environment { state: &mut state };
        let ok = ContentContract::new("bin")
            .with_support_no_game(true)
            .register_environment(&mut env);

        assert!(ok);
        assert_eq!(
            *captured_support_no_game()
                .lock()
                .expect("support-no-game capture mutex poisoned"),
            vec![true]
        );
        assert!(env.state.content_info_overrides.is_some());
    }

    #[test]
    fn set_message_is_forwarded_to_frontend() {
        let _guard = serial_test_guard();
        reset_captured_messages();

        let mut state = CoreState::default();
        state.callbacks.environment = Some(capture_message_env);

        let mut env = Environment { state: &mut state };
        assert!(env.set_message("frontend message", 120));

        assert_eq!(
            *captured_messages()
                .lock()
                .expect("message capture mutex poisoned"),
            vec![CapturedMessage {
                message: "frontend message".to_string(),
                frames: 120,
            }]
        );
    }

    #[test]
    fn extended_message_is_forwarded_to_frontend_with_typed_fields() {
        let _guard = serial_test_guard();
        reset_captured_extended_messages();

        let mut state = CoreState::default();
        state.callbacks.environment = Some(capture_message_env);

        let mut env = Environment { state: &mut state };
        assert_eq!(env.message_interface_version(), Some(1));
        assert!(
            env.set_message_ext(
                ExtendedMessage::new("loading assets")
                    .with_duration_millis(750)
                    .with_priority(3)
                    .with_level(LogLevel::Warn)
                    .with_target(MessageTarget::All)
                    .with_kind(MessageKind::Status)
                    .with_progress(MessageProgress::percent(42).expect("valid progress percent"))
            )
        );

        assert_eq!(
            *captured_extended_messages()
                .lock()
                .expect("extended message capture mutex poisoned"),
            vec![CapturedExtendedMessage {
                message: "loading assets".to_string(),
                duration: 750,
                priority: 3,
                level: LogLevel::Warn,
                target: raw::retro_message_target::All,
                kind: raw::retro_message_type::Progress,
                progress: 42,
            }]
        );
    }

    #[test]
    fn frontend_service_queries_return_rust_values() {
        let _guard = serial_test_guard();
        reset_captured_achievement_support();
        reset_captured_audio_latencies();
        reset_captured_fastforwarding_overrides();
        reset_captured_led_states();
        reset_captured_midi_interface();
        reset_captured_performance_levels();
        reset_captured_rumble_states();
        reset_captured_sensor_states();
        reset_captured_location_interface();
        reset_captured_rotations();
        reset_captured_serialization_quirks();
        reset_captured_shutdowns();
        reset_captured_hw_shared_contexts();
        reset_captured_system_av_infos();
        let mut state = CoreState::default();
        state.callbacks.environment = Some(frontend_services_env);

        let mut env = Environment { state: &mut state };

        assert_eq!(env.overscan(), Some(false));
        assert_eq!(env.can_dupe_frames(), Some(true));
        assert!(env.set_rotation(VideoRotation::CounterClockwise90));
        assert!(env.shutdown());
        assert!(env.set_hw_shared_context());
        assert!(env.set_system_av_info(system_av_info(game_geometry(256, 224), 60.0, 44_100.0)));
        assert_eq!(env.system_directory().as_deref(), Some("/system"));
        assert_eq!(
            env.libretro_path().as_deref(),
            Some("/cores/test_libretro.so")
        );
        assert_eq!(env.core_assets_directory().as_deref(), Some("/assets"));
        assert_eq!(env.content_directory().as_deref(), Some("/assets"));
        assert_eq!(env.save_directory().as_deref(), Some("/saves"));
        assert_eq!(env.username().as_deref(), Some("player"));
        assert_eq!(env.playlist_directory().as_deref(), Some("/playlists"));
        assert_eq!(
            env.file_browser_start_directory().as_deref(),
            Some("/browser")
        );
        assert_eq!(env.language(), Some(Language::PortugueseBrazil));
        assert_eq!(env.jit_capable(), Some(true));
        assert_eq!(
            env.disk_control_interface_version(),
            Some(DiskControlInterfaceVersion::new(1))
        );
        assert!(
            env.disk_control_interface_version()
                .expect("disk control version should be available")
                .supports_extended()
        );
        assert!(env.set_support_achievements(true));
        assert!(env.set_support_achievements(false));
        assert!(env.set_performance_level(PerformanceLevel::new(2)));
        assert_eq!(
            env.device_power(),
            Some(DevicePower {
                state: PowerState::Discharging,
                seconds_remaining: Some(3600),
                percent: Some(72),
            })
        );
        assert_eq!(env.netplay_client_index(), Some(NetplayClientId::new(2)));
        let input_capabilities = env
            .input_device_capabilities()
            .expect("input capabilities query should work");
        assert!(input_capabilities.contains(InputDeviceCapability::Joypad));
        assert!(input_capabilities.contains(InputDeviceCapability::Analog));
        assert!(input_capabilities.contains(InputDeviceCapability::Pointer));
        assert!(!input_capabilities.contains(InputDeviceCapability::Mouse));
        assert!(env.supports_joypad_bitmasks());
        assert_eq!(env.input_max_users(), Some(4));
        let led_interface = env
            .led_interface()
            .expect("LED interface should be available");
        assert!(led_interface.is_available());
        assert!(led_interface.set_state(LedIndex::new(2), LedState::On));
        assert!(led_interface.set_state(2, LedState::Off));
        let rumble_interface = env
            .rumble_interface()
            .expect("rumble interface should be available");
        assert!(rumble_interface.is_available());
        assert!(rumble_interface.set_state(
            InputPort::new(1),
            RumbleEffect::Strong,
            RumbleStrength::max()
        ));
        assert!(!rumble_interface.set_state(1, RumbleEffect::Weak, RumbleStrength::off()));
        let sensor_interface = env
            .sensor_interface()
            .expect("sensor interface should be available");
        assert!(sensor_interface.is_available());
        assert!(sensor_interface.enable(2, Sensor::Accelerometer, SensorRateHz::new(60)));
        assert!(sensor_interface.disable(2, Sensor::Gyroscope));
        assert!(!sensor_interface.enable(2, Sensor::Illuminance, SensorRateHz::new(15)));
        assert_eq!(
            sensor_interface.input(2, SensorInput::GyroscopeZ),
            Some(2.5)
        );
        let location_interface = env
            .location_interface()
            .expect("location interface should be available");
        assert!(location_interface.is_available());
        assert!(location_interface.set_interval(
            LocationIntervalMillis::new(1000),
            LocationIntervalMeters::new(25)
        ));
        assert!(location_interface.start());
        assert_eq!(
            location_interface.position(),
            Some(LocationPosition {
                latitude_degrees: 12.5,
                longitude_degrees: -45.25,
                horizontal_accuracy: 3.0,
                vertical_accuracy: 8.0,
            })
        );
        assert!(location_interface.stop());
        assert!(env.midi_interface_available());
        let midi_interface = env
            .midi_interface()
            .expect("MIDI interface should be available");
        assert!(midi_interface.is_available());
        assert!(midi_interface.input_enabled());
        assert!(midi_interface.output_enabled());
        assert_eq!(midi_interface.read_byte(), Some(0x90));
        assert!(midi_interface.write_byte(0x91, MidiDeltaMicros::new(240)));
        assert!(!midi_interface.write_byte(0, MidiDeltaMicros::new(480)));
        assert!(midi_interface.flush());
        let av_enable = env
            .audio_video_enable()
            .expect("AV enable query should work");
        assert!(av_enable.contains(AvEnable::Video));
        assert!(av_enable.contains(AvEnable::HardDisableAudio));
        assert!(!av_enable.contains(AvEnable::Audio));
        assert_eq!(env.fastforwarding(), Some(true));
        assert!(env.fastforwarding_override_supported());
        assert!(
            env.set_fastforwarding_override(
                FastForwardingOverride::enable()
                    .with_ratio(FastForwardRatio::multiplier(2.5).expect("valid ratio"))
                    .with_notification(false)
                    .with_inhibit_toggle(true)
            )
        );
        assert_eq!(
            env.target_refresh_rate().map(RefreshRateHz::get),
            Some(59.94)
        );
        assert_eq!(
            env.target_sample_rate().map(AudioSampleRateHz::get),
            Some(48_000)
        );
        assert_eq!(
            env.throttle_state(),
            Some(ThrottleState {
                mode: ThrottleMode::FastForward,
                rate: RunLoopRateHz::new(120.0),
            })
        );
        assert_eq!(
            env.savestate_context(),
            Some(SavestateContext::RollbackNetplay)
        );
        assert!(env.set_minimum_audio_latency(Some(AudioLatencyMillis::new(96))));
        assert!(env.set_minimum_audio_latency(None));
        let requested_quirks = SerializationQuirks::from(SerializationQuirk::MustInitialize)
            | SerializationQuirk::CoreVariableSize
            | SerializationQuirk::PlatformDependent;
        let supported_quirks = env
            .set_serialization_quirks(requested_quirks)
            .expect("serialization quirks should be supported");
        assert!(supported_quirks.contains(SerializationQuirk::MustInitialize));
        assert!(supported_quirks.contains(SerializationQuirk::PlatformDependent));
        assert!(!supported_quirks.contains(SerializationQuirk::CoreVariableSize));
        assert_eq!(
            *captured_audio_latencies()
                .lock()
                .expect("audio latency capture mutex poisoned"),
            vec![Some(96), None]
        );
        assert_eq!(
            *captured_fastforwarding_overrides()
                .lock()
                .expect("fastforwarding override capture mutex poisoned"),
            vec![
                None,
                Some(raw::retro_fastforwarding_override {
                    ratio: 2.5,
                    fastforward: true,
                    notification: false,
                    inhibit_toggle: true,
                }),
            ]
        );
        assert_eq!(
            *captured_led_states()
                .lock()
                .expect("LED state capture mutex poisoned"),
            vec![(2, 1), (2, 0)]
        );
        assert_eq!(
            *captured_rumble_states()
                .lock()
                .expect("rumble state capture mutex poisoned"),
            vec![
                (1, raw::retro_rumble_effect::Strong, u16::MAX),
                (1, raw::retro_rumble_effect::Weak, 0),
            ]
        );
        assert_eq!(
            *captured_sensor_states()
                .lock()
                .expect("sensor state capture mutex poisoned"),
            vec![
                (
                    2,
                    raw::retro_sensor_action::AccelerometerEnable,
                    SensorRateHz::new(60).as_raw(),
                ),
                (2, raw::retro_sensor_action::GyroscopeDisable, 0),
                (2, raw::retro_sensor_action::IlluminanceEnable, 15),
            ]
        );
        assert_eq!(
            *captured_location_intervals()
                .lock()
                .expect("location interval capture mutex poisoned"),
            vec![(1000, 25)]
        );
        assert_eq!(
            *captured_location_starts()
                .lock()
                .expect("location start capture mutex poisoned"),
            1
        );
        assert_eq!(
            *captured_location_stops()
                .lock()
                .expect("location stop capture mutex poisoned"),
            1
        );
        assert_eq!(
            *captured_midi_probes()
                .lock()
                .expect("MIDI probe capture mutex poisoned"),
            1
        );
        assert_eq!(
            *captured_midi_writes()
                .lock()
                .expect("MIDI write capture mutex poisoned"),
            vec![(0x91, 240), (0, 480)]
        );
        assert_eq!(
            *captured_midi_flushes()
                .lock()
                .expect("MIDI flush capture mutex poisoned"),
            1
        );
        assert_eq!(
            *captured_rotations()
                .lock()
                .expect("rotation capture mutex poisoned"),
            vec![VideoRotation::CounterClockwise90.as_raw()]
        );
        assert_eq!(
            *captured_shutdowns()
                .lock()
                .expect("shutdown capture mutex poisoned"),
            1
        );
        assert_eq!(
            *captured_hw_shared_contexts()
                .lock()
                .expect("HW shared context capture mutex poisoned"),
            1
        );
        let captured_av_infos = captured_system_av_infos()
            .lock()
            .expect("system av info capture mutex poisoned")
            .clone();
        assert_eq!(captured_av_infos.len(), 1);
        assert_eq!(captured_av_infos[0].geometry.base_width, 256);
        assert_eq!(captured_av_infos[0].geometry.base_height, 224);
        assert_eq!(captured_av_infos[0].timing.fps, 60.0);
        assert_eq!(captured_av_infos[0].timing.sample_rate, 44_100.0);
        assert_eq!(
            *captured_achievement_support()
                .lock()
                .expect("achievement support capture mutex poisoned"),
            vec![true, false]
        );
        assert_eq!(
            *captured_performance_levels()
                .lock()
                .expect("performance level capture mutex poisoned"),
            vec![2]
        );
        assert_eq!(
            *captured_serialization_quirks()
                .lock()
                .expect("serialization quirk capture mutex poisoned"),
            vec![requested_quirks.bits()]
        );
    }

    #[test]
    fn frontend_string_query_returns_none_for_available_null_value() {
        let _guard = serial_test_guard();
        let mut state = CoreState::default();
        state.callbacks.environment = Some(null_frontend_string_env);

        let mut env = Environment { state: &mut state };

        assert_eq!(env.save_directory(), None);
    }

    #[test]
    fn core_options_v2_registration_display_and_single_setter_are_forwarded() {
        let _guard = serial_test_guard();
        reset_captured_core_options();
        let mut state = CoreState::default();
        state.callbacks.environment = Some(core_options_env);
        let mut env = Environment { state: &mut state };

        let options = CoreOptions::new([CoreOptionDefinition::new(
            "demo_renderer",
            "Renderer",
            "gl",
        )
        .with_category("video")
        .with_categorized_description("API")
        .with_info("Selects the renderer")
        .with_categorized_info("Rendering API")
        .with_values([
            CoreOptionValue::new("gl").with_label("OpenGL"),
            CoreOptionValue::new("soft").with_label("Software"),
        ])])
        .with_categories([CoreOptionCategory::new("video", "Video").with_info("Video settings")]);

        assert_eq!(env.core_options_version(), CoreOptionsVersion::V2);
        assert!(
            env.set_core_options(&options)
                .expect("core options should build")
        );
        assert!(env.set_core_option_display(CoreOptionDisplay::new("demo_renderer", true)));
        assert!(env.set_variable("demo_renderer", Some("soft")));

        let captured = captured_core_options_v2()
            .lock()
            .expect("core options v2 capture mutex poisoned")
            .clone()
            .expect("core options v2 should be captured");
        assert_eq!(
            captured.categories,
            vec![CapturedCoreOptionCategory {
                key: "video".to_string(),
                description: "Video".to_string(),
                info: Some("Video settings".to_string()),
            }]
        );
        assert_eq!(
            captured.definitions,
            vec![CapturedCoreOptionDefinition {
                key: "demo_renderer".to_string(),
                description: "Renderer".to_string(),
                description_categorized: Some("API".to_string()),
                info: Some("Selects the renderer".to_string()),
                info_categorized: Some("Rendering API".to_string()),
                category_key: Some("video".to_string()),
                values: vec![
                    CapturedCoreOptionValue {
                        value: "gl".to_string(),
                        label: Some("OpenGL".to_string()),
                    },
                    CapturedCoreOptionValue {
                        value: "soft".to_string(),
                        label: Some("Software".to_string()),
                    },
                ],
                default_value: "gl".to_string(),
            }]
        );
        assert_eq!(
            *captured_core_option_displays()
                .lock()
                .expect("core option display capture mutex poisoned"),
            vec![CapturedCoreOptionDisplay {
                key: "demo_renderer".to_string(),
                visible: true,
            }]
        );
        assert_eq!(
            *captured_variables()
                .lock()
                .expect("variable capture mutex poisoned"),
            vec![CapturedVariable {
                key: "demo_renderer".to_string(),
                value: Some("soft".to_string()),
            }]
        );
    }

    #[test]
    fn core_options_legacy_v1_and_intl_paths_are_forwarded() {
        let _guard = serial_test_guard();
        reset_captured_core_options();
        let definition = CoreOptionDefinition::new("demo_speed", "Speed", "normal").with_values([
            CoreOptionValue::new("slow"),
            CoreOptionValue::new("normal"),
            CoreOptionValue::new("fast"),
        ]);
        let options = CoreOptions::new([definition.clone()]);

        let mut state = CoreState::default();
        state.callbacks.environment = Some(core_options_env);
        let mut env = Environment { state: &mut state };

        *captured_core_options_version()
            .lock()
            .expect("core options version capture mutex poisoned") = Some(1);
        assert!(
            env.set_core_options(&options)
                .expect("v1 core options should build")
        );
        assert!(
            env.set_core_options_v1_intl(
                &[definition.clone()],
                Some(std::slice::from_ref(&definition))
            )
            .expect("v1 intl core options should build")
        );
        assert!(
            env.set_core_options_v2_intl(&options, Some(&options))
                .expect("v2 intl core options should build")
        );

        *captured_core_options_version()
            .lock()
            .expect("core options version capture mutex poisoned") = None;
        assert_eq!(
            env.core_options_version(),
            CoreOptionsVersion::LEGACY_VARIABLES
        );
        assert!(
            env.set_core_options_legacy(&options)
                .expect("legacy core options should build")
        );

        assert_eq!(
            captured_core_options_v1()
                .lock()
                .expect("core options v1 capture mutex poisoned")
                .first()
                .expect("v1 option should be captured")
                .key,
            "demo_speed"
        );
        assert_eq!(
            *captured_variables()
                .lock()
                .expect("variable capture mutex poisoned"),
            vec![CapturedVariable {
                key: "demo_speed".to_string(),
                value: Some("Speed; normal|slow|fast".to_string()),
            }]
        );
    }

    #[test]
    fn core_options_update_display_callback_dispatches_to_core() {
        let _guard = serial_test_guard();
        reset_captured_core_options();
        let calls = Arc::new(Mutex::new(Vec::new()));
        install_global_test_core(CoreOptionsDisplayRecordingCore {
            calls: Arc::clone(&calls),
        });

        with_state(|state| {
            state.callbacks.environment = Some(core_options_env);
            let mut env = Environment { state };
            assert!(env.set_core_options_update_display_callback());
        });

        let callback = captured_core_options_update_display_callback()
            .lock()
            .expect("core options update display callback capture mutex poisoned")
            .expect("core options update display callback should be registered")
            .callback
            .expect("core options update display function should be set");
        assert!(unsafe { callback() });

        assert_eq!(
            *calls
                .lock()
                .expect("core options display calls mutex poisoned"),
            vec!["update"]
        );
        assert_eq!(
            *captured_core_option_displays()
                .lock()
                .expect("core option display capture mutex poisoned"),
            vec![CapturedCoreOptionDisplay {
                key: "demo_extra".to_string(),
                visible: false,
            }]
        );

        clear_global_test_core();
    }

    #[test]
    fn vfs_interface_wraps_file_directory_and_path_operations() {
        let _guard = serial_test_guard();
        reset_captured_vfs_interface();
        let mut state = CoreState::default();
        state.callbacks.environment = Some(vfs_env);
        let mut env = Environment { state: &mut state };

        let vfs = env
            .vfs_interface(VfsInterfaceVersion::new(3))
            .expect("VFS interface should be available");
        assert_eq!(vfs.version(), VfsInterfaceVersion::new(3));

        let access = VfsFileAccessFlags::from(VfsFileAccess::Read)
            | VfsFileAccess::Write
            | VfsFileAccess::UpdateExisting;
        let hints = VfsFileAccessHints::from(VfsFileAccessHint::FrequentAccess);
        let mut file = vfs
            .open_file("/tmp/test.bin", access, hints)
            .expect("file should open");

        assert_eq!(file.path().as_deref(), Some("/tmp/test.bin"));
        assert_eq!(file.size(), Some(8));
        assert_eq!(file.tell(), Some(3));
        assert_eq!(file.seek(-2, VfsSeekPosition::End), Some(6));
        let mut bytes = [0u8; 4];
        assert_eq!(file.read(&mut bytes), Some(3));
        assert_eq!(&bytes[..3], b"abc");
        assert_eq!(file.write(b"xyz"), Some(3));
        assert!(file.truncate(4));
        assert!(file.flush());
        assert!(file.close());

        let stat = vfs.stat("/tmp/test.bin").expect("path should stat");
        assert!(stat.flags.contains(VfsStatFlag::Valid));
        assert!(stat.flags.contains(VfsStatFlag::Directory));
        assert_eq!(stat.size, Some(8));
        assert!(vfs.remove_file("/tmp/remove.bin"));
        assert!(vfs.rename("/tmp/old.bin", "/tmp/new.bin"));
        assert!(vfs.create_dir("/tmp/new-dir"));

        let mut dir = vfs.open_dir("/tmp", true).expect("directory should open");
        assert!(dir.read_next());
        assert_eq!(dir.entry_name().as_deref(), Some("entry.bin"));
        assert!(!dir.entry_is_dir());
        assert!(!dir.read_next());
        assert!(dir.close());

        assert_eq!(
            *captured_vfs_interface_requests()
                .lock()
                .expect("VFS request capture mutex poisoned"),
            vec![3]
        );
        assert_eq!(
            *captured_vfs_opens()
                .lock()
                .expect("VFS open capture mutex poisoned"),
            vec![CapturedVfsOpen {
                path: "/tmp/test.bin".to_string(),
                mode: access.bits(),
                hints: hints.bits(),
            }]
        );
        assert_eq!(
            *captured_vfs_writes()
                .lock()
                .expect("VFS write capture mutex poisoned"),
            vec![b"xyz".to_vec()]
        );
        assert_eq!(
            *captured_vfs_removes()
                .lock()
                .expect("VFS remove capture mutex poisoned"),
            vec!["/tmp/remove.bin".to_string()]
        );
        assert_eq!(
            *captured_vfs_renames()
                .lock()
                .expect("VFS rename capture mutex poisoned"),
            vec![CapturedVfsRename {
                old_path: "/tmp/old.bin".to_string(),
                new_path: "/tmp/new.bin".to_string(),
            }]
        );
        assert_eq!(
            *captured_vfs_mkdirs()
                .lock()
                .expect("VFS mkdir capture mutex poisoned"),
            vec!["/tmp/new-dir".to_string()]
        );
        assert_eq!(
            *captured_vfs_closes()
                .lock()
                .expect("VFS close capture mutex poisoned"),
            1
        );
        assert_eq!(
            *captured_vfs_dir_closes()
                .lock()
                .expect("VFS dir close capture mutex poisoned"),
            1
        );
    }

    #[test]
    fn perf_interface_wraps_frontend_counter_callbacks() {
        let _guard = serial_test_guard();
        reset_captured_perf_interface();
        let mut state = CoreState::default();
        state.callbacks.environment = Some(frontend_services_env);

        let mut env = Environment { state: &mut state };
        let perf = env
            .perf_interface()
            .expect("perf interface should be available");

        assert_eq!(
            perf.time_micros(),
            Some(PerfTimeMicros::from_micros(123_456))
        );
        assert_eq!(perf.tick_counter(), Some(PerfTick::from_ticks(9_001)));
        let features = perf
            .cpu_features()
            .expect("CPU features should be available");
        assert!(features.contains(CpuFeature::Sse2));
        assert!(features.contains(CpuFeature::Neon));
        assert!(!features.contains(CpuFeature::Avx2));

        let mut counter = PerfCounter::new("hot\0path");
        assert_eq!(
            counter
                .as_ref()
                .get_ref()
                .ident()
                .to_str()
                .expect("counter identifier should be utf-8"),
            "hotpath"
        );
        assert!(perf.register_counter(counter.as_mut()));
        assert!(counter.as_ref().get_ref().is_registered());
        assert!(perf.start_counter(counter.as_mut()));
        assert_eq!(
            counter.as_ref().get_ref().last_start(),
            PerfTick::from_ticks(9_001)
        );
        assert_eq!(counter.as_ref().get_ref().call_count(), 1);
        assert!(perf.stop_counter(counter.as_mut()));
        assert_eq!(
            counter.as_ref().get_ref().total(),
            PerfTick::from_ticks(377)
        );
        assert!(perf.log());

        assert_eq!(
            *captured_perf_registered_idents()
                .lock()
                .expect("perf registered idents mutex poisoned"),
            vec!["hotpath".to_string()]
        );
        assert_eq!(
            *captured_perf_logs()
                .lock()
                .expect("perf log capture mutex poisoned"),
            1
        );
    }

    #[test]
    fn input_descriptors_are_forwarded_with_retained_strings() {
        let _guard = serial_test_guard();
        reset_captured_input_descriptors();
        let mut state = CoreState::default();
        state.callbacks.environment = Some(frontend_services_env);

        let mut env = Environment { state: &mut state };
        assert!(env.set_input_descriptors(&[
            InputDescriptor::joypad(0, JoypadButton::A, "Jump\0button"),
            InputDescriptor::analog(1, AnalogStick::Left, AnalogAxis::X, "Move X"),
        ]));

        assert_eq!(
            *captured_input_descriptors()
                .lock()
                .expect("input descriptor capture mutex poisoned"),
            vec![
                CapturedInputDescriptor {
                    port: 0,
                    device: ControllerDevice::Joypad.as_raw(),
                    index: 0,
                    id: JoypadButton::A.as_raw(),
                    description: "Jumpbutton".to_string(),
                },
                CapturedInputDescriptor {
                    port: 1,
                    device: ControllerDevice::Analog.as_raw(),
                    index: AnalogStick::Left.as_raw(),
                    id: AnalogAxis::X.as_raw(),
                    description: "Move X".to_string(),
                },
            ]
        );
        assert!(env.state.input_descriptors.is_some());
    }

    #[test]
    fn controller_info_is_forwarded_with_typed_devices() {
        let _guard = serial_test_guard();
        reset_captured_controller_info();
        let mut state = CoreState::default();
        state.callbacks.environment = Some(frontend_services_env);
        let lightgun_scope = ControllerDeviceSubclass::new(ControllerDevice::Lightgun, 0)
            .expect("lightgun subclass should be valid");

        let mut env = Environment { state: &mut state };
        assert!(env.set_controller_info(&[
            ControllerInfo::new(vec![
                ControllerDescription::new("Gamepad\0Default", ControllerDevice::Joypad),
                ControllerDescription::new(
                    "Lightgun Scope",
                    ControllerDevice::Subclass(lightgun_scope),
                ),
            ]),
            ControllerInfo::new(vec![ControllerDescription::new(
                "Mouse",
                ControllerDevice::Mouse,
            )]),
        ]));

        assert_eq!(
            *captured_controller_info()
                .lock()
                .expect("controller info capture mutex poisoned"),
            vec![
                vec![
                    CapturedControllerDescription {
                        description: "GamepadDefault".to_string(),
                        id: ControllerDevice::Joypad.as_raw(),
                    },
                    CapturedControllerDescription {
                        description: "Lightgun Scope".to_string(),
                        id: lightgun_scope.as_raw(),
                    },
                ],
                vec![CapturedControllerDescription {
                    description: "Mouse".to_string(),
                    id: ControllerDevice::Mouse.as_raw(),
                }],
            ]
        );

        assert!(!env.set_controller_info(&[ControllerInfo::default()]));
    }

    #[test]
    fn memory_maps_are_forwarded_with_typed_descriptors() {
        let _guard = serial_test_guard();
        reset_captured_memory_descriptors();
        let mut state = CoreState::default();
        state.callbacks.environment = Some(frontend_services_env);
        let mut wram = [0u8; 16];

        let descriptors = [
            MemoryMapDescriptor::from_slice(Some("WRAM\0".to_string()), 0x7e0000usize, &mut wram)
                .with_flags(MemoryDescriptorFlags::from(MemoryDescriptorFlag::SystemRam))
                .with_alignment(MemoryDescriptorAlignment::FourBytes)
                .with_min_access_size(MemoryDescriptorMinAccessSize::TwoBytes)
                .with_offset(MemoryMapOffset::new(2))
                .with_select(MemoryMapMask::new(0xff0000))
                .with_disconnect(MemoryMapMask::new(0x80))
                .with_len(MemoryMapLen::new(8)),
            MemoryMapDescriptor::new_inaccessible(
                None,
                EmulatedAddress::new(0xffffff),
                MemoryMapMask::new(0xffffff),
            ),
        ];

        let mut env = Environment { state: &mut state };
        assert!(env.set_memory_maps(&descriptors));

        assert_eq!(
            *captured_memory_descriptors()
                .lock()
                .expect("memory descriptor capture mutex poisoned"),
            vec![
                CapturedMemoryDescriptor {
                    flags: raw::RETRO_MEMDESC_SYSTEM_RAM
                        | raw::RETRO_MEMDESC_ALIGN_4
                        | raw::RETRO_MEMDESC_MINSIZE_2,
                    ptr_is_null: false,
                    offset: 2,
                    start: 0x7e0000,
                    select: 0xff0000,
                    disconnect: 0x80,
                    len: 8,
                    addrspace: Some("WRAM".to_string()),
                },
                CapturedMemoryDescriptor {
                    flags: 0,
                    ptr_is_null: true,
                    offset: 0,
                    start: 0xffffff,
                    select: 0xffffff,
                    disconnect: 0,
                    len: 0,
                    addrspace: None,
                },
            ]
        );
    }

    #[test]
    fn subsystem_info_is_forwarded_with_retained_nested_descriptors() {
        let _guard = serial_test_guard();
        reset_captured_subsystem_info();
        let mut state = CoreState::default();
        state.callbacks.environment = Some(frontend_services_env);

        let mut env = Environment { state: &mut state };
        assert!(
            env.set_subsystem_info(&[SubsystemInfo::new(
                "Super Game Boy\0",
                "sgb",
                SubsystemId::new(7),
            )
            .with_roms([
                SubsystemRomInfo::new("Game Boy ROM", "gb|gbc")
                    .with_memory([SubsystemMemoryInfo::new("sav", 0x101)]),
                SubsystemRomInfo::new("BIOS", "bin")
                    .with_need_fullpath(true)
                    .with_block_extract(true)
                    .with_required(false),
            ])])
        );

        assert_eq!(
            *captured_subsystem_info()
                .lock()
                .expect("subsystem info capture mutex poisoned"),
            vec![CapturedSubsystem {
                description: "Super Game Boy".to_string(),
                identifier: "sgb".to_string(),
                id: 7,
                roms: vec![
                    CapturedSubsystemRom {
                        description: "Game Boy ROM".to_string(),
                        valid_extensions: "gb|gbc".to_string(),
                        need_fullpath: false,
                        block_extract: false,
                        required: true,
                        memory: vec![CapturedSubsystemMemory {
                            extension: "sav".to_string(),
                            memory_type: 0x101,
                        }],
                    },
                    CapturedSubsystemRom {
                        description: "BIOS".to_string(),
                        valid_extensions: "bin".to_string(),
                        need_fullpath: true,
                        block_extract: true,
                        required: false,
                        memory: Vec::new(),
                    },
                ],
            }]
        );
        assert!(env.state.subsystem_info.is_some());
    }

    #[test]
    fn extended_game_info_query_returns_borrowed_typed_views() {
        let _guard = serial_test_guard();
        let mut state = CoreState::default();
        state.callbacks.environment = Some(frontend_services_env);
        let mut env = Environment { state: &mut state };

        {
            let infos = env
                .extended_game_infos(2)
                .expect("extended game info should be available");
            assert_eq!(infos.len(), 2);
            assert_eq!(infos[0].full_path, Some(c"/games/test.sfc"));
            assert_eq!(infos[0].archive_path, None);
            assert_eq!(infos[0].dir, Some(c"/games"));
            assert_eq!(infos[0].name, Some(c"test"));
            assert_eq!(infos[0].extension, Some(c"sfc"));
            assert_eq!(infos[0].meta, Some(c"plain"));
            assert_eq!(infos[0].data, Some(EXTENDED_GAME_CONTENT));
            assert!(!infos[0].file_in_archive);
            assert!(infos[0].persistent_data);

            assert_eq!(infos[1].full_path, None);
            assert_eq!(infos[1].archive_path, Some(c"/games/archive.zip"));
            assert_eq!(infos[1].archive_file, Some(c"inside.bin"));
            assert_eq!(infos[1].data, None);
            assert!(infos[1].file_in_archive);
            assert!(!infos[1].persistent_data);
        }

        let info = env
            .extended_game_info()
            .expect("single extended game info should be available");
        assert_eq!(info.name, Some(c"test"));
    }

    #[test]
    fn software_framebuffer_request_returns_typed_view_and_submits_frontend_buffer() {
        let _guard = serial_test_guard();
        reset_software_framebuffer_pixels();
        reset_captured_video_refreshes();
        let mut state = CoreState::default();
        state.callbacks.environment = Some(frontend_services_env);
        state.callbacks.video_refresh = Some(capture_video_refresh);

        let mut runtime = Runtime { state: &mut state };
        let request = SoftwareFramebufferRequest::new(4, 2).with_access(
            FramebufferMemoryAccessFlags::from(FramebufferMemoryAccess::Write)
                | FramebufferMemoryAccess::Read,
        );
        let mut framebuffer = runtime
            .environment()
            .current_software_framebuffer(request)
            .expect("frontend should provide a software framebuffer");

        assert_eq!(framebuffer.width(), 4);
        assert_eq!(framebuffer.height(), 2);
        assert_eq!(framebuffer.pitch(), 4 * mem::size_of::<u32>());
        assert_eq!(framebuffer.format(), PixelFormat::Xrgb8888);
        assert_eq!(
            framebuffer.access(),
            FramebufferMemoryAccessFlags::from(FramebufferMemoryAccess::Write)
                | FramebufferMemoryAccess::Read
        );
        assert_eq!(
            framebuffer.memory(),
            FramebufferMemoryTypes::from(FramebufferMemoryType::Cached)
        );

        framebuffer
            .bytes_mut()
            .expect("write access should expose writable bytes")
            .fill(0x5a);
        runtime.video_refresh_software_framebuffer(framebuffer);

        let pixels = software_framebuffer_pixels()
            .lock()
            .expect("software framebuffer pixels mutex poisoned");
        assert_eq!(*pixels, vec![0x5a5a5a5a; 8]);

        assert_eq!(
            *captured_video_refreshes()
                .lock()
                .expect("video refresh capture mutex poisoned"),
            vec![CapturedVideoRefresh {
                data_kind: CapturedVideoDataKind::Software,
                data_addr: pixels.as_ptr() as usize,
                width: 4,
                height: 2,
                pitch: 4 * mem::size_of::<u32>(),
            }]
        );
    }

    #[test]
    fn keyboard_callback_dispatches_typed_events_to_core() {
        let _guard = serial_test_guard();
        reset_captured_keyboard_callback();
        let calls = Arc::new(Mutex::new(Vec::new()));
        install_global_test_core(KeyboardRecordingCore {
            calls: Arc::clone(&calls),
        });

        __private::retro_set_environment(Some(frontend_services_env));

        let callback = captured_keyboard_callback()
            .lock()
            .expect("keyboard callback capture mutex poisoned")
            .expect("keyboard callback should be registered")
            .callback
            .expect("keyboard callback function should be set");
        unsafe {
            callback(
                true,
                KeyboardKey::A.as_raw(),
                'a' as u32,
                (KeyboardModifiers::from(KeyboardModifier::Shift) | KeyboardModifier::Ctrl).bits(),
            )
        };
        unsafe { callback(false, 65_535, 0, 0) };

        assert_eq!(
            *calls.lock().expect("keyboard event calls mutex poisoned"),
            vec![
                KeyboardEvent::new(
                    true,
                    KeyboardKey::A,
                    KeyboardCharacter::from_utf32('a' as u32),
                    KeyboardModifiers::from(KeyboardModifier::Shift) | KeyboardModifier::Ctrl,
                ),
                KeyboardEvent::new(
                    false,
                    KeyboardKey::UnknownKeycode(65_535),
                    KeyboardCharacter::from_utf32(0),
                    KeyboardModifiers::empty(),
                ),
            ]
        );

        clear_global_test_core();
    }

    #[test]
    fn configured_event_listeners_auto_register_frontend_callbacks() {
        let _guard = serial_test_guard();
        reset_captured_keyboard_callback();
        reset_captured_audio_callback();
        reset_captured_audio_buffer_status_callback();
        reset_captured_frame_time_callback();
        let keyboard_calls = Arc::new(Mutex::new(Vec::new()));
        install_global_test_core(ConfiguredEventCore {
            keyboard_calls: Arc::clone(&keyboard_calls),
        });

        __private::retro_set_environment(Some(frontend_services_env));

        let keyboard_callback = captured_keyboard_callback()
            .lock()
            .expect("keyboard callback capture mutex poisoned")
            .expect("keyboard callback should be registered")
            .callback
            .expect("keyboard callback function should be set");
        assert!(
            captured_audio_callback()
                .lock()
                .expect("audio callback capture mutex poisoned")
                .expect("audio callback should be registered")
                .callback
                .is_some()
        );
        assert!(
            captured_audio_buffer_status_callback()
                .lock()
                .expect("audio buffer status callback capture mutex poisoned")
                .expect("audio buffer status callback should be registered")
                .callback
                .is_some()
        );
        assert_eq!(
            captured_frame_time_callback()
                .lock()
                .expect("frame time callback capture mutex poisoned")
                .expect("frame time callback should be registered")
                .reference,
            16_667
        );

        unsafe { keyboard_callback(true, KeyboardKey::Return.as_raw(), '\n' as u32, 0) };
        assert_eq!(
            *keyboard_calls
                .lock()
                .expect("keyboard event calls mutex poisoned"),
            vec![KeyboardEvent::new(
                true,
                KeyboardKey::Return,
                KeyboardCharacter::from_utf32('\n' as u32),
                KeyboardModifiers::empty(),
            )]
        );

        clear_global_test_core();
    }

    #[test]
    fn event_listeners_dispatch_in_order_and_remove_by_callback() {
        let _guard = serial_test_guard();
        reset_captured_keyboard_callback();
        let calls = Arc::new(Mutex::new(Vec::new()));
        install_global_test_core(MultiKeyboardListenerCore {
            calls: Arc::clone(&calls),
        });

        __private::retro_set_environment(Some(frontend_services_env));

        let keyboard_callback = captured_keyboard_callback()
            .lock()
            .expect("keyboard callback capture mutex poisoned")
            .expect("keyboard callback should be registered")
            .callback
            .expect("keyboard callback function should be set");

        unsafe { keyboard_callback(true, KeyboardKey::Return.as_raw(), '\n' as u32, 0) };
        assert_eq!(
            *calls
                .lock()
                .expect("multi keyboard listener calls mutex poisoned"),
            vec!["first", "third"]
        );

        clear_global_test_core();
    }

    #[test]
    fn audio_callback_probe_register_clear_and_dispatch_work() {
        let _guard = serial_test_guard();
        reset_captured_audio_callback();
        let calls = Arc::new(Mutex::new(Vec::new()));
        install_global_test_core(AudioCallbackRecordingCore {
            calls: Arc::clone(&calls),
        });

        with_state(|state| {
            state.callbacks.environment = Some(frontend_services_env);
            let mut env = Environment { state };
            assert!(env.audio_callback_available());
        });
        __private::retro_set_environment(Some(frontend_services_env));
        assert_eq!(
            *captured_audio_callback_probes()
                .lock()
                .expect("audio callback probe mutex poisoned"),
            1
        );

        let callback = captured_audio_callback()
            .lock()
            .expect("audio callback capture mutex poisoned")
            .expect("audio callback should be registered");

        unsafe {
            callback
                .set_state
                .expect("audio callback set_state function should be set")(true);
            callback
                .callback
                .expect("audio callback function should be set")();
            callback
                .set_state
                .expect("audio callback set_state function should be set")(false);
        }

        assert_eq!(
            *calls.lock().expect("audio callback calls mutex poisoned"),
            vec![
                AudioCallbackEvent::State(AudioCallbackState::Active),
                AudioCallbackEvent::Request,
                AudioCallbackEvent::State(AudioCallbackState::Inactive),
            ]
        );

        with_state(|state| {
            state.callbacks.environment = Some(frontend_services_env);
            let mut env = Environment { state };
            assert!(env.clear_audio_callback());
        });
        let callback = captured_audio_callback()
            .lock()
            .expect("audio callback capture mutex poisoned")
            .expect("audio callback clear should send a non-null empty interface");
        assert!(callback.callback.is_none());
        assert!(callback.set_state.is_none());

        clear_global_test_core();
    }

    #[test]
    fn audio_buffer_status_callback_dispatches_to_core() {
        let _guard = serial_test_guard();
        reset_captured_audio_buffer_status_callback();
        let calls = Arc::new(Mutex::new(Vec::new()));
        install_global_test_core(AudioBufferStatusRecordingCore {
            calls: Arc::clone(&calls),
        });

        __private::retro_set_environment(Some(frontend_services_env));

        let callback = captured_audio_buffer_status_callback()
            .lock()
            .expect("audio buffer status callback capture mutex poisoned")
            .expect("audio buffer status callback should be registered")
            .callback
            .expect("audio buffer status callback function should be set");
        unsafe { callback(true, 75, false) };
        unsafe { callback(false, 150, true) };

        assert_eq!(
            *calls
                .lock()
                .expect("audio buffer status calls mutex poisoned"),
            vec![
                AudioBufferStatus::new(
                    true,
                    AudioBufferOccupancy::from_percent(75).expect("valid occupancy"),
                    false,
                ),
                AudioBufferStatus::from_raw(false, 150, true),
            ]
        );

        with_state(|state| {
            state.callbacks.environment = Some(frontend_services_env);
            let mut env = Environment { state };
            assert!(env.set_audio_buffer_status_callback(false));
        });
        assert!(
            captured_audio_buffer_status_callback()
                .lock()
                .expect("audio buffer status callback capture mutex poisoned")
                .is_none()
        );

        clear_global_test_core();
    }

    #[test]
    fn frame_time_callback_dispatches_to_core() {
        let _guard = serial_test_guard();
        reset_captured_frame_time_callback();
        let calls = Arc::new(Mutex::new(Vec::new()));
        install_global_test_core(FrameTimeRecordingCore {
            calls: Arc::clone(&calls),
        });

        __private::retro_set_environment(Some(frontend_services_env));

        let callback = captured_frame_time_callback()
            .lock()
            .expect("frame time callback capture mutex poisoned")
            .expect("frame time callback should be registered");
        assert_eq!(callback.reference, 16_667);

        let callback = callback
            .callback
            .expect("frame time callback function should be set");
        unsafe { callback(17_000) };
        unsafe { callback(-1) };

        assert_eq!(
            *calls.lock().expect("frame time calls mutex poisoned"),
            vec![FrameTime::from_micros(17_000), FrameTime::from_micros(-1)]
        );

        with_state(|state| {
            state.callbacks.environment = Some(frontend_services_env);
            let mut env = Environment { state };
            assert!(env.clear_frame_time_callback());
        });
        assert!(
            captured_frame_time_callback()
                .lock()
                .expect("frame time callback capture mutex poisoned")
                .is_none()
        );

        clear_global_test_core();
    }

    #[test]
    fn frame_time_callback_set_replaces_previous_callback() {
        let _guard = serial_test_guard();
        reset_captured_frame_time_callback();
        let calls = Arc::new(Mutex::new(Vec::new()));
        install_global_test_core(FrameTimeReplacementCore {
            calls: Arc::clone(&calls),
        });

        __private::retro_set_environment(Some(frontend_services_env));

        let callback = captured_frame_time_callback()
            .lock()
            .expect("frame time callback capture mutex poisoned")
            .expect("frame time callback should be registered");
        assert_eq!(callback.reference, 2_000);

        let callback = callback
            .callback
            .expect("frame time callback function should be set");
        unsafe { callback(2_100) };

        assert_eq!(
            *calls
                .lock()
                .expect("frame time replacement calls mutex poisoned"),
            vec!["second"]
        );

        clear_global_test_core();
    }

    #[test]
    fn frame_time_callback_can_be_cleared_during_event_configuration() {
        let _guard = serial_test_guard();
        reset_captured_frame_time_callback();
        install_global_test_core(FrameTimeClearedCore);

        __private::retro_set_environment(Some(frontend_services_env));

        assert!(
            captured_frame_time_callback()
                .lock()
                .expect("frame time callback capture mutex poisoned")
                .is_none()
        );

        clear_global_test_core();
    }

    #[test]
    fn proc_address_callback_dispatches_symbol_lookup_to_core() {
        let _guard = serial_test_guard();
        reset_captured_proc_address_interface();
        let calls = Arc::new(Mutex::new(Vec::new()));
        install_global_test_core(ProcAddressRecordingCore {
            calls: Arc::clone(&calls),
        });

        with_state(|state| {
            state.callbacks.environment = Some(frontend_services_env);
            let mut env = Environment { state };
            assert!(env.set_proc_address_callback());
        });

        let callback = captured_proc_address_interface()
            .lock()
            .expect("proc address interface capture mutex poisoned")
            .expect("proc address interface should be registered")
            .get_proc_address
            .expect("proc address callback should be set");

        let found = unsafe { callback(c"test_extension_proc".as_ptr()) }
            .expect("known extension should be returned");
        let missing = unsafe { callback(c"missing_extension".as_ptr()) };
        let expected: unsafe extern "C" fn() = test_extension_proc;

        assert!(std::ptr::fn_addr_eq(found, expected));
        assert!(missing.is_none());
        assert_eq!(
            *calls.lock().expect("proc address calls mutex poisoned"),
            vec![
                "test_extension_proc".to_string(),
                "missing_extension".to_string(),
            ]
        );

        clear_global_test_core();
    }

    #[test]
    fn location_lifetime_callbacks_dispatch_to_core() {
        let _guard = serial_test_guard();
        reset_captured_location_interface();
        let calls = Arc::new(Mutex::new(Vec::new()));
        install_global_test_core(LocationRecordingCore {
            calls: Arc::clone(&calls),
        });

        with_state(|state| {
            state.callbacks.environment = Some(frontend_services_env);
            let mut env = Environment { state };
            assert!(
                env.location_interface()
                    .expect("location interface should be available")
                    .is_available()
            );
        });

        let callback = captured_location_callback()
            .lock()
            .expect("location callback capture mutex poisoned")
            .expect("location callback should be captured");
        unsafe {
            callback
                .initialized
                .expect("location initialized callback should be set")();
            callback
                .deinitialized
                .expect("location deinitialized callback should be set")();
        }

        assert_eq!(
            *calls
                .lock()
                .expect("location lifecycle calls mutex poisoned"),
            vec![
                LocationLifecycleEvent::Initialized,
                LocationLifecycleEvent::Deinitialized,
            ]
        );

        clear_global_test_core();
    }

    #[test]
    fn camera_interface_dispatches_frames_and_lifecycle_to_core() {
        let _guard = serial_test_guard();
        reset_captured_camera_interface();
        let calls = Arc::new(Mutex::new(Vec::new()));
        install_global_test_core(CameraRecordingCore {
            calls: Arc::clone(&calls),
        });

        with_state(|state| {
            state.callbacks.environment = Some(frontend_services_env);
            let mut env = Environment { state };
            let interface = env
                .camera_interface(CameraRequest {
                    capabilities: CameraCapabilities::from(CameraCapability::RawFramebuffer)
                        | CameraCapability::OpenGlTexture,
                    size: CameraFrameSize::new(320, 240),
                })
                .expect("camera interface should be available");
            assert!(interface.is_available());
            assert!(
                interface
                    .capabilities()
                    .contains(CameraCapability::RawFramebuffer)
            );
            assert!(
                !interface
                    .capabilities()
                    .contains(CameraCapability::OpenGlTexture)
            );
            assert_eq!(interface.size(), CameraFrameSize::new(160, 120));
            assert!(interface.start());
            assert!(interface.stop());
        });

        let callback = captured_camera_callback()
            .lock()
            .expect("camera callback capture mutex poisoned")
            .expect("camera callback should be captured");
        let pixels = [0xff00_0001, 0xff00_0002, 0xff00_0003, 0xff00_0004];
        let affine = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.25, 0.5, 1.0];
        unsafe {
            callback
                .initialized
                .expect("camera initialized callback should be set")();
            callback
                .frame_raw_framebuffer
                .expect("raw frame callback should be set")(
                pixels.as_ptr(),
                2,
                2,
                2 * std::mem::size_of::<u32>(),
            );
            callback
                .frame_opengl_texture
                .expect("texture frame callback should be set")(
                7, 0x0de1, affine.as_ptr()
            );
            callback
                .deinitialized
                .expect("camera deinitialized callback should be set")();
        }

        assert_eq!(
            *captured_camera_starts()
                .lock()
                .expect("camera start capture mutex poisoned"),
            1
        );
        assert_eq!(
            *captured_camera_stops()
                .lock()
                .expect("camera stop capture mutex poisoned"),
            1
        );
        assert_eq!(
            *calls.lock().expect("camera calls mutex poisoned"),
            vec![
                CameraEvent::Initialized,
                CameraEvent::Raw {
                    width: 2,
                    height: 2,
                    pitch: 2 * std::mem::size_of::<u32>(),
                    pixels: pixels.to_vec(),
                },
                CameraEvent::Texture {
                    texture_id: 7,
                    texture_target: 0x0de1,
                    affine,
                },
                CameraEvent::Deinitialized,
            ]
        );

        clear_global_test_core();
    }

    #[test]
    fn disk_control_callbacks_dispatch_to_core() {
        let _guard = serial_test_guard();
        reset_captured_disk_control_callbacks();
        let calls = Arc::new(Mutex::new(Vec::new()));
        install_global_test_core(DiskControlRecordingCore {
            calls: Arc::clone(&calls),
        });

        with_state(|state| {
            state.callbacks.environment = Some(frontend_services_env);
            let mut env = Environment { state };
            assert!(env.set_disk_control_interface());
            assert!(env.set_disk_control_ext_interface());
        });

        let legacy = captured_disk_control_callback()
            .lock()
            .expect("disk control callback capture mutex poisoned")
            .expect("legacy disk control callback should be captured");
        assert!(legacy.set_eject_state.is_some());
        assert!(legacy.get_eject_state.is_some());
        assert!(legacy.get_image_index.is_some());
        assert!(legacy.set_image_index.is_some());
        assert!(legacy.get_num_images.is_some());
        assert!(legacy.replace_image_index.is_some());
        assert!(legacy.add_image_index.is_some());

        let callback = captured_disk_control_ext_callback()
            .lock()
            .expect("disk control ext callback capture mutex poisoned")
            .expect("extended disk control callback should be captured");
        unsafe {
            assert!(callback
                .set_eject_state
                .expect("set_eject_state should be set")(
                true
            ));
            assert!(callback
                .get_eject_state
                .expect("get_eject_state should be set")());
            assert_eq!(
                callback
                    .get_image_index
                    .expect("get_image_index should be set")(),
                2
            );
            assert!(callback
                .set_image_index
                .expect("set_image_index should be set")(3));
            assert_eq!(
                callback
                    .get_num_images
                    .expect("get_num_images should be set")(),
                4
            );
            assert!(callback
                .replace_image_index
                .expect("replace_image_index should be set")(
                1, ptr::null()
            ));
            assert!(callback
                .add_image_index
                .expect("add_image_index should be set")());
            assert!(callback
                .set_initial_image
                .expect("set_initial_image should be set")(
                2,
                c"/games/disc2.cue".as_ptr()
            ));

            let mut path = [0i8; 64];
            assert!(callback
                .get_image_path
                .expect("get_image_path should be set")(
                2,
                path.as_mut_ptr(),
                path.len()
            ));
            assert_eq!(CStr::from_ptr(path.as_ptr()), c"/games/disctwo.cue");

            let mut label = [0i8; 64];
            assert!(callback
                .get_image_label
                .expect("get_image_label should be set")(
                2,
                label.as_mut_ptr(),
                label.len()
            ));
            assert_eq!(CStr::from_ptr(label.as_ptr()), c"Disc Two");
        }

        assert_eq!(
            *calls.lock().expect("disk control calls mutex poisoned"),
            vec![
                DiskControlEvent::SetTray(DiskTrayState::Ejected),
                DiskControlEvent::SetImage(DiskIndex::new(3)),
                DiskControlEvent::ReplaceImage(DiskIndex::new(1), false),
                DiskControlEvent::AddImage,
                DiskControlEvent::SetInitialImage(
                    DiskIndex::new(2),
                    "/games/disc2.cue".to_string()
                ),
            ]
        );

        with_state(|state| {
            state.callbacks.environment = Some(frontend_services_env);
            let mut env = Environment { state };
            assert!(env.clear_disk_control_interface());
            assert!(env.clear_disk_control_ext_interface());
        });
        assert!(
            captured_disk_control_callback()
                .lock()
                .expect("disk control callback capture mutex poisoned")
                .is_none()
        );
        assert!(
            captured_disk_control_ext_callback()
                .lock()
                .expect("disk control ext callback capture mutex poisoned")
                .is_none()
        );

        clear_global_test_core();
    }

    #[test]
    fn netpacket_callbacks_dispatch_to_core_and_send_through_session() {
        let _guard = serial_test_guard();
        reset_captured_netpacket_interface();
        let calls = Arc::new(Mutex::new(Vec::new()));
        install_global_test_core(NetpacketRecordingCore {
            calls: Arc::clone(&calls),
        });

        with_state(|state| {
            state.callbacks.environment = Some(frontend_services_env);
            let mut env = Environment { state };
            assert!(env.set_netpacket_interface(Some("proto\0v1")));
        });

        let callback = captured_netpacket_callback()
            .lock()
            .expect("netpacket callback capture mutex poisoned")
            .expect("netpacket callback should be captured");
        assert_eq!(
            unsafe { CStr::from_ptr(callback.protocol_version as *const c_char) },
            c"protov1"
        );

        unsafe {
            callback.start.expect("start callback should be set")(
                0,
                Some(capture_netpacket_send),
                Some(capture_netpacket_poll_receive),
            );
            callback.receive.expect("receive callback should be set")(
                b"packet".as_ptr().cast(),
                6,
                3,
            );
            callback.poll.expect("poll callback should be set")();
            assert!(callback
                .connected
                .expect("connected callback should be set")(
                7
            ));
            assert!(!callback
                .connected
                .expect("connected callback should be set")(
                9
            ));
            callback
                .disconnected
                .expect("disconnected callback should be set")(7);
            callback.stop.expect("stop callback should be set")();
        }

        assert_eq!(
            *captured_netpacket_sends()
                .lock()
                .expect("netpacket sends mutex poisoned"),
            vec![
                CapturedNetpacketSend {
                    flags: NetpacketFlags::reliable().as_raw(),
                    data: b"hello".to_vec(),
                    client_id: raw::RETRO_NETPACKET_BROADCAST,
                },
                CapturedNetpacketSend {
                    flags: NetpacketFlags::reliable().with_flush_hint(true).as_raw(),
                    data: Vec::new(),
                    client_id: 0,
                },
            ]
        );
        assert_eq!(
            *captured_netpacket_polls()
                .lock()
                .expect("netpacket polls mutex poisoned"),
            1
        );
        assert_eq!(
            *calls.lock().expect("netpacket calls mutex poisoned"),
            vec![
                NetpacketEvent::Start(NetplayClientId::host(), true),
                NetpacketEvent::Receive(NetplayClientId::new(3), b"packet".to_vec()),
                NetpacketEvent::Poll,
                NetpacketEvent::Connected(NetplayClientId::new(7)),
                NetpacketEvent::Connected(NetplayClientId::new(9)),
                NetpacketEvent::Disconnected(NetplayClientId::new(7)),
                NetpacketEvent::Stop,
            ]
        );

        with_state(|state| {
            state.callbacks.environment = Some(frontend_services_env);
            let mut env = Environment { state };
            assert!(env.clear_netpacket_interface());
        });
        assert!(
            captured_netpacket_callback()
                .lock()
                .expect("netpacket callback capture mutex poisoned")
                .is_none()
        );

        clear_global_test_core();
    }

    #[test]
    fn microphone_interface_opens_reads_and_closes_handles() {
        let _guard = serial_test_guard();
        reset_captured_microphone_interface();
        let mut state = CoreState::default();
        state.callbacks.environment = Some(frontend_services_env);

        {
            let mut env = Environment { state: &mut state };
            let interface = env
                .microphone_interface()
                .expect("microphone interface should be available");
            assert!(interface.is_available());
            assert_eq!(interface.version(), raw::RETRO_MICROPHONE_INTERFACE_VERSION);

            let default_mic = interface
                .open_default()
                .expect("default microphone should open");
            drop(default_mic);

            let mut mic = interface
                .open(MicrophoneParams::new(MicrophoneRateHz::new(16_000)))
                .expect("configured microphone should open");
            assert_eq!(
                mic.params(),
                Some(MicrophoneParams::new(MicrophoneRateHz::new(22_050)))
            );
            assert!(mic.set_enabled(true));
            assert!(mic.enabled());

            let mut samples = [0i16; 4];
            assert_eq!(mic.read_samples(&mut samples), Ok(4));
            assert_eq!(samples, [1, 2, 3, 4]);
        }

        assert_eq!(
            *captured_mic_open_params()
                .lock()
                .expect("microphone open params mutex poisoned"),
            vec![None, Some(16_000)]
        );
        assert_eq!(
            *captured_mic_states()
                .lock()
                .expect("microphone states mutex poisoned"),
            vec![true]
        );
        assert_eq!(
            *captured_mic_closes()
                .lock()
                .expect("microphone closes mutex poisoned"),
            2
        );
    }

    #[test]
    fn runtime_video_refresh_frame_forwards_valid_software_frame() {
        let _guard = serial_test_guard();
        reset_captured_video_refreshes();
        let mut state = CoreState::default();
        state.callbacks.video_refresh = Some(capture_video_refresh);

        let runtime = Runtime { state: &mut state };
        let pixels = vec![0u32; 320 * 240];

        assert!(runtime.video_refresh_frame(&pixels, 320, 240, 320 * mem::size_of::<u32>()));
        assert_eq!(
            *captured_video_refreshes()
                .lock()
                .expect("video refresh capture mutex poisoned"),
            vec![CapturedVideoRefresh {
                data_kind: CapturedVideoDataKind::Software,
                data_addr: pixels.as_ptr() as usize,
                width: 320,
                height: 240,
                pitch: 320 * mem::size_of::<u32>(),
            }]
        );
    }

    #[test]
    fn runtime_video_refresh_frame_rejects_buffer_smaller_than_pitch_times_height() {
        let _guard = serial_test_guard();
        reset_captured_video_refreshes();
        let mut state = CoreState::default();
        state.callbacks.video_refresh = Some(capture_video_refresh);

        let runtime = Runtime { state: &mut state };
        let pixels = vec![0u16; (320 * 240) - 1];

        assert!(!runtime.video_refresh_frame(&pixels, 320, 240, 320 * mem::size_of::<u16>()));
        assert!(
            captured_video_refreshes()
                .lock()
                .expect("video refresh capture mutex poisoned")
                .is_empty()
        );
    }

    #[test]
    fn runtime_video_refresh_frame_rejects_pitch_smaller_than_one_row() {
        let _guard = serial_test_guard();
        reset_captured_video_refreshes();
        let mut state = CoreState::default();
        state.callbacks.video_refresh = Some(capture_video_refresh);

        let runtime = Runtime { state: &mut state };
        let pixels = vec![0u32; 4];

        assert!(!runtime.video_refresh_frame(&pixels, 2, 2, mem::size_of::<u32>()));
        assert!(
            captured_video_refreshes()
                .lock()
                .expect("video refresh capture mutex poisoned")
                .is_empty()
        );
    }

    #[test]
    fn hw_render_interface_and_context_negotiation_are_typed() {
        let _guard = serial_test_guard();
        let mut captured = captured_hw_render_state()
            .lock()
            .expect("hw render capture mutex poisoned");
        captured.reset();
        captured.context_negotiation_support_version = Some(3);
        drop(captured);

        let mut state = CoreState::default();
        state.callbacks.environment = Some(capture_hw_render_env);
        let mut env = Environment { state: &mut state };

        let interface = env
            .hw_render_interface()
            .expect("HW render interface should be available");
        assert_eq!(interface.interface_type(), HwRenderInterfaceType::Vulkan);
        assert_eq!(interface.interface_version(), 1);
        assert!(!interface.as_base_ptr().is_null());
        assert_eq!(
            env.hw_render_context_negotiation_interface_support(
                HwRenderContextNegotiationInterfaceType::Vulkan
            ),
            Some(3)
        );

        let requested = HwRenderContextNegotiationInterface::vulkan(2);
        assert!(env.set_hw_render_context_negotiation_interface(requested));

        let captured = captured_hw_render_state()
            .lock()
            .expect("hw render capture mutex poisoned");
        assert_eq!(captured.last_context_negotiation, Some(requested));
    }

    #[test]
    fn set_hw_render_from_candidates_prefers_frontend_preferred_context() {
        let _guard = serial_test_guard();
        let mut captured = captured_hw_render_state()
            .lock()
            .expect("hw render capture mutex poisoned");
        captured.reset();
        captured.preferred_context_type = HwContextType::OpenGlEs3;
        captured.supports_non_preferred_context = false;
        captured.set_accept_contexts(&[HwContextType::OpenGlEs3]);
        drop(captured);

        let mut state = CoreState::default();
        state.callbacks.environment = Some(capture_hw_render_env);
        let mut env = Environment { state: &mut state };

        let chosen = env.set_hw_render_from_candidates(&[
            HwRenderConfig {
                context_type: HwContextType::OpenGlCore,
                ..HwRenderConfig::default()
            },
            HwRenderConfig {
                context_type: HwContextType::OpenGlEs3,
                ..HwRenderConfig::default()
            },
            HwRenderConfig {
                context_type: HwContextType::OpenGlEs2,
                ..HwRenderConfig::default()
            },
        ]);

        assert_eq!(
            chosen.map(|config| config.context_type),
            Some(HwContextType::OpenGlEs3)
        );
        let captured = captured_hw_render_state()
            .lock()
            .expect("hw render capture mutex poisoned");
        assert_eq!(
            captured.attempted_contexts(),
            vec![HwContextType::OpenGlEs3]
        );
    }

    #[test]
    fn set_hw_render_from_candidates_respects_frontend_nonpreferred_restriction() {
        let _guard = serial_test_guard();
        let mut captured = captured_hw_render_state()
            .lock()
            .expect("hw render capture mutex poisoned");
        captured.reset();
        captured.preferred_context_type = HwContextType::Vulkan;
        captured.supports_non_preferred_context = false;
        captured.accept_any_context = true;
        drop(captured);

        let mut state = CoreState::default();
        state.callbacks.environment = Some(capture_hw_render_env);
        let mut env = Environment { state: &mut state };

        let chosen = env.set_hw_render_from_candidates(&[
            HwRenderConfig {
                context_type: HwContextType::OpenGlCore,
                ..HwRenderConfig::default()
            },
            HwRenderConfig {
                context_type: HwContextType::OpenGlEs2,
                ..HwRenderConfig::default()
            },
        ]);

        assert!(chosen.is_none());
        let captured = captured_hw_render_state()
            .lock()
            .expect("hw render capture mutex poisoned");
        assert!(captured.attempted_contexts().is_empty());
    }

    #[test]
    fn set_hw_render_from_candidates_falls_back_only_when_frontend_allows_it() {
        let _guard = serial_test_guard();
        let mut captured = captured_hw_render_state()
            .lock()
            .expect("hw render capture mutex poisoned");
        captured.reset();
        captured.preferred_context_type = HwContextType::Vulkan;
        captured.supports_non_preferred_context = true;
        captured.set_accept_contexts(&[HwContextType::OpenGlEs2]);
        drop(captured);

        let mut state = CoreState::default();
        state.callbacks.environment = Some(capture_hw_render_env);
        let mut env = Environment { state: &mut state };

        let chosen = env.set_hw_render_from_candidates(&[
            HwRenderConfig {
                context_type: HwContextType::OpenGlCore,
                ..HwRenderConfig::default()
            },
            HwRenderConfig {
                context_type: HwContextType::OpenGlEs3,
                ..HwRenderConfig::default()
            },
            HwRenderConfig {
                context_type: HwContextType::OpenGlEs2,
                ..HwRenderConfig::default()
            },
        ]);

        assert_eq!(
            chosen.map(|config| config.context_type),
            Some(HwContextType::OpenGlEs2)
        );
        let captured = captured_hw_render_state()
            .lock()
            .expect("hw render capture mutex poisoned");
        assert_eq!(
            captured.attempted_contexts(),
            vec![
                HwContextType::OpenGlCore,
                HwContextType::OpenGlEs3,
                HwContextType::OpenGlEs2,
            ]
        );
    }

    #[test]
    fn set_hw_render_from_candidates_recovers_from_rejected_generic_opengl_with_gles_fallbacks() {
        let _guard = serial_test_guard();
        let mut captured = captured_hw_render_state()
            .lock()
            .expect("hw render capture mutex poisoned");
        captured.reset();
        captured.preferred_context_type = HwContextType::OpenGl;
        captured.supports_non_preferred_context = false;
        captured.set_accept_contexts(&[HwContextType::OpenGlEs2]);
        drop(captured);

        let mut state = CoreState::default();
        state.callbacks.environment = Some(capture_hw_render_env);
        let mut env = Environment { state: &mut state };

        let chosen = env.set_hw_render_from_candidates(&[
            HwRenderConfig {
                context_type: HwContextType::OpenGlCore,
                ..HwRenderConfig::default()
            },
            HwRenderConfig {
                context_type: HwContextType::OpenGlEs3,
                ..HwRenderConfig::default()
            },
            HwRenderConfig {
                context_type: HwContextType::OpenGl,
                ..HwRenderConfig::default()
            },
            HwRenderConfig {
                context_type: HwContextType::OpenGlEs2,
                ..HwRenderConfig::default()
            },
        ]);

        assert_eq!(
            chosen.map(|config| config.context_type),
            Some(HwContextType::OpenGlEs2)
        );
        let captured = captured_hw_render_state()
            .lock()
            .expect("hw render capture mutex poisoned");
        assert_eq!(
            captured.attempted_contexts(),
            vec![
                HwContextType::OpenGl,
                HwContextType::OpenGlEs3,
                HwContextType::OpenGlEs2,
            ]
        );
    }

    #[test]
    fn set_hw_render_preserves_frontend_injected_runtime_callbacks() {
        let _guard = serial_test_guard();
        let mut captured = captured_hw_render_state()
            .lock()
            .expect("hw render capture mutex poisoned");
        captured.reset();
        captured.accept_any_context = true;
        captured.inject_runtime_callbacks = true;
        drop(captured);

        let mut state = CoreState::default();
        state.callbacks.environment = Some(capture_hw_render_env);
        let mut env = Environment { state: &mut state };

        let ok = env.set_hw_render(HwRenderConfig {
            context_type: HwContextType::OpenGlEs2,
            ..HwRenderConfig::default()
        });

        assert!(ok);
        let stored = env
            .state
            .hw_render
            .expect("accepted hardware-render callback should be stored on state");
        assert!(stored.context_reset.is_some());
        assert!(stored.context_destroy.is_some());
        assert_eq!(
            stored
                .get_current_framebuffer
                .map(|callback| callback as usize),
            Some(fake_current_framebuffer as usize)
        );
        assert_eq!(
            stored.get_proc_address.map(|callback| callback as usize),
            Some(fake_get_proc_address as usize)
        );
    }

    #[test]
    fn runtime_exposes_hardware_proc_addresses_without_raw_abi_types() {
        let _guard = serial_test_guard();
        let mut captured = captured_hw_render_state()
            .lock()
            .expect("hw render capture mutex poisoned");
        captured.reset();
        captured.accept_any_context = true;
        captured.inject_runtime_callbacks = true;
        drop(captured);

        let mut state = CoreState::default();
        state.callbacks.environment = Some(capture_hw_render_env);
        {
            let mut env = Environment { state: &mut state };
            assert!(env.set_hw_render(HwRenderConfig {
                context_type: HwContextType::OpenGlEs2,
                ..HwRenderConfig::default()
            }));
        }

        let runtime = Runtime { state: &mut state };
        assert_eq!(
            runtime.hw_proc_address("glClear").unwrap() as usize,
            fake_gl_proc as usize
        );
    }

    #[test]
    fn runtime_preserves_default_hardware_framebuffer() {
        let mut state = CoreState {
            hw_render: Some(RawHwRenderCallback {
                get_current_framebuffer: Some(fake_zero_current_framebuffer),
                ..RawHwRenderCallback::default()
            }),
            ..CoreState::default()
        };

        let runtime = Runtime { state: &mut state };

        assert_eq!(runtime.current_framebuffer(), Some(0));
    }

    #[test]
    fn runtime_preserves_named_hardware_framebuffer() {
        let mut state = CoreState {
            hw_render: Some(RawHwRenderCallback {
                get_current_framebuffer: Some(fake_current_framebuffer),
                ..RawHwRenderCallback::default()
            }),
            ..CoreState::default()
        };

        let runtime = Runtime { state: &mut state };
        assert_eq!(runtime.current_framebuffer(), Some(99));
    }

    #[test]
    fn runtime_distinguishes_missing_framebuffer_callback_from_default_framebuffer() {
        let mut state = CoreState::default();
        assert_eq!(Runtime { state: &mut state }.current_framebuffer(), None);

        state.hw_render = Some(RawHwRenderCallback::default());
        assert_eq!(Runtime { state: &mut state }.current_framebuffer(), None);
    }

    #[cfg(unix)]
    #[test]
    fn runtime_falls_back_to_process_global_symbols_when_frontend_proc_lookup_fails() {
        let mut state = CoreState {
            hw_render: Some(RawHwRenderCallback {
                context_type: HwContextType::OpenGlEs2,
                get_proc_address: Some(missing_get_proc_address),
                ..RawHwRenderCallback::default()
            }),
            ..CoreState::default()
        };

        let runtime = Runtime { state: &mut state };

        assert!(runtime.hw_proc_address("malloc").is_ok());
        assert!(
            runtime
                .hw_proc_address("__libretro_core_missing_symbol")
                .is_err()
        );
    }

    #[test]
    fn set_geometry_is_forwarded_to_frontend() {
        let _guard = serial_test_guard();
        reset_captured_geometries();

        let mut state = CoreState::default();
        state.callbacks.environment = Some(capture_geometry_env);
        let mut env = Environment { state: &mut state };

        assert!(env.set_geometry(GameGeometry {
            base_width: 320,
            base_height: 240,
            max_width: 2048,
            max_height: 2048,
            aspect_ratio: 4.0 / 3.0,
        }));

        let captured = captured_geometries()
            .lock()
            .expect("geometry capture mutex poisoned")
            .clone();
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].base_width, 320);
        assert_eq!(captured[0].base_height, 240);
        assert_eq!(captured[0].max_width, 2048);
        assert_eq!(captured[0].max_height, 2048);
        assert_eq!(captured[0].aspect_ratio, 4.0 / 3.0);
    }

    #[test]
    fn runtime_input_helpers_use_typed_device_queries() {
        let _guard = serial_test_guard();
        reset_captured_input_queries();

        let mut state = CoreState::default();
        state.callbacks.input_state = Some(capture_input_state);
        let runtime = Runtime { state: &mut state };

        assert!(runtime.joypad_pressed(0, JoypadButton::A));
        let buttons = runtime.joypad_buttons(0);
        assert!(buttons.contains(JoypadButton::A));
        assert!(buttons.contains(JoypadButton::B));
        assert!(!buttons.contains(JoypadButton::X));
        assert_eq!(
            runtime.analog_axis(0, AnalogStick::Left, AnalogAxis::X),
            -123
        );
        assert_eq!(runtime.analog_button(0, JoypadButton::R2), 123);
        assert_eq!(runtime.mouse_axis(0, MouseAxis::X), 7);
        assert!(runtime.mouse_button_pressed(0, MouseButton::Left));
        assert!(runtime.mouse_wheel_moved(0, MouseWheel::Up));
        assert_eq!(runtime.pointer_axis(0, 1, PointerAxis::Y), -77);
        assert!(runtime.pointer_pressed(0, 1));
        assert_eq!(runtime.pointer_count(0), 2);
        assert!(runtime.pointer_is_offscreen(0, 1));
        assert_eq!(runtime.lightgun_axis(0, LightgunAxis::ScreenX), 99);
        assert!(runtime.lightgun_button_pressed(0, LightgunButton::Trigger));
        assert!(runtime.lightgun_is_offscreen(0));

        let captured = captured_input_queries()
            .lock()
            .expect("input query capture mutex poisoned")
            .clone();
        assert!(captured.contains(&CapturedInputQuery {
            port: 0,
            device: RETRO_DEVICE_ANALOG,
            index: RETRO_DEVICE_INDEX_ANALOG_LEFT,
            id: RETRO_DEVICE_ID_ANALOG_X,
        }));
        assert!(captured.contains(&CapturedInputQuery {
            port: 0,
            device: RETRO_DEVICE_POINTER,
            index: 1,
            id: RETRO_DEVICE_ID_POINTER_PRESSED,
        }));
        assert!(captured.contains(&CapturedInputQuery {
            port: 0,
            device: RETRO_DEVICE_LIGHTGUN,
            index: 0,
            id: RETRO_DEVICE_ID_LIGHTGUN_TRIGGER,
        }));
    }

    #[test]
    fn stored_hw_render_lifecycle_trampolines_dispatch_to_core_methods() {
        let _guard = serial_test_guard();
        reset_lifecycle_call_counts();

        let mut captured = captured_hw_render_state()
            .lock()
            .expect("hw render capture mutex poisoned");
        captured.reset();
        captured.accept_any_context = true;
        drop(captured);

        let callbacks = with_state(|state| {
            state.reset_frontend_state();
            state.core = Some(Box::new(LifecycleRecordingCore));
            state.callbacks.environment = Some(capture_hw_render_env);

            let mut env = Environment { state };
            let ok = env.set_hw_render(HwRenderConfig {
                context_type: HwContextType::OpenGlEs2,
                ..HwRenderConfig::default()
            });
            assert!(ok);

            env.state
                .hw_render
                .expect("accepted hardware-render callback should be stored on state")
        });

        unsafe {
            callbacks
                .context_reset
                .expect("context_reset trampoline should be present")();
            callbacks
                .context_destroy
                .expect("context_destroy trampoline should be present")();
        }

        assert_eq!(
            snapshot_lifecycle_call_counts(),
            LifecycleCallCounts {
                resets: 1,
                destroys: 1,
            }
        );

        with_state(|state| {
            state.reset_frontend_state();
            state.core = None;
        });
    }

    #[test]
    fn retro_get_system_info_catches_core_panic_and_returns_default_info() {
        let _guard = serial_test_guard();
        install_global_test_core(PanickingCore::new(PanicAt::SystemInfo));
        let stale = c"stale".as_ptr();

        let mut info = RawSystemInfo {
            library_name: stale,
            library_version: stale,
            valid_extensions: stale,
            need_fullpath: true,
            block_extract: true,
        };

        __private::retro_get_system_info(&mut info);

        assert!(info.library_name.is_null());
        assert!(info.library_version.is_null());
        assert!(info.valid_extensions.is_null());
        assert!(!info.need_fullpath);
        assert!(!info.block_extract);

        clear_global_test_core();
    }

    #[test]
    fn retro_get_system_info_reuses_owned_strings_across_calls() {
        let _guard = serial_test_guard();
        let calls = Arc::new(AtomicUsize::new(0));
        install_global_test_core(ChangingSystemInfoCore {
            calls: Arc::clone(&calls),
        });

        let mut first = RawSystemInfo::default();
        let mut second = RawSystemInfo::default();

        __private::retro_get_system_info(&mut first);
        __private::retro_get_system_info(&mut second);

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(first.library_name, second.library_name);
        assert_eq!(first.library_version, second.library_version);
        assert_eq!(first.valid_extensions, second.valid_extensions);
        assert_eq!(
            unsafe { CStr::from_ptr(first.library_name) }
                .to_str()
                .unwrap(),
            "cached-test-core"
        );
        assert_eq!(
            unsafe { CStr::from_ptr(first.library_version) }
                .to_str()
                .unwrap(),
            "first"
        );
        assert_eq!(
            unsafe { CStr::from_ptr(first.valid_extensions) }
                .to_str()
                .unwrap(),
            "first"
        );

        clear_global_test_core();
    }

    #[test]
    fn retro_init_catches_core_panic() {
        let _guard = serial_test_guard();
        install_global_test_core(PanickingCore::new(PanicAt::Init));

        __private::retro_init();

        clear_global_test_core();
    }

    #[test]
    fn retro_load_game_catches_core_panic_and_returns_false() {
        let _guard = serial_test_guard();
        install_global_test_core(PanickingCore::new(PanicAt::LoadGame));

        assert!(!__private::retro_load_game(ptr::null()));

        clear_global_test_core();
    }

    #[test]
    fn retro_memory_callbacks_receive_typed_regions() {
        let _guard = serial_test_guard();
        let calls = Arc::new(Mutex::new(Vec::new()));
        install_global_test_core(MemoryRecordingCore::new(Arc::clone(&calls)));

        let save_ram = __private::retro_get_memory_data(RETRO_MEMORY_SAVE_RAM);
        let save_ram_size = __private::retro_get_memory_size(RETRO_MEMORY_SAVE_RAM);
        let unknown = __private::retro_get_memory_data(99);
        let unknown_size = __private::retro_get_memory_size(99);

        assert!(!save_ram.is_null());
        assert_eq!(save_ram_size, 4);
        assert!(unknown.is_null());
        assert_eq!(unknown_size, 0);
        assert_eq!(
            *calls.lock().expect("memory calls mutex poisoned"),
            vec![
                MemoryRegion::SaveRam,
                MemoryRegion::SaveRam,
                MemoryRegion::Unknown(99),
                MemoryRegion::Unknown(99),
            ]
        );

        clear_global_test_core();
    }

    #[test]
    fn retro_set_controller_port_device_converts_to_typed_values() {
        let _guard = serial_test_guard();
        let calls = Arc::new(Mutex::new(Vec::new()));
        install_global_test_core(ControllerDeviceRecordingCore {
            calls: Arc::clone(&calls),
        });

        __private::retro_set_controller_port_device(2, RETRO_DEVICE_MOUSE);
        __private::retro_set_controller_port_device(3, 123);

        assert_eq!(
            *calls
                .lock()
                .expect("controller device calls mutex poisoned"),
            vec![
                (InputPort::new(2), ControllerDevice::Mouse),
                (InputPort::new(3), ControllerDevice::Unknown(123)),
            ]
        );

        clear_global_test_core();
    }

    #[test]
    fn retro_cheat_set_converts_to_typed_values() {
        let _guard = serial_test_guard();
        let calls = Arc::new(Mutex::new(Vec::new()));
        install_global_test_core(CheatRecordingCore {
            calls: Arc::clone(&calls),
        });

        __private::retro_cheat_set(7, true, c"ABCD-EFGH".as_ptr());
        __private::retro_cheat_set(8, false, ptr::null());

        assert_eq!(
            *calls.lock().expect("cheat calls mutex poisoned"),
            vec![
                (CheatIndex::new(7), true, Some("ABCD-EFGH".to_owned())),
                (CheatIndex::new(8), false, None),
            ]
        );

        clear_global_test_core();
    }

    #[test]
    fn retro_run_panic_keeps_core_available_for_later_callbacks() {
        let _guard = serial_test_guard();
        let reset_calls = Arc::new(AtomicUsize::new(0));
        install_global_test_core(RunPanicThenResetCore {
            reset_calls: Arc::clone(&reset_calls),
        });

        __private::retro_run();
        __private::retro_reset();

        assert_eq!(reset_calls.load(Ordering::SeqCst), 1);

        clear_global_test_core();
    }
}
