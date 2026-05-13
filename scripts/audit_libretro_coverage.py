#!/usr/bin/env python3
"""Audit ergonomic coverage of the vendored libretro.h header.

This is intentionally conservative. It does not claim semantic completion from
name matching alone; it provides a repeatable inventory and a rough current
status map that the tracked coverage document can be checked against.
"""

from __future__ import annotations

import argparse
import collections
import pathlib
import re
import sys


ROOT = pathlib.Path(__file__).resolve().parents[1]
HEADER = ROOT / "crates/libretro-core/include/libretro.h"
SRC = ROOT / "crates/libretro-core/src"
COVERAGE = ROOT / "crates/libretro-core/libretro_coverage.md"

KINDS = ("function", "struct", "enum", "typedef", "constant")
REQUIRED_CATEGORIES = (
    "core-abi",
    "environment",
    "input",
    "memory",
    "options",
    "subsystems-disks",
    "callbacks",
    "frontend-services",
    "hardware",
    "vfs",
    "sensors-camera-location",
    "audio-timing",
    "netplay",
    "diagnostics-testing",
)

RAW_ONLY_OVERRIDES = set()

TYPED_WRAPPER_OVERRIDES = {
    "RETRO_API_VERSION",
    "RETRO_AV_ENABLE_AUDIO",
    "RETRO_AV_ENABLE_FAST_SAVESTATES",
    "RETRO_AV_ENABLE_HARD_DISABLE_AUDIO",
    "RETRO_AV_ENABLE_VIDEO",
    "RETRO_DEVICE_ANALOG",
    "RETRO_DEVICE_KEYBOARD",
    "RETRO_DEVICE_LIGHTGUN",
    "RETRO_DEVICE_MASK",
    "RETRO_DEVICE_MOUSE",
    "RETRO_DEVICE_NONE",
    "RETRO_DEVICE_POINTER",
    "RETRO_DEVICE_SUBCLASS",
    "RETRO_DEVICE_TYPE_SHIFT",
    "RETRO_DEVICE_ID_ANALOG_X",
    "RETRO_DEVICE_ID_ANALOG_Y",
    "RETRO_DEVICE_JOYPAD",
    "RETRO_DEVICE_ID_JOYPAD_A",
    "RETRO_DEVICE_ID_JOYPAD_B",
    "RETRO_DEVICE_ID_JOYPAD_DOWN",
    "RETRO_DEVICE_ID_JOYPAD_L",
    "RETRO_DEVICE_ID_JOYPAD_L2",
    "RETRO_DEVICE_ID_JOYPAD_L3",
    "RETRO_DEVICE_ID_JOYPAD_LEFT",
    "RETRO_DEVICE_ID_JOYPAD_R",
    "RETRO_DEVICE_ID_JOYPAD_R2",
    "RETRO_DEVICE_ID_JOYPAD_R3",
    "RETRO_DEVICE_ID_JOYPAD_RIGHT",
    "RETRO_DEVICE_ID_JOYPAD_SELECT",
    "RETRO_DEVICE_ID_JOYPAD_START",
    "RETRO_DEVICE_ID_JOYPAD_UP",
    "RETRO_DEVICE_ID_JOYPAD_X",
    "RETRO_DEVICE_ID_JOYPAD_Y",
    "RETRO_DEVICE_ID_JOYPAD_MASK",
    "RETRO_DEVICE_ID_LIGHTGUN_AUX_A",
    "RETRO_DEVICE_ID_LIGHTGUN_AUX_B",
    "RETRO_DEVICE_ID_LIGHTGUN_AUX_C",
    "RETRO_DEVICE_ID_LIGHTGUN_CURSOR",
    "RETRO_DEVICE_ID_LIGHTGUN_DPAD_DOWN",
    "RETRO_DEVICE_ID_LIGHTGUN_DPAD_LEFT",
    "RETRO_DEVICE_ID_LIGHTGUN_DPAD_RIGHT",
    "RETRO_DEVICE_ID_LIGHTGUN_DPAD_UP",
    "RETRO_DEVICE_ID_LIGHTGUN_IS_OFFSCREEN",
    "RETRO_DEVICE_ID_LIGHTGUN_PAUSE",
    "RETRO_DEVICE_ID_LIGHTGUN_RELOAD",
    "RETRO_DEVICE_ID_LIGHTGUN_SCREEN_X",
    "RETRO_DEVICE_ID_LIGHTGUN_SCREEN_Y",
    "RETRO_DEVICE_ID_LIGHTGUN_SELECT",
    "RETRO_DEVICE_ID_LIGHTGUN_START",
    "RETRO_DEVICE_ID_LIGHTGUN_TRIGGER",
    "RETRO_DEVICE_ID_LIGHTGUN_TURBO",
    "RETRO_DEVICE_ID_LIGHTGUN_X",
    "RETRO_DEVICE_ID_LIGHTGUN_Y",
    "RETRO_DEVICE_ID_MOUSE_BUTTON_4",
    "RETRO_DEVICE_ID_MOUSE_BUTTON_5",
    "RETRO_DEVICE_ID_MOUSE_HORIZ_WHEELDOWN",
    "RETRO_DEVICE_ID_MOUSE_HORIZ_WHEELUP",
    "RETRO_DEVICE_ID_MOUSE_LEFT",
    "RETRO_DEVICE_ID_MOUSE_MIDDLE",
    "RETRO_DEVICE_ID_MOUSE_RIGHT",
    "RETRO_DEVICE_ID_MOUSE_WHEELDOWN",
    "RETRO_DEVICE_ID_MOUSE_WHEELUP",
    "RETRO_DEVICE_ID_MOUSE_X",
    "RETRO_DEVICE_ID_MOUSE_Y",
    "RETRO_DEVICE_ID_POINTER_COUNT",
    "RETRO_DEVICE_ID_POINTER_IS_OFFSCREEN",
    "RETRO_DEVICE_ID_POINTER_PRESSED",
    "RETRO_DEVICE_ID_POINTER_X",
    "RETRO_DEVICE_ID_POINTER_Y",
    "RETRO_DEVICE_INDEX_ANALOG_BUTTON",
    "RETRO_DEVICE_INDEX_ANALOG_LEFT",
    "RETRO_DEVICE_INDEX_ANALOG_RIGHT",
    "RETRO_ENVIRONMENT_GET_LOG_INTERFACE",
    "RETRO_ENVIRONMENT_EXPERIMENTAL",
    "RETRO_ENVIRONMENT_GET_AUDIO_VIDEO_ENABLE",
    "RETRO_ENVIRONMENT_GET_DEVICE_POWER",
    "RETRO_ENVIRONMENT_GET_CORE_ASSETS_DIRECTORY",
    "RETRO_ENVIRONMENT_GET_CONTENT_DIRECTORY",
    "RETRO_ENVIRONMENT_GET_FILE_BROWSER_START_DIRECTORY",
    "RETRO_ENVIRONMENT_GET_JIT_CAPABLE",
    "RETRO_ENVIRONMENT_GET_LANGUAGE",
    "RETRO_ENVIRONMENT_GET_LIBRETRO_PATH",
    "RETRO_ENVIRONMENT_GET_FASTFORWARDING",
    "RETRO_ENVIRONMENT_GET_GAME_INFO_EXT",
    "RETRO_ENVIRONMENT_GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT",
    "RETRO_ENVIRONMENT_GET_HW_RENDER_INTERFACE",
    "RETRO_ENVIRONMENT_GET_INPUT_BITMASKS",
    "RETRO_ENVIRONMENT_GET_INPUT_DEVICE_CAPABILITIES",
    "RETRO_ENVIRONMENT_GET_INPUT_MAX_USERS",
    "RETRO_ENVIRONMENT_GET_LED_INTERFACE",
    "RETRO_ENVIRONMENT_GET_CAMERA_INTERFACE",
    "RETRO_ENVIRONMENT_GET_LOCATION_INTERFACE",
    "RETRO_ENVIRONMENT_GET_MESSAGE_INTERFACE_VERSION",
    "RETRO_ENVIRONMENT_GET_MIDI_INTERFACE",
    "RETRO_ENVIRONMENT_GET_MICROPHONE_INTERFACE",
    "RETRO_ENVIRONMENT_GET_NETPLAY_CLIENT_INDEX",
    "RETRO_ENVIRONMENT_GET_CAN_DUPE",
    "RETRO_ENVIRONMENT_GET_CURRENT_SOFTWARE_FRAMEBUFFER",
    "RETRO_ENVIRONMENT_GET_CORE_OPTIONS_VERSION",
    "RETRO_ENVIRONMENT_GET_DISK_CONTROL_INTERFACE_VERSION",
    "RETRO_ENVIRONMENT_GET_OVERSCAN",
    "RETRO_ENVIRONMENT_GET_PERF_INTERFACE",
    "RETRO_ENVIRONMENT_GET_PLAYLIST_DIRECTORY",
    "RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER",
    "RETRO_ENVIRONMENT_GET_RUMBLE_INTERFACE",
    "RETRO_ENVIRONMENT_GET_SAVESTATE_CONTEXT",
    "RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY",
    "RETRO_ENVIRONMENT_GET_SENSOR_INTERFACE",
    "RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY",
    "RETRO_ENVIRONMENT_GET_TARGET_REFRESH_RATE",
    "RETRO_ENVIRONMENT_GET_TARGET_SAMPLE_RATE",
    "RETRO_ENVIRONMENT_GET_THROTTLE_STATE",
    "RETRO_ENVIRONMENT_GET_USERNAME",
    "RETRO_ENVIRONMENT_GET_VARIABLE",
    "RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE",
    "RETRO_ENVIRONMENT_GET_VFS_INTERFACE",
    "RETRO_ENVIRONMENT_SET_AUDIO_BUFFER_STATUS_CALLBACK",
    "RETRO_ENVIRONMENT_SET_AUDIO_CALLBACK",
    "RETRO_ENVIRONMENT_SET_CONTENT_INFO_OVERRIDE",
    "RETRO_ENVIRONMENT_SET_CONTROLLER_INFO",
    "RETRO_ENVIRONMENT_SET_CORE_OPTIONS",
    "RETRO_ENVIRONMENT_SET_CORE_OPTIONS_DISPLAY",
    "RETRO_ENVIRONMENT_SET_CORE_OPTIONS_INTL",
    "RETRO_ENVIRONMENT_SET_CORE_OPTIONS_UPDATE_DISPLAY_CALLBACK",
    "RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2",
    "RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL",
    "RETRO_ENVIRONMENT_SET_DISK_CONTROL_EXT_INTERFACE",
    "RETRO_ENVIRONMENT_SET_DISK_CONTROL_INTERFACE",
    "RETRO_ENVIRONMENT_SET_FASTFORWARDING_OVERRIDE",
    "RETRO_ENVIRONMENT_SET_FRAME_TIME_CALLBACK",
    "RETRO_ENVIRONMENT_SET_GEOMETRY",
    "RETRO_ENVIRONMENT_SET_HW_RENDER",
    "RETRO_ENVIRONMENT_SET_HW_SHARED_CONTEXT",
    "RETRO_ENVIRONMENT_SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE",
    "RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS",
    "RETRO_ENVIRONMENT_SET_KEYBOARD_CALLBACK",
    "RETRO_ENVIRONMENT_SET_MESSAGE",
    "RETRO_ENVIRONMENT_SET_MESSAGE_EXT",
    "RETRO_ENVIRONMENT_SET_MEMORY_MAPS",
    "RETRO_ENVIRONMENT_SET_MINIMUM_AUDIO_LATENCY",
    "RETRO_ENVIRONMENT_SET_NETPACKET_INTERFACE",
    "RETRO_ENVIRONMENT_SET_PERFORMANCE_LEVEL",
    "RETRO_ENVIRONMENT_SET_PIXEL_FORMAT",
    "RETRO_ENVIRONMENT_SET_PROC_ADDRESS_CALLBACK",
    "RETRO_ENVIRONMENT_SET_ROTATION",
    "RETRO_ENVIRONMENT_SET_SYSTEM_AV_INFO",
    "RETRO_ENVIRONMENT_SET_SUPPORT_ACHIEVEMENTS",
    "RETRO_ENVIRONMENT_SET_SERIALIZATION_QUIRKS",
    "RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME",
    "RETRO_ENVIRONMENT_SET_SUBSYSTEM_INFO",
    "RETRO_ENVIRONMENT_SET_VARIABLES",
    "RETRO_ENVIRONMENT_SET_VARIABLE",
    "RETRO_ENVIRONMENT_SHUTDOWN",
    "RETRO_HW_FRAME_BUFFER_VALID",
    "RETRO_ENVIRONMENT_PRIVATE",
    "RETRO_MICROPHONE_INTERFACE_VERSION",
    "RETRO_NUM_CORE_OPTION_VALUES_MAX",
    "RETRO_MEMDESC_ALIGN_2",
    "RETRO_MEMDESC_ALIGN_4",
    "RETRO_MEMDESC_ALIGN_8",
    "RETRO_MEMDESC_BIGENDIAN",
    "RETRO_MEMDESC_CONST",
    "RETRO_MEMDESC_MINSIZE_2",
    "RETRO_MEMDESC_MINSIZE_4",
    "RETRO_MEMDESC_MINSIZE_8",
    "RETRO_MEMDESC_SAVE_RAM",
    "RETRO_MEMDESC_SYSTEM_RAM",
    "RETRO_MEMDESC_VIDEO_RAM",
    "RETRO_MEMORY_ACCESS_READ",
    "RETRO_MEMORY_ACCESS_WRITE",
    "RETRO_MEMORY_MASK",
    "RETRO_MEMORY_ROM",
    "RETRO_MEMORY_RTC",
    "RETRO_MEMORY_SAVE_RAM",
    "RETRO_MEMORY_SYSTEM_RAM",
    "RETRO_MEMORY_TYPE_CACHED",
    "RETRO_MEMORY_VIDEO_RAM",
    "RETRO_NETPACKET_BROADCAST",
    "RETRO_NETPACKET_FLUSH_HINT",
    "RETRO_NETPACKET_RELIABLE",
    "RETRO_NETPACKET_UNRELIABLE",
    "RETRO_NETPACKET_UNSEQUENCED",
    "RETRO_POWERSTATE_NO_ESTIMATE",
    "RETRO_REGION_NTSC",
    "RETRO_REGION_PAL",
    "RETRO_SENSOR_ACCELEROMETER_X",
    "RETRO_SENSOR_ACCELEROMETER_Y",
    "RETRO_SENSOR_ACCELEROMETER_Z",
    "RETRO_SENSOR_GYROSCOPE_X",
    "RETRO_SENSOR_GYROSCOPE_Y",
    "RETRO_SENSOR_GYROSCOPE_Z",
    "RETRO_SENSOR_ILLUMINANCE",
    "RETRO_SERIALIZATION_QUIRK_CORE_VARIABLE_SIZE",
    "RETRO_SERIALIZATION_QUIRK_ENDIAN_DEPENDENT",
    "RETRO_SERIALIZATION_QUIRK_FRONT_VARIABLE_SIZE",
    "RETRO_SERIALIZATION_QUIRK_INCOMPLETE",
    "RETRO_SERIALIZATION_QUIRK_MUST_INITIALIZE",
    "RETRO_SERIALIZATION_QUIRK_PLATFORM_DEPENDENT",
    "RETRO_SERIALIZATION_QUIRK_SINGLE_SESSION",
    "RETRO_SIMD_AES",
    "RETRO_SIMD_ASIMD",
    "RETRO_SIMD_AVX",
    "RETRO_SIMD_AVX2",
    "RETRO_SIMD_CMOV",
    "RETRO_SIMD_MMX",
    "RETRO_SIMD_MMXEXT",
    "RETRO_SIMD_MOVBE",
    "RETRO_SIMD_NEON",
    "RETRO_SIMD_POPCNT",
    "RETRO_SIMD_PS",
    "RETRO_SIMD_SSE",
    "RETRO_SIMD_SSE2",
    "RETRO_SIMD_SSE3",
    "RETRO_SIMD_SSE4",
    "RETRO_SIMD_SSE42",
    "RETRO_SIMD_SSSE3",
    "RETRO_SIMD_VFPV3",
    "RETRO_SIMD_VFPV4",
    "RETRO_SIMD_VFPU",
    "RETRO_SIMD_VMX",
    "RETRO_SIMD_VMX128",
    "RETRO_THROTTLE_FAST_FORWARD",
    "RETRO_THROTTLE_FRAME_STEPPING",
    "RETRO_THROTTLE_NONE",
    "RETRO_THROTTLE_REWINDING",
    "RETRO_THROTTLE_SLOW_MOTION",
    "RETRO_THROTTLE_UNBLOCKED",
    "RETRO_THROTTLE_VSYNC",
    "RETRO_VFS_FILE_ACCESS_HINT_FREQUENT_ACCESS",
    "RETRO_VFS_FILE_ACCESS_HINT_NONE",
    "RETRO_VFS_FILE_ACCESS_READ",
    "RETRO_VFS_FILE_ACCESS_READ_WRITE",
    "RETRO_VFS_FILE_ACCESS_UPDATE_EXISTING",
    "RETRO_VFS_FILE_ACCESS_WRITE",
    "RETRO_VFS_SEEK_POSITION_CURRENT",
    "RETRO_VFS_SEEK_POSITION_END",
    "RETRO_VFS_SEEK_POSITION_START",
    "RETRO_VFS_STAT_IS_CHARACTER_SPECIAL",
    "RETRO_VFS_STAT_IS_DIRECTORY",
    "RETRO_VFS_STAT_IS_VALID",
    "retro_audio_buffer_status_callback",
    "retro_audio_buffer_status_callback_t",
    "retro_audio_callback",
    "retro_audio_callback_t",
    "retro_audio_sample_batch_t",
    "retro_audio_sample_t",
    "retro_audio_set_state_callback_t",
    "retro_av_enable_flags",
    "retro_camera_callback",
    "retro_camera_buffer",
    "retro_camera_frame_opengl_texture_t",
    "retro_camera_frame_raw_framebuffer_t",
    "retro_camera_lifetime_status_t",
    "retro_camera_start_t",
    "retro_camera_stop_t",
    "retro_controller_description",
    "retro_controller_info",
    "retro_core_option_definition",
    "retro_core_option_display",
    "retro_core_option_v2_category",
    "retro_core_option_v2_definition",
    "retro_core_option_value",
    "retro_core_options_intl",
    "retro_core_options_update_display_callback",
    "retro_core_options_update_display_callback_t",
    "retro_core_options_v2",
    "retro_core_options_v2_intl",
    "retro_device_power",
    "retro_disk_control_callback",
    "retro_disk_control_ext_callback",
    "retro_environment_t",
    "retro_fastforwarding_override",
    "retro_framebuffer",
    "retro_frame_time_callback",
    "retro_frame_time_callback_t",
    "retro_game_info",
    "retro_game_geometry",
    "retro_game_info_ext",
    "retro_get_cpu_features_t",
    "retro_get_eject_state_t",
    "retro_get_image_index_t",
    "retro_get_image_label_t",
    "retro_get_image_path_t",
    "retro_get_num_images_t",
    "retro_get_proc_address_interface",
    "retro_get_proc_address_t",
    "retro_hw_context_reset_t",
    "retro_hw_context_type",
    "retro_hw_get_current_framebuffer_t",
    "retro_hw_get_proc_address_t",
    "retro_hw_render_callback",
    "retro_hw_render_context_negotiation_interface",
    "retro_hw_render_context_negotiation_interface_type",
    "retro_hw_render_interface",
    "retro_hw_render_interface_type",
    "retro_input_poll_t",
    "retro_input_descriptor",
    "retro_input_state_t",
    "retro_key",
    "retro_keyboard_callback",
    "retro_keyboard_event_t",
    "retro_led_interface",
    "retro_location_callback",
    "retro_location_get_position_t",
    "retro_location_lifetime_status_t",
    "retro_location_set_interval_t",
    "retro_location_start_t",
    "retro_location_stop_t",
    "retro_log_callback",
    "retro_log_level",
    "retro_language",
    "retro_log_printf_t",
    "retro_memory_descriptor",
    "retro_memory_map",
    "retro_message",
    "retro_message_ext",
    "retro_message_target",
    "retro_message_type",
    "retro_midi_flush_t",
    "retro_midi_input_enabled_t",
    "retro_midi_interface",
    "retro_midi_output_enabled_t",
    "retro_midi_read_t",
    "retro_midi_write_t",
    "retro_microphone_interface",
    "retro_microphone_params",
    "retro_microphone_t",
    "retro_close_mic_t",
    "retro_get_mic_params_t",
    "retro_get_mic_state_t",
    "retro_netpacket_callback",
    "retro_netpacket_connected_t",
    "retro_netpacket_disconnected_t",
    "retro_netpacket_poll_receive_t",
    "retro_netpacket_poll_t",
    "retro_netpacket_receive_t",
    "retro_netpacket_send_t",
    "retro_netpacket_start_t",
    "retro_netpacket_stop_t",
    "retro_mod",
    "retro_open_mic_t",
    "retro_perf_callback",
    "retro_perf_counter",
    "retro_perf_get_counter_t",
    "retro_perf_get_time_usec_t",
    "retro_perf_log_t",
    "retro_perf_register_t",
    "retro_perf_start_t",
    "retro_perf_stop_t",
    "retro_perf_tick_t",
    "retro_power_state",
    "retro_pixel_format",
    "retro_proc_address_t",
    "retro_read_mic_t",
    "retro_rumble_effect",
    "retro_rumble_interface",
    "retro_set_led_state_t",
    "retro_set_rumble_state_t",
    "retro_set_sensor_state_t",
    "retro_sensor_action",
    "retro_sensor_get_input_t",
    "retro_sensor_interface",
    "retro_savestate_context",
    "retro_add_image_index_t",
    "retro_replace_image_index_t",
    "retro_set_eject_state_t",
    "retro_set_image_index_t",
    "retro_set_initial_image_t",
    "retro_set_mic_state_t",
    "retro_subsystem_info",
    "retro_subsystem_memory_info",
    "retro_subsystem_rom_info",
    "retro_system_content_info_override",
    "retro_system_info",
    "retro_system_av_info",
    "retro_system_timing",
    "retro_throttle_state",
    "retro_time_t",
    "retro_usec_t",
    "retro_variable",
    "retro_video_refresh_t",
    "retro_vfs_close_t",
    "retro_vfs_closedir_t",
    "retro_vfs_dir_handle",
    "retro_vfs_dirent_get_name_t",
    "retro_vfs_dirent_is_dir_t",
    "retro_vfs_file_handle",
    "retro_vfs_flush_t",
    "retro_vfs_get_path_t",
    "retro_vfs_interface",
    "retro_vfs_interface_info",
    "retro_vfs_mkdir_t",
    "retro_vfs_open_t",
    "retro_vfs_opendir_t",
    "retro_vfs_read_t",
    "retro_vfs_readdir_t",
    "retro_vfs_remove_t",
    "retro_vfs_rename_t",
    "retro_vfs_seek_t",
    "retro_vfs_size_t",
    "retro_vfs_stat_t",
    "retro_vfs_tell_t",
    "retro_vfs_truncate_t",
    "retro_vfs_write_t",
}


