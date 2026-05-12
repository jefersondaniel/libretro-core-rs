//! Minimal libretro wrapper for implementing Rust cores.
//!
//! The crate exposes a small subset of `libretro.h` plus a trait/macro pair
//! for exporting the required `retro_*` symbols from a Rust core.
//!
//! Methodology:
//! - Keep public APIs Rust-first even when the underlying libretro ABI is C-first.
//! - Prefer strings, slices, enums, and return values over raw pointers or
//!   mutable out-params whenever the wrapper can do that conversion centrally.
//! - Keep raw ABI details private unless exposing them is necessary for a real
//!   core-development use case.
//! - Match libretro/OpenGL naming where it helps recognition, but not at the
//!   cost of forcing callers back into manual FFI plumbing.
//!
//! ```ignore
//! use libretro::{
//!     ContentContract, Core, Environment, GameGeometry, HwRenderConfig,
//!     JoypadButton, PixelFormat, Runtime, SystemAvInfo, SystemInfo, SystemTiming,
//!     VariableDefinition,
//! };
//!
//! #[derive(Default)]
//! struct GlCore {
//!     width: u32,
//!     height: u32,
//! }
//!
//! impl Core for GlCore {
//!     fn system_info(&self) -> SystemInfo {
//!         let mut info = SystemInfo::new("TestCore GL", "v1");
//!         info.need_fullpath = false;
//!         info
//!     }
//!
//!     fn av_info(&self) -> SystemAvInfo {
//!         SystemAvInfo {
//!             geometry: GameGeometry {
//!                 base_width: 320,
//!                 base_height: 240,
//!                 max_width: 1024,
//!                 max_height: 1024,
//!                 aspect_ratio: 4.0 / 3.0,
//!             },
//!             timing: SystemTiming {
//!                 fps: 60.0,
//!                 sample_rate: 0.0,
//!             },
//!         }
//!     }
//!
//!     fn on_set_environment(&mut self, env: &mut Environment<'_>) {
//!         ContentContract::new("bin")
//!             .with_support_no_game(true)
//!             .with_persistent_data(true)
//!             .register_environment(env);
//!         env.set_variables(&[VariableDefinition::new(
//!             "testgl_resolution",
//!             "Internal resolution; 320x240|640x480|1024x768",
//!         )]);
//!     }
//!
//!     fn load_game(
//!         &mut self,
//!         _game: Option<libretro::GameInfo<'_>>,
//!         runtime: &mut Runtime<'_>,
//!     ) -> bool {
//!         let mut env = runtime.environment();
//!         env.set_pixel_format(PixelFormat::Xrgb8888)
//!             && env
//!                 .set_hw_render_from_candidates(&[
//!                     HwRenderConfig::opengl()
//!                         .with_depth(true)
//!                         .with_stencil(true)
//!                         .with_bottom_left_origin(true),
//!                     HwRenderConfig::opengles3()
//!                         .with_depth(true)
//!                         .with_stencil(true)
//!                         .with_bottom_left_origin(true),
//!                 ])
//!                 .is_some()
//!     }
//!
//!     fn run(&mut self, runtime: &mut Runtime<'_>) {
//!         runtime.poll_input();
//!         if runtime.joypad_pressed(0, JoypadButton::Up) {
//!             // update state
//!         }
//!         runtime.video_refresh_hw(self.width, self.height, 0);
//!     }
//! }
//!
//! libretro::export_core!(GlCore::default());
//! ```

mod av;
mod content;
#[path = "glsym.rs"]
mod glsym_impl;
mod hw_render;
mod raw;

use std::any::Any;
use std::borrow::Cow;
use std::ffi::{CStr, CString, c_char, c_void};
use std::mem;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;
use std::sync::{Mutex, OnceLock};

