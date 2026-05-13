#![allow(non_camel_case_types)]

use std::ffi::{c_char, c_void};

pub const RETRO_API_VERSION: u32 = 1;
pub const RETRO_ENVIRONMENT_EXPERIMENTAL: u32 = 0x10000;
// ABI flag bit reserved by libretro.h for frontend-private commands.
#[allow(dead_code)]
pub const RETRO_ENVIRONMENT_PRIVATE: u32 = 0x20000;

pub const RETRO_DEVICE_TYPE_SHIFT: u32 = 8;
pub const RETRO_DEVICE_MASK: u32 = (1 << RETRO_DEVICE_TYPE_SHIFT) - 1;
pub const RETRO_DEVICE_NONE: u32 = 0;
pub const RETRO_DEVICE_JOYPAD: u32 = 1;
pub const RETRO_DEVICE_MOUSE: u32 = 2;
pub const RETRO_DEVICE_KEYBOARD: u32 = 3;
pub const RETRO_DEVICE_LIGHTGUN: u32 = 4;
pub const RETRO_DEVICE_ANALOG: u32 = 5;
pub const RETRO_DEVICE_POINTER: u32 = 6;

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
pub const RETRO_DEVICE_ID_JOYPAD_MASK: u32 = 256;

pub const RETRO_DEVICE_INDEX_ANALOG_LEFT: u32 = 0;
pub const RETRO_DEVICE_INDEX_ANALOG_RIGHT: u32 = 1;
pub const RETRO_DEVICE_INDEX_ANALOG_BUTTON: u32 = 2;
pub const RETRO_DEVICE_ID_ANALOG_X: u32 = 0;
pub const RETRO_DEVICE_ID_ANALOG_Y: u32 = 1;

pub const RETRO_DEVICE_ID_MOUSE_X: u32 = 0;
pub const RETRO_DEVICE_ID_MOUSE_Y: u32 = 1;
pub const RETRO_DEVICE_ID_MOUSE_LEFT: u32 = 2;
pub const RETRO_DEVICE_ID_MOUSE_RIGHT: u32 = 3;
pub const RETRO_DEVICE_ID_MOUSE_WHEELUP: u32 = 4;
pub const RETRO_DEVICE_ID_MOUSE_WHEELDOWN: u32 = 5;
pub const RETRO_DEVICE_ID_MOUSE_MIDDLE: u32 = 6;
pub const RETRO_DEVICE_ID_MOUSE_HORIZ_WHEELUP: u32 = 7;
pub const RETRO_DEVICE_ID_MOUSE_HORIZ_WHEELDOWN: u32 = 8;
pub const RETRO_DEVICE_ID_MOUSE_BUTTON_4: u32 = 9;
pub const RETRO_DEVICE_ID_MOUSE_BUTTON_5: u32 = 10;

pub const RETRO_DEVICE_ID_LIGHTGUN_SCREEN_X: u32 = 13;
pub const RETRO_DEVICE_ID_LIGHTGUN_SCREEN_Y: u32 = 14;
pub const RETRO_DEVICE_ID_LIGHTGUN_IS_OFFSCREEN: u32 = 15;
pub const RETRO_DEVICE_ID_LIGHTGUN_TRIGGER: u32 = 2;
pub const RETRO_DEVICE_ID_LIGHTGUN_CURSOR: u32 = 3;
pub const RETRO_DEVICE_ID_LIGHTGUN_RELOAD: u32 = 16;
pub const RETRO_DEVICE_ID_LIGHTGUN_AUX_A: u32 = 3;
pub const RETRO_DEVICE_ID_LIGHTGUN_AUX_B: u32 = 4;
pub const RETRO_DEVICE_ID_LIGHTGUN_START: u32 = 6;
pub const RETRO_DEVICE_ID_LIGHTGUN_SELECT: u32 = 7;
pub const RETRO_DEVICE_ID_LIGHTGUN_AUX_C: u32 = 8;
pub const RETRO_DEVICE_ID_LIGHTGUN_DPAD_UP: u32 = 9;
pub const RETRO_DEVICE_ID_LIGHTGUN_DPAD_DOWN: u32 = 10;
pub const RETRO_DEVICE_ID_LIGHTGUN_DPAD_LEFT: u32 = 11;
pub const RETRO_DEVICE_ID_LIGHTGUN_DPAD_RIGHT: u32 = 12;
pub const RETRO_DEVICE_ID_LIGHTGUN_X: u32 = 0;
pub const RETRO_DEVICE_ID_LIGHTGUN_Y: u32 = 1;
pub const RETRO_DEVICE_ID_LIGHTGUN_TURBO: u32 = 4;
pub const RETRO_DEVICE_ID_LIGHTGUN_PAUSE: u32 = 5;

pub const RETRO_DEVICE_ID_POINTER_X: u32 = 0;
pub const RETRO_DEVICE_ID_POINTER_Y: u32 = 1;
pub const RETRO_DEVICE_ID_POINTER_PRESSED: u32 = 2;
pub const RETRO_DEVICE_ID_POINTER_COUNT: u32 = 3;
pub const RETRO_DEVICE_ID_POINTER_IS_OFFSCREEN: u32 = 15;

pub const RETRO_REGION_NTSC: u32 = 0;
pub const RETRO_REGION_PAL: u32 = 1;

pub const RETRO_MEMORY_MASK: u32 = 0xff;
pub const RETRO_MEMORY_SAVE_RAM: u32 = 0;
pub const RETRO_MEMORY_RTC: u32 = 1;
pub const RETRO_MEMORY_SYSTEM_RAM: u32 = 2;
pub const RETRO_MEMORY_VIDEO_RAM: u32 = 3;
pub const RETRO_MEMORY_ROM: u32 = 4;

pub const RETRO_ENVIRONMENT_SET_ROTATION: u32 = 1;
pub const RETRO_ENVIRONMENT_GET_OVERSCAN: u32 = 2;
pub const RETRO_ENVIRONMENT_GET_CAN_DUPE: u32 = 3;
pub const RETRO_ENVIRONMENT_SET_MESSAGE: u32 = 6;
pub const RETRO_ENVIRONMENT_SHUTDOWN: u32 = 7;
pub const RETRO_ENVIRONMENT_SET_PERFORMANCE_LEVEL: u32 = 8;
pub const RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY: u32 = 9;
pub const RETRO_ENVIRONMENT_SET_PIXEL_FORMAT: u32 = 10;
pub const RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS: u32 = 11;
pub const RETRO_ENVIRONMENT_SET_KEYBOARD_CALLBACK: u32 = 12;
pub const RETRO_ENVIRONMENT_SET_DISK_CONTROL_INTERFACE: u32 = 13;
pub const RETRO_ENVIRONMENT_SET_HW_RENDER: u32 = 14;
pub const RETRO_ENVIRONMENT_GET_VARIABLE: u32 = 15;
pub const RETRO_ENVIRONMENT_SET_VARIABLES: u32 = 16;
pub const RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE: u32 = 17;
pub const RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME: u32 = 18;
pub const RETRO_ENVIRONMENT_GET_LIBRETRO_PATH: u32 = 19;
pub const RETRO_ENVIRONMENT_SET_FRAME_TIME_CALLBACK: u32 = 21;
pub const RETRO_ENVIRONMENT_SET_AUDIO_CALLBACK: u32 = 22;
pub const RETRO_ENVIRONMENT_GET_RUMBLE_INTERFACE: u32 = 23;
pub const RETRO_ENVIRONMENT_GET_INPUT_DEVICE_CAPABILITIES: u32 = 24;
pub const RETRO_ENVIRONMENT_GET_SENSOR_INTERFACE: u32 = 25 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_GET_CAMERA_INTERFACE: u32 = 26 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_GET_LOG_INTERFACE: u32 = 27;
pub const RETRO_ENVIRONMENT_GET_PERF_INTERFACE: u32 = 28;
pub const RETRO_ENVIRONMENT_GET_LOCATION_INTERFACE: u32 = 29;
pub const RETRO_ENVIRONMENT_GET_CORE_ASSETS_DIRECTORY: u32 = 30;
pub const RETRO_ENVIRONMENT_GET_CONTENT_DIRECTORY: u32 = 30;
pub const RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY: u32 = 31;
pub const RETRO_ENVIRONMENT_SET_SYSTEM_AV_INFO: u32 = 32;
pub const RETRO_ENVIRONMENT_SET_PROC_ADDRESS_CALLBACK: u32 = 33;
pub const RETRO_ENVIRONMENT_SET_SUBSYSTEM_INFO: u32 = 34;
pub const RETRO_ENVIRONMENT_SET_CONTROLLER_INFO: u32 = 35;
pub const RETRO_ENVIRONMENT_SET_MEMORY_MAPS: u32 = 36 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_SET_GEOMETRY: u32 = 37;
pub const RETRO_ENVIRONMENT_GET_USERNAME: u32 = 38;
pub const RETRO_ENVIRONMENT_GET_LANGUAGE: u32 = 39;
pub const RETRO_ENVIRONMENT_GET_CURRENT_SOFTWARE_FRAMEBUFFER: u32 =
    40 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_GET_HW_RENDER_INTERFACE: u32 = 41 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_SET_SUPPORT_ACHIEVEMENTS: u32 = 42 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE: u32 =
    43 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_GET_AUDIO_VIDEO_ENABLE: u32 = 47 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_SET_SERIALIZATION_QUIRKS: u32 = 44;
pub const RETRO_ENVIRONMENT_SET_HW_SHARED_CONTEXT: u32 = 44 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_GET_VFS_INTERFACE: u32 = 45 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_GET_LED_INTERFACE: u32 = 46 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_GET_MIDI_INTERFACE: u32 = 48 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_GET_FASTFORWARDING: u32 = 49 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_GET_TARGET_REFRESH_RATE: u32 = 50 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_GET_INPUT_BITMASKS: u32 = 51 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_GET_CORE_OPTIONS_VERSION: u32 = 52;
pub const RETRO_ENVIRONMENT_SET_CORE_OPTIONS: u32 = 53;
pub const RETRO_ENVIRONMENT_SET_CORE_OPTIONS_INTL: u32 = 54;
pub const RETRO_ENVIRONMENT_SET_CORE_OPTIONS_DISPLAY: u32 = 55;
pub const RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER: u32 = 56;
pub const RETRO_ENVIRONMENT_GET_DISK_CONTROL_INTERFACE_VERSION: u32 = 57;
pub const RETRO_ENVIRONMENT_SET_DISK_CONTROL_EXT_INTERFACE: u32 = 58;
pub const RETRO_ENVIRONMENT_GET_MESSAGE_INTERFACE_VERSION: u32 = 59;
pub const RETRO_ENVIRONMENT_SET_MESSAGE_EXT: u32 = 60;
pub const RETRO_ENVIRONMENT_GET_INPUT_MAX_USERS: u32 = 61;
pub const RETRO_ENVIRONMENT_SET_AUDIO_BUFFER_STATUS_CALLBACK: u32 = 62;
pub const RETRO_ENVIRONMENT_SET_MINIMUM_AUDIO_LATENCY: u32 = 63;
pub const RETRO_ENVIRONMENT_SET_FASTFORWARDING_OVERRIDE: u32 = 64;
pub const RETRO_ENVIRONMENT_SET_CONTENT_INFO_OVERRIDE: u32 = 65;
pub const RETRO_ENVIRONMENT_GET_GAME_INFO_EXT: u32 = 66;
pub const RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2: u32 = 67;
pub const RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL: u32 = 68;
pub const RETRO_ENVIRONMENT_SET_CORE_OPTIONS_UPDATE_DISPLAY_CALLBACK: u32 = 69;
pub const RETRO_ENVIRONMENT_SET_VARIABLE: u32 = 70;
pub const RETRO_ENVIRONMENT_GET_THROTTLE_STATE: u32 = 71 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_GET_SAVESTATE_CONTEXT: u32 = 72 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT: u32 =
    73 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_GET_JIT_CAPABLE: u32 = 74;
pub const RETRO_ENVIRONMENT_GET_MICROPHONE_INTERFACE: u32 = 75 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_GET_DEVICE_POWER: u32 = 77 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_SET_NETPACKET_INTERFACE: u32 = 78;
pub const RETRO_ENVIRONMENT_GET_PLAYLIST_DIRECTORY: u32 = 79;
pub const RETRO_ENVIRONMENT_GET_FILE_BROWSER_START_DIRECTORY: u32 = 80;
pub const RETRO_ENVIRONMENT_GET_TARGET_SAMPLE_RATE: u32 = 81 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_GET_NETPLAY_CLIENT_INDEX: u32 = 82 | RETRO_ENVIRONMENT_EXPERIMENTAL;

