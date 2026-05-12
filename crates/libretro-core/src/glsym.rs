//! Ergonomic OpenGL symbol access for libretro cores.
//!
//! Methodology:
//! - Keep the public API aligned with OpenGL naming where practical.
//! - Prefer Rust-native inputs and outputs (`&str`, slices, return values) over
//!   raw pointers, mutable out-params, or `CString`/`CStr` requirements.
//! - Keep ABI- and pointer-oriented helpers private to this module so core code
//!   stays focused on rendering intent instead of FFI plumbing.
//! - Add higher-level helpers only when they remove repetitive multi-call setup
//!   that every libretro OpenGL core would otherwise need to duplicate.
//!
#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::{CStr, CString, c_char, c_void};
use std::mem;
use std::sync::{Mutex, OnceLock};

use crate::{HwContextType, Runtime};

const GL_FALSE: u8 = 0;
const GL_COLOR_BUFFER_BIT: u32 = 0x0000_4000;
const GL_DEPTH_BUFFER_BIT: u32 = 0x0000_0100;
const GL_SCISSOR_TEST: u32 = 0x0C11;
const GL_BLEND: u32 = 0x0BE2;
const GL_COMPILE_STATUS: u32 = 0x8B81;
const GL_LINK_STATUS: u32 = 0x8B82;
const GL_INFO_LOG_LENGTH: u32 = 0x8B84;
const GL_VENDOR: u32 = 0x1F00;
const GL_RENDERER: u32 = 0x1F01;
const GL_VERSION: u32 = 0x1F02;
const GL_EXTENSIONS: u32 = 0x1F03;
const GL_MAX_TEXTURE_SIZE: u32 = 0x0D33;
const GL_MAX_TEXTURE_IMAGE_UNITS: u32 = 0x8872;
const GL_MAX_VARYING_VECTORS: u32 = 0x8DFC;
const GL_FLOAT: u32 = 0x1406;
const GL_TRIANGLES: u32 = 0x0004;
const GL_ARRAY_BUFFER: u32 = 0x8892;
const GL_ELEMENT_ARRAY_BUFFER: u32 = 0x8893;
const GL_STATIC_DRAW: u32 = 0x88E4;
const GL_STREAM_DRAW: u32 = 0x88E0;
const GL_DYNAMIC_DRAW: u32 = 0x88E8;
const GL_FRAMEBUFFER: u32 = 0x8D40;
const GL_DEPTH_ATTACHMENT: u32 = 0x8D00;
const GL_TEXTURE0: u32 = 0x84C0;
const GL_TEXTURE_2D: u32 = 0x0DE1;
const GL_TEXTURE_2D_ARRAY: u32 = 0x8C1A;
const GL_TEXTURE_MIN_FILTER: u32 = 0x2801;
const GL_TEXTURE_MAG_FILTER: u32 = 0x2800;
const GL_TEXTURE_WRAP_S: u32 = 0x2802;
const GL_TEXTURE_WRAP_T: u32 = 0x2803;
const GL_TEXTURE_WRAP_R: u32 = 0x8072;
const GL_CLAMP_TO_EDGE: u32 = 0x812F;
const GL_REPEAT: u32 = 0x2901;
const GL_NEAREST: u32 = 0x2600;
const GL_LINEAR: u32 = 0x2601;
const GL_NEAREST_MIPMAP_NEAREST: u32 = 0x2700;
const GL_LINEAR_MIPMAP_NEAREST: u32 = 0x2701;
const GL_RGB: u32 = 0x1907;
const GL_RED: u32 = 0x1903;
const GL_LUMINANCE: u32 = 0x1909;
const GL_RGBA: u32 = 0x1908;
const GL_UNSIGNED_BYTE: u32 = 0x1401;
const GL_UNSIGNED_SHORT: u32 = 0x1403;
const GL_UNSIGNED_SHORT_4_4_4_4: u32 = 0x8033;
const GL_UNSIGNED_SHORT_5_6_5: u32 = 0x8363;
const GL_UNPACK_ALIGNMENT: u32 = 0x0CF5;
const GL_ONE: u32 = 1;
const GL_SRC_ALPHA: u32 = 0x0302;
const GL_ONE_MINUS_SRC_ALPHA: u32 = 0x0303;
const GL_FUNC_ADD: u32 = 0x8006;
const GL_FUNC_REVERSE_SUBTRACT: u32 = 0x800B;
const GL_R8: u32 = 0x8229;
const GL_RGBA8: u32 = 0x8058;
const GL_VERTEX_SHADER: u32 = 0x8B31;
const GL_FRAGMENT_SHADER: u32 = 0x8B30;
const GL_HIGH_FLOAT: u32 = 0x8DF2;
const GL_NO_ERROR: u32 = 0;
const GL_INVALID_ENUM: u32 = 0x0500;
const GL_INVALID_VALUE: u32 = 0x0501;
const GL_INVALID_OPERATION: u32 = 0x0502;
const GL_OUT_OF_MEMORY: u32 = 0x0505;
const GL_INVALID_FRAMEBUFFER_OPERATION: u32 = 0x0506;
const GL_FRAMEBUFFER_COMPLETE: u32 = 0x8CD5;
const GL_FRAMEBUFFER_INCOMPLETE_ATTACHMENT: u32 = 0x8CD6;
const GL_FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT: u32 = 0x8CD7;
const GL_FRAMEBUFFER_INCOMPLETE_DIMENSIONS: u32 = 0x8CD9;
const GL_FRAMEBUFFER_UNSUPPORTED: u32 = 0x8CDD;

type GlClearColor = unsafe extern "C" fn(f32, f32, f32, f32);
type GlClear = unsafe extern "C" fn(u32);
type GlEnable = unsafe extern "C" fn(u32);
type GlDisable = unsafe extern "C" fn(u32);
type GlViewport = unsafe extern "C" fn(i32, i32, i32, i32);
type GlScissor = unsafe extern "C" fn(i32, i32, i32, i32);
type GlCreateShader = unsafe extern "C" fn(u32) -> u32;
type GlShaderSource = unsafe extern "C" fn(u32, i32, *const *const c_char, *const i32);
type GlCompileShader = unsafe extern "C" fn(u32);
type GlGetShaderIv = unsafe extern "C" fn(u32, u32, *mut i32);
type GlGetShaderInfoLog = unsafe extern "C" fn(u32, i32, *mut i32, *mut c_char);
type GlDeleteShader = unsafe extern "C" fn(u32);
type GlCreateProgram = unsafe extern "C" fn() -> u32;
type GlAttachShader = unsafe extern "C" fn(u32, u32);
type GlLinkProgram = unsafe extern "C" fn(u32);
type GlGetProgramIv = unsafe extern "C" fn(u32, u32, *mut i32);
type GlGetProgramInfoLog = unsafe extern "C" fn(u32, i32, *mut i32, *mut c_char);
type GlDeleteProgram = unsafe extern "C" fn(u32);
type GlUseProgram = unsafe extern "C" fn(u32);
type GlGenBuffers = unsafe extern "C" fn(i32, *mut u32);
type GlBindBuffer = unsafe extern "C" fn(u32, u32);
type GlBufferData = unsafe extern "C" fn(u32, isize, *const c_void, u32);
type GlDeleteBuffers = unsafe extern "C" fn(i32, *const u32);
type GlGenTextures = unsafe extern "C" fn(i32, *mut u32);
type GlBindTexture = unsafe extern "C" fn(u32, u32);
type GlActiveTexture = unsafe extern "C" fn(u32);
type GlTexParameteri = unsafe extern "C" fn(u32, u32, i32);
type GlPixelStorei = unsafe extern "C" fn(u32, i32);
type GlTexImage2D = unsafe extern "C" fn(u32, i32, i32, i32, i32, i32, u32, u32, *const c_void);
type GlTexImage3D =
    unsafe extern "C" fn(u32, i32, i32, i32, i32, i32, i32, u32, u32, *const c_void);
type GlTexSubImage3D =
    unsafe extern "C" fn(u32, i32, i32, i32, i32, i32, i32, i32, u32, u32, *const c_void);
type GlGenerateMipmap = unsafe extern "C" fn(u32);
type GlDeleteTextures = unsafe extern "C" fn(i32, *const u32);
type GlGenVertexArrays = unsafe extern "C" fn(i32, *mut u32);
type GlBindVertexArray = unsafe extern "C" fn(u32);
type GlDeleteVertexArrays = unsafe extern "C" fn(i32, *const u32);
type GlEnableVertexAttribArray = unsafe extern "C" fn(u32);
type GlDisableVertexAttribArray = unsafe extern "C" fn(u32);
type GlVertexAttribPointer = unsafe extern "C" fn(u32, i32, u32, u8, i32, *const c_void);
type GlVertexAttribDivisor = unsafe extern "C" fn(u32, u32);
type GlGetUniformLocation = unsafe extern "C" fn(u32, *const c_char) -> i32;
type GlGetAttribLocation = unsafe extern "C" fn(u32, *const c_char) -> i32;
type GlUniform1i = unsafe extern "C" fn(i32, i32);
type GlUniform1f = unsafe extern "C" fn(i32, f32);
type GlUniform2f = unsafe extern "C" fn(i32, f32, f32);
type GlUniform4fv = unsafe extern "C" fn(i32, i32, *const f32);
type GlUniformMatrix4fv = unsafe extern "C" fn(i32, i32, u8, *const f32);
type GlDrawArrays = unsafe extern "C" fn(u32, i32, i32);
type GlDrawElements = unsafe extern "C" fn(u32, i32, u32, *const c_void);
type GlDrawElementsInstanced = unsafe extern "C" fn(u32, i32, u32, *const c_void, i32);
type GlBlendFunc = unsafe extern "C" fn(u32, u32);
type GlBlendEquationFn = unsafe extern "C" fn(u32);
type GlBindFramebuffer = unsafe extern "C" fn(u32, u32);
type GlGetString = unsafe extern "C" fn(u32) -> *const u8;
type GlGetIntegerv = unsafe extern "C" fn(u32, *mut i32);
type GlGetShaderPrecisionFormat = unsafe extern "C" fn(u32, u32, *mut i32, *mut i32);
type GlGetError = unsafe extern "C" fn() -> u32;
type GlCheckFramebufferStatus = unsafe extern "C" fn(u32) -> u32;
type GlInvalidateFramebuffer = unsafe extern "C" fn(u32, i32, *const u32);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GlVersionInfo {
    pub is_gles: bool,
    pub major: Option<u32>,
    pub minor: Option<u32>,
}