def strip_comments(text: str) -> str:
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    return re.sub(r"//.*", "", text)


def extract_symbols(header: str) -> list[tuple[str, str]]:
    cleaned = strip_comments(header)
    symbols: set[tuple[str, str]] = set()

    for name in re.findall(r"RETRO_API\s+[^{;]*?\b(retro_[A-Za-z0-9_]+)\s*\(", cleaned):
        symbols.add(("function", name))

    for name in re.findall(r"^\s*struct\s+(retro_[A-Za-z0-9_]+)\b", cleaned, flags=re.M):
        symbols.add(("struct", name))

    for name in re.findall(r"^\s*enum\s+(retro_[A-Za-z0-9_]+)\b", cleaned, flags=re.M):
        symbols.add(("enum", name))

    for statement in re.findall(r"\btypedef\b.*?;", cleaned, flags=re.S):
        fn = re.search(r"\*\s*(retro_[A-Za-z0-9_]+)\s*\)", statement)
        if fn:
            symbols.add(("typedef", fn.group(1)))
            continue
        simple = re.search(r"\b(retro_[A-Za-z0-9_]+)\s*;", statement)
        if simple:
            symbols.add(("typedef", simple.group(1)))

    for name in re.findall(r"^\s*#\s*define\s+(RETRO_[A-Za-z0-9_]+)\b", cleaned, flags=re.M):
        if name in {"RETRO_API", "RETRO_CALLCONV", "RETRO_IMPORT_SYMBOLS"}:
            continue
        symbols.add(("constant", name))

    return sorted(symbols, key=lambda item: (KINDS.index(item[0]), item[1]))