pub const RETRO_AV_ENABLE_VIDEO: u32 = 1 << 0;
pub const RETRO_AV_ENABLE_AUDIO: u32 = 1 << 1;
pub const RETRO_AV_ENABLE_FAST_SAVESTATES: u32 = 1 << 2;
pub const RETRO_AV_ENABLE_HARD_DISABLE_AUDIO: u32 = 1 << 3;
pub const RETRO_THROTTLE_NONE: u32 = 0;
pub const RETRO_THROTTLE_FRAME_STEPPING: u32 = 1;
pub const RETRO_THROTTLE_FAST_FORWARD: u32 = 2;
pub const RETRO_THROTTLE_SLOW_MOTION: u32 = 3;
pub const RETRO_THROTTLE_REWINDING: u32 = 4;
pub const RETRO_THROTTLE_VSYNC: u32 = 5;
pub const RETRO_THROTTLE_UNBLOCKED: u32 = 6;
pub const RETRO_POWERSTATE_NO_ESTIMATE: i32 = -1;
pub const RETRO_NETPACKET_UNRELIABLE: i32 = 0;
pub const RETRO_NETPACKET_RELIABLE: i32 = 1 << 0;
pub const RETRO_NETPACKET_UNSEQUENCED: i32 = 1 << 1;
pub const RETRO_NETPACKET_FLUSH_HINT: i32 = 1 << 2;
pub const RETRO_NETPACKET_BROADCAST: u16 = 0xffff;
pub const RETRO_MICROPHONE_INTERFACE_VERSION: u32 = 1;
pub const RETRO_NUM_CORE_OPTION_VALUES_MAX: usize = 128;
pub const RETRO_SERIALIZATION_QUIRK_INCOMPLETE: u64 = 1 << 0;
pub const RETRO_SERIALIZATION_QUIRK_MUST_INITIALIZE: u64 = 1 << 1;
pub const RETRO_SERIALIZATION_QUIRK_CORE_VARIABLE_SIZE: u64 = 1 << 2;
pub const RETRO_SERIALIZATION_QUIRK_FRONT_VARIABLE_SIZE: u64 = 1 << 3;
pub const RETRO_SERIALIZATION_QUIRK_SINGLE_SESSION: u64 = 1 << 4;
pub const RETRO_SERIALIZATION_QUIRK_ENDIAN_DEPENDENT: u64 = 1 << 5;
pub const RETRO_SERIALIZATION_QUIRK_PLATFORM_DEPENDENT: u64 = 1 << 6;
pub const RETRO_MEMDESC_CONST: u64 = 1 << 0;
pub const RETRO_MEMDESC_BIGENDIAN: u64 = 1 << 1;
pub const RETRO_MEMDESC_SYSTEM_RAM: u64 = 1 << 2;
pub const RETRO_MEMDESC_SAVE_RAM: u64 = 1 << 3;
pub const RETRO_MEMDESC_VIDEO_RAM: u64 = 1 << 4;
pub const RETRO_MEMDESC_ALIGN_2: u64 = 1 << 16;
pub const RETRO_MEMDESC_ALIGN_4: u64 = 2 << 16;
pub const RETRO_MEMDESC_ALIGN_8: u64 = 3 << 16;
pub const RETRO_MEMDESC_MINSIZE_2: u64 = 1 << 24;
pub const RETRO_MEMDESC_MINSIZE_4: u64 = 2 << 24;
pub const RETRO_MEMDESC_MINSIZE_8: u64 = 3 << 24;
pub const RETRO_MEMORY_ACCESS_WRITE: u32 = 1 << 0;
pub const RETRO_MEMORY_ACCESS_READ: u32 = 1 << 1;
pub const RETRO_MEMORY_TYPE_CACHED: u32 = 1 << 0;
pub const RETRO_VFS_FILE_ACCESS_READ: u32 = 1 << 0;
pub const RETRO_VFS_FILE_ACCESS_WRITE: u32 = 1 << 1;
#[allow(dead_code)]
pub const RETRO_VFS_FILE_ACCESS_READ_WRITE: u32 =
    RETRO_VFS_FILE_ACCESS_READ | RETRO_VFS_FILE_ACCESS_WRITE;
pub const RETRO_VFS_FILE_ACCESS_UPDATE_EXISTING: u32 = 1 << 2;
#[allow(dead_code)]
pub const RETRO_VFS_FILE_ACCESS_HINT_NONE: u32 = 0;
pub const RETRO_VFS_FILE_ACCESS_HINT_FREQUENT_ACCESS: u32 = 1 << 0;
pub const RETRO_VFS_SEEK_POSITION_START: i32 = 0;
pub const RETRO_VFS_SEEK_POSITION_CURRENT: i32 = 1;
pub const RETRO_VFS_SEEK_POSITION_END: i32 = 2;
pub const RETRO_VFS_STAT_IS_VALID: u32 = 1 << 0;
pub const RETRO_VFS_STAT_IS_DIRECTORY: u32 = 1 << 1;
pub const RETRO_VFS_STAT_IS_CHARACTER_SPECIAL: u32 = 1 << 2;
pub const RETRO_SENSOR_ACCELEROMETER_X: u32 = 0;
pub const RETRO_SENSOR_ACCELEROMETER_Y: u32 = 1;
pub const RETRO_SENSOR_ACCELEROMETER_Z: u32 = 2;
pub const RETRO_SENSOR_GYROSCOPE_X: u32 = 3;
pub const RETRO_SENSOR_GYROSCOPE_Y: u32 = 4;
pub const RETRO_SENSOR_GYROSCOPE_Z: u32 = 5;
pub const RETRO_SENSOR_ILLUMINANCE: u32 = 6;
pub const RETRO_SIMD_SSE: u64 = 1 << 0;
pub const RETRO_SIMD_SSE2: u64 = 1 << 1;
pub const RETRO_SIMD_VMX: u64 = 1 << 2;
pub const RETRO_SIMD_VMX128: u64 = 1 << 3;
pub const RETRO_SIMD_AVX: u64 = 1 << 4;
pub const RETRO_SIMD_NEON: u64 = 1 << 5;
pub const RETRO_SIMD_SSE3: u64 = 1 << 6;
pub const RETRO_SIMD_SSSE3: u64 = 1 << 7;
pub const RETRO_SIMD_MMX: u64 = 1 << 8;
pub const RETRO_SIMD_MMXEXT: u64 = 1 << 9;
pub const RETRO_SIMD_SSE4: u64 = 1 << 10;
pub const RETRO_SIMD_SSE42: u64 = 1 << 11;
pub const RETRO_SIMD_AVX2: u64 = 1 << 12;
pub const RETRO_SIMD_VFPU: u64 = 1 << 13;
pub const RETRO_SIMD_PS: u64 = 1 << 14;
pub const RETRO_SIMD_AES: u64 = 1 << 15;
pub const RETRO_SIMD_VFPV3: u64 = 1 << 16;
pub const RETRO_SIMD_VFPV4: u64 = 1 << 17;
pub const RETRO_SIMD_POPCNT: u64 = 1 << 18;
pub const RETRO_SIMD_MOVBE: u64 = 1 << 19;
pub const RETRO_SIMD_CMOV: u64 = 1 << 20;
pub const RETRO_SIMD_ASIMD: u64 = 1 << 21;

pub const RETRO_HW_FRAME_BUFFER_VALID: *const c_void = usize::MAX as *const c_void;

pub type retro_proc_address_t = Option<unsafe extern "C" fn()>;
pub type retro_get_proc_address_t =
    Option<unsafe extern "C" fn(sym: *const c_char) -> retro_proc_address_t>;
pub type retro_environment_t = Option<unsafe extern "C" fn(cmd: u32, data: *mut c_void) -> bool>;
pub type retro_video_refresh_t =
    Option<unsafe extern "C" fn(data: *const c_void, width: u32, height: u32, pitch: usize)>;
pub type retro_audio_sample_t = Option<unsafe extern "C" fn(left: i16, right: i16)>;
pub type retro_audio_sample_batch_t =
    Option<unsafe extern "C" fn(data: *const i16, frames: usize) -> usize>;
pub type retro_audio_callback_t = Option<unsafe extern "C" fn()>;
pub type retro_audio_set_state_callback_t = Option<unsafe extern "C" fn(enabled: bool)>;
pub type retro_audio_buffer_status_callback_t =
    Option<unsafe extern "C" fn(active: bool, occupancy: u32, underrun_likely: bool)>;
pub type retro_usec_t = i64;
pub type retro_frame_time_callback_t = Option<unsafe extern "C" fn(usec: retro_usec_t)>;
pub type retro_input_poll_t = Option<unsafe extern "C" fn()>;
pub type retro_input_state_t =
    Option<unsafe extern "C" fn(port: u32, device: u32, index: u32, id: u32) -> i16>;
pub type retro_key = u32;
pub type retro_mod = u16;
pub type retro_keyboard_event_t = Option<
    unsafe extern "C" fn(down: bool, keycode: retro_key, character: u32, key_modifiers: retro_mod),
>;
pub type retro_log_printf_t =
    Option<unsafe extern "C" fn(level: retro_log_level, fmt: *const c_char, ...)>;
pub type retro_set_led_state_t = Option<unsafe extern "C" fn(led: i32, state: i32)>;
pub type retro_set_rumble_state_t =
    Option<unsafe extern "C" fn(port: u32, effect: retro_rumble_effect, strength: u16) -> bool>;
pub type retro_set_sensor_state_t =
    Option<unsafe extern "C" fn(port: u32, action: retro_sensor_action, rate: u32) -> bool>;
pub type retro_sensor_get_input_t = Option<unsafe extern "C" fn(port: u32, id: u32) -> f32>;
pub type retro_location_set_interval_t =
    Option<unsafe extern "C" fn(interval_ms: u32, interval_distance: u32)>;
pub type retro_location_start_t = Option<unsafe extern "C" fn() -> bool>;
pub type retro_location_stop_t = Option<unsafe extern "C" fn()>;
pub type retro_location_get_position_t = Option<
    unsafe extern "C" fn(
        lat: *mut f64,
        lon: *mut f64,
        horiz_accuracy: *mut f64,
        vert_accuracy: *mut f64,
    ) -> bool,
>;
pub type retro_location_lifetime_status_t = Option<unsafe extern "C" fn()>;
pub type retro_camera_start_t = Option<unsafe extern "C" fn() -> bool>;
pub type retro_camera_stop_t = Option<unsafe extern "C" fn()>;
pub type retro_camera_lifetime_status_t = Option<unsafe extern "C" fn()>;
pub type retro_camera_frame_raw_framebuffer_t =
    Option<unsafe extern "C" fn(buffer: *const u32, width: u32, height: u32, pitch: usize)>;
pub type retro_camera_frame_opengl_texture_t =
    Option<unsafe extern "C" fn(texture_id: u32, texture_target: u32, affine: *const f32)>;
pub type retro_set_eject_state_t = Option<unsafe extern "C" fn(ejected: bool) -> bool>;
pub type retro_get_eject_state_t = Option<unsafe extern "C" fn() -> bool>;
pub type retro_get_image_index_t = Option<unsafe extern "C" fn() -> u32>;
pub type retro_set_image_index_t = Option<unsafe extern "C" fn(index: u32) -> bool>;
pub type retro_get_num_images_t = Option<unsafe extern "C" fn() -> u32>;
pub type retro_replace_image_index_t =
    Option<unsafe extern "C" fn(index: u32, info: *const retro_game_info) -> bool>;
pub type retro_add_image_index_t = Option<unsafe extern "C" fn() -> bool>;
pub type retro_set_initial_image_t =
    Option<unsafe extern "C" fn(index: u32, path: *const c_char) -> bool>;
pub type retro_get_image_path_t =
    Option<unsafe extern "C" fn(index: u32, s: *mut c_char, len: usize) -> bool>;
pub type retro_get_image_label_t =
    Option<unsafe extern "C" fn(index: u32, s: *mut c_char, len: usize) -> bool>;
pub type retro_netpacket_send_t =
    Option<unsafe extern "C" fn(flags: i32, buf: *const c_void, len: usize, client_id: u16)>;
pub type retro_netpacket_poll_receive_t = Option<unsafe extern "C" fn()>;
pub type retro_netpacket_start_t = Option<
    unsafe extern "C" fn(
        client_id: u16,
        send_fn: retro_netpacket_send_t,
        poll_receive_fn: retro_netpacket_poll_receive_t,
    ),
