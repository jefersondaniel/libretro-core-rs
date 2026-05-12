#![allow(non_camel_case_types)]

use std::ffi::{c_char, c_void};

pub const RETRO_API_VERSION: u32 = 1;

pub const RETRO_DEVICE_NONE: u32 = 0;
pub const RETRO_DEVICE_JOYPAD: u32 = 1;

pub const RETRO_DEVICE_ID_JOYPAD_B: u32 = 0;
pub const RETRO_DEVICE_ID_JOYPAD_Y: u32 = 1;
pub const RETRO_DEVICE_ID_JOYPAD_SELECT: u32 = 2;
pub const RETRO_DEVICE_ID_JOYPAD_START: u32 = 3;
pub const RETRO_DEVICE_ID_JOYPAD_UP: u32 = 4;
pub const RETRO_DEVICE_ID_JOYPAD_DOWN: u32 = 5;
pub const RETRO_DEVICE_ID_JOYPAD_LEFT: u32 = 6;
pub const RETRO_DEVICE_ID_JOYPAD_RIGHT: u32 = 7;
pub const RETRO_DEVICE_ID_JOYPAD_A: u32 = 8;
pub const RETRO_DEVICE_ID_JOYPAD_X: u32 = 9;
pub const RETRO_DEVICE_ID_JOYPAD_L: u32 = 10;
pub const RETRO_DEVICE_ID_JOYPAD_R: u32 = 11;
pub const RETRO_DEVICE_ID_JOYPAD_L2: u32 = 12;
pub const RETRO_DEVICE_ID_JOYPAD_R2: u32 = 13;
pub const RETRO_DEVICE_ID_JOYPAD_L3: u32 = 14;
pub const RETRO_DEVICE_ID_JOYPAD_R3: u32 = 15;

pub const RETRO_REGION_NTSC: u32 = 0;
pub const RETRO_REGION_PAL: u32 = 1;

pub const RETRO_ENVIRONMENT_SET_MESSAGE: u32 = 6;
pub const RETRO_ENVIRONMENT_SET_PIXEL_FORMAT: u32 = 10;
pub const RETRO_ENVIRONMENT_SET_HW_RENDER: u32 = 14;
pub const RETRO_ENVIRONMENT_GET_VARIABLE: u32 = 15;
pub const RETRO_ENVIRONMENT_SET_VARIABLES: u32 = 16;
pub const RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE: u32 = 17;
pub const RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME: u32 = 18;
pub const RETRO_ENVIRONMENT_GET_LOG_INTERFACE: u32 = 27;
pub const RETRO_ENVIRONMENT_SET_GEOMETRY: u32 = 37;
pub const RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER: u32 = 56;
pub const RETRO_ENVIRONMENT_SET_CONTENT_INFO_OVERRIDE: u32 = 65;

pub const RETRO_HW_FRAME_BUFFER_VALID: *const c_void = usize::MAX as *const c_void;

pub type retro_proc_address_t = Option<unsafe extern "C" fn()>;
pub type retro_environment_t = Option<unsafe extern "C" fn(cmd: u32, data: *mut c_void) -> bool>;
pub type retro_video_refresh_t =
    Option<unsafe extern "C" fn(data: *const c_void, width: u32, height: u32, pitch: usize)>;
pub type retro_audio_sample_t = Option<unsafe extern "C" fn(left: i16, right: i16)>;
pub type retro_audio_sample_batch_t =
    Option<unsafe extern "C" fn(data: *const i16, frames: usize) -> usize>;
pub type retro_input_poll_t = Option<unsafe extern "C" fn()>;
pub type retro_input_state_t =
    Option<unsafe extern "C" fn(port: u32, device: u32, index: u32, id: u32) -> i16>;
pub type retro_log_printf_t =
    Option<unsafe extern "C" fn(level: retro_log_level, fmt: *const c_char, ...)>;