def rust_source_text() -> str:
    return "\n".join(path.read_text() for path in SRC.glob("*.rs"))


def category_for(kind: str, name: str) -> str:
    if kind == "function":
        return "core-abi"
    if name.startswith("RETRO_ENVIRONMENT"):
        if "AUDIO" in name or "FRAME_TIME" in name or "TARGET_SAMPLE_RATE" in name:
            return "audio-timing"
        if "CORE_OPTIONS" in name or name in {
            "RETRO_ENVIRONMENT_SET_VARIABLE",
            "RETRO_ENVIRONMENT_SET_VARIABLES",
            "RETRO_ENVIRONMENT_GET_VARIABLE",
            "RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE",
        }:
            return "options"
        if "DISK" in name or "SUBSYSTEM" in name:
            return "subsystems-disks"
        if "HW_RENDER" in name or "SOFTWARE_FRAMEBUFFER" in name or name == "RETRO_ENVIRONMENT_SET_PIXEL_FORMAT":
            return "hardware"
        if "VFS" in name:
            return "vfs"
        if any(token in name for token in ("SENSOR", "CAMERA", "LOCATION", "MICROPHONE")):
            return "sensors-camera-location"
        if "NET" in name:
            return "netplay"
        if any(token in name for token in ("LOG", "MESSAGE", "PERF")):
            return "diagnostics-testing"
        if any(token in name for token in ("INPUT", "KEYBOARD", "RUMBLE", "LED")):
            return "input"
        if any(token in name for token in ("MEMORY", "SAVE", "SERIALIZATION", "GAME_INFO", "CONTENT")):
            return "memory"
        return "frontend-services"
    if name.startswith(("RETRO_DEVICE", "RETRO_KEY", "RETRO_MOD")) or any(
        token in name for token in ("input", "keyboard", "rumble", "led")
    ):
        return "input"
    if name.startswith(("RETRO_MEMORY", "RETRO_MEMDESC", "RETRO_SERIALIZATION")) or any(
        token in name for token in ("memory", "savestate", "game_info", "framebuffer")
    ):
        return "memory"
    if "option" in name or name.startswith("RETRO_NUM_CORE_OPTION"):
        return "options"
    if any(token in name for token in ("disk", "subsystem")):
        return "subsystems-disks"
    if any(token in name for token in ("callback", "environment_t", "video_refresh", "audio_sample", "input_poll", "input_state")):
        return "callbacks"
    if name.startswith("RETRO_HW") or "hw_render" in name or "pixel_format" in name:
        return "hardware"
    if name.startswith("RETRO_VFS") or "vfs" in name:
        return "vfs"
    if any(token in name for token in ("sensor", "camera", "location", "microphone")):
        return "sensors-camera-location"
    if any(token in name for token in ("audio", "frame_time", "usec")):
        return "audio-timing"
    if "netpacket" in name or name.startswith("RETRO_NETPACKET"):
        return "netplay"
    if any(token in name for token in ("log", "perf", "SIMD")) or name.startswith("RETRO_SIMD"):
        return "diagnostics-testing"
    return "frontend-services"