pub use av::{
    bounded_game_geometry, exact_audio_frames_per_video_frame, fixed_system_av_info, game_geometry,
    silent_stereo_frames, silent_stereo_frames_for_video_frame, system_av_info,
};
pub use content::ContentContract;
pub use glsym_impl::{
    CompatGl, CompatGlClear, CompatTextureGl, FakeGlConfig, FakeGlSnapshot, GlBlendEquation,
    GlBlendFactor, GlBufferTarget, GlBufferUsage, GlCapability, GlDrawMode, GlFramebufferTarget,
    GlIndexType, GlTextureDataType, GlTextureFilter, GlTextureFormat, GlTextureInternalFormat,
    GlTextureParameter, GlTextureTarget, GlTextureWrap, GlVersionInfo,
    configure_fake_gl_for_testing, fake_get_proc_address_for_testing, glsym,
    reset_fake_gl_for_testing, snapshot_fake_gl_for_testing,
};
pub use hw_render::{
    OPENGL_COMPATIBILITY_HW_RENDER_LABEL, OPENGL_MODERN_PREFERRED_HW_RENDER_LABEL,
    opengl_compatibility_hw_render_candidates, opengl_modern_preferred_hw_render_candidates,
};
pub use raw::{
    RETRO_API_VERSION, RETRO_DEVICE_ID_JOYPAD_A, RETRO_DEVICE_ID_JOYPAD_B,
    RETRO_DEVICE_ID_JOYPAD_DOWN, RETRO_DEVICE_ID_JOYPAD_L, RETRO_DEVICE_ID_JOYPAD_L2,
    RETRO_DEVICE_ID_JOYPAD_L3, RETRO_DEVICE_ID_JOYPAD_LEFT, RETRO_DEVICE_ID_JOYPAD_R,
    RETRO_DEVICE_ID_JOYPAD_R2, RETRO_DEVICE_ID_JOYPAD_R3, RETRO_DEVICE_ID_JOYPAD_RIGHT,
    RETRO_DEVICE_ID_JOYPAD_SELECT, RETRO_DEVICE_ID_JOYPAD_START, RETRO_DEVICE_ID_JOYPAD_UP,
    RETRO_DEVICE_ID_JOYPAD_X, RETRO_DEVICE_ID_JOYPAD_Y, RETRO_DEVICE_JOYPAD, RETRO_DEVICE_NONE,
    RETRO_ENVIRONMENT_GET_LOG_INTERFACE, RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER,
    RETRO_ENVIRONMENT_GET_VARIABLE, RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE,
    RETRO_ENVIRONMENT_SET_CONTENT_INFO_OVERRIDE, RETRO_ENVIRONMENT_SET_GEOMETRY,
    RETRO_ENVIRONMENT_SET_HW_RENDER, RETRO_ENVIRONMENT_SET_MESSAGE,
    RETRO_ENVIRONMENT_SET_PIXEL_FORMAT, RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME,
    RETRO_ENVIRONMENT_SET_VARIABLES, RETRO_HW_FRAME_BUFFER_VALID, RETRO_REGION_NTSC,
    RETRO_REGION_PAL, retro_audio_sample_batch_t, retro_audio_sample_t, retro_environment_t,
    retro_game_geometry as GameGeometry, retro_game_info as RawGameInfo,
    retro_hw_context_type as HwContextType, retro_hw_render_callback as RawHwRenderCallback,
    retro_input_poll_t, retro_input_state_t, retro_log_callback as RawLogCallback,
    retro_log_level as LogLevel, retro_message as RawMessage, retro_pixel_format as PixelFormat,
    retro_system_av_info as SystemAvInfo,
    retro_system_content_info_override as RawContentInfoOverride,
    retro_system_info as RawSystemInfo, retro_system_timing as SystemTiming,
    retro_variable as RawVariable, retro_video_refresh_t,
};

type CoreFactory = fn() -> Box<dyn Core>;

static FACTORY: OnceLock<CoreFactory> = OnceLock::new();
static STATE: OnceLock<Mutex<CoreState>> = OnceLock::new();

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

#[derive(Clone, Debug)]
pub struct VariableDefinition {
    pub key: String,
    pub value: String,
}