>;
pub type retro_netpacket_receive_t =
    Option<unsafe extern "C" fn(buf: *const c_void, len: usize, client_id: u16)>;
pub type retro_netpacket_stop_t = Option<unsafe extern "C" fn()>;
pub type retro_netpacket_poll_t = Option<unsafe extern "C" fn()>;
pub type retro_netpacket_connected_t = Option<unsafe extern "C" fn(client_id: u16) -> bool>;
pub type retro_netpacket_disconnected_t = Option<unsafe extern "C" fn(client_id: u16)>;
pub type retro_open_mic_t =
    Option<unsafe extern "C" fn(params: *const retro_microphone_params) -> *mut retro_microphone>;
pub type retro_close_mic_t = Option<unsafe extern "C" fn(microphone: *mut retro_microphone)>;
pub type retro_get_mic_params_t = Option<
    unsafe extern "C" fn(
        microphone: *const retro_microphone,
        params: *mut retro_microphone_params,
    ) -> bool,
>;
pub type retro_set_mic_state_t =
    Option<unsafe extern "C" fn(microphone: *mut retro_microphone, state: bool) -> bool>;
pub type retro_get_mic_state_t =
    Option<unsafe extern "C" fn(microphone: *const retro_microphone) -> bool>;
pub type retro_read_mic_t = Option<
    unsafe extern "C" fn(
        microphone: *mut retro_microphone,
        samples: *mut i16,
        num_samples: usize,
    ) -> i32,
>;
pub type retro_midi_input_enabled_t = Option<unsafe extern "C" fn() -> bool>;
pub type retro_midi_output_enabled_t = Option<unsafe extern "C" fn() -> bool>;
pub type retro_midi_read_t = Option<unsafe extern "C" fn(byte: *mut u8) -> bool>;
pub type retro_midi_write_t = Option<unsafe extern "C" fn(byte: u8, delta_time: u32) -> bool>;
pub type retro_midi_flush_t = Option<unsafe extern "C" fn() -> bool>;
pub type retro_core_options_update_display_callback_t = Option<unsafe extern "C" fn() -> bool>;
pub type retro_vfs_get_path_t =
    Option<unsafe extern "C" fn(stream: *mut retro_vfs_file_handle) -> *const c_char>;
pub type retro_vfs_open_t = Option<
    unsafe extern "C" fn(path: *const c_char, mode: u32, hints: u32) -> *mut retro_vfs_file_handle,
>;
pub type retro_vfs_close_t =
    Option<unsafe extern "C" fn(stream: *mut retro_vfs_file_handle) -> i32>;
pub type retro_vfs_size_t = Option<unsafe extern "C" fn(stream: *mut retro_vfs_file_handle) -> i64>;
pub type retro_vfs_truncate_t =
    Option<unsafe extern "C" fn(stream: *mut retro_vfs_file_handle, length: i64) -> i64>;
pub type retro_vfs_tell_t = Option<unsafe extern "C" fn(stream: *mut retro_vfs_file_handle) -> i64>;
pub type retro_vfs_seek_t = Option<
    unsafe extern "C" fn(
        stream: *mut retro_vfs_file_handle,
        offset: i64,
        seek_position: i32,
    ) -> i64,
>;
pub type retro_vfs_read_t = Option<
    unsafe extern "C" fn(stream: *mut retro_vfs_file_handle, s: *mut c_void, len: u64) -> i64,
>;
pub type retro_vfs_write_t = Option<
    unsafe extern "C" fn(stream: *mut retro_vfs_file_handle, s: *const c_void, len: u64) -> i64,
>;
pub type retro_vfs_flush_t =
    Option<unsafe extern "C" fn(stream: *mut retro_vfs_file_handle) -> i32>;
pub type retro_vfs_remove_t = Option<unsafe extern "C" fn(path: *const c_char) -> i32>;
pub type retro_vfs_rename_t =
    Option<unsafe extern "C" fn(old_path: *const c_char, new_path: *const c_char) -> i32>;
pub type retro_vfs_stat_t =
    Option<unsafe extern "C" fn(path: *const c_char, size: *mut i32) -> i32>;
pub type retro_vfs_mkdir_t = Option<unsafe extern "C" fn(dir: *const c_char) -> i32>;
pub type retro_vfs_opendir_t = Option<
    unsafe extern "C" fn(dir: *const c_char, include_hidden: bool) -> *mut retro_vfs_dir_handle,
>;
pub type retro_vfs_readdir_t =
    Option<unsafe extern "C" fn(dirstream: *mut retro_vfs_dir_handle) -> bool>;
pub type retro_vfs_dirent_get_name_t =
    Option<unsafe extern "C" fn(dirstream: *mut retro_vfs_dir_handle) -> *const c_char>;
pub type retro_vfs_dirent_is_dir_t =
    Option<unsafe extern "C" fn(dirstream: *mut retro_vfs_dir_handle) -> bool>;
pub type retro_vfs_closedir_t =
    Option<unsafe extern "C" fn(dirstream: *mut retro_vfs_dir_handle) -> i32>;