pub type retro_hw_context_reset_t = Option<unsafe extern "C" fn()>;
pub type retro_hw_get_current_framebuffer_t = Option<unsafe extern "C" fn() -> usize>;
pub type retro_hw_get_proc_address_t =
    Option<unsafe extern "C" fn(sym: *const c_char) -> retro_proc_address_t>;

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum retro_hw_context_type {
    #[default]
    None = 0,
    OpenGl = 1,
    OpenGlEs2 = 2,
    OpenGlCore = 3,
    OpenGlEs3 = 4,
    OpenGlEsVersion = 5,
    Vulkan = 6,
    D3d11 = 7,
    D3d10 = 8,
    D3d12 = 9,
    D3d9 = 10,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum retro_pixel_format {
    #[default]
    _0Rgb1555 = 0,
    Xrgb8888 = 1,
    Rgb565 = 2,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum retro_log_level {
    #[default]
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct retro_system_info {
    pub library_name: *const c_char,
    pub library_version: *const c_char,
    pub valid_extensions: *const c_char,
    pub need_fullpath: bool,
    pub block_extract: bool,
}

impl Default for retro_system_info {
    fn default() -> Self {
        Self {
            library_name: std::ptr::null(),
            library_version: std::ptr::null(),
            valid_extensions: std::ptr::null(),
            need_fullpath: false,
            block_extract: false,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_game_geometry {
    pub base_width: u32,
    pub base_height: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub aspect_ratio: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_system_timing {
    pub fps: f64,
    pub sample_rate: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct retro_message {
    pub msg: *const c_char,
    pub frames: u32,
}

impl Default for retro_message {
    fn default() -> Self {
        Self {
            msg: std::ptr::null(),
            frames: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_system_av_info {
    pub geometry: retro_game_geometry,
    pub timing: retro_system_timing,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct retro_variable {
    pub key: *const c_char,
    pub value: *const c_char,
}

impl Default for retro_variable {
    fn default() -> Self {
        Self {
            key: std::ptr::null(),
            value: std::ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_log_callback {
    pub log: retro_log_printf_t,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct retro_system_content_info_override {
    pub extensions: *const c_char,
    pub need_fullpath: bool,
    pub persistent_data: bool,
}

impl Default for retro_system_content_info_override {
    fn default() -> Self {
        Self {
            extensions: std::ptr::null(),
            need_fullpath: false,
            persistent_data: false,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct retro_game_info {
    pub path: *const c_char,
    pub data: *const c_void,
    pub size: usize,
    pub meta: *const c_char,
}

impl Default for retro_game_info {
    fn default() -> Self {
        Self {
            path: std::ptr::null(),
            data: std::ptr::null(),
            size: 0,
            meta: std::ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_hw_render_callback {
    pub context_type: retro_hw_context_type,
    pub context_reset: retro_hw_context_reset_t,
    pub get_current_framebuffer: retro_hw_get_current_framebuffer_t,
    pub get_proc_address: retro_hw_get_proc_address_t,
    pub depth: bool,
    pub stencil: bool,
    pub bottom_left_origin: bool,
    pub version_major: u32,
    pub version_minor: u32,
    pub cache_context: bool,
    pub context_destroy: retro_hw_context_reset_t,
    pub debug_context: bool,
}

// Mirrors `include/libretro.h` for 32-bit frontends. These assertions compile
// into ARMv7 builds so callback structs cannot silently drift from the C ABI.
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(std::mem::size_of::<retro_system_info>() == 16);
    assert!(std::mem::align_of::<retro_system_info>() == 4);
    assert!(std::mem::offset_of!(retro_system_info, need_fullpath) == 12);

    assert!(std::mem::size_of::<retro_system_av_info>() == 40);

    assert!(std::mem::size_of::<retro_message>() == 8);
    assert!(std::mem::align_of::<retro_message>() == 4);
    assert!(std::mem::offset_of!(retro_message, msg) == 0);
    assert!(std::mem::offset_of!(retro_message, frames) == 4);

    assert!(std::mem::size_of::<retro_variable>() == 8);
    assert!(std::mem::align_of::<retro_variable>() == 4);
    assert!(std::mem::offset_of!(retro_variable, key) == 0);
    assert!(std::mem::offset_of!(retro_variable, value) == 4);

    assert!(std::mem::size_of::<retro_system_content_info_override>() == 8);
    assert!(std::mem::align_of::<retro_system_content_info_override>() == 4);
    assert!(std::mem::offset_of!(retro_system_content_info_override, extensions) == 0);
    assert!(std::mem::offset_of!(retro_system_content_info_override, need_fullpath) == 4);
    assert!(std::mem::offset_of!(retro_system_content_info_override, persistent_data) == 5);

    assert!(std::mem::size_of::<retro_game_info>() == 16);
    assert!(std::mem::align_of::<retro_game_info>() == 4);

    assert!(std::mem::size_of::<retro_hw_render_callback>() == 40);
    assert!(std::mem::align_of::<retro_hw_render_callback>() == 4);
    assert!(std::mem::offset_of!(retro_hw_render_callback, context_type) == 0);
    assert!(std::mem::offset_of!(retro_hw_render_callback, context_reset) == 4);
    assert!(std::mem::offset_of!(retro_hw_render_callback, get_current_framebuffer) == 8);
    assert!(std::mem::offset_of!(retro_hw_render_callback, get_proc_address) == 12);
    assert!(std::mem::offset_of!(retro_hw_render_callback, depth) == 16);
    assert!(std::mem::offset_of!(retro_hw_render_callback, stencil) == 17);
    assert!(std::mem::offset_of!(retro_hw_render_callback, bottom_left_origin) == 18);
    assert!(std::mem::offset_of!(retro_hw_render_callback, version_major) == 20);
    assert!(std::mem::offset_of!(retro_hw_render_callback, version_minor) == 24);
    assert!(std::mem::offset_of!(retro_hw_render_callback, cache_context) == 28);
    assert!(std::mem::offset_of!(retro_hw_render_callback, context_destroy) == 32);
    assert!(std::mem::offset_of!(retro_hw_render_callback, debug_context) == 36);
};

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(std::mem::size_of::<retro_system_info>() == 32);
    assert!(std::mem::align_of::<retro_system_info>() == 8);
    assert!(std::mem::offset_of!(retro_system_info, need_fullpath) == 24);
    assert!(std::mem::offset_of!(retro_system_info, block_extract) == 25);

    assert!(std::mem::size_of::<retro_system_av_info>() == 40);

    assert!(std::mem::size_of::<retro_message>() == 16);
    assert!(std::mem::align_of::<retro_message>() == 8);
    assert!(std::mem::offset_of!(retro_message, msg) == 0);
    assert!(std::mem::offset_of!(retro_message, frames) == 8);

    assert!(std::mem::size_of::<retro_variable>() == 16);
    assert!(std::mem::align_of::<retro_variable>() == 8);
    assert!(std::mem::offset_of!(retro_variable, key) == 0);
    assert!(std::mem::offset_of!(retro_variable, value) == 8);

    assert!(std::mem::size_of::<retro_system_content_info_override>() == 16);
    assert!(std::mem::align_of::<retro_system_content_info_override>() == 8);
    assert!(std::mem::offset_of!(retro_system_content_info_override, extensions) == 0);
    assert!(std::mem::offset_of!(retro_system_content_info_override, need_fullpath) == 8);
    assert!(std::mem::offset_of!(retro_system_content_info_override, persistent_data) == 9);

    assert!(std::mem::size_of::<retro_game_info>() == 32);
    assert!(std::mem::align_of::<retro_game_info>() == 8);
    assert!(std::mem::offset_of!(retro_game_info, path) == 0);
    assert!(std::mem::offset_of!(retro_game_info, data) == 8);
    assert!(std::mem::offset_of!(retro_game_info, size) == 16);
    assert!(std::mem::offset_of!(retro_game_info, meta) == 24);

    assert!(std::mem::size_of::<retro_hw_render_callback>() == 64);
    assert!(std::mem::align_of::<retro_hw_render_callback>() == 8);
    assert!(std::mem::offset_of!(retro_hw_render_callback, context_type) == 0);
    assert!(std::mem::offset_of!(retro_hw_render_callback, context_reset) == 8);
    assert!(std::mem::offset_of!(retro_hw_render_callback, get_current_framebuffer) == 16);
    assert!(std::mem::offset_of!(retro_hw_render_callback, get_proc_address) == 24);
    assert!(std::mem::offset_of!(retro_hw_render_callback, depth) == 32);
    assert!(std::mem::offset_of!(retro_hw_render_callback, stencil) == 33);
    assert!(std::mem::offset_of!(retro_hw_render_callback, bottom_left_origin) == 34);
    assert!(std::mem::offset_of!(retro_hw_render_callback, version_major) == 36);
    assert!(std::mem::offset_of!(retro_hw_render_callback, version_minor) == 40);
    assert!(std::mem::offset_of!(retro_hw_render_callback, cache_context) == 44);
    assert!(std::mem::offset_of!(retro_hw_render_callback, context_destroy) == 48);
    assert!(std::mem::offset_of!(retro_hw_render_callback, debug_context) == 56);
};