def module_for(category: str) -> str:
    return {
        "core-abi": "lib.rs + raw.rs",
        "environment": "environment.rs",
        "input": "input.rs",
        "memory": "memory.rs",
        "options": "options.rs",
        "subsystems-disks": "subsystem.rs + disk.rs",
        "callbacks": "callbacks.rs",
        "frontend-services": "environment.rs",
        "hardware": "hardware.rs",
        "vfs": "vfs.rs",
        "sensors-camera-location": "sensors.rs",
        "audio-timing": "callbacks.rs",
        "netplay": "netplay.rs",
        "diagnostics-testing": "perf.rs",
    }[category]


def status_for(kind: str, name: str, source: str) -> str:
    if kind == "function":
        return "typed-wrapper"
    if name in RAW_ONLY_OVERRIDES:
        return "raw-only"
    if name in TYPED_WRAPPER_OVERRIDES:
        return "typed-wrapper"
    if re.search(rf"\b{name}\b", source):
        return "raw-only"
    return "missing"


def build_inventory() -> list[dict[str, str]]:
    source = rust_source_text()
    return [
        {
            "kind": kind,
            "name": name,
            "category": category_for(kind, name),
            "module": module_for(category_for(kind, name)),
            "status": status_for(kind, name, source),
        }
        for kind, name in extract_symbols(HEADER.read_text())
    ]