impl VariableDefinition {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JoypadButton {
    B,
    Y,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
    A,
    X,
    L,
    R,
    L2,
    R2,
    L3,
    R3,
}

impl JoypadButton {
    fn as_raw(self) -> u32 {
        match self {
            Self::B => RETRO_DEVICE_ID_JOYPAD_B,
            Self::Y => RETRO_DEVICE_ID_JOYPAD_Y,
            Self::Select => RETRO_DEVICE_ID_JOYPAD_SELECT,
            Self::Start => RETRO_DEVICE_ID_JOYPAD_START,
            Self::Up => RETRO_DEVICE_ID_JOYPAD_UP,
            Self::Down => RETRO_DEVICE_ID_JOYPAD_DOWN,
            Self::Left => RETRO_DEVICE_ID_JOYPAD_LEFT,
            Self::Right => RETRO_DEVICE_ID_JOYPAD_RIGHT,
            Self::A => RETRO_DEVICE_ID_JOYPAD_A,
            Self::X => RETRO_DEVICE_ID_JOYPAD_X,
            Self::L => RETRO_DEVICE_ID_JOYPAD_L,
            Self::R => RETRO_DEVICE_ID_JOYPAD_R,
            Self::L2 => RETRO_DEVICE_ID_JOYPAD_L2,
            Self::R2 => RETRO_DEVICE_ID_JOYPAD_R2,
            Self::L3 => RETRO_DEVICE_ID_JOYPAD_L3,
            Self::R3 => RETRO_DEVICE_ID_JOYPAD_R3,
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

pub trait Core: Send + 'static {
    fn system_info(&self) -> SystemInfo;
    fn av_info(&self) -> SystemAvInfo;
    fn run(&mut self, runtime: &mut Runtime<'_>);

    fn on_set_environment(&mut self, _env: &mut Environment<'_>) {}
    fn init(&mut self, _env: &mut Environment<'_>) {}
    fn deinit(&mut self) {}
    fn set_controller_port_device(&mut self, _port: u32, _device: u32) {}
    fn reset(&mut self) {}
    fn load_game(&mut self, _game: Option<GameInfo<'_>>, _runtime: &mut Runtime<'_>) -> bool {
        true
    }
    fn load_game_special(
        &mut self,
        _game_type: u32,
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
    fn cheat_set(&mut self, _index: u32, _enabled: bool, _code: Option<&CStr>) {}
    fn region(&self) -> Region {
        Region::Ntsc
    }
    fn memory_data(&mut self, _id: u32) -> *mut c_void {
        ptr::null_mut()
    }
    fn memory_size(&self, _id: u32) -> usize {
        0
    }
    fn hw_context_reset(&mut self, _runtime: &mut Runtime<'_>) {}
    fn hw_context_destroy(&mut self, _runtime: &mut Runtime<'_>) {}
}

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
        let mut owned = Vec::with_capacity(variables.len());
        for variable in variables {
            owned.push((
                sanitize_cstring(&variable.key),
                sanitize_cstring(&variable.value),
            ));
        }

        let mut raw = Vec::with_capacity(owned.len() + 1);
        for (key, value) in &owned {
            raw.push(RawVariable {
                key: key.as_ptr(),
                value: value.as_ptr(),
            });
        }
        raw.push(RawVariable::default());

        let ok = self.call_env(
            RETRO_ENVIRONMENT_SET_VARIABLES,
            raw.as_mut_ptr().cast::<c_void>(),
        );
        if ok {
            self.state.variables = Some(VariableStorage {
                _owned: owned,
                _raw: raw,
            });
        }
        ok
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
        let mut geometry = geometry;
        self.call_env(
            RETRO_ENVIRONMENT_SET_GEOMETRY,
            (&mut geometry as *mut GameGeometry).cast::<c_void>(),
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

    fn call_env(&mut self, command: u32, data: *mut c_void) -> bool {
        let Some(callback) = self.state.callbacks.environment else {
            return false;
        };
        // SAFETY: `callback` comes from the frontend via the libretro ABI.
        unsafe { callback(command, data) }
    }
}

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

    pub fn input_state(&self, port: u32, device: u32, index: u32, id: u32) -> i16 {
        let Some(callback) = self.state.callbacks.input_state else {
            return 0;
        };
        // SAFETY: `callback` comes from the frontend via the libretro ABI.
        unsafe { callback(port, device, index, id) }
    }

    pub fn joypad_pressed(&self, port: u32, button: JoypadButton) -> bool {
        self.input_state(port, RETRO_DEVICE_JOYPAD, 0, button.as_raw()) != 0
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

    pub fn current_framebuffer(&self) -> Option<u32> {
        let callback = self.state.hw_render?.get_current_framebuffer?;
        // SAFETY: Frontend provided the callback through `SET_HW_RENDER`.
        let framebuffer = u32::try_from(unsafe { callback() }).ok()?;
        if framebuffer == 0 {
            None
        } else {
            Some(framebuffer)
        }
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
                    let mut env = Environment { state };
                    core.on_set_environment(&mut env);
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

    pub fn retro_get_system_av_info(info: *mut SystemAvInfo) {
        if info.is_null() {
            return;
        }

        // Keep the frontend-facing out-param initialized even if the core callback fails.
        unsafe { *info = SystemAvInfo::default() };
        with_state(|state| {
            catch_state_callback(state, "retro_get_system_av_info", (), |state| {
                let av = state.with_core(|core, _| core.av_info());
                // SAFETY: `info` is provided by the frontend.
                unsafe { *info = av };
            });
        });
    }

    pub fn retro_set_controller_port_device(port: u32, device: u32) {
        with_state(|state| {
            catch_state_callback(state, "retro_set_controller_port_device", (), |state| {
                state.with_core(|core, _| core.set_controller_port_device(port, device));
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
                Some(unsafe { CStr::from_ptr(code) })
            };
            catch_state_callback(state, "retro_cheat_set", (), |state| {
                state.with_core(|core, _| core.cheat_set(index, enabled, code));
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
                    core.load_game_special(game_type, &games, &mut runtime)
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
                state.with_core(|core, _| core.memory_data(id))
            })
        })
    }

    pub fn retro_get_memory_size(id: u32) -> usize {
        with_state(|state| {
            catch_state_callback(state, "retro_get_memory_size", 0, |state| {
                state.with_core(|core, _| core.memory_size(id))
            })
        })
    }
}

#[macro_export]
macro_rules! export_core {
    ($factory:expr) => {
        #[doc(hidden)]
        fn __libretro_create_core() -> Box<dyn $crate::Core> {
            Box::new($factory)
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
        pub extern "C" fn retro_get_system_av_info(info: *mut $crate::SystemAvInfo) {
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

struct VariableStorage {
    _owned: Vec<(CString, CString)>,
    _raw: Vec<RawVariable>,
}

struct ContentInfoOverrideStorage {
    _extensions: Vec<CString>,
    _raw: Vec<RawContentInfoOverride>,
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
    callbacks: CoreCallbacks,
    system_info: Option<OwnedSystemInfo>,
    variables: Option<VariableStorage>,
    content_info_overrides: Option<ContentInfoOverrideStorage>,
    log_callback: Option<RawLogCallback>,
    hw_render: Option<RawHwRenderCallback>,
}

impl CoreState {
    fn with_core<T>(&mut self, f: impl FnOnce(&mut dyn Core, &mut CoreState) -> T) -> T {
        let core = self.core.take().unwrap_or_else(|| {
            let factory = *FACTORY
                .get()
                .expect("libretro core factory was not registered");
            factory()
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
        self.log_callback = None;
        self.hw_render = None;
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

fn sanitize_cstring(value: impl AsRef<str>) -> CString {
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
    with_state(|state| {
        catch_state_callback(state, "hw_context_reset", (), |state| {
            state.with_core(|core, state| {
                let mut runtime = Runtime { state };
                core.hw_context_reset(&mut runtime);
            });
        });
    });
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
    struct CapturedVideoRefresh {
        data_kind: CapturedVideoDataKind,
        width: u32,
        height: u32,
        pitch: usize,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum CapturedVideoDataKind {
        Software,
        Hardware,
        Dupe,
    }

    static CAPTURED_CONTENT_OVERRIDES: OnceLock<Mutex<Vec<CapturedContentOverride>>> =
        OnceLock::new();
    static CAPTURED_SUPPORT_NO_GAME: OnceLock<Mutex<Vec<bool>>> = OnceLock::new();
    static CAPTURED_MESSAGES: OnceLock<Mutex<Vec<CapturedMessage>>> = OnceLock::new();
    static CAPTURED_VIDEO_REFRESHES: OnceLock<Mutex<Vec<CapturedVideoRefresh>>> = OnceLock::new();
    static CAPTURED_HW_RENDER_STATE: OnceLock<Mutex<CapturedHwRenderState>> = OnceLock::new();
    static CAPTURED_GEOMETRIES: OnceLock<Mutex<Vec<GameGeometry>>> = OnceLock::new();
    static CAPTURED_LIFECYCLE_COUNTS: OnceLock<Mutex<LifecycleCallCounts>> = OnceLock::new();
    static TEST_SERIAL_GUARD: OnceLock<Mutex<()>> = OnceLock::new();

    #[derive(Clone, Copy, Debug, Default)]
    struct CapturedHwRenderState {
        preferred_context_type: HwContextType,
        supports_non_preferred_context: bool,
        accept_contexts: [Option<HwContextType>; 4],
        accept_any_context: bool,
        attempts: [Option<HwContextType>; 4],
        attempt_count: usize,
        last_callback: Option<RawHwRenderCallback>,
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

    fn captured_video_refreshes() -> &'static Mutex<Vec<CapturedVideoRefresh>> {
        CAPTURED_VIDEO_REFRESHES.get_or_init(|| Mutex::new(Vec::new()))
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

    fn reset_captured_video_refreshes() {
        captured_video_refreshes()
            .lock()
            .expect("video refresh capture mutex poisoned")
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
            state.core = Some(Box::new(core));
        });
    }

    fn clear_global_test_core() {
        with_state(|state| {
            state.reset_frontend_state();
            state.core = None;
        });
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
        if command != RETRO_ENVIRONMENT_SET_MESSAGE {
            return false;
        }

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
            .push(unsafe { *data.cast::<GameGeometry>() });
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
    fn runtime_treats_zero_hardware_framebuffer_as_unavailable() {
        let mut state = CoreState {
            hw_render: Some(RawHwRenderCallback {
                get_current_framebuffer: Some(fake_zero_current_framebuffer),
                ..RawHwRenderCallback::default()
            }),
            ..CoreState::default()
        };

        let runtime = Runtime { state: &mut state };

        assert_eq!(runtime.current_framebuffer(), None);
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
        assert!(runtime.hw_proc_address("__libretro_core_missing_symbol").is_err());
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