pub type retro_perf_tick_t = u64;
pub type retro_time_t = i64;
pub type retro_perf_get_time_usec_t = Option<unsafe extern "C" fn() -> retro_time_t>;
pub type retro_perf_get_counter_t = Option<unsafe extern "C" fn() -> retro_perf_tick_t>;
pub type retro_get_cpu_features_t = Option<unsafe extern "C" fn() -> u64>;
pub type retro_perf_log_t = Option<unsafe extern "C" fn()>;
pub type retro_perf_register_t = Option<unsafe extern "C" fn(counter: *mut retro_perf_counter)>;
pub type retro_perf_start_t = Option<unsafe extern "C" fn(counter: *mut retro_perf_counter)>;
pub type retro_perf_stop_t = Option<unsafe extern "C" fn(counter: *mut retro_perf_counter)>;
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum retro_hw_render_interface_type {
    Vulkan = 0,
    D3d9 = 1,
    D3d10 = 2,
    D3d11 = 3,
    D3d12 = 4,
    GskitPs2 = 5,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum retro_hw_render_context_negotiation_interface_type {
    Vulkan = 0,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum retro_pixel_format {
    #[default]
    _0Rgb1555 = 0,
    Xrgb8888 = 1,
    Rgb565 = 2,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum retro_camera_buffer {
    OpenGlTexture = 0,
    RawFramebuffer = 1,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum retro_rumble_effect {
    #[default]
    Strong = 0,
    Weak = 1,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum retro_sensor_action {
    #[default]
    AccelerometerEnable = 0,
    AccelerometerDisable = 1,
    GyroscopeEnable = 2,
    GyroscopeDisable = 3,
    IlluminanceEnable = 4,
    IlluminanceDisable = 5,
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

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum retro_message_target {
    #[default]
    All = 0,
    Osd = 1,
    Log = 2,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum retro_message_type {
    #[default]
    Notification = 0,
    NotificationAlt = 1,
    Status = 2,
    Progress = 3,
}

#[repr(i32)]
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum retro_power_state {
    #[default]
    Unknown = 0,
    Discharging = 1,
    Charging = 2,
    Charged = 3,
    PluggedIn = 4,
}

#[repr(i32)]
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum retro_savestate_context {
    #[default]
    Normal = 0,
    RunaheadSameInstance = 1,
    RunaheadSameBinary = 2,
    RollbackNetplay = 3,
    Unknown = i32::MAX,
}

#[repr(i32)]
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum retro_language {
    #[default]
    English = 0,
    Japanese = 1,
    French = 2,
    Spanish = 3,
    German = 4,
    Italian = 5,
    Dutch = 6,
    PortugueseBrazil = 7,
    PortuguesePortugal = 8,
    Russian = 9,
    Korean = 10,
    ChineseTraditional = 11,
    ChineseSimplified = 12,
    Esperanto = 13,
    Polish = 14,
    Vietnamese = 15,
    Arabic = 16,
    Greek = 17,
    Turkish = 18,
    Slovak = 19,
    Persian = 20,
    Hebrew = 21,
    Asturian = 22,
    Finnish = 23,
    Indonesian = 24,
    Swedish = 25,
    Ukrainian = 26,
    Czech = 27,
    CatalanValencia = 28,
    Catalan = 29,
    BritishEnglish = 30,
    Hungarian = 31,
    Belarusian = 32,
    Galician = 33,
    Norwegian = 34,
    Irish = 35,
}

const _: () = {
    assert!(std::mem::size_of::<retro_language>() == 4);
    assert!(std::mem::align_of::<retro_language>() == 4);
    assert!(std::mem::size_of::<retro_message_target>() == 4);
    assert!(std::mem::align_of::<retro_message_target>() == 4);
    assert!(std::mem::size_of::<retro_message_type>() == 4);
    assert!(std::mem::align_of::<retro_message_type>() == 4);
    assert!(std::mem::size_of::<retro_power_state>() == 4);
    assert!(std::mem::align_of::<retro_power_state>() == 4);
    assert!(std::mem::size_of::<retro_savestate_context>() == 4);
    assert!(std::mem::align_of::<retro_savestate_context>() == 4);
    assert!(std::mem::size_of::<retro_rumble_effect>() == 4);
    assert!(std::mem::align_of::<retro_rumble_effect>() == 4);
};

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
#[derive(Clone, Copy, Debug)]
pub struct retro_message_ext {
    pub msg: *const c_char,
    pub duration: u32,
    pub priority: u32,
    pub level: retro_log_level,
    pub target: retro_message_target,
    pub type_: retro_message_type,
    pub progress: i8,
}

impl Default for retro_message_ext {
    fn default() -> Self {
        Self {
            msg: std::ptr::null(),
            duration: 0,
            priority: 0,
            level: retro_log_level::Info,
            target: retro_message_target::Osd,
            type_: retro_message_type::Notification,
            progress: -1,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct retro_input_descriptor {
    pub port: u32,
    pub device: u32,
    pub index: u32,
    pub id: u32,
    pub description: *const c_char,
}

impl Default for retro_input_descriptor {
    fn default() -> Self {
        Self {
            port: 0,
            device: 0,
            index: 0,
            id: 0,
            description: std::ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct retro_device_power {
    pub state: retro_power_state,
    pub seconds: i32,
    pub percent: i8,
}

impl Default for retro_device_power {
    fn default() -> Self {
        Self {
            state: retro_power_state::Unknown,
            seconds: RETRO_POWERSTATE_NO_ESTIMATE,
            percent: -1,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct retro_memory_descriptor {
    pub flags: u64,
    pub ptr: *mut c_void,
    pub offset: usize,
    pub start: usize,
    pub select: usize,
    pub disconnect: usize,
    pub len: usize,
    pub addrspace: *const c_char,
}

impl Default for retro_memory_descriptor {
    fn default() -> Self {
        Self {
            flags: 0,
            ptr: std::ptr::null_mut(),
            offset: 0,
            start: 0,
            select: 0,
            disconnect: 0,
            len: 0,
            addrspace: std::ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct retro_memory_map {
    pub descriptors: *const retro_memory_descriptor,
    pub num_descriptors: u32,
}

impl Default for retro_memory_map {
    fn default() -> Self {
        Self {
            descriptors: std::ptr::null(),
            num_descriptors: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct retro_controller_description {
    pub desc: *const c_char,
    pub id: u32,
}

impl Default for retro_controller_description {
    fn default() -> Self {
        Self {
            desc: std::ptr::null(),
            id: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct retro_controller_info {
    pub types: *const retro_controller_description,
    pub num_types: u32,
}

impl Default for retro_controller_info {
    fn default() -> Self {
        Self {
            types: std::ptr::null(),
            num_types: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct retro_framebuffer {
    pub data: *mut c_void,
    pub width: u32,
    pub height: u32,
    pub pitch: usize,
    pub format: retro_pixel_format,
    pub access_flags: u32,
    pub memory_flags: u32,
}

impl Default for retro_framebuffer {
    fn default() -> Self {
        Self {
            data: std::ptr::null_mut(),
            width: 0,
            height: 0,
            pitch: 0,
            format: retro_pixel_format::default(),
            access_flags: 0,
            memory_flags: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_led_interface {
    pub set_led_state: retro_set_led_state_t,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_rumble_interface {
    pub set_rumble_state: retro_set_rumble_state_t,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct retro_perf_counter {
    pub ident: *const c_char,
    pub start: retro_perf_tick_t,
    pub total: retro_perf_tick_t,
    pub call_cnt: retro_perf_tick_t,
    pub registered: bool,
}

impl Default for retro_perf_counter {
    fn default() -> Self {
        Self {
            ident: std::ptr::null(),
            start: 0,
            total: 0,
            call_cnt: 0,
            registered: false,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_perf_callback {
    pub get_time_usec: retro_perf_get_time_usec_t,
    pub get_cpu_features: retro_get_cpu_features_t,
    pub get_perf_counter: retro_perf_get_counter_t,
    pub perf_register: retro_perf_register_t,
    pub perf_start: retro_perf_start_t,
    pub perf_stop: retro_perf_stop_t,
    pub perf_log: retro_perf_log_t,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_keyboard_callback {
    pub callback: retro_keyboard_event_t,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_sensor_interface {
    pub set_sensor_state: retro_set_sensor_state_t,
    pub get_sensor_input: retro_sensor_get_input_t,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_camera_callback {
    pub caps: u64,
    pub width: u32,
    pub height: u32,
    pub start: retro_camera_start_t,
    pub stop: retro_camera_stop_t,
    pub frame_raw_framebuffer: retro_camera_frame_raw_framebuffer_t,
    pub frame_opengl_texture: retro_camera_frame_opengl_texture_t,
    pub initialized: retro_camera_lifetime_status_t,
    pub deinitialized: retro_camera_lifetime_status_t,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_location_callback {
    pub start: retro_location_start_t,
    pub stop: retro_location_stop_t,
    pub get_position: retro_location_get_position_t,
    pub set_interval: retro_location_set_interval_t,
    pub initialized: retro_location_lifetime_status_t,
    pub deinitialized: retro_location_lifetime_status_t,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct retro_subsystem_memory_info {
    pub extension: *const c_char,
    pub memory_type: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct retro_subsystem_rom_info {
    pub desc: *const c_char,
    pub valid_extensions: *const c_char,
    pub need_fullpath: bool,
    pub block_extract: bool,
    pub required: bool,
    pub memory: *const retro_subsystem_memory_info,
    pub num_memory: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct retro_subsystem_info {
    pub desc: *const c_char,
    pub ident: *const c_char,
    pub roms: *const retro_subsystem_rom_info,
    pub num_roms: u32,
    pub id: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_disk_control_callback {
    pub set_eject_state: retro_set_eject_state_t,
    pub get_eject_state: retro_get_eject_state_t,
    pub get_image_index: retro_get_image_index_t,
    pub set_image_index: retro_set_image_index_t,
    pub get_num_images: retro_get_num_images_t,
    pub replace_image_index: retro_replace_image_index_t,
    pub add_image_index: retro_add_image_index_t,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_disk_control_ext_callback {
    pub set_eject_state: retro_set_eject_state_t,
    pub get_eject_state: retro_get_eject_state_t,
    pub get_image_index: retro_get_image_index_t,
    pub set_image_index: retro_set_image_index_t,
    pub get_num_images: retro_get_num_images_t,
    pub replace_image_index: retro_replace_image_index_t,
    pub add_image_index: retro_add_image_index_t,
    pub set_initial_image: retro_set_initial_image_t,
    pub get_image_path: retro_get_image_path_t,
    pub get_image_label: retro_get_image_label_t,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_netpacket_callback {
    pub start: retro_netpacket_start_t,
    pub receive: retro_netpacket_receive_t,
    pub stop: retro_netpacket_stop_t,
    pub poll: retro_netpacket_poll_t,
    pub connected: retro_netpacket_connected_t,
    pub disconnected: retro_netpacket_disconnected_t,
    pub protocol_version: *const c_char,
}

#[repr(C)]
#[derive(Debug)]
pub struct retro_microphone {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Debug)]
pub struct retro_vfs_file_handle {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Debug)]
pub struct retro_vfs_dir_handle {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct retro_microphone_params {
    pub rate: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_microphone_interface {
    pub interface_version: u32,
    pub open_mic: retro_open_mic_t,
    pub close_mic: retro_close_mic_t,
    pub get_params: retro_get_mic_params_t,
    pub set_mic_state: retro_set_mic_state_t,
    pub get_mic_state: retro_get_mic_state_t,
    pub read_mic: retro_read_mic_t,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_vfs_interface {
    pub get_path: retro_vfs_get_path_t,
    pub open: retro_vfs_open_t,
    pub close: retro_vfs_close_t,
    pub size: retro_vfs_size_t,
    pub tell: retro_vfs_tell_t,
    pub seek: retro_vfs_seek_t,
    pub read: retro_vfs_read_t,
    pub write: retro_vfs_write_t,
    pub flush: retro_vfs_flush_t,
    pub remove: retro_vfs_remove_t,
    pub rename: retro_vfs_rename_t,
    pub truncate: retro_vfs_truncate_t,
    pub stat: retro_vfs_stat_t,
    pub mkdir: retro_vfs_mkdir_t,
    pub opendir: retro_vfs_opendir_t,
    pub readdir: retro_vfs_readdir_t,
    pub dirent_get_name: retro_vfs_dirent_get_name_t,
    pub dirent_is_dir: retro_vfs_dirent_is_dir_t,
    pub closedir: retro_vfs_closedir_t,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_vfs_interface_info {
    pub required_interface_version: u32,
    pub iface: *mut retro_vfs_interface,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_midi_interface {
    pub input_enabled: retro_midi_input_enabled_t,
    pub output_enabled: retro_midi_output_enabled_t,
    pub read: retro_midi_read_t,
    pub write: retro_midi_write_t,
    pub flush: retro_midi_flush_t,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_audio_callback {
    pub callback: retro_audio_callback_t,
    pub set_state: retro_audio_set_state_callback_t,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_audio_buffer_status_callback {
    pub callback: retro_audio_buffer_status_callback_t,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_frame_time_callback {
    pub callback: retro_frame_time_callback_t,
    pub reference: retro_usec_t,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_get_proc_address_interface {
    pub get_proc_address: retro_get_proc_address_t,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct retro_fastforwarding_override {
    pub ratio: f32,
    pub fastforward: bool,
    pub notification: bool,
    pub inhibit_toggle: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct retro_throttle_state {
    pub mode: u32,
    pub rate: f32,
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
pub struct retro_core_option_display {
    pub key: *const c_char,
    pub visible: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_core_option_value {
    pub value: *const c_char,
    pub label: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct retro_core_option_definition {
    pub key: *const c_char,
    pub desc: *const c_char,
    pub info: *const c_char,
    pub values: [retro_core_option_value; RETRO_NUM_CORE_OPTION_VALUES_MAX],
    pub default_value: *const c_char,
}

impl Default for retro_core_option_definition {
    fn default() -> Self {
        Self {
            key: std::ptr::null(),
            desc: std::ptr::null(),
            info: std::ptr::null(),
            values: [retro_core_option_value::default(); RETRO_NUM_CORE_OPTION_VALUES_MAX],
            default_value: std::ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_core_options_intl {
    pub us: *mut retro_core_option_definition,
    pub local: *mut retro_core_option_definition,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_core_option_v2_category {
    pub key: *const c_char,
    pub desc: *const c_char,
    pub info: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct retro_core_option_v2_definition {
    pub key: *const c_char,
    pub desc: *const c_char,
    pub desc_categorized: *const c_char,
    pub info: *const c_char,
    pub info_categorized: *const c_char,
    pub category_key: *const c_char,
    pub values: [retro_core_option_value; RETRO_NUM_CORE_OPTION_VALUES_MAX],
    pub default_value: *const c_char,
}

impl Default for retro_core_option_v2_definition {
    fn default() -> Self {
        Self {
            key: std::ptr::null(),
            desc: std::ptr::null(),
            desc_categorized: std::ptr::null(),
            info: std::ptr::null(),
            info_categorized: std::ptr::null(),
            category_key: std::ptr::null(),
            values: [retro_core_option_value::default(); RETRO_NUM_CORE_OPTION_VALUES_MAX],
            default_value: std::ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_core_options_v2 {
    pub categories: *mut retro_core_option_v2_category,
    pub definitions: *mut retro_core_option_v2_definition,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_core_options_v2_intl {
    pub us: *mut retro_core_options_v2,
    pub local: *mut retro_core_options_v2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_core_options_update_display_callback {
    pub callback: retro_core_options_update_display_callback_t,
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
#[derive(Clone, Copy, Debug)]
pub struct retro_game_info_ext {
    pub full_path: *const c_char,
    pub archive_path: *const c_char,
    pub archive_file: *const c_char,
    pub dir: *const c_char,
    pub name: *const c_char,
    pub ext: *const c_char,
    pub meta: *const c_char,
    pub data: *const c_void,
    pub size: usize,
    pub file_in_archive: bool,
    pub persistent_data: bool,
}

impl Default for retro_game_info_ext {
    fn default() -> Self {
        Self {
            full_path: std::ptr::null(),
            archive_path: std::ptr::null(),
            archive_file: std::ptr::null(),
            dir: std::ptr::null(),
            name: std::ptr::null(),
            ext: std::ptr::null(),
            meta: std::ptr::null(),
            data: std::ptr::null(),
            size: 0,
            file_in_archive: false,
            persistent_data: false,
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

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_hw_render_interface {
    pub interface_type: i32,
    pub interface_version: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct retro_hw_render_context_negotiation_interface {
    pub interface_type: i32,
    pub interface_version: u32,
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

    assert!(std::mem::size_of::<retro_message_ext>() == 28);
    assert!(std::mem::align_of::<retro_message_ext>() == 4);
    assert!(std::mem::offset_of!(retro_message_ext, msg) == 0);
    assert!(std::mem::offset_of!(retro_message_ext, duration) == 4);
    assert!(std::mem::offset_of!(retro_message_ext, priority) == 8);
    assert!(std::mem::offset_of!(retro_message_ext, level) == 12);
    assert!(std::mem::offset_of!(retro_message_ext, target) == 16);
    assert!(std::mem::offset_of!(retro_message_ext, type_) == 20);
    assert!(std::mem::offset_of!(retro_message_ext, progress) == 24);

    assert!(std::mem::size_of::<retro_input_descriptor>() == 20);
    assert!(std::mem::align_of::<retro_input_descriptor>() == 4);
    assert!(std::mem::offset_of!(retro_input_descriptor, port) == 0);
    assert!(std::mem::offset_of!(retro_input_descriptor, device) == 4);
    assert!(std::mem::offset_of!(retro_input_descriptor, index) == 8);
    assert!(std::mem::offset_of!(retro_input_descriptor, id) == 12);
    assert!(std::mem::offset_of!(retro_input_descriptor, description) == 16);

    assert!(std::mem::size_of::<retro_device_power>() == 12);
    assert!(std::mem::align_of::<retro_device_power>() == 4);
    assert!(std::mem::offset_of!(retro_device_power, state) == 0);
    assert!(std::mem::offset_of!(retro_device_power, seconds) == 4);
    assert!(std::mem::offset_of!(retro_device_power, percent) == 8);

    let memory_descriptor_ptr_offset =
        (std::mem::size_of::<u64>() + std::mem::align_of::<*mut c_void>() - 1)
            & !(std::mem::align_of::<*mut c_void>() - 1);
    let memory_descriptor_offset_offset =
        memory_descriptor_ptr_offset + std::mem::size_of::<*mut c_void>();
    let memory_descriptor_start_offset =
        memory_descriptor_offset_offset + std::mem::size_of::<usize>();
    let memory_descriptor_select_offset =
        memory_descriptor_start_offset + std::mem::size_of::<usize>();
    let memory_descriptor_disconnect_offset =
        memory_descriptor_select_offset + std::mem::size_of::<usize>();
    let memory_descriptor_len_offset =
        memory_descriptor_disconnect_offset + std::mem::size_of::<usize>();
    let memory_descriptor_addrspace_offset =
        memory_descriptor_len_offset + std::mem::size_of::<usize>();
    let memory_descriptor_size = (memory_descriptor_addrspace_offset
        + std::mem::size_of::<*const c_char>()
        + std::mem::align_of::<retro_memory_descriptor>()
        - 1)
        & !(std::mem::align_of::<retro_memory_descriptor>() - 1);
    assert!(std::mem::size_of::<retro_memory_descriptor>() == memory_descriptor_size);
    assert!(
        std::mem::align_of::<retro_memory_descriptor>()
            == std::mem::align_of::<u64>()
                .max(std::mem::align_of::<*mut c_void>())
                .max(std::mem::align_of::<usize>())
                .max(std::mem::align_of::<*const c_char>())
    );
    assert!(std::mem::offset_of!(retro_memory_descriptor, flags) == 0);
    assert!(std::mem::offset_of!(retro_memory_descriptor, ptr) == memory_descriptor_ptr_offset);
    assert!(
        std::mem::offset_of!(retro_memory_descriptor, offset) == memory_descriptor_offset_offset
    );
    assert!(std::mem::offset_of!(retro_memory_descriptor, start) == memory_descriptor_start_offset);
    assert!(
        std::mem::offset_of!(retro_memory_descriptor, select) == memory_descriptor_select_offset
    );
    assert!(
        std::mem::offset_of!(retro_memory_descriptor, disconnect)
            == memory_descriptor_disconnect_offset
    );
    assert!(std::mem::offset_of!(retro_memory_descriptor, len) == memory_descriptor_len_offset);
    assert!(
        std::mem::offset_of!(retro_memory_descriptor, addrspace)
            == memory_descriptor_addrspace_offset
    );

    let memory_map_num_descriptors_offset =
        (std::mem::size_of::<*const retro_memory_descriptor>() + std::mem::align_of::<u32>() - 1)
            & !(std::mem::align_of::<u32>() - 1);
    let memory_map_size = (memory_map_num_descriptors_offset
        + std::mem::size_of::<u32>()
        + std::mem::align_of::<retro_memory_map>()
        - 1)
        & !(std::mem::align_of::<retro_memory_map>() - 1);
    assert!(std::mem::size_of::<retro_memory_map>() == memory_map_size);
    assert!(
        std::mem::align_of::<retro_memory_map>()
            == std::mem::align_of::<*const retro_memory_descriptor>()
                .max(std::mem::align_of::<u32>())
    );
    assert!(std::mem::offset_of!(retro_memory_map, descriptors) == 0);
    assert!(
        std::mem::offset_of!(retro_memory_map, num_descriptors)
            == memory_map_num_descriptors_offset
    );

    assert!(std::mem::size_of::<retro_controller_description>() == 8);
    assert!(std::mem::align_of::<retro_controller_description>() == 4);
    assert!(std::mem::offset_of!(retro_controller_description, desc) == 0);
    assert!(std::mem::offset_of!(retro_controller_description, id) == 4);

    assert!(std::mem::size_of::<retro_controller_info>() == 8);
    assert!(std::mem::align_of::<retro_controller_info>() == 4);
    assert!(std::mem::offset_of!(retro_controller_info, types) == 0);
    assert!(std::mem::offset_of!(retro_controller_info, num_types) == 4);

    assert!(std::mem::size_of::<retro_framebuffer>() == 28);
    assert!(std::mem::align_of::<retro_framebuffer>() == 4);
    assert!(std::mem::offset_of!(retro_framebuffer, data) == 0);
    assert!(std::mem::offset_of!(retro_framebuffer, width) == 4);
    assert!(std::mem::offset_of!(retro_framebuffer, height) == 8);
    assert!(std::mem::offset_of!(retro_framebuffer, pitch) == 12);
    assert!(std::mem::offset_of!(retro_framebuffer, format) == 16);
    assert!(std::mem::offset_of!(retro_framebuffer, access_flags) == 20);
    assert!(std::mem::offset_of!(retro_framebuffer, memory_flags) == 24);

    assert!(std::mem::size_of::<retro_led_interface>() == 4);
    assert!(std::mem::align_of::<retro_led_interface>() == 4);
    assert!(std::mem::offset_of!(retro_led_interface, set_led_state) == 0);

    assert!(std::mem::size_of::<retro_rumble_interface>() == 4);
    assert!(std::mem::align_of::<retro_rumble_interface>() == 4);
    assert!(std::mem::offset_of!(retro_rumble_interface, set_rumble_state) == 0);

    assert!(std::mem::size_of::<retro_sensor_interface>() == 8);
    assert!(std::mem::align_of::<retro_sensor_interface>() == 4);
    assert!(std::mem::offset_of!(retro_sensor_interface, set_sensor_state) == 0);
    assert!(std::mem::offset_of!(retro_sensor_interface, get_sensor_input) == 4);

    assert!(std::mem::size_of::<retro_camera_callback>() == 40);
    assert!(std::mem::align_of::<retro_camera_callback>() == 4);
    assert!(std::mem::offset_of!(retro_camera_callback, caps) == 0);
    assert!(std::mem::offset_of!(retro_camera_callback, width) == 8);
    assert!(std::mem::offset_of!(retro_camera_callback, height) == 12);
    assert!(std::mem::offset_of!(retro_camera_callback, start) == 16);
    assert!(std::mem::offset_of!(retro_camera_callback, stop) == 20);
    assert!(std::mem::offset_of!(retro_camera_callback, frame_raw_framebuffer) == 24);
    assert!(std::mem::offset_of!(retro_camera_callback, frame_opengl_texture) == 28);
    assert!(std::mem::offset_of!(retro_camera_callback, initialized) == 32);
    assert!(std::mem::offset_of!(retro_camera_callback, deinitialized) == 36);

    assert!(std::mem::size_of::<retro_location_callback>() == 24);
    assert!(std::mem::align_of::<retro_location_callback>() == 4);
    assert!(std::mem::offset_of!(retro_location_callback, start) == 0);
    assert!(std::mem::offset_of!(retro_location_callback, stop) == 4);
    assert!(std::mem::offset_of!(retro_location_callback, get_position) == 8);
    assert!(std::mem::offset_of!(retro_location_callback, set_interval) == 12);
    assert!(std::mem::offset_of!(retro_location_callback, initialized) == 16);
    assert!(std::mem::offset_of!(retro_location_callback, deinitialized) == 20);

    assert!(std::mem::size_of::<retro_subsystem_memory_info>() == 8);
    assert!(std::mem::align_of::<retro_subsystem_memory_info>() == 4);
    assert!(std::mem::offset_of!(retro_subsystem_memory_info, extension) == 0);
    assert!(std::mem::offset_of!(retro_subsystem_memory_info, memory_type) == 4);

    assert!(std::mem::size_of::<retro_subsystem_rom_info>() == 20);
    assert!(std::mem::align_of::<retro_subsystem_rom_info>() == 4);
    assert!(std::mem::offset_of!(retro_subsystem_rom_info, desc) == 0);
    assert!(std::mem::offset_of!(retro_subsystem_rom_info, valid_extensions) == 4);
    assert!(std::mem::offset_of!(retro_subsystem_rom_info, need_fullpath) == 8);
    assert!(std::mem::offset_of!(retro_subsystem_rom_info, block_extract) == 9);
    assert!(std::mem::offset_of!(retro_subsystem_rom_info, required) == 10);
    assert!(std::mem::offset_of!(retro_subsystem_rom_info, memory) == 12);
    assert!(std::mem::offset_of!(retro_subsystem_rom_info, num_memory) == 16);

    assert!(std::mem::size_of::<retro_subsystem_info>() == 20);
    assert!(std::mem::align_of::<retro_subsystem_info>() == 4);
    assert!(std::mem::offset_of!(retro_subsystem_info, desc) == 0);
    assert!(std::mem::offset_of!(retro_subsystem_info, ident) == 4);
    assert!(std::mem::offset_of!(retro_subsystem_info, roms) == 8);
    assert!(std::mem::offset_of!(retro_subsystem_info, num_roms) == 12);
    assert!(std::mem::offset_of!(retro_subsystem_info, id) == 16);

    assert!(std::mem::size_of::<retro_disk_control_callback>() == 28);
    assert!(std::mem::align_of::<retro_disk_control_callback>() == 4);
    assert!(std::mem::offset_of!(retro_disk_control_callback, set_eject_state) == 0);
    assert!(std::mem::offset_of!(retro_disk_control_callback, get_eject_state) == 4);
    assert!(std::mem::offset_of!(retro_disk_control_callback, get_image_index) == 8);
    assert!(std::mem::offset_of!(retro_disk_control_callback, set_image_index) == 12);
    assert!(std::mem::offset_of!(retro_disk_control_callback, get_num_images) == 16);
    assert!(std::mem::offset_of!(retro_disk_control_callback, replace_image_index) == 20);
    assert!(std::mem::offset_of!(retro_disk_control_callback, add_image_index) == 24);

    assert!(std::mem::size_of::<retro_disk_control_ext_callback>() == 40);
    assert!(std::mem::align_of::<retro_disk_control_ext_callback>() == 4);
    assert!(std::mem::offset_of!(retro_disk_control_ext_callback, set_eject_state) == 0);
    assert!(std::mem::offset_of!(retro_disk_control_ext_callback, get_eject_state) == 4);
    assert!(std::mem::offset_of!(retro_disk_control_ext_callback, get_image_index) == 8);
    assert!(std::mem::offset_of!(retro_disk_control_ext_callback, set_image_index) == 12);
    assert!(std::mem::offset_of!(retro_disk_control_ext_callback, get_num_images) == 16);
    assert!(std::mem::offset_of!(retro_disk_control_ext_callback, replace_image_index) == 20);
    assert!(std::mem::offset_of!(retro_disk_control_ext_callback, add_image_index) == 24);
    assert!(std::mem::offset_of!(retro_disk_control_ext_callback, set_initial_image) == 28);
    assert!(std::mem::offset_of!(retro_disk_control_ext_callback, get_image_path) == 32);
    assert!(std::mem::offset_of!(retro_disk_control_ext_callback, get_image_label) == 36);

    assert!(std::mem::size_of::<retro_netpacket_callback>() == 28);
    assert!(std::mem::align_of::<retro_netpacket_callback>() == 4);
    assert!(std::mem::offset_of!(retro_netpacket_callback, start) == 0);
    assert!(std::mem::offset_of!(retro_netpacket_callback, receive) == 4);
    assert!(std::mem::offset_of!(retro_netpacket_callback, stop) == 8);
    assert!(std::mem::offset_of!(retro_netpacket_callback, poll) == 12);
    assert!(std::mem::offset_of!(retro_netpacket_callback, connected) == 16);
    assert!(std::mem::offset_of!(retro_netpacket_callback, disconnected) == 20);
    assert!(std::mem::offset_of!(retro_netpacket_callback, protocol_version) == 24);

    assert!(std::mem::size_of::<retro_microphone_params>() == 4);
    assert!(std::mem::align_of::<retro_microphone_params>() == 4);
    assert!(std::mem::offset_of!(retro_microphone_params, rate) == 0);

    assert!(std::mem::size_of::<retro_microphone_interface>() == 28);
    assert!(std::mem::align_of::<retro_microphone_interface>() == 4);
    assert!(std::mem::offset_of!(retro_microphone_interface, interface_version) == 0);
    assert!(std::mem::offset_of!(retro_microphone_interface, open_mic) == 4);
    assert!(std::mem::offset_of!(retro_microphone_interface, close_mic) == 8);
    assert!(std::mem::offset_of!(retro_microphone_interface, get_params) == 12);
    assert!(std::mem::offset_of!(retro_microphone_interface, set_mic_state) == 16);
    assert!(std::mem::offset_of!(retro_microphone_interface, get_mic_state) == 20);
    assert!(std::mem::offset_of!(retro_microphone_interface, read_mic) == 24);

    assert!(std::mem::size_of::<retro_vfs_interface>() == 76);
    assert!(std::mem::align_of::<retro_vfs_interface>() == 4);
    assert!(std::mem::offset_of!(retro_vfs_interface, get_path) == 0);
    assert!(std::mem::offset_of!(retro_vfs_interface, open) == 4);
    assert!(std::mem::offset_of!(retro_vfs_interface, close) == 8);
    assert!(std::mem::offset_of!(retro_vfs_interface, size) == 12);
    assert!(std::mem::offset_of!(retro_vfs_interface, tell) == 16);
    assert!(std::mem::offset_of!(retro_vfs_interface, seek) == 20);
    assert!(std::mem::offset_of!(retro_vfs_interface, read) == 24);
    assert!(std::mem::offset_of!(retro_vfs_interface, write) == 28);
    assert!(std::mem::offset_of!(retro_vfs_interface, flush) == 32);
    assert!(std::mem::offset_of!(retro_vfs_interface, remove) == 36);
    assert!(std::mem::offset_of!(retro_vfs_interface, rename) == 40);
    assert!(std::mem::offset_of!(retro_vfs_interface, truncate) == 44);
    assert!(std::mem::offset_of!(retro_vfs_interface, stat) == 48);
    assert!(std::mem::offset_of!(retro_vfs_interface, mkdir) == 52);
    assert!(std::mem::offset_of!(retro_vfs_interface, opendir) == 56);
    assert!(std::mem::offset_of!(retro_vfs_interface, readdir) == 60);
    assert!(std::mem::offset_of!(retro_vfs_interface, dirent_get_name) == 64);
    assert!(std::mem::offset_of!(retro_vfs_interface, dirent_is_dir) == 68);
    assert!(std::mem::offset_of!(retro_vfs_interface, closedir) == 72);

    assert!(std::mem::size_of::<retro_vfs_interface_info>() == 8);
    assert!(std::mem::align_of::<retro_vfs_interface_info>() == 4);
    assert!(std::mem::offset_of!(retro_vfs_interface_info, required_interface_version) == 0);
    assert!(std::mem::offset_of!(retro_vfs_interface_info, iface) == 4);

    assert!(std::mem::size_of::<retro_midi_interface>() == 20);
    assert!(std::mem::align_of::<retro_midi_interface>() == 4);
    assert!(std::mem::offset_of!(retro_midi_interface, input_enabled) == 0);
    assert!(std::mem::offset_of!(retro_midi_interface, output_enabled) == 4);
    assert!(std::mem::offset_of!(retro_midi_interface, read) == 8);
    assert!(std::mem::offset_of!(retro_midi_interface, write) == 12);
    assert!(std::mem::offset_of!(retro_midi_interface, flush) == 16);

    let perf_counter_start_offset =
        (std::mem::size_of::<*const c_char>() + std::mem::align_of::<retro_perf_tick_t>() - 1)
            & !(std::mem::align_of::<retro_perf_tick_t>() - 1);
    let perf_counter_total_offset =
        perf_counter_start_offset + std::mem::size_of::<retro_perf_tick_t>();
    let perf_counter_call_count_offset =
        perf_counter_total_offset + std::mem::size_of::<retro_perf_tick_t>();
    let perf_counter_registered_offset =
        perf_counter_call_count_offset + std::mem::size_of::<retro_perf_tick_t>();
    let perf_counter_size = (perf_counter_registered_offset
        + std::mem::size_of::<bool>()
        + std::mem::align_of::<retro_perf_counter>()
        - 1)
        & !(std::mem::align_of::<retro_perf_counter>() - 1);
    assert!(std::mem::size_of::<retro_perf_counter>() == perf_counter_size);
    assert!(
        std::mem::align_of::<retro_perf_counter>()
            == std::mem::align_of::<*const c_char>()
                .max(std::mem::align_of::<retro_perf_tick_t>())
                .max(std::mem::align_of::<bool>())
    );
    assert!(std::mem::offset_of!(retro_perf_counter, ident) == 0);
    assert!(std::mem::offset_of!(retro_perf_counter, start) == perf_counter_start_offset);
    assert!(std::mem::offset_of!(retro_perf_counter, total) == perf_counter_total_offset);
    assert!(std::mem::offset_of!(retro_perf_counter, call_cnt) == perf_counter_call_count_offset);
    assert!(std::mem::offset_of!(retro_perf_counter, registered) == perf_counter_registered_offset);

    assert!(std::mem::size_of::<retro_perf_callback>() == 28);
    assert!(std::mem::align_of::<retro_perf_callback>() == 4);
    assert!(std::mem::offset_of!(retro_perf_callback, get_time_usec) == 0);
    assert!(std::mem::offset_of!(retro_perf_callback, get_cpu_features) == 4);
    assert!(std::mem::offset_of!(retro_perf_callback, get_perf_counter) == 8);
    assert!(std::mem::offset_of!(retro_perf_callback, perf_register) == 12);
    assert!(std::mem::offset_of!(retro_perf_callback, perf_start) == 16);
    assert!(std::mem::offset_of!(retro_perf_callback, perf_stop) == 20);
    assert!(std::mem::offset_of!(retro_perf_callback, perf_log) == 24);

    assert!(std::mem::size_of::<retro_keyboard_callback>() == 4);
    assert!(std::mem::align_of::<retro_keyboard_callback>() == 4);
    assert!(std::mem::offset_of!(retro_keyboard_callback, callback) == 0);

    assert!(std::mem::size_of::<retro_audio_callback>() == 8);
    assert!(std::mem::align_of::<retro_audio_callback>() == 4);
    assert!(std::mem::offset_of!(retro_audio_callback, callback) == 0);
    assert!(std::mem::offset_of!(retro_audio_callback, set_state) == 4);

    assert!(std::mem::size_of::<retro_audio_buffer_status_callback>() == 4);
    assert!(std::mem::align_of::<retro_audio_buffer_status_callback>() == 4);
    assert!(std::mem::offset_of!(retro_audio_buffer_status_callback, callback) == 0);

    let frame_time_reference_offset = (std::mem::size_of::<retro_frame_time_callback_t>()
        + std::mem::align_of::<retro_usec_t>()
        - 1)
        & !(std::mem::align_of::<retro_usec_t>() - 1);
    assert!(
        std::mem::size_of::<retro_frame_time_callback>()
            == frame_time_reference_offset + std::mem::size_of::<retro_usec_t>()
    );
    assert!(
        std::mem::align_of::<retro_frame_time_callback>()
            == std::mem::align_of::<retro_frame_time_callback_t>()
                .max(std::mem::align_of::<retro_usec_t>())
    );
    assert!(std::mem::offset_of!(retro_frame_time_callback, callback) == 0);
    assert!(
        std::mem::offset_of!(retro_frame_time_callback, reference) == frame_time_reference_offset
    );

    assert!(std::mem::size_of::<retro_get_proc_address_interface>() == 4);
    assert!(std::mem::align_of::<retro_get_proc_address_interface>() == 4);
    assert!(std::mem::offset_of!(retro_get_proc_address_interface, get_proc_address) == 0);

    assert!(std::mem::size_of::<retro_fastforwarding_override>() == 8);
    assert!(std::mem::align_of::<retro_fastforwarding_override>() == 4);
    assert!(std::mem::offset_of!(retro_fastforwarding_override, ratio) == 0);
    assert!(std::mem::offset_of!(retro_fastforwarding_override, fastforward) == 4);
    assert!(std::mem::offset_of!(retro_fastforwarding_override, notification) == 5);
    assert!(std::mem::offset_of!(retro_fastforwarding_override, inhibit_toggle) == 6);

    assert!(std::mem::size_of::<retro_throttle_state>() == 8);
    assert!(std::mem::align_of::<retro_throttle_state>() == 4);
    assert!(std::mem::offset_of!(retro_throttle_state, mode) == 0);
    assert!(std::mem::offset_of!(retro_throttle_state, rate) == 4);

    assert!(std::mem::size_of::<retro_variable>() == 8);
    assert!(std::mem::align_of::<retro_variable>() == 4);
    assert!(std::mem::offset_of!(retro_variable, key) == 0);
    assert!(std::mem::offset_of!(retro_variable, value) == 4);

    assert!(std::mem::size_of::<retro_core_option_display>() == 8);
    assert!(std::mem::align_of::<retro_core_option_display>() == 4);
    assert!(std::mem::offset_of!(retro_core_option_display, key) == 0);
    assert!(std::mem::offset_of!(retro_core_option_display, visible) == 4);

    assert!(std::mem::size_of::<retro_core_option_value>() == 8);
    assert!(std::mem::align_of::<retro_core_option_value>() == 4);
    assert!(std::mem::offset_of!(retro_core_option_value, value) == 0);
    assert!(std::mem::offset_of!(retro_core_option_value, label) == 4);

    assert!(std::mem::size_of::<retro_core_option_definition>() == 1040);
    assert!(std::mem::align_of::<retro_core_option_definition>() == 4);
    assert!(std::mem::offset_of!(retro_core_option_definition, key) == 0);
    assert!(std::mem::offset_of!(retro_core_option_definition, desc) == 4);
    assert!(std::mem::offset_of!(retro_core_option_definition, info) == 8);
    assert!(std::mem::offset_of!(retro_core_option_definition, values) == 12);
    assert!(std::mem::offset_of!(retro_core_option_definition, default_value) == 1036);

    assert!(std::mem::size_of::<retro_core_options_intl>() == 8);
    assert!(std::mem::align_of::<retro_core_options_intl>() == 4);
    assert!(std::mem::offset_of!(retro_core_options_intl, us) == 0);
    assert!(std::mem::offset_of!(retro_core_options_intl, local) == 4);

    assert!(std::mem::size_of::<retro_core_option_v2_category>() == 12);
    assert!(std::mem::align_of::<retro_core_option_v2_category>() == 4);
    assert!(std::mem::offset_of!(retro_core_option_v2_category, key) == 0);
    assert!(std::mem::offset_of!(retro_core_option_v2_category, desc) == 4);
    assert!(std::mem::offset_of!(retro_core_option_v2_category, info) == 8);

    assert!(std::mem::size_of::<retro_core_option_v2_definition>() == 1052);
    assert!(std::mem::align_of::<retro_core_option_v2_definition>() == 4);
    assert!(std::mem::offset_of!(retro_core_option_v2_definition, key) == 0);
    assert!(std::mem::offset_of!(retro_core_option_v2_definition, desc) == 4);
    assert!(std::mem::offset_of!(retro_core_option_v2_definition, desc_categorized) == 8);
    assert!(std::mem::offset_of!(retro_core_option_v2_definition, info) == 12);
    assert!(std::mem::offset_of!(retro_core_option_v2_definition, info_categorized) == 16);
    assert!(std::mem::offset_of!(retro_core_option_v2_definition, category_key) == 20);
    assert!(std::mem::offset_of!(retro_core_option_v2_definition, values) == 24);
    assert!(std::mem::offset_of!(retro_core_option_v2_definition, default_value) == 1048);

    assert!(std::mem::size_of::<retro_core_options_v2>() == 8);
    assert!(std::mem::align_of::<retro_core_options_v2>() == 4);
    assert!(std::mem::offset_of!(retro_core_options_v2, categories) == 0);
    assert!(std::mem::offset_of!(retro_core_options_v2, definitions) == 4);

    assert!(std::mem::size_of::<retro_core_options_v2_intl>() == 8);
    assert!(std::mem::align_of::<retro_core_options_v2_intl>() == 4);
    assert!(std::mem::offset_of!(retro_core_options_v2_intl, us) == 0);
    assert!(std::mem::offset_of!(retro_core_options_v2_intl, local) == 4);

    assert!(std::mem::size_of::<retro_core_options_update_display_callback>() == 4);
    assert!(std::mem::align_of::<retro_core_options_update_display_callback>() == 4);
    assert!(std::mem::offset_of!(retro_core_options_update_display_callback, callback) == 0);

    assert!(std::mem::size_of::<retro_system_content_info_override>() == 8);
    assert!(std::mem::align_of::<retro_system_content_info_override>() == 4);
    assert!(std::mem::offset_of!(retro_system_content_info_override, extensions) == 0);
    assert!(std::mem::offset_of!(retro_system_content_info_override, need_fullpath) == 4);
    assert!(std::mem::offset_of!(retro_system_content_info_override, persistent_data) == 5);

    assert!(std::mem::size_of::<retro_game_info>() == 16);
    assert!(std::mem::align_of::<retro_game_info>() == 4);

    assert!(std::mem::size_of::<retro_game_info_ext>() == 40);
    assert!(std::mem::align_of::<retro_game_info_ext>() == 4);
    assert!(std::mem::offset_of!(retro_game_info_ext, full_path) == 0);
    assert!(std::mem::offset_of!(retro_game_info_ext, archive_path) == 4);
    assert!(std::mem::offset_of!(retro_game_info_ext, archive_file) == 8);
    assert!(std::mem::offset_of!(retro_game_info_ext, dir) == 12);
    assert!(std::mem::offset_of!(retro_game_info_ext, name) == 16);
    assert!(std::mem::offset_of!(retro_game_info_ext, ext) == 20);
    assert!(std::mem::offset_of!(retro_game_info_ext, meta) == 24);
    assert!(std::mem::offset_of!(retro_game_info_ext, data) == 28);
    assert!(std::mem::offset_of!(retro_game_info_ext, size) == 32);
    assert!(std::mem::offset_of!(retro_game_info_ext, file_in_archive) == 36);
    assert!(std::mem::offset_of!(retro_game_info_ext, persistent_data) == 37);

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

    assert!(std::mem::size_of::<retro_hw_render_interface>() == 8);
    assert!(std::mem::align_of::<retro_hw_render_interface>() == 4);
    assert!(std::mem::offset_of!(retro_hw_render_interface, interface_type) == 0);
    assert!(std::mem::offset_of!(retro_hw_render_interface, interface_version) == 4);

    assert!(std::mem::size_of::<retro_hw_render_context_negotiation_interface>() == 8);
    assert!(std::mem::align_of::<retro_hw_render_context_negotiation_interface>() == 4);
    assert!(
        std::mem::offset_of!(
            retro_hw_render_context_negotiation_interface,
            interface_type
        ) == 0
    );
    assert!(
        std::mem::offset_of!(
            retro_hw_render_context_negotiation_interface,
            interface_version
        ) == 4
    );
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

    assert!(std::mem::size_of::<retro_message_ext>() == 32);
    assert!(std::mem::align_of::<retro_message_ext>() == 8);
    assert!(std::mem::offset_of!(retro_message_ext, msg) == 0);
    assert!(std::mem::offset_of!(retro_message_ext, duration) == 8);
    assert!(std::mem::offset_of!(retro_message_ext, priority) == 12);
    assert!(std::mem::offset_of!(retro_message_ext, level) == 16);
    assert!(std::mem::offset_of!(retro_message_ext, target) == 20);
    assert!(std::mem::offset_of!(retro_message_ext, type_) == 24);
    assert!(std::mem::offset_of!(retro_message_ext, progress) == 28);

    assert!(std::mem::size_of::<retro_input_descriptor>() == 24);
    assert!(std::mem::align_of::<retro_input_descriptor>() == 8);
    assert!(std::mem::offset_of!(retro_input_descriptor, port) == 0);
    assert!(std::mem::offset_of!(retro_input_descriptor, device) == 4);
    assert!(std::mem::offset_of!(retro_input_descriptor, index) == 8);
    assert!(std::mem::offset_of!(retro_input_descriptor, id) == 12);
    assert!(std::mem::offset_of!(retro_input_descriptor, description) == 16);

    assert!(std::mem::size_of::<retro_device_power>() == 12);
    assert!(std::mem::align_of::<retro_device_power>() == 4);
    assert!(std::mem::offset_of!(retro_device_power, state) == 0);
    assert!(std::mem::offset_of!(retro_device_power, seconds) == 4);
    assert!(std::mem::offset_of!(retro_device_power, percent) == 8);

    assert!(std::mem::size_of::<retro_memory_descriptor>() == 64);
    assert!(std::mem::align_of::<retro_memory_descriptor>() == 8);
    assert!(std::mem::offset_of!(retro_memory_descriptor, flags) == 0);
    assert!(std::mem::offset_of!(retro_memory_descriptor, ptr) == 8);
    assert!(std::mem::offset_of!(retro_memory_descriptor, offset) == 16);
    assert!(std::mem::offset_of!(retro_memory_descriptor, start) == 24);
    assert!(std::mem::offset_of!(retro_memory_descriptor, select) == 32);
    assert!(std::mem::offset_of!(retro_memory_descriptor, disconnect) == 40);
    assert!(std::mem::offset_of!(retro_memory_descriptor, len) == 48);
    assert!(std::mem::offset_of!(retro_memory_descriptor, addrspace) == 56);

    assert!(std::mem::size_of::<retro_memory_map>() == 16);
    assert!(std::mem::align_of::<retro_memory_map>() == 8);
    assert!(std::mem::offset_of!(retro_memory_map, descriptors) == 0);
    assert!(std::mem::offset_of!(retro_memory_map, num_descriptors) == 8);

    assert!(std::mem::size_of::<retro_controller_description>() == 16);
    assert!(std::mem::align_of::<retro_controller_description>() == 8);
    assert!(std::mem::offset_of!(retro_controller_description, desc) == 0);
    assert!(std::mem::offset_of!(retro_controller_description, id) == 8);

    assert!(std::mem::size_of::<retro_controller_info>() == 16);
    assert!(std::mem::align_of::<retro_controller_info>() == 8);
    assert!(std::mem::offset_of!(retro_controller_info, types) == 0);
    assert!(std::mem::offset_of!(retro_controller_info, num_types) == 8);

    assert!(std::mem::size_of::<retro_framebuffer>() == 40);
    assert!(std::mem::align_of::<retro_framebuffer>() == 8);
    assert!(std::mem::offset_of!(retro_framebuffer, data) == 0);
    assert!(std::mem::offset_of!(retro_framebuffer, width) == 8);
    assert!(std::mem::offset_of!(retro_framebuffer, height) == 12);
    assert!(std::mem::offset_of!(retro_framebuffer, pitch) == 16);
    assert!(std::mem::offset_of!(retro_framebuffer, format) == 24);
    assert!(std::mem::offset_of!(retro_framebuffer, access_flags) == 28);
    assert!(std::mem::offset_of!(retro_framebuffer, memory_flags) == 32);

    assert!(std::mem::size_of::<retro_led_interface>() == 8);
    assert!(std::mem::align_of::<retro_led_interface>() == 8);
    assert!(std::mem::offset_of!(retro_led_interface, set_led_state) == 0);

    assert!(std::mem::size_of::<retro_rumble_interface>() == 8);
    assert!(std::mem::align_of::<retro_rumble_interface>() == 8);
    assert!(std::mem::offset_of!(retro_rumble_interface, set_rumble_state) == 0);

    assert!(std::mem::size_of::<retro_sensor_interface>() == 16);
    assert!(std::mem::align_of::<retro_sensor_interface>() == 8);
    assert!(std::mem::offset_of!(retro_sensor_interface, set_sensor_state) == 0);
    assert!(std::mem::offset_of!(retro_sensor_interface, get_sensor_input) == 8);

    assert!(std::mem::size_of::<retro_camera_callback>() == 64);
    assert!(std::mem::align_of::<retro_camera_callback>() == 8);
    assert!(std::mem::offset_of!(retro_camera_callback, caps) == 0);
    assert!(std::mem::offset_of!(retro_camera_callback, width) == 8);
    assert!(std::mem::offset_of!(retro_camera_callback, height) == 12);
    assert!(std::mem::offset_of!(retro_camera_callback, start) == 16);
    assert!(std::mem::offset_of!(retro_camera_callback, stop) == 24);
    assert!(std::mem::offset_of!(retro_camera_callback, frame_raw_framebuffer) == 32);
    assert!(std::mem::offset_of!(retro_camera_callback, frame_opengl_texture) == 40);
    assert!(std::mem::offset_of!(retro_camera_callback, initialized) == 48);
    assert!(std::mem::offset_of!(retro_camera_callback, deinitialized) == 56);

    assert!(std::mem::size_of::<retro_location_callback>() == 48);
    assert!(std::mem::align_of::<retro_location_callback>() == 8);
    assert!(std::mem::offset_of!(retro_location_callback, start) == 0);
    assert!(std::mem::offset_of!(retro_location_callback, stop) == 8);
    assert!(std::mem::offset_of!(retro_location_callback, get_position) == 16);
    assert!(std::mem::offset_of!(retro_location_callback, set_interval) == 24);
    assert!(std::mem::offset_of!(retro_location_callback, initialized) == 32);
    assert!(std::mem::offset_of!(retro_location_callback, deinitialized) == 40);

    assert!(std::mem::size_of::<retro_subsystem_memory_info>() == 16);
    assert!(std::mem::align_of::<retro_subsystem_memory_info>() == 8);
    assert!(std::mem::offset_of!(retro_subsystem_memory_info, extension) == 0);
    assert!(std::mem::offset_of!(retro_subsystem_memory_info, memory_type) == 8);

    assert!(std::mem::size_of::<retro_subsystem_rom_info>() == 40);
    assert!(std::mem::align_of::<retro_subsystem_rom_info>() == 8);
    assert!(std::mem::offset_of!(retro_subsystem_rom_info, desc) == 0);
    assert!(std::mem::offset_of!(retro_subsystem_rom_info, valid_extensions) == 8);
    assert!(std::mem::offset_of!(retro_subsystem_rom_info, need_fullpath) == 16);
    assert!(std::mem::offset_of!(retro_subsystem_rom_info, block_extract) == 17);
    assert!(std::mem::offset_of!(retro_subsystem_rom_info, required) == 18);
    assert!(std::mem::offset_of!(retro_subsystem_rom_info, memory) == 24);
    assert!(std::mem::offset_of!(retro_subsystem_rom_info, num_memory) == 32);

    assert!(std::mem::size_of::<retro_subsystem_info>() == 32);
    assert!(std::mem::align_of::<retro_subsystem_info>() == 8);
    assert!(std::mem::offset_of!(retro_subsystem_info, desc) == 0);
    assert!(std::mem::offset_of!(retro_subsystem_info, ident) == 8);
    assert!(std::mem::offset_of!(retro_subsystem_info, roms) == 16);
    assert!(std::mem::offset_of!(retro_subsystem_info, num_roms) == 24);
    assert!(std::mem::offset_of!(retro_subsystem_info, id) == 28);

    assert!(std::mem::size_of::<retro_disk_control_callback>() == 56);
    assert!(std::mem::align_of::<retro_disk_control_callback>() == 8);
    assert!(std::mem::offset_of!(retro_disk_control_callback, set_eject_state) == 0);
    assert!(std::mem::offset_of!(retro_disk_control_callback, get_eject_state) == 8);
    assert!(std::mem::offset_of!(retro_disk_control_callback, get_image_index) == 16);
    assert!(std::mem::offset_of!(retro_disk_control_callback, set_image_index) == 24);
    assert!(std::mem::offset_of!(retro_disk_control_callback, get_num_images) == 32);
    assert!(std::mem::offset_of!(retro_disk_control_callback, replace_image_index) == 40);
    assert!(std::mem::offset_of!(retro_disk_control_callback, add_image_index) == 48);

    assert!(std::mem::size_of::<retro_disk_control_ext_callback>() == 80);
    assert!(std::mem::align_of::<retro_disk_control_ext_callback>() == 8);
    assert!(std::mem::offset_of!(retro_disk_control_ext_callback, set_eject_state) == 0);
    assert!(std::mem::offset_of!(retro_disk_control_ext_callback, get_eject_state) == 8);
    assert!(std::mem::offset_of!(retro_disk_control_ext_callback, get_image_index) == 16);
    assert!(std::mem::offset_of!(retro_disk_control_ext_callback, set_image_index) == 24);
    assert!(std::mem::offset_of!(retro_disk_control_ext_callback, get_num_images) == 32);
    assert!(std::mem::offset_of!(retro_disk_control_ext_callback, replace_image_index) == 40);
    assert!(std::mem::offset_of!(retro_disk_control_ext_callback, add_image_index) == 48);
    assert!(std::mem::offset_of!(retro_disk_control_ext_callback, set_initial_image) == 56);
    assert!(std::mem::offset_of!(retro_disk_control_ext_callback, get_image_path) == 64);
    assert!(std::mem::offset_of!(retro_disk_control_ext_callback, get_image_label) == 72);

    assert!(std::mem::size_of::<retro_netpacket_callback>() == 56);
    assert!(std::mem::align_of::<retro_netpacket_callback>() == 8);
    assert!(std::mem::offset_of!(retro_netpacket_callback, start) == 0);
    assert!(std::mem::offset_of!(retro_netpacket_callback, receive) == 8);
    assert!(std::mem::offset_of!(retro_netpacket_callback, stop) == 16);
    assert!(std::mem::offset_of!(retro_netpacket_callback, poll) == 24);
    assert!(std::mem::offset_of!(retro_netpacket_callback, connected) == 32);
    assert!(std::mem::offset_of!(retro_netpacket_callback, disconnected) == 40);
    assert!(std::mem::offset_of!(retro_netpacket_callback, protocol_version) == 48);

    assert!(std::mem::size_of::<retro_microphone_params>() == 4);
    assert!(std::mem::align_of::<retro_microphone_params>() == 4);
    assert!(std::mem::offset_of!(retro_microphone_params, rate) == 0);

    assert!(std::mem::size_of::<retro_microphone_interface>() == 56);
    assert!(std::mem::align_of::<retro_microphone_interface>() == 8);
    assert!(std::mem::offset_of!(retro_microphone_interface, interface_version) == 0);
    assert!(std::mem::offset_of!(retro_microphone_interface, open_mic) == 8);
    assert!(std::mem::offset_of!(retro_microphone_interface, close_mic) == 16);
    assert!(std::mem::offset_of!(retro_microphone_interface, get_params) == 24);
    assert!(std::mem::offset_of!(retro_microphone_interface, set_mic_state) == 32);
    assert!(std::mem::offset_of!(retro_microphone_interface, get_mic_state) == 40);
    assert!(std::mem::offset_of!(retro_microphone_interface, read_mic) == 48);

    assert!(std::mem::size_of::<retro_vfs_interface>() == 152);
    assert!(std::mem::align_of::<retro_vfs_interface>() == 8);
    assert!(std::mem::offset_of!(retro_vfs_interface, get_path) == 0);
    assert!(std::mem::offset_of!(retro_vfs_interface, open) == 8);
    assert!(std::mem::offset_of!(retro_vfs_interface, close) == 16);
    assert!(std::mem::offset_of!(retro_vfs_interface, size) == 24);
    assert!(std::mem::offset_of!(retro_vfs_interface, tell) == 32);
    assert!(std::mem::offset_of!(retro_vfs_interface, seek) == 40);
    assert!(std::mem::offset_of!(retro_vfs_interface, read) == 48);
    assert!(std::mem::offset_of!(retro_vfs_interface, write) == 56);
    assert!(std::mem::offset_of!(retro_vfs_interface, flush) == 64);
    assert!(std::mem::offset_of!(retro_vfs_interface, remove) == 72);
    assert!(std::mem::offset_of!(retro_vfs_interface, rename) == 80);
    assert!(std::mem::offset_of!(retro_vfs_interface, truncate) == 88);
    assert!(std::mem::offset_of!(retro_vfs_interface, stat) == 96);
    assert!(std::mem::offset_of!(retro_vfs_interface, mkdir) == 104);
    assert!(std::mem::offset_of!(retro_vfs_interface, opendir) == 112);
    assert!(std::mem::offset_of!(retro_vfs_interface, readdir) == 120);
    assert!(std::mem::offset_of!(retro_vfs_interface, dirent_get_name) == 128);
    assert!(std::mem::offset_of!(retro_vfs_interface, dirent_is_dir) == 136);
    assert!(std::mem::offset_of!(retro_vfs_interface, closedir) == 144);

    assert!(std::mem::size_of::<retro_vfs_interface_info>() == 16);
    assert!(std::mem::align_of::<retro_vfs_interface_info>() == 8);
    assert!(std::mem::offset_of!(retro_vfs_interface_info, required_interface_version) == 0);
    assert!(std::mem::offset_of!(retro_vfs_interface_info, iface) == 8);

    assert!(std::mem::size_of::<retro_midi_interface>() == 40);
    assert!(std::mem::align_of::<retro_midi_interface>() == 8);
    assert!(std::mem::offset_of!(retro_midi_interface, input_enabled) == 0);
    assert!(std::mem::offset_of!(retro_midi_interface, output_enabled) == 8);
    assert!(std::mem::offset_of!(retro_midi_interface, read) == 16);
    assert!(std::mem::offset_of!(retro_midi_interface, write) == 24);
    assert!(std::mem::offset_of!(retro_midi_interface, flush) == 32);

    assert!(std::mem::size_of::<retro_perf_counter>() == 40);
    assert!(std::mem::align_of::<retro_perf_counter>() == 8);
    assert!(std::mem::offset_of!(retro_perf_counter, ident) == 0);
    assert!(std::mem::offset_of!(retro_perf_counter, start) == 8);
    assert!(std::mem::offset_of!(retro_perf_counter, total) == 16);
    assert!(std::mem::offset_of!(retro_perf_counter, call_cnt) == 24);
    assert!(std::mem::offset_of!(retro_perf_counter, registered) == 32);

    assert!(std::mem::size_of::<retro_perf_callback>() == 56);
    assert!(std::mem::align_of::<retro_perf_callback>() == 8);
    assert!(std::mem::offset_of!(retro_perf_callback, get_time_usec) == 0);
    assert!(std::mem::offset_of!(retro_perf_callback, get_cpu_features) == 8);
    assert!(std::mem::offset_of!(retro_perf_callback, get_perf_counter) == 16);
    assert!(std::mem::offset_of!(retro_perf_callback, perf_register) == 24);
    assert!(std::mem::offset_of!(retro_perf_callback, perf_start) == 32);
    assert!(std::mem::offset_of!(retro_perf_callback, perf_stop) == 40);
    assert!(std::mem::offset_of!(retro_perf_callback, perf_log) == 48);

    assert!(std::mem::size_of::<retro_keyboard_callback>() == 8);
    assert!(std::mem::align_of::<retro_keyboard_callback>() == 8);
    assert!(std::mem::offset_of!(retro_keyboard_callback, callback) == 0);

    assert!(std::mem::size_of::<retro_audio_callback>() == 16);
    assert!(std::mem::align_of::<retro_audio_callback>() == 8);
    assert!(std::mem::offset_of!(retro_audio_callback, callback) == 0);
    assert!(std::mem::offset_of!(retro_audio_callback, set_state) == 8);

    assert!(std::mem::size_of::<retro_audio_buffer_status_callback>() == 8);
    assert!(std::mem::align_of::<retro_audio_buffer_status_callback>() == 8);
    assert!(std::mem::offset_of!(retro_audio_buffer_status_callback, callback) == 0);

    assert!(std::mem::size_of::<retro_frame_time_callback>() == 16);
    assert!(std::mem::align_of::<retro_frame_time_callback>() == 8);
    assert!(std::mem::offset_of!(retro_frame_time_callback, callback) == 0);
    assert!(std::mem::offset_of!(retro_frame_time_callback, reference) == 8);

    assert!(std::mem::size_of::<retro_get_proc_address_interface>() == 8);
    assert!(std::mem::align_of::<retro_get_proc_address_interface>() == 8);
    assert!(std::mem::offset_of!(retro_get_proc_address_interface, get_proc_address) == 0);

    assert!(std::mem::size_of::<retro_fastforwarding_override>() == 8);
    assert!(std::mem::align_of::<retro_fastforwarding_override>() == 4);
    assert!(std::mem::offset_of!(retro_fastforwarding_override, ratio) == 0);
    assert!(std::mem::offset_of!(retro_fastforwarding_override, fastforward) == 4);
    assert!(std::mem::offset_of!(retro_fastforwarding_override, notification) == 5);
    assert!(std::mem::offset_of!(retro_fastforwarding_override, inhibit_toggle) == 6);

    assert!(std::mem::size_of::<retro_throttle_state>() == 8);
    assert!(std::mem::align_of::<retro_throttle_state>() == 4);
    assert!(std::mem::offset_of!(retro_throttle_state, mode) == 0);
    assert!(std::mem::offset_of!(retro_throttle_state, rate) == 4);

    assert!(std::mem::size_of::<retro_variable>() == 16);
    assert!(std::mem::align_of::<retro_variable>() == 8);
    assert!(std::mem::offset_of!(retro_variable, key) == 0);
    assert!(std::mem::offset_of!(retro_variable, value) == 8);

    assert!(std::mem::size_of::<retro_core_option_display>() == 16);
    assert!(std::mem::align_of::<retro_core_option_display>() == 8);
    assert!(std::mem::offset_of!(retro_core_option_display, key) == 0);
    assert!(std::mem::offset_of!(retro_core_option_display, visible) == 8);

    assert!(std::mem::size_of::<retro_core_option_value>() == 16);
    assert!(std::mem::align_of::<retro_core_option_value>() == 8);
    assert!(std::mem::offset_of!(retro_core_option_value, value) == 0);
    assert!(std::mem::offset_of!(retro_core_option_value, label) == 8);

    assert!(std::mem::size_of::<retro_core_option_definition>() == 2080);
    assert!(std::mem::align_of::<retro_core_option_definition>() == 8);
    assert!(std::mem::offset_of!(retro_core_option_definition, key) == 0);
    assert!(std::mem::offset_of!(retro_core_option_definition, desc) == 8);
    assert!(std::mem::offset_of!(retro_core_option_definition, info) == 16);
    assert!(std::mem::offset_of!(retro_core_option_definition, values) == 24);
    assert!(std::mem::offset_of!(retro_core_option_definition, default_value) == 2072);

    assert!(std::mem::size_of::<retro_core_options_intl>() == 16);
    assert!(std::mem::align_of::<retro_core_options_intl>() == 8);
    assert!(std::mem::offset_of!(retro_core_options_intl, us) == 0);
    assert!(std::mem::offset_of!(retro_core_options_intl, local) == 8);

    assert!(std::mem::size_of::<retro_core_option_v2_category>() == 24);
    assert!(std::mem::align_of::<retro_core_option_v2_category>() == 8);
    assert!(std::mem::offset_of!(retro_core_option_v2_category, key) == 0);
    assert!(std::mem::offset_of!(retro_core_option_v2_category, desc) == 8);
    assert!(std::mem::offset_of!(retro_core_option_v2_category, info) == 16);

    assert!(std::mem::size_of::<retro_core_option_v2_definition>() == 2104);
    assert!(std::mem::align_of::<retro_core_option_v2_definition>() == 8);
    assert!(std::mem::offset_of!(retro_core_option_v2_definition, key) == 0);
    assert!(std::mem::offset_of!(retro_core_option_v2_definition, desc) == 8);
    assert!(std::mem::offset_of!(retro_core_option_v2_definition, desc_categorized) == 16);
    assert!(std::mem::offset_of!(retro_core_option_v2_definition, info) == 24);
    assert!(std::mem::offset_of!(retro_core_option_v2_definition, info_categorized) == 32);
    assert!(std::mem::offset_of!(retro_core_option_v2_definition, category_key) == 40);
    assert!(std::mem::offset_of!(retro_core_option_v2_definition, values) == 48);
    assert!(std::mem::offset_of!(retro_core_option_v2_definition, default_value) == 2096);

    assert!(std::mem::size_of::<retro_core_options_v2>() == 16);
    assert!(std::mem::align_of::<retro_core_options_v2>() == 8);
    assert!(std::mem::offset_of!(retro_core_options_v2, categories) == 0);
    assert!(std::mem::offset_of!(retro_core_options_v2, definitions) == 8);

    assert!(std::mem::size_of::<retro_core_options_v2_intl>() == 16);
    assert!(std::mem::align_of::<retro_core_options_v2_intl>() == 8);
    assert!(std::mem::offset_of!(retro_core_options_v2_intl, us) == 0);
    assert!(std::mem::offset_of!(retro_core_options_v2_intl, local) == 8);

    assert!(std::mem::size_of::<retro_core_options_update_display_callback>() == 8);
    assert!(std::mem::align_of::<retro_core_options_update_display_callback>() == 8);
    assert!(std::mem::offset_of!(retro_core_options_update_display_callback, callback) == 0);

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

    assert!(std::mem::size_of::<retro_game_info_ext>() == 80);
    assert!(std::mem::align_of::<retro_game_info_ext>() == 8);
    assert!(std::mem::offset_of!(retro_game_info_ext, full_path) == 0);
    assert!(std::mem::offset_of!(retro_game_info_ext, archive_path) == 8);
    assert!(std::mem::offset_of!(retro_game_info_ext, archive_file) == 16);
    assert!(std::mem::offset_of!(retro_game_info_ext, dir) == 24);
    assert!(std::mem::offset_of!(retro_game_info_ext, name) == 32);
    assert!(std::mem::offset_of!(retro_game_info_ext, ext) == 40);
    assert!(std::mem::offset_of!(retro_game_info_ext, meta) == 48);
    assert!(std::mem::offset_of!(retro_game_info_ext, data) == 56);
    assert!(std::mem::offset_of!(retro_game_info_ext, size) == 64);
    assert!(std::mem::offset_of!(retro_game_info_ext, file_in_archive) == 72);
    assert!(std::mem::offset_of!(retro_game_info_ext, persistent_data) == 73);

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

    assert!(std::mem::size_of::<retro_hw_render_interface>() == 8);
    assert!(std::mem::align_of::<retro_hw_render_interface>() == 4);
    assert!(std::mem::offset_of!(retro_hw_render_interface, interface_type) == 0);
    assert!(std::mem::offset_of!(retro_hw_render_interface, interface_version) == 4);

    assert!(std::mem::size_of::<retro_hw_render_context_negotiation_interface>() == 8);
    assert!(std::mem::align_of::<retro_hw_render_context_negotiation_interface>() == 4);
    assert!(
        std::mem::offset_of!(
            retro_hw_render_context_negotiation_interface,
            interface_type
        ) == 0
    );
    assert!(
        std::mem::offset_of!(
            retro_hw_render_context_negotiation_interface,
            interface_version
        ) == 4
    );
};