def count_by(inventory: list[dict[str, str]], key: str) -> collections.Counter[str]:
    return collections.Counter(item[key] for item in inventory)


def summary_counts(inventory: list[dict[str, str]]) -> dict[str, int]:
    counts: dict[str, int] = {"total": len(inventory)}
    counts.update({f"kind.{key}": value for key, value in count_by(inventory, "kind").items()})
    counts.update({f"status.{key}": value for key, value in count_by(inventory, "status").items()})
    return dict(sorted(counts.items()))


def format_counts_block(counts: dict[str, int]) -> str:
    body = "\n".join(f"{key}={value}" for key, value in counts.items())
    return f"<!-- libretro-coverage-counts\n{body}\n-->"


def parse_counts_block(text: str) -> dict[str, int]:
    match = re.search(r"<!-- libretro-coverage-counts\n(.*?)\n-->", text, flags=re.S)
    if not match:
        raise ValueError("missing libretro-coverage-counts block")
    counts: dict[str, int] = {}
    for line in match.group(1).splitlines():
        key, value = line.split("=", 1)
        counts[key] = int(value)
    return counts


def print_summary(inventory: list[dict[str, str]]) -> None:
    print(format_counts_block(summary_counts(inventory)))
    print()
    grouped = collections.defaultdict(list)
    for item in inventory:
        grouped[item["category"]].append(item)
    for category in REQUIRED_CATEGORIES:
        items = grouped[category]
        statuses = count_by(items, "status")
        print(
            f"{category}: total={len(items)} typed-wrapper={statuses['typed-wrapper']} "
            f"raw-only={statuses['raw-only']} missing={statuses['missing']}"
        )
        missing = [item["name"] for item in items if item["status"] == "missing"][:12]
        if missing:
            print("  missing:", ", ".join(missing))


def check_coverage_doc(inventory: list[dict[str, str]]) -> int:
    if not COVERAGE.exists():
        print(f"{COVERAGE} does not exist", file=sys.stderr)
        return 1

    text = COVERAGE.read_text()
    expected = summary_counts(inventory)
    try:
        actual = parse_counts_block(text)
    except ValueError as error:
        print(f"{COVERAGE}: {error}", file=sys.stderr)
        return 1

    if actual != expected:
        print(f"{COVERAGE}: libretro-coverage-counts is stale", file=sys.stderr)
        print("expected:")
        print(format_counts_block(expected))
        return 1

    missing_categories = [category for category in REQUIRED_CATEGORIES if f"`{category}`" not in text]
    if missing_categories:
        print(
            f"{COVERAGE}: missing category entries: {', '.join(missing_categories)}",
            file=sys.stderr,
        )
        return 1

    print(f"{COVERAGE}: inventory counts and required categories are current")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="check libretro_coverage.md")
    args = parser.parse_args()

    inventory = build_inventory()
    if args.check:
        return check_coverage_doc(inventory)
    print_summary(inventory)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