impl GlVersionInfo {
    pub fn version_at_least(self, major: u32, minor: u32) -> bool {
        match (self.major, self.minor) {
            (Some(actual_major), Some(actual_minor)) => {
                actual_major > major || (actual_major == major && actual_minor >= minor)
            }
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GlBufferTarget {
    ArrayBuffer,
    ElementArrayBuffer,
}

impl GlBufferTarget {
    fn as_raw(self) -> u32 {
        match self {
            Self::ArrayBuffer => GL_ARRAY_BUFFER,
            Self::ElementArrayBuffer => GL_ELEMENT_ARRAY_BUFFER,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GlBufferUsage {
    StaticDraw,
    StreamDraw,
    DynamicDraw,
}

impl GlBufferUsage {
    fn as_raw(self) -> u32 {
        match self {
            Self::StaticDraw => GL_STATIC_DRAW,
            Self::StreamDraw => GL_STREAM_DRAW,
            Self::DynamicDraw => GL_DYNAMIC_DRAW,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GlTextureTarget {
    Texture2D,
    Texture2DArray,
}

impl GlTextureTarget {
    pub fn as_raw(self) -> u32 {
        match self {
            Self::Texture2D => GL_TEXTURE_2D,
            Self::Texture2DArray => GL_TEXTURE_2D_ARRAY,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GlTextureParameter {
    MinFilter,
    MagFilter,
    WrapS,
    WrapT,
    WrapR,
}

impl GlTextureParameter {
    pub fn as_raw(self) -> u32 {
        match self {
            Self::MinFilter => GL_TEXTURE_MIN_FILTER,
            Self::MagFilter => GL_TEXTURE_MAG_FILTER,
            Self::WrapS => GL_TEXTURE_WRAP_S,
            Self::WrapT => GL_TEXTURE_WRAP_T,
            Self::WrapR => GL_TEXTURE_WRAP_R,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GlTextureFilter {
    Nearest,
    Linear,
    NearestMipmapNearest,
    LinearMipmapNearest,
}

impl GlTextureFilter {
    pub fn as_raw(self) -> u32 {
        match self {
            Self::Nearest => GL_NEAREST,
            Self::Linear => GL_LINEAR,
            Self::NearestMipmapNearest => GL_NEAREST_MIPMAP_NEAREST,
            Self::LinearMipmapNearest => GL_LINEAR_MIPMAP_NEAREST,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GlTextureWrap {
    ClampToEdge,
    Repeat,
}

impl GlTextureWrap {
    pub fn as_raw(self) -> u32 {
        match self {
            Self::ClampToEdge => GL_CLAMP_TO_EDGE,
            Self::Repeat => GL_REPEAT,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlTextureInternalFormat {
    R8,
    Luminance,
    Rgb,
    Rgba,
    Rgba8,
}

impl GlTextureInternalFormat {
    pub fn as_raw(self) -> u32 {
        match self {
            Self::R8 => GL_R8,
            Self::Luminance => GL_LUMINANCE,
            Self::Rgb => GL_RGB,
            Self::Rgba => GL_RGBA,
            Self::Rgba8 => GL_RGBA8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlTextureFormat {
    Red,
    Luminance,
    Rgb,
    Rgba,
}

impl GlTextureFormat {
    pub fn as_raw(self) -> u32 {
        match self {
            Self::Red => GL_RED,
            Self::Luminance => GL_LUMINANCE,
            Self::Rgb => GL_RGB,
            Self::Rgba => GL_RGBA,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlTextureDataType {
    UnsignedByte,
    UnsignedShort4444,
    UnsignedShort565,
}

impl GlTextureDataType {
    pub fn as_raw(self) -> u32 {
        match self {
            Self::UnsignedByte => GL_UNSIGNED_BYTE,
            Self::UnsignedShort4444 => GL_UNSIGNED_SHORT_4_4_4_4,
            Self::UnsignedShort565 => GL_UNSIGNED_SHORT_5_6_5,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GlCapability {
    Blend,
    ScissorTest,
}

impl GlCapability {
    fn as_raw(self) -> u32 {
        match self {
            Self::Blend => GL_BLEND,
            Self::ScissorTest => GL_SCISSOR_TEST,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GlBlendFactor {
    One,
    SourceAlpha,
    OneMinusSourceAlpha,
}

impl GlBlendFactor {
    fn as_raw(self) -> u32 {
        match self {
            Self::One => GL_ONE,
            Self::SourceAlpha => GL_SRC_ALPHA,
            Self::OneMinusSourceAlpha => GL_ONE_MINUS_SRC_ALPHA,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GlBlendEquation {
    Add,
    ReverseSubtract,
}

impl GlBlendEquation {
    fn as_raw(self) -> u32 {
        match self {
            Self::Add => GL_FUNC_ADD,
            Self::ReverseSubtract => GL_FUNC_REVERSE_SUBTRACT,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GlIndexType {
    UnsignedShort,
}

impl GlIndexType {
    fn as_raw(self) -> u32 {
        match self {
            Self::UnsignedShort => GL_UNSIGNED_SHORT,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GlDrawMode {
    Triangles,
}

impl GlDrawMode {
    fn as_raw(self) -> u32 {
        match self {
            Self::Triangles => GL_TRIANGLES,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GlFramebufferTarget {
    Framebuffer,
}

impl GlFramebufferTarget {
    fn as_raw(self) -> u32 {
        match self {
            Self::Framebuffer => GL_FRAMEBUFFER,
        }
    }
}

/// Typed OpenGL symbol table resolved from libretro's hardware-render callbacks.
///
/// `init()` expects the frontend-owned GL context to be active, which in libretro
/// means after `hw_context_reset` and before `hw_context_destroy`.
#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct glsym {
    context_type: HwContextType,
    version_info: GlVersionInfo,
    vendor_string: String,
    renderer_string: String,
    version_string: String,
    extensions_string: String,
    max_texture_size: Option<u32>,
    max_texture_image_units: Option<u32>,
    max_varying_vectors: Option<u32>,
    fragment_highp_float: Option<bool>,
    clear_color: GlClearColor,
    clear: GlClear,
    enable: GlEnable,
    disable: GlDisable,
    viewport: GlViewport,
    scissor: GlScissor,
    create_shader: GlCreateShader,
    shader_source: GlShaderSource,
    compile_shader: GlCompileShader,
    get_shader_iv: GlGetShaderIv,
    get_shader_info_log: GlGetShaderInfoLog,
    delete_shader: GlDeleteShader,
    create_program: GlCreateProgram,
    attach_shader: GlAttachShader,
    link_program: GlLinkProgram,
    get_program_iv: GlGetProgramIv,
    get_program_info_log: GlGetProgramInfoLog,
    delete_program: GlDeleteProgram,
    use_program: GlUseProgram,
    gen_buffers: GlGenBuffers,
    bind_buffer: GlBindBuffer,
    buffer_data: GlBufferData,
    delete_buffers: GlDeleteBuffers,
    gen_textures: GlGenTextures,
    bind_texture: GlBindTexture,
    active_texture: GlActiveTexture,
    tex_parameter_i: GlTexParameteri,
    pixel_store_i: GlPixelStorei,
    tex_image_2d: GlTexImage2D,
    tex_image_3d: Option<GlTexImage3D>,
    tex_sub_image_3d: Option<GlTexSubImage3D>,
    generate_mipmap: Option<GlGenerateMipmap>,
    delete_textures: GlDeleteTextures,
    gen_vertex_arrays: Option<GlGenVertexArrays>,
    bind_vertex_array: Option<GlBindVertexArray>,
    delete_vertex_arrays: Option<GlDeleteVertexArrays>,
    enable_vertex_attrib_array: GlEnableVertexAttribArray,
    disable_vertex_attrib_array: GlDisableVertexAttribArray,
    vertex_attrib_pointer: GlVertexAttribPointer,
    vertex_attrib_divisor: Option<GlVertexAttribDivisor>,
    get_uniform_location: GlGetUniformLocation,
    get_attrib_location: GlGetAttribLocation,
    uniform_1i: GlUniform1i,
    uniform_1f: GlUniform1f,
    uniform_2f: GlUniform2f,
    uniform_4fv: GlUniform4fv,
    uniform_matrix_4fv: GlUniformMatrix4fv,
    draw_arrays: GlDrawArrays,
    draw_elements: GlDrawElements,
    draw_elements_instanced: Option<GlDrawElementsInstanced>,
    blend_func: GlBlendFunc,
    blend_equation: GlBlendEquationFn,
    bind_framebuffer: GlBindFramebuffer,
    get_error: Option<GlGetError>,
    check_framebuffer_status: Option<GlCheckFramebufferStatus>,
    invalidate_framebuffer: Option<GlInvalidateFramebuffer>,
    discard_framebuffer_ext: Option<GlInvalidateFramebuffer>,
}

/// Minimal GL entry points needed for legal hardware framebuffer setup.
///
/// This intentionally loads only framebuffer binding, viewport, and clear calls
/// so cores can present a hardware frame before probing shader, texture, or
/// product-renderer symbol coverage.
#[derive(Clone, Copy)]
pub struct CompatGlClear {
    context_type: HwContextType,
    version_info: GlVersionInfo,
    clear_color: GlClearColor,
    clear: GlClear,
    viewport: GlViewport,
    bind_framebuffer: GlBindFramebuffer,
    get_error: Option<GlGetError>,
    check_framebuffer_status: Option<GlCheckFramebufferStatus>,
}

/// GLES2-era drawing symbols for simple compatibility rendering.
///
/// This is narrower than `glsym::init()`: it deliberately avoids product
/// renderer symbols such as scissor, draw-elements, matrices, texture arrays,
/// VAOs, instancing, and blend equations.
#[derive(Clone, Copy)]
pub struct CompatGl {
    clear: CompatGlClear,
    create_shader: GlCreateShader,
    shader_source: GlShaderSource,
    compile_shader: GlCompileShader,
    get_shader_iv: GlGetShaderIv,
    get_shader_info_log: GlGetShaderInfoLog,
    delete_shader: GlDeleteShader,
    create_program: GlCreateProgram,
    attach_shader: GlAttachShader,
    link_program: GlLinkProgram,
    get_program_iv: GlGetProgramIv,
    get_program_info_log: GlGetProgramInfoLog,
    delete_program: GlDeleteProgram,
    use_program: GlUseProgram,
    gen_buffers: GlGenBuffers,
    bind_buffer: GlBindBuffer,
    buffer_data: GlBufferData,
    delete_buffers: GlDeleteBuffers,
    enable_vertex_attrib_array: GlEnableVertexAttribArray,
    disable_vertex_attrib_array: GlDisableVertexAttribArray,
    vertex_attrib_pointer: GlVertexAttribPointer,
    get_uniform_location: GlGetUniformLocation,
    uniform_4fv: GlUniform4fv,
    get_attrib_location: GlGetAttribLocation,
    draw_arrays: GlDrawArrays,
}

/// Texture/uniform/blend symbols layered on top of `CompatGl`.
///
/// Loading this separately keeps the known-good clear/triangle path alive when
/// older libretro GL proc lookup fails for texture or blending entry points.
/// `glActiveTexture` is required so callers can restore texture unit 0
/// explicitly after uploads and draws.
#[derive(Clone, Copy)]
pub struct CompatTextureGl {
    max_texture_size: Option<u32>,
    get_error: Option<GlGetError>,
    enable: GlEnable,
    disable: GlDisable,
    gen_textures: GlGenTextures,
    bind_texture: GlBindTexture,
    active_texture: GlActiveTexture,
    tex_parameter_i: GlTexParameteri,
    pixel_store_i: GlPixelStorei,
    tex_image_2d: GlTexImage2D,
    delete_textures: GlDeleteTextures,
    get_uniform_location: GlGetUniformLocation,
    uniform_1i: GlUniform1i,
    uniform_4fv: GlUniform4fv,
    blend_func: GlBlendFunc,
}

fn parse_gl_version_info(version_string: &str) -> Option<GlVersionInfo> {
    let (is_gles, version_token) = if let Some(rest) = version_string.strip_prefix("OpenGL ES-CM ")
    {
        (true, rest.split_whitespace().next()?)
    } else if let Some(rest) = version_string.strip_prefix("OpenGL ES-CL ") {
        (true, rest.split_whitespace().next()?)
    } else if let Some(rest) = version_string.strip_prefix("OpenGL ES ") {
        (true, rest.split_whitespace().next()?)
    } else {
        (false, version_string.split_whitespace().next()?)
    };

    let mut parts = version_token.split('.');
    let major = parts.next()?.parse::<u32>().ok()?;
    let minor = parts.next()?.parse::<u32>().ok()?;
    Some(GlVersionInfo {
        is_gles,
        major: Some(major),
        minor: Some(minor),
    })
}

fn fallback_gl_version_info(context_type: HwContextType) -> GlVersionInfo {
    match context_type {
        HwContextType::OpenGlEs2 => GlVersionInfo {
            is_gles: true,
            major: Some(2),
            minor: Some(0),
        },
        HwContextType::OpenGlEs3 => GlVersionInfo {
            is_gles: true,
            major: Some(3),
            minor: Some(0),
        },
        HwContextType::OpenGlEsVersion => GlVersionInfo {
            is_gles: true,
            major: None,
            minor: None,
        },
        _ => GlVersionInfo::default(),
    }
}

fn query_gl_version_info(get_string: GlGetString, context_type: HwContextType) -> GlVersionInfo {
    let version = query_gl_string(get_string, GL_VERSION);
    if let Some(parsed) = parse_gl_version_info(&version) {
        return parsed;
    }

    fallback_gl_version_info(context_type)
}

fn query_gl_string(get_string: GlGetString, name: u32) -> String {
    let raw = unsafe { get_string(name) };
    if raw.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(raw.cast::<c_char>()) }
        .to_string_lossy()
        .into_owned()
}

fn query_fragment_highp_float(
    get_shader_precision_format: Option<GlGetShaderPrecisionFormat>,
    version_info: GlVersionInfo,
) -> Option<bool> {
    if !version_info.is_gles {
        return Some(true);
    }
    let get_shader_precision_format = get_shader_precision_format?;
    let mut range = [0i32; 2];
    let mut precision = 0i32;
    unsafe {
        get_shader_precision_format(
            GL_FRAGMENT_SHADER,
            GL_HIGH_FLOAT,
            range.as_mut_ptr(),
            &mut precision,
        )
    };
    Some(range != [0, 0] || precision > 0)
}

fn normalize_positive_gl_limit(value: i32) -> Option<u32> {
    u32::try_from(value).ok().filter(|value| *value > 0)
}

fn query_gl_max_texture_size(get_integer_v: GlGetIntegerv) -> Option<u32> {
    let mut value = 0;
    unsafe { get_integer_v(GL_MAX_TEXTURE_SIZE, &mut value) };
    normalize_positive_gl_limit(value)
}

fn query_gl_max_texture_image_units(get_integer_v: GlGetIntegerv) -> Option<u32> {
    let mut value = 0;
    unsafe { get_integer_v(GL_MAX_TEXTURE_IMAGE_UNITS, &mut value) };
    normalize_positive_gl_limit(value)
}

fn query_gl_max_varying_vectors(get_integer_v: GlGetIntegerv) -> Option<u32> {
    let mut value = 0;
    unsafe { get_integer_v(GL_MAX_VARYING_VECTORS, &mut value) };
    normalize_positive_gl_limit(value)
}

fn is_gles_context(context_type: HwContextType, version_info: GlVersionInfo) -> bool {
    version_info.is_gles
        || matches!(
            context_type,
            HwContextType::OpenGlEs2 | HwContextType::OpenGlEs3 | HwContextType::OpenGlEsVersion
        )
}

fn supports_core_texture_arrays(_context_type: HwContextType, version_info: GlVersionInfo) -> bool {
    // Function-pointer presence is not enough after process-global fallback:
    // desktop GL loaders often expose dispatch stubs for unsupported contexts.
    version_info.version_at_least(3, 0)
}

fn supports_core_instancing(context_type: HwContextType, version_info: GlVersionInfo) -> bool {
    let is_gles = is_gles_context(context_type, version_info);
    if is_gles {
        version_info.version_at_least(3, 0)
    } else {
        // The product modern desktop path uses core glVertexAttribDivisor and
        // OpenGL 3.3-era shader expectations. Do not promote from process-global
        // dispatch stubs alone on older compatibility contexts.
        version_info.version_at_least(3, 3)
    }
}

fn describe_gl_error(error: u32) -> &'static str {
    match error {
        GL_NO_ERROR => "GL_NO_ERROR",
        GL_INVALID_ENUM => "GL_INVALID_ENUM",
        GL_INVALID_VALUE => "GL_INVALID_VALUE",
        GL_INVALID_OPERATION => "GL_INVALID_OPERATION",
        GL_OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
        GL_INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
        _ => "unknown GL error",
    }
}

fn describe_framebuffer_status(status: u32) -> &'static str {
    match status {
        GL_FRAMEBUFFER_COMPLETE => "GL_FRAMEBUFFER_COMPLETE",
        GL_FRAMEBUFFER_INCOMPLETE_ATTACHMENT => "GL_FRAMEBUFFER_INCOMPLETE_ATTACHMENT",
        GL_FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => {
            "GL_FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT"
        }
        GL_FRAMEBUFFER_INCOMPLETE_DIMENSIONS => "GL_FRAMEBUFFER_INCOMPLETE_DIMENSIONS",
        GL_FRAMEBUFFER_UNSUPPORTED => "GL_FRAMEBUFFER_UNSUPPORTED",
        _ => "unknown framebuffer status",
    }
}

fn collect_gl_errors(get_error: Option<GlGetError>, operation: &str) -> Result<(), String> {
    let Some(get_error) = get_error else {
        return Ok(());
    };

    let mut errors = Vec::new();
    for _ in 0..16 {
        let error = unsafe { get_error() };
        if error == GL_NO_ERROR {
            break;
        }
        errors.push(error);
    }

    if errors.is_empty() {
        return Ok(());
    }

    let joined = errors
        .into_iter()
        .map(|error| format!("{} ({error:#06x})", describe_gl_error(error)))
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!("{operation} observed GL error(s): {joined}"))
}

fn check_bound_framebuffer_complete(
    check_framebuffer_status: Option<GlCheckFramebufferStatus>,
    target: GlFramebufferTarget,
    operation: &str,
) -> Result<(), String> {
    let Some(check_framebuffer_status) = check_framebuffer_status else {
        return Ok(());
    };

    let status = unsafe { check_framebuffer_status(target.as_raw()) };
    if status == GL_FRAMEBUFFER_COMPLETE {
        return Ok(());
    }

    Err(format!(
        "{operation} found incomplete framebuffer: {} ({status:#06x})",
        describe_framebuffer_status(status)
    ))
}

impl CompatGlClear {
    pub fn init(runtime: &Runtime<'_>) -> Result<Self, String> {
        let context_type = runtime
            .hw_context_type()
            .ok_or_else(|| "hardware render context type is not available".to_string())?;
        if !context_type.is_opengl_family() {
            return Err(format!(
                "CompatGlClear requires an OpenGL-family context, got {context_type:?}"
            ));
        }
        let get_string: Option<GlGetString> = load_optional_gl_symbol(runtime, "glGetString")?;

        Ok(Self {
            context_type,
            version_info: get_string
                .map(|get_string| query_gl_version_info(get_string, context_type))
                .unwrap_or_else(|| fallback_gl_version_info(context_type)),
            clear_color: load_gl_symbol(runtime, "glClearColor")?,
            clear: load_gl_symbol(runtime, "glClear")?,
            viewport: load_gl_symbol(runtime, "glViewport")?,
            bind_framebuffer: load_gl_symbol(runtime, "glBindFramebuffer")?,
            get_error: load_optional_gl_symbol(runtime, "glGetError")?,
            check_framebuffer_status: load_optional_gl_symbol(runtime, "glCheckFramebufferStatus")?,
        })
    }

    pub fn context_type(&self) -> HwContextType {
        self.context_type
    }

    pub fn version_info(&self) -> GlVersionInfo {
        self.version_info
    }

    pub fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        unsafe { (self.clear_color)(r, g, b, a) };
    }

    pub fn clear_color_depth_buffer(&self) {
        unsafe { (self.clear)(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT) };
    }

    pub fn viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe { (self.viewport)(x, y, width, height) };
    }

    pub fn bind_framebuffer(&self, target: GlFramebufferTarget, framebuffer: u32) {
        unsafe { (self.bind_framebuffer)(target.as_raw(), framebuffer) };
    }

    pub fn bind_framebuffer_checked(
        &self,
        target: GlFramebufferTarget,
        framebuffer: u32,
    ) -> Result<(), String> {
        self.bind_framebuffer(target, framebuffer);
        self.check_no_error("glBindFramebuffer")?;
        if framebuffer != 0 {
            self.check_bound_framebuffer_complete(target)?;
        }
        Ok(())
    }

    pub fn check_no_error(&self, operation: &str) -> Result<(), String> {
        collect_gl_errors(self.get_error, operation)
    }

    pub fn check_bound_framebuffer_complete(
        &self,
        target: GlFramebufferTarget,
    ) -> Result<(), String> {
        check_bound_framebuffer_complete(
            self.check_framebuffer_status,
            target,
            "glCheckFramebufferStatus",
        )
    }

    #[doc(hidden)]
    pub fn fake_for_testing(config: FakeGlConfig) -> Self {
        let gl = glsym::fake_for_testing(config);
        Self::from_glsym(gl)
    }

    pub fn from_glsym(gl: glsym) -> Self {
        Self {
            context_type: gl.context_type,
            version_info: gl.version_info,
            clear_color: gl.clear_color,
            clear: gl.clear,
            viewport: gl.viewport,
            bind_framebuffer: gl.bind_framebuffer,
            get_error: gl.get_error,
            check_framebuffer_status: gl.check_framebuffer_status,
        }
    }
}

impl CompatGl {
    pub fn init(runtime: &Runtime<'_>) -> Result<Self, String> {
        let clear = CompatGlClear::init(runtime)?;
        Self::init_from_clear(runtime, clear)
    }

    pub fn init_from_clear(runtime: &Runtime<'_>, clear: CompatGlClear) -> Result<Self, String> {
        Ok(Self {
            clear,
            create_shader: load_gl_symbol(runtime, "glCreateShader")?,
            shader_source: load_gl_symbol(runtime, "glShaderSource")?,
            compile_shader: load_gl_symbol(runtime, "glCompileShader")?,
            get_shader_iv: load_gl_symbol(runtime, "glGetShaderiv")?,
            get_shader_info_log: load_gl_symbol(runtime, "glGetShaderInfoLog")?,
            delete_shader: load_gl_symbol(runtime, "glDeleteShader")?,
            create_program: load_gl_symbol(runtime, "glCreateProgram")?,
            attach_shader: load_gl_symbol(runtime, "glAttachShader")?,
            link_program: load_gl_symbol(runtime, "glLinkProgram")?,
            get_program_iv: load_gl_symbol(runtime, "glGetProgramiv")?,
            get_program_info_log: load_gl_symbol(runtime, "glGetProgramInfoLog")?,
            delete_program: load_gl_symbol(runtime, "glDeleteProgram")?,
            use_program: load_gl_symbol(runtime, "glUseProgram")?,
            gen_buffers: load_gl_symbol(runtime, "glGenBuffers")?,
            bind_buffer: load_gl_symbol(runtime, "glBindBuffer")?,
            buffer_data: load_gl_symbol(runtime, "glBufferData")?,
            delete_buffers: load_gl_symbol(runtime, "glDeleteBuffers")?,
            enable_vertex_attrib_array: load_gl_symbol(runtime, "glEnableVertexAttribArray")?,
            disable_vertex_attrib_array: load_gl_symbol(runtime, "glDisableVertexAttribArray")?,
            vertex_attrib_pointer: load_gl_symbol(runtime, "glVertexAttribPointer")?,
            get_uniform_location: load_gl_symbol(runtime, "glGetUniformLocation")?,
            uniform_4fv: load_gl_symbol(runtime, "glUniform4fv")?,
            get_attrib_location: load_gl_symbol(runtime, "glGetAttribLocation")?,
            draw_arrays: load_gl_symbol(runtime, "glDrawArrays")?,
        })
    }

    pub fn clear_symbols(&self) -> CompatGlClear {
        self.clear
    }

    pub fn context_type(&self) -> HwContextType {
        self.clear.context_type()
    }

    pub fn version_info(&self) -> GlVersionInfo {
        self.clear.version_info()
    }

    pub fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        self.clear.clear_color(r, g, b, a);
    }

    pub fn clear_color_depth_buffer(&self) {
        self.clear.clear_color_depth_buffer();
    }

    pub fn viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        self.clear.viewport(x, y, width, height);
    }

    pub fn bind_framebuffer(&self, target: GlFramebufferTarget, framebuffer: u32) {
        self.clear.bind_framebuffer(target, framebuffer);
    }

    pub fn bind_framebuffer_checked(
        &self,
        target: GlFramebufferTarget,
        framebuffer: u32,
    ) -> Result<(), String> {
        self.clear.bind_framebuffer_checked(target, framebuffer)
    }

    pub fn check_no_error(&self, operation: &str) -> Result<(), String> {
        self.clear.check_no_error(operation)
    }

    pub fn check_bound_framebuffer_complete(
        &self,
        target: GlFramebufferTarget,
    ) -> Result<(), String> {
        self.clear.check_bound_framebuffer_complete(target)
    }

    pub fn build_program(&self, vertex_shader: &str, fragment_shader: &str) -> Result<u32, String> {
        let vertex = self.compile_shader_source(GL_VERTEX_SHADER, vertex_shader)?;
        let fragment = match self.compile_shader_source(GL_FRAGMENT_SHADER, fragment_shader) {
            Ok(shader) => shader,
            Err(error) => {
                self.delete_shader(vertex);
                return Err(error);
            }
        };

        let program = self.create_program();
        if program == 0 {
            self.delete_shader(vertex);
            self.delete_shader(fragment);
            return Err("glCreateProgram returned 0".to_string());
        }

        self.attach_shader(program, vertex);
        self.attach_shader(program, fragment);
        self.link_program(program);
        self.delete_shader(vertex);
        self.delete_shader(fragment);

        let status = self.get_program_iv(program, GL_LINK_STATUS);
        if status == 0 {
            let log = self.get_program_info_log(program);
            self.delete_program(program);
            return Err(format!("program link failed: {log}"));
        }

        if let Err(error) = self.check_no_error("CompatGl::build_program") {
            self.delete_program(program);
            return Err(error);
        }
        Ok(program)
    }

    pub fn compile_shader_source(&self, shader_type: u32, source: &str) -> Result<u32, String> {
        let shader = self.create_shader(shader_type);
        if shader == 0 {
            return Err(format!("glCreateShader({shader_type:#06x}) returned 0"));
        }
        if let Err(error) = self.shader_source(shader, source) {
            self.delete_shader(shader);
            return Err(error);
        }
        self.compile_shader(shader);

        let status = self.get_shader_iv(shader, GL_COMPILE_STATUS);
        if status == 0 {
            let log = self.get_shader_info_log(shader);
            self.delete_shader(shader);
            return Err(format!("shader compile failed: {log}"));
        }

        if let Err(error) = self.check_no_error("CompatGl::compile_shader_source") {
            self.delete_shader(shader);
            return Err(error);
        }
        Ok(shader)
    }

    fn create_shader(&self, shader_type: u32) -> u32 {
        unsafe { (self.create_shader)(shader_type) }
    }

    fn shader_source(&self, shader: u32, source: &str) -> Result<(), String> {
        let source = gl_string(source)?;
        let ptr = source.as_ptr();
        unsafe { (self.shader_source)(shader, 1, &ptr, std::ptr::null()) };
        Ok(())
    }

    fn compile_shader(&self, shader: u32) {
        unsafe { (self.compile_shader)(shader) };
    }

    fn get_shader_iv(&self, shader: u32, parameter: u32) -> i32 {
        let mut value = 0;
        unsafe { (self.get_shader_iv)(shader, parameter, &mut value) };
        value
    }

    fn get_shader_info_log(&self, shader: u32) -> String {
        let length = self.get_shader_iv(shader, GL_INFO_LOG_LENGTH);
        if length <= 1 {
            return "no log".to_string();
        }

        let mut buffer = vec![0u8; length as usize];
        let mut written = 0;
        unsafe {
            (self.get_shader_info_log)(
                shader,
                length,
                &mut written,
                buffer.as_mut_ptr().cast::<c_char>(),
            )
        };
        String::from_utf8_lossy(&buffer[..written as usize]).into_owned()
    }

    fn delete_shader(&self, shader: u32) {
        unsafe { (self.delete_shader)(shader) };
    }

    fn create_program(&self) -> u32 {
        unsafe { (self.create_program)() }
    }

    fn attach_shader(&self, program: u32, shader: u32) {
        unsafe { (self.attach_shader)(program, shader) };
    }

    fn link_program(&self, program: u32) {
        unsafe { (self.link_program)(program) };
    }

    fn get_program_iv(&self, program: u32, parameter: u32) -> i32 {
        let mut value = 0;
        unsafe { (self.get_program_iv)(program, parameter, &mut value) };
        value
    }

    fn get_program_info_log(&self, program: u32) -> String {
        let length = self.get_program_iv(program, GL_INFO_LOG_LENGTH);
        if length <= 1 {
            return "no log".to_string();
        }

        let mut buffer = vec![0u8; length as usize];
        let mut written = 0;
        unsafe {
            (self.get_program_info_log)(
                program,
                length,
                &mut written,
                buffer.as_mut_ptr().cast::<c_char>(),
            )
        };
        String::from_utf8_lossy(&buffer[..written as usize]).into_owned()
    }

    pub fn delete_program(&self, program: u32) {
        unsafe { (self.delete_program)(program) };
    }

    pub fn use_program(&self, program: u32) {
        unsafe { (self.use_program)(program) };
    }

    pub fn gen_buffer(&self) -> u32 {
        let mut id = 0;
        unsafe { (self.gen_buffers)(1, &mut id) };
        id
    }

    pub fn bind_buffer(&self, target: GlBufferTarget, buffer: u32) {
        unsafe { (self.bind_buffer)(target.as_raw(), buffer) };
    }

    pub fn buffer_data<T>(&self, target: GlBufferTarget, data: &[T], usage: GlBufferUsage) {
        let byte_len = std::mem::size_of_val(data);
        unsafe {
            (self.buffer_data)(
                target.as_raw(),
                byte_len as isize,
                data.as_ptr().cast::<c_void>(),
                usage.as_raw(),
            );
        }
    }

    pub fn delete_buffer(&self, id: u32) {
        unsafe { (self.delete_buffers)(1, &id) };
    }

    pub fn enable_vertex_attrib_array(&self, index: u32) {
        unsafe { (self.enable_vertex_attrib_array)(index) };
    }

    pub fn disable_vertex_attrib_array(&self, index: u32) {
        unsafe { (self.disable_vertex_attrib_array)(index) };
    }

    pub fn vertex_attrib_pointer_f32(
        &self,
        index: u32,
        size: i32,
        normalized: bool,
        stride: i32,
        offset: usize,
    ) {
        unsafe {
            (self.vertex_attrib_pointer)(
                index,
                size,
                GL_FLOAT,
                if normalized { 1 } else { GL_FALSE },
                stride,
                offset as *const c_void,
            );
        }
    }

    pub fn required_attrib_location(&self, program: u32, name: &str) -> Result<u32, String> {
        let name = gl_string(name)?;
        let location = unsafe { (self.get_attrib_location)(program, name.as_ptr()) };
        if location < 0 {
            return Err(format!(
                "shader linked without required active attribute {name:?}"
            ));
        }
        Ok(location as u32)
    }

    pub fn required_uniform_location(&self, program: u32, name: &str) -> Result<i32, String> {
        let name = gl_string(name)?;
        let location = unsafe { (self.get_uniform_location)(program, name.as_ptr()) };
        if location < 0 {
            return Err(format!(
                "shader linked without required active uniform {name:?}"
            ));
        }
        Ok(location)
    }

    pub fn uniform_4fv(&self, location: i32, values: &[f32; 4]) {
        unsafe { (self.uniform_4fv)(location, 1, values.as_ptr()) };
    }

    pub fn draw_arrays(&self, mode: GlDrawMode, first: i32, count: i32) {
        unsafe { (self.draw_arrays)(mode.as_raw(), first, count) };
    }

    #[doc(hidden)]
    pub fn fake_for_testing(config: FakeGlConfig) -> Self {
        let gl = glsym::fake_for_testing(config);
        Self::from_glsym(gl)
    }

    pub fn from_glsym(gl: glsym) -> Self {
        Self {
            clear: CompatGlClear::from_glsym(gl.clone()),
            create_shader: gl.create_shader,
            shader_source: gl.shader_source,
            compile_shader: gl.compile_shader,
            get_shader_iv: gl.get_shader_iv,
            get_shader_info_log: gl.get_shader_info_log,
            delete_shader: gl.delete_shader,
            create_program: gl.create_program,
            attach_shader: gl.attach_shader,
            link_program: gl.link_program,
            get_program_iv: gl.get_program_iv,
            get_program_info_log: gl.get_program_info_log,
            delete_program: gl.delete_program,
            use_program: gl.use_program,
            gen_buffers: gl.gen_buffers,
            bind_buffer: gl.bind_buffer,
            buffer_data: gl.buffer_data,
            delete_buffers: gl.delete_buffers,
            enable_vertex_attrib_array: gl.enable_vertex_attrib_array,
            disable_vertex_attrib_array: gl.disable_vertex_attrib_array,
            vertex_attrib_pointer: gl.vertex_attrib_pointer,
            get_uniform_location: gl.get_uniform_location,
            uniform_4fv: gl.uniform_4fv,
            get_attrib_location: gl.get_attrib_location,
            draw_arrays: gl.draw_arrays,
        }
    }
}

impl CompatTextureGl {
    pub fn init(runtime: &Runtime<'_>) -> Result<Self, String> {
        let get_integer_v: Option<GlGetIntegerv> =
            load_optional_gl_symbol(runtime, "glGetIntegerv")?;
        Ok(Self {
            max_texture_size: get_integer_v.and_then(query_gl_max_texture_size),
            get_error: load_optional_gl_symbol(runtime, "glGetError")?,
            enable: load_gl_symbol(runtime, "glEnable")?,
            disable: load_gl_symbol(runtime, "glDisable")?,
            gen_textures: load_gl_symbol(runtime, "glGenTextures")?,
            bind_texture: load_gl_symbol(runtime, "glBindTexture")?,
            active_texture: load_gl_symbol(runtime, "glActiveTexture")?,
            tex_parameter_i: load_gl_symbol(runtime, "glTexParameteri")?,
            pixel_store_i: load_gl_symbol(runtime, "glPixelStorei")?,
            tex_image_2d: load_gl_symbol(runtime, "glTexImage2D")?,
            delete_textures: load_gl_symbol(runtime, "glDeleteTextures")?,
            get_uniform_location: load_gl_symbol(runtime, "glGetUniformLocation")?,
            uniform_1i: load_gl_symbol(runtime, "glUniform1i")?,
            uniform_4fv: load_gl_symbol(runtime, "glUniform4fv")?,
            blend_func: load_gl_symbol(runtime, "glBlendFunc")?,
        })
    }

    pub fn max_texture_size(&self) -> Option<u32> {
        self.max_texture_size
    }

    pub fn from_glsym(gl: glsym) -> Self {
        Self {
            max_texture_size: gl.max_texture_size,
            get_error: gl.get_error,
            enable: gl.enable,
            disable: gl.disable,
            gen_textures: gl.gen_textures,
            bind_texture: gl.bind_texture,
            active_texture: gl.active_texture,
            tex_parameter_i: gl.tex_parameter_i,
            pixel_store_i: gl.pixel_store_i,
            tex_image_2d: gl.tex_image_2d,
            delete_textures: gl.delete_textures,
            get_uniform_location: gl.get_uniform_location,
            uniform_1i: gl.uniform_1i,
            uniform_4fv: gl.uniform_4fv,
            blend_func: gl.blend_func,
        }
    }

    pub fn enable(&self, capability: GlCapability) {
        unsafe { (self.enable)(capability.as_raw()) };
    }

    pub fn disable(&self, capability: GlCapability) {
        unsafe { (self.disable)(capability.as_raw()) };
    }

    pub fn gen_texture(&self) -> u32 {
        let mut id = 0;
        unsafe { (self.gen_textures)(1, &mut id) };
        id
    }

    pub fn bind_texture(&self, target: GlTextureTarget, texture: u32) {
        unsafe { (self.bind_texture)(target.as_raw(), texture) };
    }

    pub fn active_texture(&self, unit_index: u32) {
        unsafe { (self.active_texture)(GL_TEXTURE0 + unit_index) };
    }

    pub fn tex_parameter_i(
        &self,
        target: GlTextureTarget,
        parameter: GlTextureParameter,
        value: i32,
    ) {
        unsafe { (self.tex_parameter_i)(target.as_raw(), parameter.as_raw(), value) };
    }

    pub fn pixel_store_unpack_alignment(&self, alignment: i32) {
        unsafe { (self.pixel_store_i)(GL_UNPACK_ALIGNMENT, alignment) };
    }

    pub fn tex_image_2d_u8(
        &self,
        target: GlTextureTarget,
        internal_format: GlTextureInternalFormat,
        width: i32,
        height: i32,
        format: GlTextureFormat,
        bytes: Option<&[u8]>,
    ) {
        self.tex_image_2d_typed_u8(
            target,
            internal_format,
            width,
            height,
            format,
            GlTextureDataType::UnsignedByte,
            bytes,
        );
    }

    pub fn tex_image_2d_typed_u8(
        &self,
        target: GlTextureTarget,
        internal_format: GlTextureInternalFormat,
        width: i32,
        height: i32,
        format: GlTextureFormat,
        data_type: GlTextureDataType,
        bytes: Option<&[u8]>,
    ) {
        let pixels = bytes
            .map(|bytes| bytes.as_ptr().cast::<c_void>())
            .unwrap_or(std::ptr::null());
        unsafe {
            (self.tex_image_2d)(
                target.as_raw(),
                0,
                internal_format.as_raw() as i32,
                width,
                height,
                0,
                format.as_raw(),
                data_type.as_raw(),
                pixels,
            );
        }
    }

    pub fn delete_texture(&self, id: u32) {
        unsafe { (self.delete_textures)(1, &id) };
    }

    pub fn required_uniform_location(&self, program: u32, name: &str) -> Result<i32, String> {
        let name = gl_string(name)?;
        let location = unsafe { (self.get_uniform_location)(program, name.as_ptr()) };
        if location < 0 {
            return Err(format!(
                "shader linked without required active uniform {name:?}"
            ));
        }
        Ok(location)
    }

    pub fn uniform_1i(&self, location: i32, value: i32) {
        unsafe { (self.uniform_1i)(location, value) };
    }

    pub fn uniform_4fv(&self, location: i32, values: &[f32; 4]) {
        unsafe { (self.uniform_4fv)(location, 1, values.as_ptr()) };
    }

    pub fn blend_func(&self, source: GlBlendFactor, destination: GlBlendFactor) {
        unsafe { (self.blend_func)(source.as_raw(), destination.as_raw()) };
    }

    pub fn check_no_error(&self, operation: &str) -> Result<(), String> {
        collect_gl_errors(self.get_error, operation)
    }

    #[doc(hidden)]
    pub fn fake_for_testing(config: FakeGlConfig) -> Self {
        let gl = glsym::fake_for_testing(config);
        Self::from_glsym(gl)
    }
}

impl glsym {
    pub fn init(runtime: &Runtime<'_>) -> Result<Self, String> {
        let context_type = runtime
            .hw_context_type()
            .ok_or_else(|| "hardware render context type is not available".to_string())?;
        if !context_type.is_opengl_family() {
            return Err(format!(
                "glsym requires an OpenGL-family context, got {context_type:?}"
            ));
        }
        let get_string: GlGetString = load_gl_symbol(runtime, "glGetString")?;
        let get_integer_v: GlGetIntegerv = load_gl_symbol(runtime, "glGetIntegerv")?;
        let version_info = query_gl_version_info(get_string, context_type);
        let extensions_string = if version_info.is_gles || !version_info.version_at_least(3, 0) {
            query_gl_string(get_string, GL_EXTENSIONS)
        } else {
            // Desktop OpenGL 3+ core profiles require glGetStringi for
            // extensions; glGetString(GL_EXTENSIONS) is invalid and can leave a
            // sticky GL error that later shader/bootstrap validation reports.
            String::new()
        };
        let get_shader_precision_format: Option<GlGetShaderPrecisionFormat> =
            load_optional_gl_symbol(runtime, "glGetShaderPrecisionFormat")?;
        Ok(Self {
            context_type,
            version_info,
            vendor_string: query_gl_string(get_string, GL_VENDOR),
            renderer_string: query_gl_string(get_string, GL_RENDERER),
            version_string: query_gl_string(get_string, GL_VERSION),
            extensions_string,
            max_texture_size: query_gl_max_texture_size(get_integer_v),
            max_texture_image_units: query_gl_max_texture_image_units(get_integer_v),
            max_varying_vectors: query_gl_max_varying_vectors(get_integer_v),
            fragment_highp_float: query_fragment_highp_float(
                get_shader_precision_format,
                version_info,
            ),
            clear_color: load_gl_symbol(runtime, "glClearColor")?,
            clear: load_gl_symbol(runtime, "glClear")?,
            enable: load_gl_symbol(runtime, "glEnable")?,
            disable: load_gl_symbol(runtime, "glDisable")?,
            viewport: load_gl_symbol(runtime, "glViewport")?,
            scissor: load_gl_symbol(runtime, "glScissor")?,
            create_shader: load_gl_symbol(runtime, "glCreateShader")?,
            shader_source: load_gl_symbol(runtime, "glShaderSource")?,
            compile_shader: load_gl_symbol(runtime, "glCompileShader")?,
            get_shader_iv: load_gl_symbol(runtime, "glGetShaderiv")?,
            get_shader_info_log: load_gl_symbol(runtime, "glGetShaderInfoLog")?,
            delete_shader: load_gl_symbol(runtime, "glDeleteShader")?,
            create_program: load_gl_symbol(runtime, "glCreateProgram")?,
            attach_shader: load_gl_symbol(runtime, "glAttachShader")?,
            link_program: load_gl_symbol(runtime, "glLinkProgram")?,
            get_program_iv: load_gl_symbol(runtime, "glGetProgramiv")?,
            get_program_info_log: load_gl_symbol(runtime, "glGetProgramInfoLog")?,
            delete_program: load_gl_symbol(runtime, "glDeleteProgram")?,
            use_program: load_gl_symbol(runtime, "glUseProgram")?,
            gen_buffers: load_gl_symbol(runtime, "glGenBuffers")?,
            bind_buffer: load_gl_symbol(runtime, "glBindBuffer")?,
            buffer_data: load_gl_symbol(runtime, "glBufferData")?,
            delete_buffers: load_gl_symbol(runtime, "glDeleteBuffers")?,
            gen_textures: load_gl_symbol(runtime, "glGenTextures")?,
            bind_texture: load_gl_symbol(runtime, "glBindTexture")?,
            active_texture: load_gl_symbol(runtime, "glActiveTexture")?,
            tex_parameter_i: load_gl_symbol(runtime, "glTexParameteri")?,
            pixel_store_i: load_gl_symbol(runtime, "glPixelStorei")?,
            tex_image_2d: load_gl_symbol(runtime, "glTexImage2D")?,
            tex_image_3d: load_optional_gl_symbol(runtime, "glTexImage3D")?,
            tex_sub_image_3d: load_optional_gl_symbol(runtime, "glTexSubImage3D")?,
            generate_mipmap: load_optional_gl_symbol(runtime, "glGenerateMipmap")?,
            delete_textures: load_gl_symbol(runtime, "glDeleteTextures")?,
            gen_vertex_arrays: load_optional_gl_symbol(runtime, "glGenVertexArrays")?,
            bind_vertex_array: load_optional_gl_symbol(runtime, "glBindVertexArray")?,
            delete_vertex_arrays: load_optional_gl_symbol(runtime, "glDeleteVertexArrays")?,
            enable_vertex_attrib_array: load_gl_symbol(runtime, "glEnableVertexAttribArray")?,
            disable_vertex_attrib_array: load_gl_symbol(runtime, "glDisableVertexAttribArray")?,
            vertex_attrib_pointer: load_gl_symbol(runtime, "glVertexAttribPointer")?,
            vertex_attrib_divisor: load_optional_gl_symbol(runtime, "glVertexAttribDivisor")?,
            get_uniform_location: load_gl_symbol(runtime, "glGetUniformLocation")?,
            get_attrib_location: load_gl_symbol(runtime, "glGetAttribLocation")?,
            uniform_1i: load_gl_symbol(runtime, "glUniform1i")?,
            uniform_1f: load_gl_symbol(runtime, "glUniform1f")?,
            uniform_2f: load_gl_symbol(runtime, "glUniform2f")?,
            uniform_4fv: load_gl_symbol(runtime, "glUniform4fv")?,
            uniform_matrix_4fv: load_gl_symbol(runtime, "glUniformMatrix4fv")?,
            draw_arrays: load_gl_symbol(runtime, "glDrawArrays")?,
            draw_elements: load_gl_symbol(runtime, "glDrawElements")?,
            draw_elements_instanced: load_optional_gl_symbol(runtime, "glDrawElementsInstanced")?,
            blend_func: load_gl_symbol(runtime, "glBlendFunc")?,
            blend_equation: load_gl_symbol(runtime, "glBlendEquation")?,
            bind_framebuffer: load_gl_symbol(runtime, "glBindFramebuffer")?,
            // Product rendering treats GL error/FBO checks as part of the
            // libretro shared-context contract. Narrow compatibility symbol
            // groups keep these optional, but the full renderer must not
            // silently disable them.
            get_error: Some(load_gl_symbol(runtime, "glGetError")?),
            check_framebuffer_status: Some(load_gl_symbol(runtime, "glCheckFramebufferStatus")?),
            invalidate_framebuffer: load_optional_gl_symbol(runtime, "glInvalidateFramebuffer")?,
            discard_framebuffer_ext: load_optional_gl_symbol(runtime, "glDiscardFramebufferEXT")?,
        })
    }

    pub fn context_type(&self) -> HwContextType {
        self.context_type
    }

    pub fn version_info(&self) -> GlVersionInfo {
        self.version_info
    }

    pub fn vendor_string(&self) -> &str {
        &self.vendor_string
    }

    pub fn renderer_string(&self) -> &str {
        &self.renderer_string
    }

    pub fn version_string(&self) -> &str {
        &self.version_string
    }

    pub fn extensions_string(&self) -> &str {
        &self.extensions_string
    }

    pub fn max_texture_size(&self) -> Option<u32> {
        self.max_texture_size
    }

    pub fn max_texture_image_units(&self) -> Option<u32> {
        self.max_texture_image_units
    }

    pub fn max_varying_vectors(&self) -> Option<u32> {
        self.max_varying_vectors
    }

    pub fn fragment_highp_float(&self) -> Option<bool> {
        self.fragment_highp_float
    }

    pub fn supports_npot_repeat(&self) -> bool {
        !self.version_info.is_gles || self.version_info.version_at_least(3, 0)
    }

    pub fn supports_discard_framebuffer(&self) -> bool {
        self.discard_framebuffer_ext.is_some()
    }

    pub fn supports_invalidate_framebuffer(&self) -> bool {
        self.invalidate_framebuffer.is_some()
    }

    pub fn has_extension(&self, name: &str) -> bool {
        self.extensions_string
            .split_whitespace()
            .any(|extension| extension == name)
    }

    pub fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        unsafe { (self.clear_color)(r, g, b, a) };
    }

    pub fn clear_color_buffer(&self) {
        unsafe { (self.clear)(GL_COLOR_BUFFER_BIT) };
    }

    pub fn clear_color_depth_buffer(&self) {
        unsafe { (self.clear)(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT) };
    }

    pub fn enable(&self, capability: GlCapability) {
        unsafe { (self.enable)(capability.as_raw()) };
    }

    pub fn disable(&self, capability: GlCapability) {
        unsafe { (self.disable)(capability.as_raw()) };
    }

    pub fn viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe { (self.viewport)(x, y, width, height) };
    }

    pub fn scissor(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe { (self.scissor)(x, y, width, height) };
    }

    pub fn create_shader(&self, shader_type: u32) -> u32 {
        unsafe { (self.create_shader)(shader_type) }
    }

    pub fn compile_shader_source(&self, shader_type: u32, source: &str) -> Result<u32, String> {
        let shader = self.create_shader(shader_type);
        if shader == 0 {
            return Err(format!("glCreateShader({shader_type:#06x}) returned 0"));
        }
        if let Err(error) = self.shader_source(shader, source) {
            self.delete_shader(shader);
            return Err(error);
        }
        self.compile_shader(shader);

        let status = self.get_shader_iv(shader, GL_COMPILE_STATUS);
        if status == 0 {
            let log = self.get_shader_info_log(shader);
            self.delete_shader(shader);
            return Err(format!("shader compile failed: {log}"));
        }

        if let Err(error) = self.check_no_error("glsym::compile_shader_source") {
            self.delete_shader(shader);
            return Err(error);
        }
        Ok(shader)
    }

    pub fn shader_source(&self, shader: u32, source: &str) -> Result<(), String> {
        let source = gl_string(source)?;
        self.shader_source_raw(shader, source.as_c_str());
        Ok(())
    }

    fn shader_source_raw(&self, shader: u32, source: &CStr) {
        let source_ptr = source.as_ptr();
        unsafe { (self.shader_source)(shader, 1, &source_ptr, std::ptr::null()) };
    }

    pub fn compile_shader(&self, shader: u32) {
        unsafe { (self.compile_shader)(shader) };
    }

    pub fn get_shader_iv(&self, shader: u32, pname: u32) -> i32 {
        let mut value = 0;
        self.get_shader_iv_raw(shader, pname, &mut value);
        value
    }

    fn get_shader_iv_raw(&self, shader: u32, pname: u32, params: &mut i32) {
        unsafe { (self.get_shader_iv)(shader, pname, params) };
    }

    pub fn get_shader_info_log(&self, shader: u32) -> String {
        let length = self.get_shader_iv(shader, GL_INFO_LOG_LENGTH);
        if length <= 1 {
            return "no log".to_string();
        }

        let mut buffer = vec![0u8; length as usize];
        let mut written = 0;
        self.get_shader_info_log_raw(
            shader,
            length,
            &mut written,
            buffer.as_mut_ptr().cast::<c_char>(),
        );
        String::from_utf8_lossy(&buffer[..written as usize]).into_owned()
    }

    fn get_shader_info_log_raw(
        &self,
        shader: u32,
        length: i32,
        written: &mut i32,
        buffer: *mut c_char,
    ) {
        unsafe { (self.get_shader_info_log)(shader, length, written, buffer) };
    }

    pub fn delete_shader(&self, shader: u32) {
        unsafe { (self.delete_shader)(shader) };
    }

    pub fn create_program(&self) -> u32 {
        unsafe { (self.create_program)() }
    }

    pub fn build_program(&self, vertex_source: &str, fragment_source: &str) -> Result<u32, String> {
        let vertex_shader = self.compile_shader_source(GL_VERTEX_SHADER, vertex_source)?;
        let fragment_shader = match self.compile_shader_source(GL_FRAGMENT_SHADER, fragment_source)
        {
            Ok(shader) => shader,
            Err(error) => {
                self.delete_shader(vertex_shader);
                return Err(error);
            }
        };

        let program = self.create_program();
        if program == 0 {
            self.delete_shader(vertex_shader);
            self.delete_shader(fragment_shader);
            return Err("glCreateProgram returned 0".to_string());
        }
        self.attach_shader(program, vertex_shader);
        self.attach_shader(program, fragment_shader);
        self.link_program(program);

        let status = self.get_program_iv(program, GL_LINK_STATUS);
        self.delete_shader(vertex_shader);
        self.delete_shader(fragment_shader);

        if status == 0 {
            let log = self.get_program_info_log(program);
            self.delete_program(program);
            return Err(format!("program link failed: {log}"));
        }

        if let Err(error) = self.check_no_error("glsym::build_program") {
            self.delete_program(program);
            return Err(error);
        }
        Ok(program)
    }

    pub fn attach_shader(&self, program: u32, shader: u32) {
        unsafe { (self.attach_shader)(program, shader) };
    }

    pub fn link_program(&self, program: u32) {
        unsafe { (self.link_program)(program) };
    }

    pub fn get_program_iv(&self, program: u32, pname: u32) -> i32 {
        let mut value = 0;
        self.get_program_iv_raw(program, pname, &mut value);
        value
    }

    fn get_program_iv_raw(&self, program: u32, pname: u32, params: &mut i32) {
        unsafe { (self.get_program_iv)(program, pname, params) };
    }

    pub fn get_program_info_log(&self, program: u32) -> String {
        let length = self.get_program_iv(program, GL_INFO_LOG_LENGTH);
        if length <= 1 {
            return "no log".to_string();
        }

        let mut buffer = vec![0u8; length as usize];
        let mut written = 0;
        self.get_program_info_log_raw(
            program,
            length,
            &mut written,
            buffer.as_mut_ptr().cast::<c_char>(),
        );
        String::from_utf8_lossy(&buffer[..written as usize]).into_owned()
    }

    fn get_program_info_log_raw(
        &self,
        program: u32,
        length: i32,
        written: &mut i32,
        buffer: *mut c_char,
    ) {
        unsafe { (self.get_program_info_log)(program, length, written, buffer) };
    }

    pub fn delete_program(&self, program: u32) {
        unsafe { (self.delete_program)(program) };
    }

    pub fn use_program(&self, program: u32) {
        unsafe { (self.use_program)(program) };
    }

    pub fn gen_buffer(&self) -> u32 {
        let mut id = 0;
        unsafe { (self.gen_buffers)(1, &mut id) };
        id
    }

    pub fn bind_buffer(&self, target: GlBufferTarget, buffer: u32) {
        unsafe { (self.bind_buffer)(target.as_raw(), buffer) };
    }

    pub fn buffer_data<T>(&self, target: GlBufferTarget, data: &[T], usage: GlBufferUsage) {
        unsafe {
            self.buffer_data_raw(
                target.as_raw(),
                std::mem::size_of_val(data) as isize,
                data.as_ptr().cast::<c_void>(),
                usage.as_raw(),
            );
        }
    }

    pub fn buffer_data_u8(&self, target: GlBufferTarget, data: &[u8], usage: GlBufferUsage) {
        unsafe {
            self.buffer_data_raw(
                target.as_raw(),
                data.len() as isize,
                data.as_ptr().cast::<c_void>(),
                usage.as_raw(),
            );
        }
    }

    pub fn buffer_data_empty(&self, target: GlBufferTarget, byte_len: usize, usage: GlBufferUsage) {
        unsafe {
            self.buffer_data_raw(
                target.as_raw(),
                byte_len as isize,
                std::ptr::null(),
                usage.as_raw(),
            );
        }
    }

    unsafe fn buffer_data_raw(&self, target: u32, size: isize, data: *const c_void, usage: u32) {
        (self.buffer_data)(target, size, data, usage);
    }

    pub fn delete_buffer(&self, id: u32) {
        unsafe { (self.delete_buffers)(1, &id) };
    }

    pub fn gen_texture(&self) -> u32 {
        let mut id = 0;
        unsafe { (self.gen_textures)(1, &mut id) };
        id
    }

    pub fn bind_texture(&self, target: GlTextureTarget, texture: u32) {
        unsafe { (self.bind_texture)(target.as_raw(), texture) };
    }

    pub fn active_texture(&self, unit_index: u32) {
        unsafe { (self.active_texture)(GL_TEXTURE0 + unit_index) };
    }

    pub fn tex_parameter_i(
        &self,
        target: GlTextureTarget,
        parameter: GlTextureParameter,
        value: i32,
    ) {
        unsafe { (self.tex_parameter_i)(target.as_raw(), parameter.as_raw(), value) };
    }

    pub fn pixel_store_unpack_alignment(&self, alignment: i32) {
        unsafe { (self.pixel_store_i)(GL_UNPACK_ALIGNMENT, alignment) };
    }

    pub fn tex_image_2d_u8(
        &self,
        target: GlTextureTarget,
        internal_format: GlTextureInternalFormat,
        width: i32,
        height: i32,
        format: GlTextureFormat,
        bytes: Option<&[u8]>,
    ) {
        self.tex_image_2d_typed_u8(
            target,
            internal_format,
            width,
            height,
            format,
            GlTextureDataType::UnsignedByte,
            bytes,
        );
    }

    pub fn tex_image_2d_typed_u8(
        &self,
        target: GlTextureTarget,
        internal_format: GlTextureInternalFormat,
        width: i32,
        height: i32,
        format: GlTextureFormat,
        data_type: GlTextureDataType,
        bytes: Option<&[u8]>,
    ) {
        let pixels = bytes
            .map(|bytes| bytes.as_ptr().cast::<c_void>())
            .unwrap_or(std::ptr::null());
        unsafe {
            (self.tex_image_2d)(
                target.as_raw(),
                0,
                internal_format.as_raw() as i32,
                width,
                height,
                0,
                format.as_raw(),
                data_type.as_raw(),
                pixels,
            );
        }
    }

    pub fn supports_texture_arrays(&self) -> bool {
        self.tex_image_3d.is_some()
            && self.tex_sub_image_3d.is_some()
            && supports_core_texture_arrays(self.context_type, self.version_info)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn tex_image_3d_u8(
        &self,
        target: GlTextureTarget,
        internal_format: GlTextureInternalFormat,
        width: i32,
        height: i32,
        depth: i32,
        format: GlTextureFormat,
        bytes: Option<&[u8]>,
    ) -> Result<(), String> {
        let Some(tex_image_3d) = self.tex_image_3d else {
            return Err("texture arrays are not available for this GL context".to_string());
        };
        let pixels = bytes
            .map(|bytes| bytes.as_ptr().cast::<c_void>())
            .unwrap_or(std::ptr::null());
        unsafe {
            tex_image_3d(
                target.as_raw(),
                0,
                internal_format.as_raw() as i32,
                width,
                height,
                depth,
                0,
                format.as_raw(),
                GL_UNSIGNED_BYTE,
                pixels,
            );
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn tex_sub_image_3d_u8(
        &self,
        target: GlTextureTarget,
        zoffset: i32,
        width: i32,
        height: i32,
        depth: i32,
        format: GlTextureFormat,
        bytes: &[u8],
    ) -> Result<(), String> {
        let Some(tex_sub_image_3d) = self.tex_sub_image_3d else {
            return Err("texture arrays are not available for this GL context".to_string());
        };
        unsafe {
            tex_sub_image_3d(
                target.as_raw(),
                0,
                0,
                0,
                zoffset,
                width,
                height,
                depth,
                format.as_raw(),
                GL_UNSIGNED_BYTE,
                bytes.as_ptr().cast::<c_void>(),
            );
        }
        Ok(())
    }

    pub fn supports_generate_mipmap(&self) -> bool {
        self.generate_mipmap.is_some()
    }

    pub fn generate_mipmap(&self, target: GlTextureTarget) -> Result<(), String> {
        let Some(generate_mipmap) = self.generate_mipmap else {
            return Err("glGenerateMipmap is not available for this GL context".to_string());
        };
        unsafe { generate_mipmap(target.as_raw()) };
        Ok(())
    }

    pub fn delete_texture(&self, id: u32) {
        unsafe { (self.delete_textures)(1, &id) };
    }

    pub fn supports_vertex_arrays(&self) -> bool {
        self.gen_vertex_arrays.is_some()
            && self.bind_vertex_array.is_some()
            && self.delete_vertex_arrays.is_some()
    }

    pub fn supports_instancing(&self) -> bool {
        self.vertex_attrib_divisor.is_some()
            && self.draw_elements_instanced.is_some()
            && supports_core_instancing(self.context_type, self.version_info)
    }

    pub fn gen_vertex_array(&self) -> Result<u32, String> {
        let Some(gen_vertex_arrays) = self.gen_vertex_arrays else {
            return Err("vertex arrays are not available for this GL context".to_string());
        };

        let mut id = 0;
        unsafe { gen_vertex_arrays(1, &mut id) };
        if id == 0 {
            return Err("glGenVertexArrays returned 0".to_string());
        }
        Ok(id)
    }

    pub fn bind_vertex_array(&self, array: u32) -> Result<(), String> {
        let Some(bind_vertex_array) = self.bind_vertex_array else {
            return Err("vertex arrays are not available for this GL context".to_string());
        };

        unsafe { bind_vertex_array(array) };
        Ok(())
    }

    pub fn delete_vertex_array(&self, array: u32) -> Result<(), String> {
        let Some(delete_vertex_arrays) = self.delete_vertex_arrays else {
            return Err("vertex arrays are not available for this GL context".to_string());
        };

        unsafe { delete_vertex_arrays(1, &array) };
        Ok(())
    }

    pub fn enable_vertex_attrib_array(&self, index: u32) {
        unsafe { (self.enable_vertex_attrib_array)(index) };
    }

    pub fn disable_vertex_attrib_array(&self, index: u32) {
        unsafe { (self.disable_vertex_attrib_array)(index) };
    }

    pub fn vertex_attrib_pointer_f32(
        &self,
        index: u32,
        size: i32,
        normalized: bool,
        stride: i32,
        offset: usize,
    ) {
        unsafe {
            (self.vertex_attrib_pointer)(
                index,
                size,
                GL_FLOAT,
                if normalized { 1 } else { GL_FALSE },
                stride,
                offset as *const c_void,
            )
        };
    }

    pub fn vertex_attrib_divisor(&self, index: u32, divisor: u32) -> Result<(), String> {
        let Some(vertex_attrib_divisor) = self.vertex_attrib_divisor else {
            return Err("instanced attributes are not available for this GL context".to_string());
        };

        unsafe { vertex_attrib_divisor(index, divisor) };
        Ok(())
    }

    pub fn get_uniform_location(&self, program: u32, name: &str) -> Result<i32, String> {
        let name = gl_string(name)?;
        Ok(self.get_uniform_location_raw(program, name.as_c_str()))
    }

    pub fn required_uniform_location(&self, program: u32, name: &str) -> Result<i32, String> {
        let location = self
            .get_uniform_location(program, name)
            .map_err(|error| format!("failed to query uniform {name}: {error}"))?;
        if location < 0 {
            return Err(format!(
                "shader linked without required active uniform {name}"
            ));
        }
        Ok(location)
    }

    fn get_uniform_location_raw(&self, program: u32, name: &CStr) -> i32 {
        unsafe { (self.get_uniform_location)(program, name.as_ptr()) }
    }

    pub fn get_attrib_location(&self, program: u32, name: &str) -> Result<i32, String> {
        let name = gl_string(name)?;
        Ok(self.get_attrib_location_raw(program, name.as_c_str()))
    }

    pub fn required_attrib_location(&self, program: u32, name: &str) -> Result<u32, String> {
        let location = self
            .get_attrib_location(program, name)
            .map_err(|error| format!("failed to query attribute {name}: {error}"))?;
        if location < 0 {
            return Err(format!(
                "shader linked without required active attribute {name}"
            ));
        }
        Ok(location as u32)
    }

    fn get_attrib_location_raw(&self, program: u32, name: &CStr) -> i32 {
        unsafe { (self.get_attrib_location)(program, name.as_ptr()) }
    }

    pub fn uniform_1f(&self, location: i32, value: f32) {
        unsafe { (self.uniform_1f)(location, value) };
    }

    pub fn uniform_1i(&self, location: i32, value: i32) {
        unsafe { (self.uniform_1i)(location, value) };
    }

    pub fn uniform_2f(&self, location: i32, x: f32, y: f32) {
        unsafe { (self.uniform_2f)(location, x, y) };
    }

    pub fn uniform_4fv(&self, location: i32, values: &[f32; 4]) {
        unsafe { (self.uniform_4fv)(location, 1, values.as_ptr()) };
    }

    pub fn uniform_matrix_4fv(&self, location: i32, transpose: bool, values: &[f32; 16]) {
        unsafe {
            (self.uniform_matrix_4fv)(
                location,
                1,
                if transpose { 1 } else { GL_FALSE },
                values.as_ptr(),
            )
        };
    }

    pub fn draw_arrays(&self, mode: GlDrawMode, first: i32, count: i32) {
        unsafe { (self.draw_arrays)(mode.as_raw(), first, count) };
    }

    pub fn draw_elements(
        &self,
        mode: GlDrawMode,
        count: i32,
        index_type: GlIndexType,
        offset: usize,
    ) {
        unsafe {
            (self.draw_elements)(
                mode.as_raw(),
                count,
                index_type.as_raw(),
                offset as *const c_void,
            )
        };
    }

    pub fn draw_elements_instanced(
        &self,
        mode: GlDrawMode,
        count: i32,
        index_type: GlIndexType,
        offset: usize,
        instance_count: i32,
    ) -> Result<(), String> {
        let Some(draw_elements_instanced) = self.draw_elements_instanced else {
            return Err("instanced draws are not available for this GL context".to_string());
        };
        unsafe {
            draw_elements_instanced(
                mode.as_raw(),
                count,
                index_type.as_raw(),
                offset as *const c_void,
                instance_count,
            );
        }
        Ok(())
    }

    pub fn blend_func(&self, source: GlBlendFactor, destination: GlBlendFactor) {
        unsafe { (self.blend_func)(source.as_raw(), destination.as_raw()) };
    }

    pub fn blend_equation(&self, equation: GlBlendEquation) {
        unsafe { (self.blend_equation)(equation.as_raw()) };
    }

    pub fn bind_framebuffer(&self, target: GlFramebufferTarget, framebuffer: u32) {
        unsafe { (self.bind_framebuffer)(target.as_raw(), framebuffer) };
    }

    pub fn bind_framebuffer_checked(
        &self,
        target: GlFramebufferTarget,
        framebuffer: u32,
    ) -> Result<(), String> {
        self.bind_framebuffer(target, framebuffer);
        self.check_no_error("glBindFramebuffer")?;
        if framebuffer != 0 {
            self.check_bound_framebuffer_complete(target)?;
        }
        Ok(())
    }

    pub fn check_no_error(&self, operation: &str) -> Result<(), String> {
        collect_gl_errors(self.get_error, operation)
    }

    pub fn check_bound_framebuffer_complete(
        &self,
        target: GlFramebufferTarget,
    ) -> Result<(), String> {
        check_bound_framebuffer_complete(
            self.check_framebuffer_status,
            target,
            "glCheckFramebufferStatus",
        )
    }

    pub fn discard_depth_framebuffer_attachment(&self) -> bool {
        let attachments = [GL_DEPTH_ATTACHMENT];
        if let Some(invalidate_framebuffer) = self.invalidate_framebuffer {
            unsafe {
                invalidate_framebuffer(
                    GL_FRAMEBUFFER,
                    attachments.len() as i32,
                    attachments.as_ptr(),
                );
            }
            true
        } else if let Some(discard_framebuffer_ext) = self.discard_framebuffer_ext {
            unsafe {
                discard_framebuffer_ext(
                    GL_FRAMEBUFFER,
                    attachments.len() as i32,
                    attachments.as_ptr(),
                );
            }
            true
        } else {
            false
        }
    }

    #[doc(hidden)]
    pub fn fake_for_testing(config: FakeGlConfig) -> Self {
        let mut state = fake_gl_state()
            .lock()
            .expect("fake GL state mutex poisoned");
        state.configure(config);
        Self {
            context_type: state.config.context_type,
            version_info: state.config.version_info,
            vendor_string: state.config.vendor_string.clone(),
            renderer_string: state.config.renderer_string.clone(),
            version_string: state.config.version_string.clone(),
            extensions_string: state.config.extensions_string.clone(),
            max_texture_size: state.config.max_texture_size,
            max_texture_image_units: state.config.max_texture_image_units,
            max_varying_vectors: state.config.max_varying_vectors,
            fragment_highp_float: state.config.fragment_highp_float,
            clear_color: fake_gl_clear_color,
            clear: fake_gl_clear,
            enable: fake_gl_enable,
            disable: fake_gl_disable,
            viewport: fake_gl_viewport,
            scissor: fake_gl_scissor,
            create_shader: fake_gl_create_shader,
            shader_source: fake_gl_shader_source,
            compile_shader: fake_gl_compile_shader,
            get_shader_iv: fake_gl_get_shader_iv,
            get_shader_info_log: fake_gl_get_shader_info_log,
            delete_shader: fake_gl_delete_shader,
            create_program: fake_gl_create_program,
            attach_shader: fake_gl_attach_shader,
            link_program: fake_gl_link_program,
            get_program_iv: fake_gl_get_program_iv,
            get_program_info_log: fake_gl_get_program_info_log,
            delete_program: fake_gl_delete_program,
            use_program: fake_gl_use_program,
            gen_buffers: fake_gl_gen_buffers,
            bind_buffer: fake_gl_bind_buffer,
            buffer_data: fake_gl_buffer_data,
            delete_buffers: fake_gl_delete_buffers,
            gen_textures: fake_gl_gen_textures,
            bind_texture: fake_gl_bind_texture,
            active_texture: fake_gl_active_texture,
            tex_parameter_i: fake_gl_tex_parameter_i,
            pixel_store_i: fake_gl_pixel_store_i,
            tex_image_2d: fake_gl_tex_image_2d,
            tex_image_3d: state
                .config
                .supports_texture_arrays
                .then_some(fake_gl_tex_image_3d),
            tex_sub_image_3d: state
                .config
                .supports_texture_arrays
                .then_some(fake_gl_tex_sub_image_3d),
            generate_mipmap: state
                .config
                .supports_generate_mipmap
                .then_some(fake_gl_generate_mipmap),
            delete_textures: fake_gl_delete_textures,
            gen_vertex_arrays: state
                .config
                .supports_vertex_arrays
                .then_some(fake_gl_gen_vertex_arrays),
            bind_vertex_array: state
                .config
                .supports_vertex_arrays
                .then_some(fake_gl_bind_vertex_array),
            delete_vertex_arrays: state
                .config
                .supports_vertex_arrays
                .then_some(fake_gl_delete_vertex_arrays),
            enable_vertex_attrib_array: fake_gl_enable_vertex_attrib_array,
            disable_vertex_attrib_array: fake_gl_disable_vertex_attrib_array,
            vertex_attrib_pointer: fake_gl_vertex_attrib_pointer,
            vertex_attrib_divisor: state
                .config
                .supports_instancing
                .then_some(fake_gl_vertex_attrib_divisor),
            get_uniform_location: fake_gl_get_uniform_location,
            get_attrib_location: fake_gl_get_attrib_location,
            uniform_1i: fake_gl_uniform_1i,
            uniform_1f: fake_gl_uniform_1f,
            uniform_2f: fake_gl_uniform_2f,
            uniform_4fv: fake_gl_uniform_4fv,
            uniform_matrix_4fv: fake_gl_uniform_matrix_4fv,
            draw_arrays: fake_gl_draw_arrays,
            draw_elements: fake_gl_draw_elements,
            draw_elements_instanced: state
                .config
                .supports_instancing
                .then_some(fake_gl_draw_elements_instanced),
            blend_func: fake_gl_blend_func,
            blend_equation: fake_gl_blend_equation,
            bind_framebuffer: fake_gl_bind_framebuffer,
            get_error: Some(fake_gl_get_error),
            check_framebuffer_status: Some(fake_gl_check_framebuffer_status),
            invalidate_framebuffer: state
                .config
                .version_info
                .version_at_least(3, 0)
                .then_some(fake_gl_invalidate_framebuffer),
            discard_framebuffer_ext: state
                .config
                .version_info
                .is_gles
                .then_some(fake_gl_discard_framebuffer_ext),
        }
    }

    #[doc(hidden)]
    pub fn reset_fake_state_for_testing() {
        reset_fake_gl_for_testing();
    }

    #[doc(hidden)]
    pub fn snapshot_fake_state_for_testing() -> FakeGlSnapshot {
        snapshot_fake_gl_for_testing()
    }
}

fn load_gl_symbol<T>(runtime: &Runtime<'_>, name: &str) -> Result<T, String>
where
    T: Copy,
{
    let raw = runtime.hw_proc_address(name)?;
    Ok(unsafe { mem::transmute_copy(&raw) })
}

fn load_optional_gl_symbol<T>(runtime: &Runtime<'_>, name: &str) -> Result<Option<T>, String>
where
    T: Copy,
{
    Ok(runtime
        .hw_proc_address(name)
        .ok()
        .map(|raw| unsafe { mem::transmute_copy(&raw) }))
}

fn gl_string(value: &str) -> Result<CString, String> {
    CString::new(value).map_err(|_| format!("GL strings cannot contain interior NULs: {value:?}"))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FakeGlConfig {
    pub context_type: HwContextType,
    pub version_info: GlVersionInfo,
    pub vendor_string: String,
    pub renderer_string: String,
    pub version_string: String,
    pub extensions_string: String,
    pub max_texture_size: Option<u32>,
    pub max_texture_image_units: Option<u32>,
    pub max_varying_vectors: Option<u32>,
    pub fragment_highp_float: Option<bool>,
    pub supports_vertex_arrays: bool,
    pub supports_texture_arrays: bool,
    pub supports_instancing: bool,
    pub supports_generate_mipmap: bool,
    pub next_error: Option<u32>,
    pub framebuffer_status: u32,
}

impl Default for FakeGlConfig {
    fn default() -> Self {
        Self {
            context_type: HwContextType::OpenGlEs2,
            version_info: GlVersionInfo {
                is_gles: true,
                major: Some(2),
                minor: Some(0),
            },
            vendor_string: "FakeVendor".to_string(),
            renderer_string: "FakeRenderer".to_string(),
            version_string: "OpenGL ES 2.0 Fake".to_string(),
            extensions_string: String::new(),
            max_texture_size: Some(1024),
            max_texture_image_units: Some(8),
            max_varying_vectors: Some(8),
            fragment_highp_float: Some(false),
            supports_vertex_arrays: false,
            supports_texture_arrays: false,
            supports_instancing: false,
            supports_generate_mipmap: true,
            next_error: None,
            framebuffer_status: GL_FRAMEBUFFER_COMPLETE,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeTextureParameterCall {
    pub target: u32,
    pub parameter: u32,
    pub value: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeTextureUpload2D {
    pub target: u32,
    pub internal_format: u32,
    pub width: i32,
    pub height: i32,
    pub format: u32,
    pub type_: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FakeGlSnapshot {
    pub clear_calls: usize,
    pub current_clear_color: [u32; 4],
    pub clear_colors: Vec<[u32; 4]>,
    pub clear_scissor_enabled: Vec<bool>,
    pub draw_arrays_calls: usize,
    pub draw_elements_calls: usize,
    pub draw_elements_instanced_calls: usize,
    pub buffer_data_calls: usize,
    pub buffer_data_bytes: usize,
    pub deleted_buffers: Vec<u32>,
    pub deleted_shaders: Vec<u32>,
    pub deleted_programs: Vec<u32>,
    pub texture_parameter_calls: Vec<FakeTextureParameterCall>,
    pub texture_uploads_2d: Vec<FakeTextureUpload2D>,
    pub texture_uploads_3d_calls: usize,
    pub generate_mipmap_calls: Vec<u32>,
    pub deleted_textures: Vec<u32>,
    pub current_program: u32,
    pub use_program_calls: Vec<u32>,
    pub bound_array_buffer: u32,
    pub bound_element_array_buffer: u32,
    pub bound_vertex_array: u32,
    pub vertex_array_bindings: Vec<u32>,
    pub bound_framebuffer: u32,
    pub framebuffer_bindings: Vec<u32>,
    pub framebuffer_invalidations: Vec<Vec<u32>>,
    pub viewport_calls: Vec<(i32, i32, i32, i32)>,
    pub unpack_alignment: i32,
    pub unpack_alignment_calls: Vec<i32>,
    pub active_texture_unit: u32,
    pub bound_texture_2d: u32,
    pub bound_texture_2d_units: Vec<(u32, u32)>,
    pub bound_texture_2d_array: u32,
    pub bound_texture_2d_array_units: Vec<(u32, u32)>,
    pub blend_enabled: bool,
    pub scissor_enabled: bool,
    pub scissor_calls: Vec<(i32, i32, i32, i32)>,
    pub enabled_vertex_attribs: Vec<u32>,
}

struct FakeGlState {
    config: FakeGlConfig,
    vendor_string: CString,
    renderer_string: CString,
    version_string: CString,
    extensions_string: CString,
    next_id: u32,
    snapshot: FakeGlSnapshot,
}

impl Default for FakeGlState {
    fn default() -> Self {
        let config = FakeGlConfig::default();
        let vendor_string =
            CString::new(config.vendor_string.clone()).expect("fake GL vendor string");
        let renderer_string =
            CString::new(config.renderer_string.clone()).expect("fake GL renderer string");
        let version_string =
            CString::new(config.version_string.clone()).expect("fake GL version string");
        let extensions_string =
            CString::new(config.extensions_string.clone()).expect("fake GL extensions string");
        Self {
            config,
            vendor_string,
            renderer_string,
            version_string,
            extensions_string,
            next_id: 1,
            snapshot: FakeGlState::default_snapshot(),
        }
    }
}

impl FakeGlState {
    fn default_snapshot() -> FakeGlSnapshot {
        FakeGlSnapshot {
            unpack_alignment: 4,
            ..FakeGlSnapshot::default()
        }
    }

    fn reset(&mut self) {
        *self = Self::default();
    }

    fn configure(&mut self, config: FakeGlConfig) {
        self.vendor_string =
            CString::new(config.vendor_string.clone()).expect("fake GL vendor string");
        self.renderer_string =
            CString::new(config.renderer_string.clone()).expect("fake GL renderer string");
        self.version_string =
            CString::new(config.version_string.clone()).expect("fake GL version string");
        self.extensions_string =
            CString::new(config.extensions_string.clone()).expect("fake GL extensions string");
        self.config = config;
        self.next_id = 1;
        self.snapshot = Self::default_snapshot();
    }

    fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        id
    }

    fn set_texture_binding(bindings: &mut Vec<(u32, u32)>, unit: u32, texture: u32) {
        if let Some((_, binding)) = bindings
            .iter_mut()
            .find(|(bound_unit, _)| *bound_unit == unit)
        {
            *binding = texture;
        } else {
            bindings.push((unit, texture));
        }
        bindings.sort_unstable_by_key(|(bound_unit, _)| *bound_unit);
    }

    fn set_texture_2d_binding(&mut self, unit: u32, texture: u32) {
        if unit == 0 {
            self.snapshot.bound_texture_2d = texture;
        }
        Self::set_texture_binding(&mut self.snapshot.bound_texture_2d_units, unit, texture);
    }

    fn set_texture_2d_array_binding(&mut self, unit: u32, texture: u32) {
        if unit == 0 {
            self.snapshot.bound_texture_2d_array = texture;
        }
        Self::set_texture_binding(
            &mut self.snapshot.bound_texture_2d_array_units,
            unit,
            texture,
        );
    }
}

fn fake_gl_state() -> &'static Mutex<FakeGlState> {
    static FAKE_GL_STATE: OnceLock<Mutex<FakeGlState>> = OnceLock::new();
    FAKE_GL_STATE.get_or_init(|| Mutex::new(FakeGlState::default()))
}

pub fn configure_fake_gl_for_testing(config: FakeGlConfig) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .configure(config);
}

pub fn reset_fake_gl_for_testing() {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .reset();
}

pub fn snapshot_fake_gl_for_testing() -> FakeGlSnapshot {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .clone()
}

/// Returns fake GL procedure addresses for tests.
///
/// # Safety
///
/// `symbol` must be either null or point to a valid NUL-terminated C string for
/// the duration of the call.
pub unsafe extern "C" fn fake_get_proc_address_for_testing(
    symbol: *const c_char,
) -> Option<unsafe extern "C" fn()> {
    if symbol.is_null() {
        return None;
    }

    let symbol = unsafe { CStr::from_ptr(symbol) }.to_str().ok()?;
    match symbol {
        "glGetString" => Some(mem::transmute::<GlGetString, unsafe extern "C" fn()>(
            fake_gl_get_string,
        )),
        "glGetIntegerv" => Some(mem::transmute::<GlGetIntegerv, unsafe extern "C" fn()>(
            fake_gl_get_integer_v,
        )),
        "glGetShaderPrecisionFormat" => Some(mem::transmute::<
            GlGetShaderPrecisionFormat,
            unsafe extern "C" fn(),
        >(fake_gl_get_shader_precision_format)),
        "glGetError" => Some(mem::transmute::<GlGetError, unsafe extern "C" fn()>(
            fake_gl_get_error,
        )),
        "glCheckFramebufferStatus" => Some(mem::transmute::<
            GlCheckFramebufferStatus,
            unsafe extern "C" fn(),
        >(fake_gl_check_framebuffer_status)),
        "glInvalidateFramebuffer" => {
            let state = fake_gl_state()
                .lock()
                .expect("fake GL state mutex poisoned");
            if state.config.version_info.version_at_least(3, 0) {
                Some(mem::transmute::<
                    GlInvalidateFramebuffer,
                    unsafe extern "C" fn(),
                >(fake_gl_invalidate_framebuffer))
            } else {
                None
            }
        }
        "glDiscardFramebufferEXT" => {
            let state = fake_gl_state()
                .lock()
                .expect("fake GL state mutex poisoned");
            if state.config.version_info.is_gles {
                Some(mem::transmute::<
                    GlInvalidateFramebuffer,
                    unsafe extern "C" fn(),
                >(fake_gl_discard_framebuffer_ext))
            } else {
                None
            }
        }
        "glClearColor" => Some(mem::transmute::<GlClearColor, unsafe extern "C" fn()>(
            fake_gl_clear_color,
        )),
        "glClear" => Some(mem::transmute::<GlClear, unsafe extern "C" fn()>(
            fake_gl_clear,
        )),
        "glEnable" => Some(mem::transmute::<GlEnable, unsafe extern "C" fn()>(
            fake_gl_enable,
        )),
        "glDisable" => Some(mem::transmute::<GlDisable, unsafe extern "C" fn()>(
            fake_gl_disable,
        )),
        "glViewport" => Some(mem::transmute::<GlViewport, unsafe extern "C" fn()>(
            fake_gl_viewport,
        )),
        "glScissor" => Some(mem::transmute::<GlScissor, unsafe extern "C" fn()>(
            fake_gl_scissor,
        )),
        "glCreateShader" => Some(mem::transmute::<GlCreateShader, unsafe extern "C" fn()>(
            fake_gl_create_shader,
        )),
        "glShaderSource" => Some(mem::transmute::<GlShaderSource, unsafe extern "C" fn()>(
            fake_gl_shader_source,
        )),
        "glCompileShader" => Some(mem::transmute::<GlCompileShader, unsafe extern "C" fn()>(
            fake_gl_compile_shader,
        )),
        "glGetShaderiv" => Some(mem::transmute::<GlGetShaderIv, unsafe extern "C" fn()>(
            fake_gl_get_shader_iv,
        )),
        "glGetShaderInfoLog" => Some(
            mem::transmute::<GlGetShaderInfoLog, unsafe extern "C" fn()>(
                fake_gl_get_shader_info_log,
            ),
        ),
        "glDeleteShader" => Some(mem::transmute::<GlDeleteShader, unsafe extern "C" fn()>(
            fake_gl_delete_shader,
        )),
        "glCreateProgram" => Some(mem::transmute::<GlCreateProgram, unsafe extern "C" fn()>(
            fake_gl_create_program,
        )),
        "glAttachShader" => Some(mem::transmute::<GlAttachShader, unsafe extern "C" fn()>(
            fake_gl_attach_shader,
        )),
        "glLinkProgram" => Some(mem::transmute::<GlLinkProgram, unsafe extern "C" fn()>(
            fake_gl_link_program,
        )),
        "glGetProgramiv" => Some(mem::transmute::<GlGetProgramIv, unsafe extern "C" fn()>(
            fake_gl_get_program_iv,
        )),
        "glGetProgramInfoLog" => Some(
            mem::transmute::<GlGetProgramInfoLog, unsafe extern "C" fn()>(
                fake_gl_get_program_info_log,
            ),
        ),
        "glDeleteProgram" => Some(mem::transmute::<GlDeleteProgram, unsafe extern "C" fn()>(
            fake_gl_delete_program,
        )),
        "glUseProgram" => Some(mem::transmute::<GlUseProgram, unsafe extern "C" fn()>(
            fake_gl_use_program,
        )),
        "glGenBuffers" => Some(mem::transmute::<GlGenBuffers, unsafe extern "C" fn()>(
            fake_gl_gen_buffers,
        )),
        "glBindBuffer" => Some(mem::transmute::<GlBindBuffer, unsafe extern "C" fn()>(
            fake_gl_bind_buffer,
        )),
        "glBufferData" => Some(mem::transmute::<GlBufferData, unsafe extern "C" fn()>(
            fake_gl_buffer_data,
        )),
        "glDeleteBuffers" => Some(mem::transmute::<GlDeleteBuffers, unsafe extern "C" fn()>(
            fake_gl_delete_buffers,
        )),
        "glGenTextures" => Some(mem::transmute::<GlGenTextures, unsafe extern "C" fn()>(
            fake_gl_gen_textures,
        )),
        "glBindTexture" => Some(mem::transmute::<GlBindTexture, unsafe extern "C" fn()>(
            fake_gl_bind_texture,
        )),
        "glActiveTexture" => Some(mem::transmute::<GlActiveTexture, unsafe extern "C" fn()>(
            fake_gl_active_texture,
        )),
        "glTexParameteri" => Some(mem::transmute::<GlTexParameteri, unsafe extern "C" fn()>(
            fake_gl_tex_parameter_i,
        )),
        "glPixelStorei" => Some(mem::transmute::<GlPixelStorei, unsafe extern "C" fn()>(
            fake_gl_pixel_store_i,
        )),
        "glTexImage2D" => Some(mem::transmute::<GlTexImage2D, unsafe extern "C" fn()>(
            fake_gl_tex_image_2d,
        )),
        "glTexImage3D" => fake_gl_state()
            .lock()
            .expect("fake GL state mutex poisoned")
            .config
            .supports_texture_arrays
            .then_some(mem::transmute::<GlTexImage3D, unsafe extern "C" fn()>(
                fake_gl_tex_image_3d,
            )),
        "glTexSubImage3D" => fake_gl_state()
            .lock()
            .expect("fake GL state mutex poisoned")
            .config
            .supports_texture_arrays
            .then_some(mem::transmute::<GlTexSubImage3D, unsafe extern "C" fn()>(
                fake_gl_tex_sub_image_3d,
            )),
        "glGenerateMipmap" => fake_gl_state()
            .lock()
            .expect("fake GL state mutex poisoned")
            .config
            .supports_generate_mipmap
            .then_some(mem::transmute::<GlGenerateMipmap, unsafe extern "C" fn()>(
                fake_gl_generate_mipmap,
            )),
        "glDeleteTextures" => Some(mem::transmute::<GlDeleteTextures, unsafe extern "C" fn()>(
            fake_gl_delete_textures,
        )),
        "glGenVertexArrays" => fake_gl_state()
            .lock()
            .expect("fake GL state mutex poisoned")
            .config
            .supports_vertex_arrays
            .then_some(mem::transmute::<GlGenVertexArrays, unsafe extern "C" fn()>(
                fake_gl_gen_vertex_arrays,
            )),
        "glBindVertexArray" => fake_gl_state()
            .lock()
            .expect("fake GL state mutex poisoned")
            .config
            .supports_vertex_arrays
            .then_some(mem::transmute::<GlBindVertexArray, unsafe extern "C" fn()>(
                fake_gl_bind_vertex_array,
            )),
        "glDeleteVertexArrays" => fake_gl_state()
            .lock()
            .expect("fake GL state mutex poisoned")
            .config
            .supports_vertex_arrays
            .then_some(
                mem::transmute::<GlDeleteVertexArrays, unsafe extern "C" fn()>(
                    fake_gl_delete_vertex_arrays,
                ),
            ),
        "glEnableVertexAttribArray" => Some(mem::transmute::<
            GlEnableVertexAttribArray,
            unsafe extern "C" fn(),
        >(fake_gl_enable_vertex_attrib_array)),
        "glDisableVertexAttribArray" => Some(mem::transmute::<
            GlDisableVertexAttribArray,
            unsafe extern "C" fn(),
        >(fake_gl_disable_vertex_attrib_array)),
        "glVertexAttribPointer" => Some(mem::transmute::<
            GlVertexAttribPointer,
            unsafe extern "C" fn(),
        >(fake_gl_vertex_attrib_pointer)),
        "glVertexAttribDivisor" => fake_gl_state()
            .lock()
            .expect("fake GL state mutex poisoned")
            .config
            .supports_instancing
            .then_some(mem::transmute::<
                GlVertexAttribDivisor,
                unsafe extern "C" fn(),
            >(fake_gl_vertex_attrib_divisor)),
        "glGetUniformLocation" => Some(mem::transmute::<
            GlGetUniformLocation,
            unsafe extern "C" fn(),
        >(fake_gl_get_uniform_location)),
        "glGetAttribLocation" => Some(
            mem::transmute::<GlGetAttribLocation, unsafe extern "C" fn()>(
                fake_gl_get_attrib_location,
            ),
        ),
        "glUniform1i" => Some(mem::transmute::<GlUniform1i, unsafe extern "C" fn()>(
            fake_gl_uniform_1i,
        )),
        "glUniform1f" => Some(mem::transmute::<GlUniform1f, unsafe extern "C" fn()>(
            fake_gl_uniform_1f,
        )),
        "glUniform2f" => Some(mem::transmute::<GlUniform2f, unsafe extern "C" fn()>(
            fake_gl_uniform_2f,
        )),
        "glUniform4fv" => Some(mem::transmute::<GlUniform4fv, unsafe extern "C" fn()>(
            fake_gl_uniform_4fv,
        )),
        "glUniformMatrix4fv" => Some(
            mem::transmute::<GlUniformMatrix4fv, unsafe extern "C" fn()>(
                fake_gl_uniform_matrix_4fv,
            ),
        ),
        "glDrawArrays" => Some(mem::transmute::<GlDrawArrays, unsafe extern "C" fn()>(
            fake_gl_draw_arrays,
        )),
        "glDrawElements" => Some(mem::transmute::<GlDrawElements, unsafe extern "C" fn()>(
            fake_gl_draw_elements,
        )),
        "glDrawElementsInstanced" => fake_gl_state()
            .lock()
            .expect("fake GL state mutex poisoned")
            .config
            .supports_instancing
            .then_some(mem::transmute::<
                GlDrawElementsInstanced,
                unsafe extern "C" fn(),
            >(fake_gl_draw_elements_instanced)),
        "glBlendFunc" => Some(mem::transmute::<GlBlendFunc, unsafe extern "C" fn()>(
            fake_gl_blend_func,
        )),
        "glBlendEquation" => Some(mem::transmute::<GlBlendEquationFn, unsafe extern "C" fn()>(
            fake_gl_blend_equation,
        )),
        "glBindFramebuffer" => Some(mem::transmute::<GlBindFramebuffer, unsafe extern "C" fn()>(
            fake_gl_bind_framebuffer,
        )),
        _ => None,
    }
}

unsafe extern "C" fn fake_gl_get_string(name: u32) -> *const u8 {
    let state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    match name {
        GL_VENDOR => state.vendor_string.as_ptr().cast::<u8>(),
        GL_RENDERER => state.renderer_string.as_ptr().cast::<u8>(),
        GL_VERSION => state.version_string.as_ptr().cast::<u8>(),
        GL_EXTENSIONS => state.extensions_string.as_ptr().cast::<u8>(),
        _ => std::ptr::null(),
    }
}

unsafe extern "C" fn fake_gl_get_integer_v(name: u32, value: *mut i32) {
    let state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    let result = match name {
        GL_MAX_TEXTURE_SIZE => state.config.max_texture_size,
        GL_MAX_TEXTURE_IMAGE_UNITS => state.config.max_texture_image_units,
        GL_MAX_VARYING_VECTORS => state.config.max_varying_vectors,
        _ => None,
    }
    .and_then(|value| i32::try_from(value).ok())
    .unwrap_or(0);

    if !value.is_null() {
        unsafe { *value = result };
    }
}

unsafe extern "C" fn fake_gl_get_shader_precision_format(
    _shader_type: u32,
    _precision_type: u32,
    range: *mut i32,
    precision: *mut i32,
) {
    let supported = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .config
        .fragment_highp_float
        .unwrap_or(false);
    if !range.is_null() {
        unsafe {
            *range = if supported { 127 } else { 0 };
            *range.add(1) = if supported { 127 } else { 0 };
        }
    }
    if !precision.is_null() {
        unsafe {
            *precision = if supported { 23 } else { 0 };
        }
    }
}

unsafe extern "C" fn fake_gl_get_error() -> u32 {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    state.config.next_error.take().unwrap_or(GL_NO_ERROR)
}

unsafe extern "C" fn fake_gl_check_framebuffer_status(_target: u32) -> u32 {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .config
        .framebuffer_status
}

unsafe extern "C" fn fake_gl_clear_color(r: f32, g: f32, b: f32, a: f32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .current_clear_color = [r.to_bits(), g.to_bits(), b.to_bits(), a.to_bits()];
}

unsafe extern "C" fn fake_gl_clear(_mask: u32) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    let scissor_enabled = state.snapshot.scissor_enabled;
    let clear_color = state.snapshot.current_clear_color;
    state.snapshot.clear_calls += 1;
    state.snapshot.clear_colors.push(clear_color);
    state.snapshot.clear_scissor_enabled.push(scissor_enabled);
}

unsafe extern "C" fn fake_gl_enable(cap: u32) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    match cap {
        GL_BLEND => state.snapshot.blend_enabled = true,
        GL_SCISSOR_TEST => state.snapshot.scissor_enabled = true,
        _ => {}
    }
}

unsafe extern "C" fn fake_gl_disable(cap: u32) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    match cap {
        GL_BLEND => state.snapshot.blend_enabled = false,
        GL_SCISSOR_TEST => state.snapshot.scissor_enabled = false,
        _ => {}
    }
}
unsafe extern "C" fn fake_gl_viewport(x: i32, y: i32, w: i32, h: i32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .viewport_calls
        .push((x, y, w, h));
}
unsafe extern "C" fn fake_gl_scissor(x: i32, y: i32, w: i32, h: i32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .scissor_calls
        .push((x, y, w, h));
}

unsafe extern "C" fn fake_gl_create_shader(_kind: u32) -> u32 {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .next_id()
}

unsafe extern "C" fn fake_gl_shader_source(
    _shader: u32,
    _count: i32,
    _source: *const *const c_char,
    _length: *const i32,
) {
}

unsafe extern "C" fn fake_gl_compile_shader(_shader: u32) {}

unsafe extern "C" fn fake_gl_get_shader_iv(_shader: u32, pname: u32, params: *mut i32) {
    let value = match pname {
        GL_COMPILE_STATUS => 1,
        GL_INFO_LOG_LENGTH => 0,
        _ => 0,
    };
    if !params.is_null() {
        unsafe { *params = value };
    }
}

unsafe extern "C" fn fake_gl_get_shader_info_log(
    _shader: u32,
    _buf_size: i32,
    length: *mut i32,
    info_log: *mut c_char,
) {
    if !length.is_null() {
        unsafe { *length = 0 };
    }
    if !info_log.is_null() {
        unsafe { *info_log = 0 };
    }
}

unsafe extern "C" fn fake_gl_delete_shader(shader: u32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .deleted_shaders
        .push(shader);
}

unsafe extern "C" fn fake_gl_create_program() -> u32 {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .next_id()
}

unsafe extern "C" fn fake_gl_attach_shader(_program: u32, _shader: u32) {}
unsafe extern "C" fn fake_gl_link_program(_program: u32) {}

unsafe extern "C" fn fake_gl_get_program_iv(_program: u32, pname: u32, params: *mut i32) {
    let value = match pname {
        GL_LINK_STATUS => 1,
        GL_INFO_LOG_LENGTH => 0,
        _ => 0,
    };
    if !params.is_null() {
        unsafe { *params = value };
    }
}

unsafe extern "C" fn fake_gl_get_program_info_log(
    _program: u32,
    _buf_size: i32,
    length: *mut i32,
    info_log: *mut c_char,
) {
    if !length.is_null() {
        unsafe { *length = 0 };
    }
    if !info_log.is_null() {
        unsafe { *info_log = 0 };
    }
}

unsafe extern "C" fn fake_gl_delete_program(program: u32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .deleted_programs
        .push(program);
}
unsafe extern "C" fn fake_gl_use_program(program: u32) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    state.snapshot.current_program = program;
    state.snapshot.use_program_calls.push(program);
}

unsafe extern "C" fn fake_gl_gen_buffers(n: i32, buffers: *mut u32) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    for index in 0..usize::try_from(n.max(0)).unwrap_or(0) {
        unsafe { *buffers.add(index) = state.next_id() };
    }
}

unsafe extern "C" fn fake_gl_bind_buffer(target: u32, buffer: u32) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    match target {
        GL_ARRAY_BUFFER => state.snapshot.bound_array_buffer = buffer,
        GL_ELEMENT_ARRAY_BUFFER => state.snapshot.bound_element_array_buffer = buffer,
        _ => {}
    }
}
unsafe extern "C" fn fake_gl_buffer_data(
    _target: u32,
    size: isize,
    _data: *const c_void,
    _usage: u32,
) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    state.snapshot.buffer_data_calls = state.snapshot.buffer_data_calls.saturating_add(1);
    if let Ok(size) = usize::try_from(size.max(0)) {
        state.snapshot.buffer_data_bytes = state.snapshot.buffer_data_bytes.saturating_add(size);
    }
}

unsafe extern "C" fn fake_gl_delete_buffers(n: i32, buffers: *const u32) {
    if buffers.is_null() {
        return;
    }

    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    for index in 0..usize::try_from(n.max(0)).unwrap_or(0) {
        let buffer = unsafe { *buffers.add(index) };
        state.snapshot.deleted_buffers.push(buffer);
    }
}

unsafe extern "C" fn fake_gl_gen_textures(n: i32, textures: *mut u32) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    for index in 0..usize::try_from(n.max(0)).unwrap_or(0) {
        unsafe { *textures.add(index) = state.next_id() };
    }
}

unsafe extern "C" fn fake_gl_bind_texture(target: u32, texture: u32) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    let unit = state.snapshot.active_texture_unit;
    match target {
        GL_TEXTURE_2D => state.set_texture_2d_binding(unit, texture),
        GL_TEXTURE_2D_ARRAY => state.set_texture_2d_array_binding(unit, texture),
        _ => {}
    }
}
unsafe extern "C" fn fake_gl_active_texture(texture: u32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .active_texture_unit = texture.saturating_sub(GL_TEXTURE0);
}
unsafe extern "C" fn fake_gl_tex_parameter_i(target: u32, parameter: u32, value: i32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .texture_parameter_calls
        .push(FakeTextureParameterCall {
            target,
            parameter,
            value,
        });
}
unsafe extern "C" fn fake_gl_pixel_store_i(parameter: u32, value: i32) {
    if parameter == GL_UNPACK_ALIGNMENT {
        let mut state = fake_gl_state()
            .lock()
            .expect("fake GL state mutex poisoned");
        state.snapshot.unpack_alignment = value;
        state.snapshot.unpack_alignment_calls.push(value);
    }
}

unsafe extern "C" fn fake_gl_tex_image_2d(
    target: u32,
    _level: i32,
    internal_format: i32,
    width: i32,
    height: i32,
    _border: i32,
    format: u32,
    type_: u32,
    _pixels: *const c_void,
) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .texture_uploads_2d
        .push(FakeTextureUpload2D {
            target,
            internal_format: internal_format.max(0) as u32,
            width,
            height,
            format,
            type_,
        });
}

unsafe extern "C" fn fake_gl_tex_image_3d(
    _target: u32,
    _level: i32,
    _internal_format: i32,
    _width: i32,
    _height: i32,
    _depth: i32,
    _border: i32,
    _format: u32,
    _type_: u32,
    _pixels: *const c_void,
) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .texture_uploads_3d_calls += 1;
}

unsafe extern "C" fn fake_gl_tex_sub_image_3d(
    _target: u32,
    _level: i32,
    _xoffset: i32,
    _yoffset: i32,
    _zoffset: i32,
    _width: i32,
    _height: i32,
    _depth: i32,
    _format: u32,
    _type_: u32,
    _pixels: *const c_void,
) {
}

unsafe extern "C" fn fake_gl_generate_mipmap(target: u32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .generate_mipmap_calls
        .push(target);
}

unsafe extern "C" fn fake_gl_delete_textures(n: i32, textures: *const u32) {
    if textures.is_null() {
        return;
    }

    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    for index in 0..usize::try_from(n.max(0)).unwrap_or(0) {
        let texture = unsafe { *textures.add(index) };
        state.snapshot.deleted_textures.push(texture);
    }
}

unsafe extern "C" fn fake_gl_gen_vertex_arrays(n: i32, arrays: *mut u32) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    for index in 0..usize::try_from(n.max(0)).unwrap_or(0) {
        unsafe { *arrays.add(index) = state.next_id() };
    }
}

unsafe extern "C" fn fake_gl_bind_vertex_array(array: u32) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    state.snapshot.bound_vertex_array = array;
    state.snapshot.vertex_array_bindings.push(array);
}
unsafe extern "C" fn fake_gl_delete_vertex_arrays(_n: i32, _arrays: *const u32) {}
unsafe extern "C" fn fake_gl_enable_vertex_attrib_array(index: u32) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    if state.snapshot.bound_vertex_array != 0 {
        return;
    }
    if !state.snapshot.enabled_vertex_attribs.contains(&index) {
        state.snapshot.enabled_vertex_attribs.push(index);
        state.snapshot.enabled_vertex_attribs.sort_unstable();
    }
}

unsafe extern "C" fn fake_gl_disable_vertex_attrib_array(index: u32) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    if state.snapshot.bound_vertex_array != 0 {
        return;
    }
    state
        .snapshot
        .enabled_vertex_attribs
        .retain(|enabled| *enabled != index);
}

unsafe extern "C" fn fake_gl_vertex_attrib_pointer(
    _index: u32,
    _size: i32,
    _type_: u32,
    _normalized: u8,
    _stride: i32,
    _pointer: *const c_void,
) {
}

unsafe extern "C" fn fake_gl_vertex_attrib_divisor(_index: u32, _divisor: u32) {}

fn fake_uniform_location(name: &str) -> i32 {
    match name {
        "projection" => 0,
        "canvas_transform" => 1,
        "u_viewport" => 2,
        "u_font" => 3,
        "u_color" => 4,
        "definitely_missing" => -1,
        _ => 5,
    }
}

fn fake_attribute_location(name: &str) -> i32 {
    match name {
        "a_corner" | "position" | "a_pos" => 0,
        "a_rect" | "uv" | "a_uv" => 1,
        "a_uv_rect" | "color" => 2,
        "a_color" | "sprite_data" => 3,
        "a_clip_rect" => 4,
        "a_sprite_data" => 5,
        _ => -1,
    }
}

unsafe extern "C" fn fake_gl_get_uniform_location(_program: u32, name: *const c_char) -> i32 {
    if name.is_null() {
        return -1;
    }
    let name = unsafe { CStr::from_ptr(name) }.to_str().unwrap_or_default();
    fake_uniform_location(name)
}

unsafe extern "C" fn fake_gl_get_attrib_location(_program: u32, name: *const c_char) -> i32 {
    if name.is_null() {
        return -1;
    }
    let name = unsafe { CStr::from_ptr(name) }.to_str().unwrap_or_default();
    fake_attribute_location(name)
}

unsafe extern "C" fn fake_gl_uniform_1i(_location: i32, _value: i32) {}
unsafe extern "C" fn fake_gl_uniform_1f(_location: i32, _value: f32) {}
unsafe extern "C" fn fake_gl_uniform_2f(_location: i32, _x: f32, _y: f32) {}
unsafe extern "C" fn fake_gl_uniform_4fv(_location: i32, _count: i32, _value: *const f32) {}

unsafe extern "C" fn fake_gl_uniform_matrix_4fv(
    _location: i32,
    _count: i32,
    _transpose: u8,
    _value: *const f32,
) {
}

unsafe extern "C" fn fake_gl_draw_arrays(_mode: u32, _first: i32, _count: i32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .draw_arrays_calls += 1;
}

unsafe extern "C" fn fake_gl_draw_elements(
    _mode: u32,
    _count: i32,
    _type_: u32,
    _indices: *const c_void,
) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .draw_elements_calls += 1;
}

unsafe extern "C" fn fake_gl_draw_elements_instanced(
    _mode: u32,
    _count: i32,
    _type_: u32,
    _indices: *const c_void,
    _instance_count: i32,
) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .draw_elements_instanced_calls += 1;
}

unsafe extern "C" fn fake_gl_blend_func(_src: u32, _dst: u32) {}
unsafe extern "C" fn fake_gl_blend_equation(_mode: u32) {}
unsafe extern "C" fn fake_gl_bind_framebuffer(_target: u32, framebuffer: u32) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    state.snapshot.bound_framebuffer = framebuffer;
    state.snapshot.framebuffer_bindings.push(framebuffer);
}

unsafe extern "C" fn fake_gl_invalidate_framebuffer(
    _target: u32,
    num_attachments: i32,
    attachments: *const u32,
) {
    fake_gl_record_framebuffer_invalidation(num_attachments, attachments);
}

unsafe extern "C" fn fake_gl_discard_framebuffer_ext(
    _target: u32,
    num_attachments: i32,
    attachments: *const u32,
) {
    fake_gl_record_framebuffer_invalidation(num_attachments, attachments);
}

fn fake_gl_record_framebuffer_invalidation(num_attachments: i32, attachments: *const u32) {
    let attachments = if num_attachments <= 0 || attachments.is_null() {
        Vec::new()
    } else {
        // Safety: fake GL callers pass a stack slice that is valid for the
        // duration of the call, matching the GL API contract.
        unsafe { std::slice::from_raw_parts(attachments, num_attachments as usize).to_vec() }
    };
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .framebuffer_invalidations
        .push(attachments);
}

#[cfg(test)]
pub(crate) fn fake_gl_test_guard() -> std::sync::MutexGuard<'static, ()> {
    static FAKE_GL_TEST_GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    FAKE_GL_TEST_GUARD
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("fake GL test guard mutex poisoned")
}

#[cfg(test)]
mod tests {
    use super::{
        CompatGl, FakeGlConfig, GlBufferTarget, GlCapability, GlFramebufferTarget, GlTextureFormat,
        GlTextureInternalFormat, GlTextureTarget, GlVersionInfo, HwContextType, fake_gl_test_guard,
        fallback_gl_version_info, glsym, normalize_positive_gl_limit, parse_gl_version_info,
    };

    #[test]
    fn parses_embedded_and_desktop_gl_version_strings() {
        assert_eq!(
            parse_gl_version_info("OpenGL ES 2.0 Mesa 23.1.0"),
            Some(GlVersionInfo {
                is_gles: true,
                major: Some(2),
                minor: Some(0),
            })
        );
        assert_eq!(
            parse_gl_version_info("4.6 (Core Profile) Mesa 24.0.1"),
            Some(GlVersionInfo {
                is_gles: false,
                major: Some(4),
                minor: Some(6),
            })
        );
    }

    #[test]
    fn falls_back_to_explicit_gles_enums_when_version_query_is_ambiguous() {
        assert_eq!(
            fallback_gl_version_info(HwContextType::OpenGlEs2),
            GlVersionInfo {
                is_gles: true,
                major: Some(2),
                minor: Some(0),
            }
        );
        assert_eq!(
            fallback_gl_version_info(HwContextType::OpenGl),
            GlVersionInfo::default()
        );
    }

    #[test]
    fn ignores_non_positive_gl_limits() {
        assert_eq!(normalize_positive_gl_limit(-1), None);
        assert_eq!(normalize_positive_gl_limit(0), None);
        assert_eq!(normalize_positive_gl_limit(4096), Some(4096));
    }

    #[test]
    fn fake_gl_reports_configured_limits_and_capabilities() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig {
            context_type: HwContextType::OpenGl,
            version_info: GlVersionInfo {
                is_gles: false,
                major: Some(2),
                minor: Some(1),
            },
            version_string: "2.1 Fake".to_string(),
            max_texture_size: Some(2048),
            max_texture_image_units: Some(4),
            max_varying_vectors: Some(6),
            supports_vertex_arrays: true,
            supports_texture_arrays: false,
            supports_instancing: false,
            next_error: None,
            framebuffer_status: super::GL_FRAMEBUFFER_COMPLETE,
            ..FakeGlConfig::default()
        });

        assert_eq!(gl.context_type(), HwContextType::OpenGl);
        assert_eq!(gl.max_texture_size(), Some(2048));
        assert_eq!(gl.max_texture_image_units(), Some(4));
        assert_eq!(gl.max_varying_vectors(), Some(6));
        assert!(gl.supports_vertex_arrays());
        let vertex_array = gl.gen_vertex_array().expect("fake VAO allocation");
        gl.bind_vertex_array(vertex_array).expect("fake VAO bind");
        gl.bind_vertex_array(0).expect("fake VAO unbind");
        gl.clear_color(0.25, 0.5, 0.75, 1.0);
        gl.clear_color_buffer();
        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(snapshot.bound_vertex_array, 0);
        assert_eq!(snapshot.vertex_array_bindings, vec![vertex_array, 0]);
        assert_eq!(
            snapshot.clear_colors,
            vec![[
                0.25f32.to_bits(),
                0.5f32.to_bits(),
                0.75f32.to_bits(),
                1.0f32.to_bits()
            ]]
        );
        assert!(!gl.supports_texture_arrays());
        assert!(!gl.supports_instancing());
        assert!(gl.supports_npot_repeat());
    }

    #[test]
    fn supports_npot_repeat_matches_es2_and_modern_context_classes() {
        let _guard = fake_gl_test_guard();
        let es2 = glsym::fake_for_testing(FakeGlConfig::default());
        assert!(!es2.supports_npot_repeat());

        let es3 = glsym::fake_for_testing(FakeGlConfig {
            version_info: GlVersionInfo {
                is_gles: true,
                major: Some(3),
                minor: Some(0),
            },
            version_string: "OpenGL ES 3.0 Fake".to_string(),
            supports_texture_arrays: true,
            supports_instancing: true,
            ..FakeGlConfig::default()
        });
        assert!(es3.supports_npot_repeat());
    }

    #[test]
    fn modern_feature_helpers_require_context_version_not_only_symbol_presence() {
        let _guard = fake_gl_test_guard();
        let es2_with_global_symbols = glsym::fake_for_testing(FakeGlConfig {
            supports_texture_arrays: true,
            supports_instancing: true,
            ..FakeGlConfig::default()
        });
        assert!(!es2_with_global_symbols.supports_texture_arrays());
        assert!(!es2_with_global_symbols.supports_instancing());

        let desktop_21_with_global_symbols = glsym::fake_for_testing(FakeGlConfig {
            context_type: HwContextType::OpenGl,
            version_info: GlVersionInfo {
                is_gles: false,
                major: Some(2),
                minor: Some(1),
            },
            version_string: "2.1 Fake".to_string(),
            supports_vertex_arrays: true,
            supports_texture_arrays: true,
            supports_instancing: true,
            ..FakeGlConfig::default()
        });
        assert!(!desktop_21_with_global_symbols.supports_texture_arrays());
        assert!(!desktop_21_with_global_symbols.supports_instancing());

        let desktop_33 = glsym::fake_for_testing(FakeGlConfig {
            context_type: HwContextType::OpenGl,
            version_info: GlVersionInfo {
                is_gles: false,
                major: Some(3),
                minor: Some(3),
            },
            version_string: "3.3 Fake".to_string(),
            supports_vertex_arrays: true,
            supports_texture_arrays: true,
            supports_instancing: true,
            ..FakeGlConfig::default()
        });
        assert!(desktop_33.supports_texture_arrays());
        assert!(desktop_33.supports_instancing());
    }

    #[test]
    fn required_locations_reject_inactive_shader_inputs() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig::default());

        assert_eq!(gl.required_attrib_location(1, "position"), Ok(0));
        assert_eq!(gl.required_uniform_location(1, "projection"), Ok(0));
        assert!(
            gl.required_attrib_location(1, "definitely_missing")
                .unwrap_err()
                .contains("required active attribute")
        );
        assert!(
            gl.required_uniform_location(1, "definitely_missing")
                .unwrap_err()
                .contains("required active uniform")
        );
        gl.uniform_2f(0, 320.0, 240.0);
    }

    #[test]
    fn checked_gl_calls_report_errors_and_incomplete_framebuffers() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig {
            next_error: Some(super::GL_INVALID_OPERATION),
            ..FakeGlConfig::default()
        });
        let error = gl.check_no_error("unit test draw").unwrap_err();
        assert!(error.contains("unit test draw"));
        assert!(error.contains("GL_INVALID_OPERATION"));

        let gl = glsym::fake_for_testing(FakeGlConfig {
            framebuffer_status: super::GL_FRAMEBUFFER_INCOMPLETE_ATTACHMENT,
            ..FakeGlConfig::default()
        });
        let error = gl
            .bind_framebuffer_checked(GlFramebufferTarget::Framebuffer, 7)
            .unwrap_err();
        assert!(error.contains("incomplete framebuffer"));
        assert!(error.contains("GL_FRAMEBUFFER_INCOMPLETE_ATTACHMENT"));
    }

    #[test]
    fn shader_source_errors_delete_unowned_shader_objects() {
        let _guard = fake_gl_test_guard();
        let compat = CompatGl::fake_for_testing(FakeGlConfig::default());

        let error = compat
            .compile_shader_source(super::GL_VERTEX_SHADER, "void\0main")
            .unwrap_err();

        assert!(error.contains("interior NUL"));
        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(snapshot.deleted_shaders, vec![1]);

        let gl = glsym::fake_for_testing(FakeGlConfig::default());
        let error = gl
            .compile_shader_source(super::GL_FRAGMENT_SHADER, "void\0main")
            .unwrap_err();

        assert!(error.contains("interior NUL"));
        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(snapshot.deleted_shaders, vec![1]);
    }

    #[test]
    fn fake_gl_snapshot_tracks_texture_upload_formats() {
        let _guard = fake_gl_test_guard();
        let _gl = glsym::fake_for_testing(FakeGlConfig::default());
        unsafe {
            super::fake_gl_tex_image_2d(
                super::GlTextureTarget::Texture2D.as_raw(),
                0,
                GlTextureInternalFormat::Luminance.as_raw() as i32,
                32,
                16,
                0,
                GlTextureFormat::Luminance.as_raw(),
                super::GL_UNSIGNED_BYTE,
                std::ptr::null(),
            );
        }

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(snapshot.texture_uploads_2d.len(), 1);
        assert_eq!(
            snapshot.texture_uploads_2d[0].internal_format,
            GlTextureInternalFormat::Luminance.as_raw()
        );
        assert_eq!(
            snapshot.texture_uploads_2d[0].format,
            GlTextureFormat::Luminance.as_raw()
        );
        assert_eq!(snapshot.texture_uploads_2d[0].width, 32);
        assert_eq!(snapshot.texture_uploads_2d[0].height, 16);
    }

    #[test]
    fn fake_gl_snapshot_tracks_shared_context_state() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig::default());

        gl.enable(GlCapability::Blend);
        gl.enable(GlCapability::ScissorTest);
        gl.bind_buffer(GlBufferTarget::ArrayBuffer, 11);
        gl.bind_buffer(GlBufferTarget::ElementArrayBuffer, 12);
        gl.bind_framebuffer(GlFramebufferTarget::Framebuffer, 13);
        gl.viewport(1, 2, 320, 240);
        gl.scissor(3, 4, 160, 120);
        gl.pixel_store_unpack_alignment(1);
        gl.active_texture(1);
        gl.bind_texture(GlTextureTarget::Texture2D, 21);
        gl.bind_texture(GlTextureTarget::Texture2DArray, 31);
        gl.active_texture(0);
        gl.bind_texture(GlTextureTarget::Texture2D, 20);
        gl.bind_texture(GlTextureTarget::Texture2DArray, 30);
        gl.clear_color_buffer();

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(snapshot.clear_scissor_enabled, vec![true]);
        assert!(snapshot.blend_enabled);
        assert!(snapshot.scissor_enabled);
        assert_eq!(snapshot.bound_array_buffer, 11);
        assert_eq!(snapshot.bound_element_array_buffer, 12);
        assert_eq!(snapshot.bound_framebuffer, 13);
        assert_eq!(snapshot.framebuffer_bindings, vec![13]);
        assert_eq!(snapshot.viewport_calls, vec![(1, 2, 320, 240)]);
        assert_eq!(snapshot.scissor_calls, vec![(3, 4, 160, 120)]);
        assert_eq!(snapshot.unpack_alignment, 1);
        assert_eq!(snapshot.unpack_alignment_calls, vec![1]);
        assert_eq!(snapshot.active_texture_unit, 0);
        assert_eq!(snapshot.bound_texture_2d, 20);
        assert_eq!(snapshot.bound_texture_2d_units, vec![(0, 20), (1, 21)]);
        assert_eq!(snapshot.bound_texture_2d_array, 30);
        assert_eq!(
            snapshot.bound_texture_2d_array_units,
            vec![(0, 30), (1, 31)]
        );

        gl.disable(GlCapability::Blend);
        gl.disable(GlCapability::ScissorTest);
        gl.bind_buffer(GlBufferTarget::ArrayBuffer, 0);
        gl.bind_buffer(GlBufferTarget::ElementArrayBuffer, 0);
        gl.bind_framebuffer(GlFramebufferTarget::Framebuffer, 0);
        gl.pixel_store_unpack_alignment(4);
        gl.active_texture(1);
        gl.bind_texture(GlTextureTarget::Texture2D, 0);
        gl.bind_texture(GlTextureTarget::Texture2DArray, 0);
        gl.active_texture(0);
        gl.bind_texture(GlTextureTarget::Texture2D, 0);
        gl.bind_texture(GlTextureTarget::Texture2DArray, 0);
        gl.clear_color_buffer();

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(snapshot.clear_scissor_enabled, vec![true, false]);
        assert!(!snapshot.blend_enabled);
        assert!(!snapshot.scissor_enabled);
        assert_eq!(snapshot.bound_array_buffer, 0);
        assert_eq!(snapshot.bound_element_array_buffer, 0);
        assert_eq!(snapshot.bound_framebuffer, 0);
        assert_eq!(snapshot.framebuffer_bindings, vec![13, 0]);
        assert_eq!(snapshot.unpack_alignment, 4);
        assert_eq!(snapshot.unpack_alignment_calls, vec![1, 4]);
        assert_eq!(snapshot.active_texture_unit, 0);
        assert_eq!(snapshot.bound_texture_2d, 0);
        assert_eq!(snapshot.bound_texture_2d_units, vec![(0, 0), (1, 0)]);
        assert_eq!(snapshot.bound_texture_2d_array, 0);
        assert_eq!(snapshot.bound_texture_2d_array_units, vec![(0, 0), (1, 0)]);
    }
}
