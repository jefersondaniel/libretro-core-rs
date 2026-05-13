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

use enumflags2::{BitFlags, bitflags};

use crate::{HwContextType, Runtime};

const GL_FALSE: u8 = 0;
const GL_NONE: u32 = 0;
const GL_COLOR_BUFFER_BIT: u32 = 0x0000_4000;
const GL_DEPTH_BUFFER_BIT: u32 = 0x0000_0100;
const GL_STENCIL_BUFFER_BIT: u32 = 0x0000_0400;
const GL_SCISSOR_TEST: u32 = 0x0C11;
const GL_BLEND: u32 = 0x0BE2;
const GL_DEPTH_TEST: u32 = 0x0B71;
const GL_STENCIL_TEST: u32 = 0x0B90;
const GL_CULL_FACE: u32 = 0x0B44;
const GL_MULTISAMPLE: u32 = 0x809D;
const GL_DITHER: u32 = 0x0BD0;
const GL_POLYGON_OFFSET_FILL: u32 = 0x8037;
const GL_PRIMITIVE_RESTART_FIXED_INDEX: u32 = 0x8D69;
const GL_COMPILE_STATUS: u32 = 0x8B81;
const GL_LINK_STATUS: u32 = 0x8B82;
const GL_INFO_LOG_LENGTH: u32 = 0x8B84;
const GL_VENDOR: u32 = 0x1F00;
const GL_RENDERER: u32 = 0x1F01;
const GL_VERSION: u32 = 0x1F02;
const GL_EXTENSIONS: u32 = 0x1F03;
const GL_NUM_EXTENSIONS: u32 = 0x821D;
const GL_MAX_TEXTURE_SIZE: u32 = 0x0D33;
const GL_MAX_TEXTURE_IMAGE_UNITS: u32 = 0x8872;
const GL_MAX_VARYING_VECTORS: u32 = 0x8DFC;
const GL_FLOAT: u32 = 0x1406;
const GL_TRIANGLES: u32 = 0x0004;
const GL_ARRAY_BUFFER: u32 = 0x8892;
const GL_ELEMENT_ARRAY_BUFFER: u32 = 0x8893;
const GL_COPY_READ_BUFFER: u32 = 0x8F36;
const GL_COPY_WRITE_BUFFER: u32 = 0x8F37;
const GL_UNIFORM_BUFFER: u32 = 0x8A11;
const GL_TRANSFORM_FEEDBACK_BUFFER: u32 = 0x8C8E;
const GL_STATIC_DRAW: u32 = 0x88E4;
const GL_STREAM_DRAW: u32 = 0x88E0;
const GL_DYNAMIC_DRAW: u32 = 0x88E8;
const GL_FRAMEBUFFER: u32 = 0x8D40;
const GL_RENDERBUFFER: u32 = 0x8D41;
const GL_COLOR_ATTACHMENT0: u32 = 0x8CE0;
const GL_DEPTH_ATTACHMENT: u32 = 0x8D00;
const GL_STENCIL_ATTACHMENT: u32 = 0x8D20;
const GL_DEPTH_STENCIL_ATTACHMENT: u32 = 0x821A;
const GL_DEPTH_COMPONENT16: u32 = 0x81A5;
const GL_STENCIL_INDEX8: u32 = 0x8D48;
const GL_RGBA4: u32 = 0x8056;
const GL_RGB565: u32 = 0x8D62;
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
const GL_NEAREST_MIPMAP_LINEAR: u32 = 0x2702;
const GL_LINEAR_MIPMAP_LINEAR: u32 = 0x2703;
const GL_RGB: u32 = 0x1907;
const GL_RED: u32 = 0x1903;
const GL_LUMINANCE: u32 = 0x1909;
const GL_RGBA: u32 = 0x1908;
const GL_UNSIGNED_BYTE: u32 = 0x1401;
const GL_UNSIGNED_SHORT: u32 = 0x1403;
const GL_UNSIGNED_SHORT_4_4_4_4: u32 = 0x8033;
const GL_UNSIGNED_SHORT_5_6_5: u32 = 0x8363;
const GL_PACK_ALIGNMENT: u32 = 0x0D05;
const GL_UNPACK_ALIGNMENT: u32 = 0x0CF5;
const GL_ONE: u32 = 1;
const GL_NEVER: u32 = 0x0200;
const GL_LESS: u32 = 0x0201;
const GL_EQUAL: u32 = 0x0202;
const GL_LEQUAL: u32 = 0x0203;
const GL_GREATER: u32 = 0x0204;
const GL_NOTEQUAL: u32 = 0x0205;
const GL_GEQUAL: u32 = 0x0206;
const GL_ALWAYS: u32 = 0x0207;
const GL_FRONT: u32 = 0x0404;
const GL_BACK: u32 = 0x0405;
const GL_FRONT_AND_BACK: u32 = 0x0408;
const GL_CW: u32 = 0x0900;
const GL_CCW: u32 = 0x0901;
const GL_KEEP: u32 = 0x1E00;
const GL_REPLACE: u32 = 0x1E01;
const GL_INCR: u32 = 0x1E02;
const GL_DECR: u32 = 0x1E03;
const GL_INVERT: u32 = 0x150A;
const GL_INCR_WRAP: u32 = 0x8507;
const GL_DECR_WRAP: u32 = 0x8508;
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
const GL_SAMPLES_PASSED: u32 = 0x8914;
const GL_ANY_SAMPLES_PASSED: u32 = 0x8C2F;
const GL_ANY_SAMPLES_PASSED_CONSERVATIVE: u32 = 0x8D6A;
const GL_PRIMITIVES_GENERATED: u32 = 0x8C87;
const GL_QUERY_RESULT: u32 = 0x8866;
const GL_QUERY_RESULT_AVAILABLE: u32 = 0x8867;
const GL_SYNC_GPU_COMMANDS_COMPLETE: u32 = 0x9117;
const GL_SYNC_FLUSH_COMMANDS_BIT: u32 = 0x0000_0001;
const GL_ALREADY_SIGNALED: u32 = 0x911A;
const GL_TIMEOUT_EXPIRED: u32 = 0x911B;
const GL_CONDITION_SATISFIED: u32 = 0x911C;
const GL_WAIT_FAILED: u32 = 0x911D;
const GL_TIMEOUT_IGNORED: u64 = u64::MAX;

type GlClearColor = unsafe extern "C" fn(f32, f32, f32, f32);
type GlClear = unsafe extern "C" fn(u32);
type GlEnable = unsafe extern "C" fn(u32);
type GlDisable = unsafe extern "C" fn(u32);
type GlDepthFunc = unsafe extern "C" fn(u32);
type GlDepthMask = unsafe extern "C" fn(u8);
type GlDepthRangef = unsafe extern "C" fn(f32, f32);
type GlCullFace = unsafe extern "C" fn(u32);
type GlFrontFace = unsafe extern "C" fn(u32);
type GlStencilFunc = unsafe extern "C" fn(u32, i32, u32);
type GlStencilMaskFn = unsafe extern "C" fn(u32);
type GlStencilOp = unsafe extern "C" fn(u32, u32, u32);
type GlStencilFuncSeparate = unsafe extern "C" fn(u32, u32, i32, u32);
type GlStencilMaskSeparate = unsafe extern "C" fn(u32, u32);
type GlStencilOpSeparate = unsafe extern "C" fn(u32, u32, u32, u32);
type GlColorMaskFn = unsafe extern "C" fn(u8, u8, u8, u8);
type GlPolygonOffsetFn = unsafe extern "C" fn(f32, f32);
type GlGenQueries = unsafe extern "C" fn(i32, *mut u32);
type GlDeleteQueries = unsafe extern "C" fn(i32, *const u32);
type GlBeginQuery = unsafe extern "C" fn(u32, u32);
type GlEndQuery = unsafe extern "C" fn(u32);
type GlGetQueryObjectuiv = unsafe extern "C" fn(u32, u32, *mut u32);
type GlFenceSync = unsafe extern "C" fn(u32, u32) -> *const c_void;
type GlClientWaitSync = unsafe extern "C" fn(*const c_void, u32, u64) -> u32;
type GlWaitSync = unsafe extern "C" fn(*const c_void, u32, u64);
type GlDeleteSync = unsafe extern "C" fn(*const c_void);
type GlReadPixels = unsafe extern "C" fn(i32, i32, i32, i32, u32, u32, *mut c_void);
type GlReadBuffer = unsafe extern "C" fn(u32);
type GlDrawBuffers = unsafe extern "C" fn(i32, *const u32);
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
type GlBindBufferBase = unsafe extern "C" fn(u32, u32, u32);
type GlBindBufferRange = unsafe extern "C" fn(u32, u32, u32, isize, isize);
type GlBufferData = unsafe extern "C" fn(u32, isize, *const c_void, u32);
type GlBufferSubData = unsafe extern "C" fn(u32, isize, isize, *const c_void);
type GlCopyBufferSubData = unsafe extern "C" fn(u32, u32, isize, isize, isize);
type GlDeleteBuffers = unsafe extern "C" fn(i32, *const u32);
type GlGenTextures = unsafe extern "C" fn(i32, *mut u32);
type GlBindTexture = unsafe extern "C" fn(u32, u32);
type GlActiveTexture = unsafe extern "C" fn(u32);
type GlTexParameteri = unsafe extern "C" fn(u32, u32, i32);
type GlPixelStorei = unsafe extern "C" fn(u32, i32);
type GlTexImage2D = unsafe extern "C" fn(u32, i32, i32, i32, i32, i32, u32, u32, *const c_void);
type GlTexSubImage2D = unsafe extern "C" fn(u32, i32, i32, i32, i32, i32, u32, u32, *const c_void);
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
type GlVertexAttribDivisorFn = unsafe extern "C" fn(u32, u32);
type GlGetUniformLocation = unsafe extern "C" fn(u32, *const c_char) -> i32;
type GlGetAttribLocation = unsafe extern "C" fn(u32, *const c_char) -> i32;
type GlBindAttribLocation = unsafe extern "C" fn(u32, u32, *const c_char);
type GlUniform1i = unsafe extern "C" fn(i32, i32);
type GlUniform1f = unsafe extern "C" fn(i32, f32);
type GlUniform2f = unsafe extern "C" fn(i32, f32, f32);
type GlUniform3f = unsafe extern "C" fn(i32, f32, f32, f32);
type GlUniform4f = unsafe extern "C" fn(i32, f32, f32, f32, f32);
type GlUniform4fv = unsafe extern "C" fn(i32, i32, *const f32);
type GlUniformMatrix3fv = unsafe extern "C" fn(i32, i32, u8, *const f32);
type GlUniformMatrix4fv = unsafe extern "C" fn(i32, i32, u8, *const f32);
type GlDrawArrays = unsafe extern "C" fn(u32, i32, i32);
type GlDrawArraysInstanced = unsafe extern "C" fn(u32, i32, i32, i32);
type GlDrawElements = unsafe extern "C" fn(u32, i32, u32, *const c_void);
type GlDrawRangeElements = unsafe extern "C" fn(u32, u32, u32, i32, u32, *const c_void);
type GlDrawElementsInstanced = unsafe extern "C" fn(u32, i32, u32, *const c_void, i32);
type GlBlendColor = unsafe extern "C" fn(f32, f32, f32, f32);
type GlBlendFunc = unsafe extern "C" fn(u32, u32);
type GlBlendFuncSeparate = unsafe extern "C" fn(u32, u32, u32, u32);
type GlBlendEquationFn = unsafe extern "C" fn(u32);
type GlBlendEquationSeparate = unsafe extern "C" fn(u32, u32);
type GlGenFramebuffers = unsafe extern "C" fn(i32, *mut u32);
type GlBindFramebuffer = unsafe extern "C" fn(u32, u32);
type GlDeleteFramebuffers = unsafe extern "C" fn(i32, *const u32);
type GlFramebufferTexture2D = unsafe extern "C" fn(u32, u32, u32, u32, i32);
type GlGenRenderbuffers = unsafe extern "C" fn(i32, *mut u32);
type GlBindRenderbuffer = unsafe extern "C" fn(u32, u32);
type GlRenderbufferStorage = unsafe extern "C" fn(u32, u32, i32, i32);
type GlDeleteRenderbuffers = unsafe extern "C" fn(i32, *const u32);
type GlFramebufferRenderbuffer = unsafe extern "C" fn(u32, u32, u32, u32);
type GlBlitFramebuffer = unsafe extern "C" fn(i32, i32, i32, i32, i32, i32, i32, i32, u32, u32);
type GlGetString = unsafe extern "C" fn(u32) -> *const u8;
type GlGetStringi = unsafe extern "C" fn(u32, u32) -> *const u8;
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
    CopyReadBuffer,
    CopyWriteBuffer,
}

impl GlBufferTarget {
    fn as_raw(self) -> u32 {
        match self {
            Self::ArrayBuffer => GL_ARRAY_BUFFER,
            Self::ElementArrayBuffer => GL_ELEMENT_ARRAY_BUFFER,
            Self::CopyReadBuffer => GL_COPY_READ_BUFFER,
            Self::CopyWriteBuffer => GL_COPY_WRITE_BUFFER,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlIndexedBufferTarget {
    UniformBuffer,
    TransformFeedbackBuffer,
}

impl GlIndexedBufferTarget {
    fn as_raw(self) -> u32 {
        match self {
            Self::UniformBuffer => GL_UNIFORM_BUFFER,
            Self::TransformFeedbackBuffer => GL_TRANSFORM_FEEDBACK_BUFFER,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlBufferBindingIndex(u32);

impl GlBufferBindingIndex {
    pub const ZERO: Self = Self(0);

    pub const fn from_index(index: u32) -> Self {
        Self(index)
    }

    pub const fn as_raw(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlBufferByteOffset(usize);

impl GlBufferByteOffset {
    pub const fn from_bytes(bytes: usize) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(self) -> usize {
        self.0
    }

    fn as_isize(self) -> Result<isize, String> {
        isize::try_from(self.0)
            .map_err(|_| format!("GL buffer byte offset {} exceeds isize::MAX", self.0))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlBufferByteSize(usize);

impl GlBufferByteSize {
    pub const fn from_bytes(bytes: usize) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(self) -> usize {
        self.0
    }

    fn as_isize(self, operation: &str) -> Result<isize, String> {
        isize::try_from(self.0)
            .map_err(|_| format!("{operation} byte length {} exceeds isize::MAX", self.0))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlBufferRange {
    pub offset: GlBufferByteOffset,
    pub size: GlBufferByteSize,
}

impl GlBufferRange {
    pub const fn from_start(size: GlBufferByteSize) -> Self {
        Self {
            offset: GlBufferByteOffset::from_bytes(0),
            size,
        }
    }

    pub const fn new(offset: GlBufferByteOffset, size: GlBufferByteSize) -> Self {
        Self { offset, size }
    }

    fn as_gl_args(self, operation: &str) -> Result<(isize, isize), String> {
        Ok((self.offset.as_isize()?, self.size.as_isize(operation)?))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlBuffer(u32);

impl GlBuffer {
    pub fn as_raw(self) -> u32 {
        self.0
    }

    fn from_nonzero(id: u32) -> Result<Self, String> {
        if id == 0 {
            Err("glGenBuffers returned 0".to_string())
        } else {
            Ok(Self(id))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlUniformLocation(i32);

impl GlUniformLocation {
    pub fn as_raw(self) -> i32 {
        self.0
    }

    fn from_raw(raw: i32) -> Option<Self> {
        (raw >= 0).then_some(Self(raw))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlShaderStage {
    Vertex,
    Fragment,
}

impl GlShaderStage {
    pub fn as_raw(self) -> u32 {
        match self {
            Self::Vertex => GL_VERTEX_SHADER,
            Self::Fragment => GL_FRAGMENT_SHADER,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlShader(u32);

impl GlShader {
    pub fn as_raw(self) -> u32 {
        self.0
    }

    fn from_nonzero(id: u32, stage: GlShaderStage) -> Result<Self, String> {
        if id == 0 {
            Err(format!("glCreateShader({stage:?}) returned 0"))
        } else {
            Ok(Self(id))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlProgram(u32);

impl GlProgram {
    pub fn as_raw(self) -> u32 {
        self.0
    }

    fn from_nonzero(id: u32) -> Result<Self, String> {
        if id == 0 {
            Err("glCreateProgram returned 0".to_string())
        } else {
            Ok(Self(id))
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlVertexAttribLocation(u32);

impl GlVertexAttribLocation {
    pub const ZERO: Self = Self(0);

    pub const fn from_index(index: u32) -> Self {
        Self(index)
    }

    pub fn as_raw(self) -> u32 {
        self.0
    }

    fn from_raw(raw: i32) -> Option<Self> {
        (raw >= 0).then_some(Self(raw as u32))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlVertexAttribDivisor(u32);

impl GlVertexAttribDivisor {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);

    pub const fn new(divisor: u32) -> Self {
        Self(divisor)
    }

    fn as_raw(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlVertexAttribF32Components {
    One,
    Two,
    Three,
    Four,
}

impl GlVertexAttribF32Components {
    pub const fn as_count(self) -> usize {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
        }
    }

    const fn as_gl_size(self) -> i32 {
        self.as_count() as i32
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GlVertexAttribByteOffset(usize);

impl GlVertexAttribByteOffset {
    pub const ZERO: Self = Self(0);

    pub const fn from_bytes(bytes: usize) -> Self {
        Self(bytes)
    }

    pub fn from_f32_count(count: usize) -> Result<Self, String> {
        count
            .checked_mul(mem::size_of::<f32>())
            .map(Self)
            .ok_or_else(|| format!("vertex attribute f32 offset {count} overflows usize"))
    }

    const fn as_bytes(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GlVertexAttribStride(i32);

impl GlVertexAttribStride {
    pub const TIGHTLY_PACKED: Self = Self(0);

    pub fn from_bytes(bytes: usize) -> Result<Self, String> {
        i32::try_from(bytes)
            .map(Self)
            .map_err(|_| format!("vertex attribute stride {bytes} exceeds i32::MAX"))
    }

    pub fn from_f32_count(count: usize) -> Result<Self, String> {
        let bytes = count
            .checked_mul(mem::size_of::<f32>())
            .ok_or_else(|| format!("vertex attribute f32 stride {count} overflows usize"))?;
        Self::from_bytes(bytes)
    }

    const fn as_i32(self) -> i32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlVertexAttribF32Layout {
    components: GlVertexAttribF32Components,
    normalized: bool,
    stride: GlVertexAttribStride,
    offset: GlVertexAttribByteOffset,
}

impl GlVertexAttribF32Layout {
    pub const fn tightly_packed(components: GlVertexAttribF32Components) -> Self {
        Self {
            components,
            normalized: false,
            stride: GlVertexAttribStride::TIGHTLY_PACKED,
            offset: GlVertexAttribByteOffset::ZERO,
        }
    }

    pub fn interleaved(
        components: GlVertexAttribF32Components,
        stride_f32_count: usize,
    ) -> Result<Self, String> {
        Ok(Self {
            components,
            normalized: false,
            stride: GlVertexAttribStride::from_f32_count(stride_f32_count)?,
            offset: GlVertexAttribByteOffset::ZERO,
        })
    }

    pub const fn normalized(mut self, normalized: bool) -> Self {
        self.normalized = normalized;
        self
    }

    pub fn with_offset_f32_count(mut self, offset_f32_count: usize) -> Result<Self, String> {
        self.offset = GlVertexAttribByteOffset::from_f32_count(offset_f32_count)?;
        Ok(self)
    }

    pub const fn with_offset_components(mut self, components: GlVertexAttribF32Components) -> Self {
        self.offset = GlVertexAttribByteOffset(components.as_count() * mem::size_of::<f32>());
        self
    }

    pub const fn with_offset(mut self, offset: GlVertexAttribByteOffset) -> Self {
        self.offset = offset;
        self
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlTexture(u32);

impl GlTexture {
    pub fn as_raw(self) -> u32 {
        self.0
    }

    fn from_nonzero(id: u32) -> Result<Self, String> {
        if id == 0 {
            Err("glGenTextures returned 0".to_string())
        } else {
            Ok(Self(id))
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlTextureMinFilter {
    Nearest,
    Linear,
    NearestMipmapNearest,
    LinearMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapLinear,
}

impl GlTextureMinFilter {
    pub fn as_raw(self) -> u32 {
        match self {
            Self::Nearest => GL_NEAREST,
            Self::Linear => GL_LINEAR,
            Self::NearestMipmapNearest => GL_NEAREST_MIPMAP_NEAREST,
            Self::LinearMipmapNearest => GL_LINEAR_MIPMAP_NEAREST,
            Self::NearestMipmapLinear => GL_NEAREST_MIPMAP_LINEAR,
            Self::LinearMipmapLinear => GL_LINEAR_MIPMAP_LINEAR,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlTextureMagFilter {
    Nearest,
    Linear,
}

impl GlTextureMagFilter {
    pub fn as_raw(self) -> u32 {
        match self {
            Self::Nearest => GL_NEAREST,
            Self::Linear => GL_LINEAR,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlPixelStoreAlignment {
    One,
    Two,
    Four,
    Eight,
}

impl GlPixelStoreAlignment {
    pub fn as_raw(self) -> i32 {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Four => 4,
            Self::Eight => 8,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlTextureUnit(u32);

impl GlTextureUnit {
    pub const ZERO: Self = Self(0);

    pub const fn from_index(index: u32) -> Self {
        Self(index)
    }

    pub const fn index(self) -> u32 {
        self.0
    }

    fn as_raw(self) -> Result<u32, String> {
        GL_TEXTURE0
            .checked_add(self.0)
            .ok_or_else(|| format!("texture unit index {} overflows GLenum", self.0))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlTextureLevel(u32);

impl GlTextureLevel {
    pub const ZERO: Self = Self(0);

    pub const fn from_index(index: u32) -> Self {
        Self(index)
    }

    fn as_i32(self, operation: &str) -> Result<i32, String> {
        i32::try_from(self.0)
            .map_err(|_| format!("{operation} mip level {} exceeds i32::MAX", self.0))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlTextureOffset2D {
    pub x: i32,
    pub y: i32,
}

impl GlTextureOffset2D {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlTextureOffset3D {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl GlTextureOffset3D {
    pub const ZERO: Self = Self::new(0, 0, 0);

    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlTextureSize2D {
    pub width: u32,
    pub height: u32,
}

impl GlTextureSize2D {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    fn as_gl_args(self, operation: &str) -> Result<(i32, i32), String> {
        let width = i32::try_from(self.width)
            .map_err(|_| format!("{operation} width {} exceeds i32::MAX", self.width))?;
        let height = i32::try_from(self.height)
            .map_err(|_| format!("{operation} height {} exceeds i32::MAX", self.height))?;
        Ok((width, height))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlTextureSize3D {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl GlTextureSize3D {
    pub const fn new(width: u32, height: u32, depth: u32) -> Self {
        Self {
            width,
            height,
            depth,
        }
    }

    fn as_gl_args(self, operation: &str) -> Result<(i32, i32, i32), String> {
        let width = glsizei_from_u32(self.width, &format!("{operation} width"))?;
        let height = glsizei_from_u32(self.height, &format!("{operation} height"))?;
        let depth = glsizei_from_u32(self.depth, &format!("{operation} depth"))?;
        Ok((width, height, depth))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlRenderbufferSize {
    pub width: u32,
    pub height: u32,
}

impl GlRenderbufferSize {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    fn as_gl_args(self) -> Result<(i32, i32), String> {
        Ok((
            glsizei_from_u32(self.width, "renderbuffer width")?,
            glsizei_from_u32(self.height, "renderbuffer height")?,
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl GlRect {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    fn as_gl_args(self, operation: &str) -> Result<(i32, i32, i32, i32), String> {
        let width = i32::try_from(self.width)
            .map_err(|_| format!("{operation} width {} exceeds i32::MAX", self.width))?;
        let height = i32::try_from(self.height)
            .map_err(|_| format!("{operation} height {} exceeds i32::MAX", self.height))?;
        Ok((self.x, self.y, width, height))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlFramebufferBuffer {
    None,
    Front,
    Back,
    ColorAttachment(u32),
}

impl GlFramebufferBuffer {
    fn as_raw(self) -> Result<u32, String> {
        match self {
            Self::None => Ok(GL_NONE),
            Self::Front => Ok(GL_FRONT),
            Self::Back => Ok(GL_BACK),
            Self::ColorAttachment(index) => GL_COLOR_ATTACHMENT0
                .checked_add(index)
                .ok_or_else(|| format!("color attachment index {index} overflows GLenum")),
        }
    }
}

#[bitflags]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlFramebufferBlitBuffer {
    Color = GL_COLOR_BUFFER_BIT,
    Depth = GL_DEPTH_BUFFER_BIT,
    Stencil = GL_STENCIL_BUFFER_BIT,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlFramebufferBlitFilter {
    Nearest,
    Linear,
}

impl GlFramebufferBlitFilter {
    fn as_raw(self) -> u32 {
        match self {
            Self::Nearest => GL_NEAREST,
            Self::Linear => GL_LINEAR,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GlCapability {
    Blend,
    CullFace,
    DepthTest,
    Dither,
    Multisample,
    PolygonOffsetFill,
    PrimitiveRestartFixedIndex,
    ScissorTest,
    StencilTest,
}

impl GlCapability {
    fn as_raw(self) -> u32 {
        match self {
            Self::Blend => GL_BLEND,
            Self::CullFace => GL_CULL_FACE,
            Self::DepthTest => GL_DEPTH_TEST,
            Self::Dither => GL_DITHER,
            Self::Multisample => GL_MULTISAMPLE,
            Self::PolygonOffsetFill => GL_POLYGON_OFFSET_FILL,
            Self::PrimitiveRestartFixedIndex => GL_PRIMITIVE_RESTART_FIXED_INDEX,
            Self::ScissorTest => GL_SCISSOR_TEST,
            Self::StencilTest => GL_STENCIL_TEST,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlDepthFunction {
    Never,
    Less,
    Equal,
    LessOrEqual,
    Greater,
    NotEqual,
    GreaterOrEqual,
    Always,
}

impl GlDepthFunction {
    fn as_raw(self) -> u32 {
        match self {
            Self::Never => GL_NEVER,
            Self::Less => GL_LESS,
            Self::Equal => GL_EQUAL,
            Self::LessOrEqual => GL_LEQUAL,
            Self::Greater => GL_GREATER,
            Self::NotEqual => GL_NOTEQUAL,
            Self::GreaterOrEqual => GL_GEQUAL,
            Self::Always => GL_ALWAYS,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlDepthRange {
    near: f64,
    far: f64,
}

impl GlDepthRange {
    pub const DEFAULT: Self = Self::new(0.0, 1.0);

    pub const fn new(near: f64, far: f64) -> Self {
        Self { near, far }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlCullFaceMode {
    Front,
    Back,
    FrontAndBack,
}

impl GlCullFaceMode {
    fn as_raw(self) -> u32 {
        match self {
            Self::Front => GL_FRONT,
            Self::Back => GL_BACK,
            Self::FrontAndBack => GL_FRONT_AND_BACK,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlFrontFaceWinding {
    Clockwise,
    CounterClockwise,
}

impl GlFrontFaceWinding {
    fn as_raw(self) -> u32 {
        match self {
            Self::Clockwise => GL_CW,
            Self::CounterClockwise => GL_CCW,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlStencilFunction {
    Never,
    Less,
    Equal,
    LessOrEqual,
    Greater,
    NotEqual,
    GreaterOrEqual,
    Always,
}

impl GlStencilFunction {
    fn as_raw(self) -> u32 {
        match self {
            Self::Never => GL_NEVER,
            Self::Less => GL_LESS,
            Self::Equal => GL_EQUAL,
            Self::LessOrEqual => GL_LEQUAL,
            Self::Greater => GL_GREATER,
            Self::NotEqual => GL_NOTEQUAL,
            Self::GreaterOrEqual => GL_GEQUAL,
            Self::Always => GL_ALWAYS,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlStencilReference(i32);

impl GlStencilReference {
    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    fn as_raw(self) -> i32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlStencilMask(u32);

impl GlStencilMask {
    pub const NONE: Self = Self(0);
    pub const ALL: Self = Self(u32::MAX);

    pub const fn new(bits: u32) -> Self {
        Self(bits)
    }

    fn as_raw(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlStencilFace {
    Front,
    Back,
    FrontAndBack,
}

impl GlStencilFace {
    fn as_raw(self) -> u32 {
        match self {
            Self::Front => GL_FRONT,
            Self::Back => GL_BACK,
            Self::FrontAndBack => GL_FRONT_AND_BACK,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlStencilOperation {
    Keep,
    Zero,
    Replace,
    IncrementClamp,
    DecrementClamp,
    Invert,
    IncrementWrap,
    DecrementWrap,
}

impl GlStencilOperation {
    fn as_raw(self) -> u32 {
        match self {
            Self::Keep => GL_KEEP,
            Self::Zero => 0,
            Self::Replace => GL_REPLACE,
            Self::IncrementClamp => GL_INCR,
            Self::DecrementClamp => GL_DECR,
            Self::Invert => GL_INVERT,
            Self::IncrementWrap => GL_INCR_WRAP,
            Self::DecrementWrap => GL_DECR_WRAP,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlColorWriteMask {
    red: bool,
    green: bool,
    blue: bool,
    alpha: bool,
}

impl GlColorWriteMask {
    pub const ALL: Self = Self::new(true, true, true, true);
    pub const NONE: Self = Self::new(false, false, false, false);
    pub const RGB: Self = Self::new(true, true, true, false);
    pub const ALPHA: Self = Self::new(false, false, false, true);

    pub const fn new(red: bool, green: bool, blue: bool, alpha: bool) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    fn as_raw(self) -> [u8; 4] {
        [
            if self.red { 1 } else { GL_FALSE },
            if self.green { 1 } else { GL_FALSE },
            if self.blue { 1 } else { GL_FALSE },
            if self.alpha { 1 } else { GL_FALSE },
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlPolygonOffset {
    factor: f32,
    units: f32,
}

impl GlPolygonOffset {
    pub const fn new(factor: f32, units: f32) -> Self {
        Self { factor, units }
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

    const fn byte_len(self) -> usize {
        match self {
            Self::UnsignedShort => mem::size_of::<u16>(),
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GlDrawRange {
    pub first: u32,
    pub count: u32,
}

impl GlDrawRange {
    pub const fn new(first: u32, count: u32) -> Self {
        Self { first, count }
    }

    pub const fn from_start(count: u32) -> Self {
        Self { first: 0, count }
    }

    fn as_gl_args(self, operation: &str) -> Result<(i32, i32), String> {
        let first = i32::try_from(self.first)
            .map_err(|_| format!("{operation} first vertex {} exceeds i32::MAX", self.first))?;
        let count = i32::try_from(self.count)
            .map_err(|_| format!("{operation} vertex count {} exceeds i32::MAX", self.count))?;
        Ok((first, count))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlElementVertexRange {
    pub start: u32,
    pub end: u32,
}

impl GlElementVertexRange {
    pub const fn new_unchecked(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub fn new(start: u32, end: u32) -> Result<Self, String> {
        if start > end {
            Err(format!(
                "element vertex range start {start} exceeds end {end}"
            ))
        } else {
            Ok(Self { start, end })
        }
    }

    fn as_gl_args(self) -> (u32, u32) {
        (self.start, self.end)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlElementByteOffset(usize);

impl GlElementByteOffset {
    pub const ZERO: Self = Self(0);

    pub const fn from_bytes(bytes: usize) -> Self {
        Self(bytes)
    }

    pub fn from_indices(index_type: GlIndexType, index_count: usize) -> Result<Self, String> {
        index_count
            .checked_mul(index_type.byte_len())
            .map(Self)
            .ok_or_else(|| format!("element index offset {index_count} overflows usize"))
    }

    const fn as_bytes(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlElementRange {
    pub count: u32,
    pub offset: GlElementByteOffset,
}

impl GlElementRange {
    pub const fn from_start(count: u32) -> Self {
        Self {
            count,
            offset: GlElementByteOffset::ZERO,
        }
    }

    pub const fn new(count: u32, offset: GlElementByteOffset) -> Self {
        Self { count, offset }
    }

    fn as_gl_args(self, operation: &str) -> Result<(i32, usize), String> {
        let count = i32::try_from(self.count)
            .map_err(|_| format!("{operation} index count {} exceeds i32::MAX", self.count))?;
        Ok((count, self.offset.as_bytes()))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlInstanceCount(u32);

impl GlInstanceCount {
    pub const fn new(count: u32) -> Self {
        Self(count)
    }

    fn as_i32(self, operation: &str) -> Result<i32, String> {
        i32::try_from(self.0)
            .map_err(|_| format!("{operation} instance count {} exceeds i32::MAX", self.0))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlVertexArray(u32);

impl GlVertexArray {
    pub fn as_raw(self) -> u32 {
        self.0
    }

    fn from_nonzero(id: u32) -> Result<Self, String> {
        if id == 0 {
            Err("glGenVertexArrays returned 0".to_string())
        } else {
            Ok(Self(id))
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlFramebuffer(u32);

impl GlFramebuffer {
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if raw == 0 { None } else { Some(Self(raw)) }
    }

    pub fn as_raw(self) -> u32 {
        self.0
    }

    fn from_nonzero(id: u32) -> Result<Self, String> {
        if id == 0 {
            Err("glGenFramebuffers returned 0".to_string())
        } else {
            Ok(Self(id))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlRenderbuffer(u32);

impl GlRenderbuffer {
    pub fn as_raw(self) -> u32 {
        self.0
    }

    fn from_nonzero(id: u32) -> Result<Self, String> {
        if id == 0 {
            Err("glGenRenderbuffers returned 0".to_string())
        } else {
            Ok(Self(id))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlFramebufferAttachment {
    Color(u32),
    Depth,
    Stencil,
    DepthStencil,
}

impl GlFramebufferAttachment {
    fn as_raw(self) -> Result<u32, String> {
        match self {
            Self::Color(index) => GL_COLOR_ATTACHMENT0
                .checked_add(index)
                .ok_or_else(|| format!("framebuffer color attachment index {index} overflowed")),
            Self::Depth => Ok(GL_DEPTH_ATTACHMENT),
            Self::Stencil => Ok(GL_STENCIL_ATTACHMENT),
            Self::DepthStencil => Ok(GL_DEPTH_STENCIL_ATTACHMENT),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlFramebufferTexture2DTarget {
    Texture2D,
}

impl GlFramebufferTexture2DTarget {
    fn as_raw(self) -> u32 {
        match self {
            Self::Texture2D => GL_TEXTURE_2D,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlRenderbufferTarget {
    Renderbuffer,
}

impl GlRenderbufferTarget {
    fn as_raw(self) -> u32 {
        match self {
            Self::Renderbuffer => GL_RENDERBUFFER,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlRenderbufferInternalFormat {
    Rgba4,
    Rgb565,
    DepthComponent16,
    StencilIndex8,
}

impl GlRenderbufferInternalFormat {
    fn as_raw(self) -> u32 {
        match self {
            Self::Rgba4 => GL_RGBA4,
            Self::Rgb565 => GL_RGB565,
            Self::DepthComponent16 => GL_DEPTH_COMPONENT16,
            Self::StencilIndex8 => GL_STENCIL_INDEX8,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlQuery(u32);

impl GlQuery {
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if raw == 0 { None } else { Some(Self(raw)) }
    }

    pub const fn as_raw(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlQueryTarget {
    SamplesPassed,
    AnySamplesPassed,
    AnySamplesPassedConservative,
    PrimitivesGenerated,
}

impl GlQueryTarget {
    fn as_raw(self) -> u32 {
        match self {
            Self::SamplesPassed => GL_SAMPLES_PASSED,
            Self::AnySamplesPassed => GL_ANY_SAMPLES_PASSED,
            Self::AnySamplesPassedConservative => GL_ANY_SAMPLES_PASSED_CONSERVATIVE,
            Self::PrimitivesGenerated => GL_PRIMITIVES_GENERATED,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlSync(*const c_void);

impl GlSync {
    fn from_raw(raw: *const c_void) -> Option<Self> {
        (!raw.is_null()).then_some(Self(raw))
    }

    fn as_raw(self) -> *const c_void {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlSyncTimeout(u64);

impl GlSyncTimeout {
    pub const ZERO: Self = Self(0);
    pub const IGNORED: Self = Self(GL_TIMEOUT_IGNORED);

    pub const fn from_nanos(nanos: u64) -> Self {
        Self(nanos)
    }

    fn as_raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlSyncWaitResult {
    AlreadySignaled,
    TimeoutExpired,
    ConditionSatisfied,
}

impl GlSyncWaitResult {
    fn from_raw(raw: u32) -> Result<Self, String> {
        match raw {
            GL_ALREADY_SIGNALED => Ok(Self::AlreadySignaled),
            GL_TIMEOUT_EXPIRED => Ok(Self::TimeoutExpired),
            GL_CONDITION_SATISFIED => Ok(Self::ConditionSatisfied),
            GL_WAIT_FAILED => Err("glClientWaitSync reported GL_WAIT_FAILED".to_string()),
            _ => Err(format!(
                "glClientWaitSync returned unknown status {raw:#06x}"
            )),
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
    depth_func: Option<GlDepthFunc>,
    depth_mask: Option<GlDepthMask>,
    depth_range_f: Option<GlDepthRangef>,
    cull_face: Option<GlCullFace>,
    front_face: Option<GlFrontFace>,
    stencil_func: Option<GlStencilFunc>,
    stencil_mask: Option<GlStencilMaskFn>,
    stencil_op: Option<GlStencilOp>,
    stencil_func_separate: Option<GlStencilFuncSeparate>,
    stencil_mask_separate: Option<GlStencilMaskSeparate>,
    stencil_op_separate: Option<GlStencilOpSeparate>,
    color_mask: Option<GlColorMaskFn>,
    polygon_offset: Option<GlPolygonOffsetFn>,
    gen_queries: Option<GlGenQueries>,
    delete_queries: Option<GlDeleteQueries>,
    begin_query: Option<GlBeginQuery>,
    end_query: Option<GlEndQuery>,
    get_query_object_uiv: Option<GlGetQueryObjectuiv>,
    fence_sync: Option<GlFenceSync>,
    client_wait_sync: Option<GlClientWaitSync>,
    wait_sync: Option<GlWaitSync>,
    delete_sync: Option<GlDeleteSync>,
    read_pixels: Option<GlReadPixels>,
    read_buffer: Option<GlReadBuffer>,
    draw_buffers: Option<GlDrawBuffers>,
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
    bind_buffer_base: Option<GlBindBufferBase>,
    bind_buffer_range: Option<GlBindBufferRange>,
    buffer_data: GlBufferData,
    buffer_sub_data: GlBufferSubData,
    copy_buffer_sub_data: Option<GlCopyBufferSubData>,
    delete_buffers: GlDeleteBuffers,
    gen_textures: GlGenTextures,
    bind_texture: GlBindTexture,
    active_texture: GlActiveTexture,
    tex_parameter_i: GlTexParameteri,
    pixel_store_i: GlPixelStorei,
    tex_image_2d: GlTexImage2D,
    tex_sub_image_2d: GlTexSubImage2D,
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
    vertex_attrib_divisor: Option<GlVertexAttribDivisorFn>,
    get_uniform_location: GlGetUniformLocation,
    get_attrib_location: GlGetAttribLocation,
    bind_attrib_location: Option<GlBindAttribLocation>,
    uniform_1i: GlUniform1i,
    uniform_1f: GlUniform1f,
    uniform_2f: GlUniform2f,
    uniform_3f: GlUniform3f,
    uniform_4f: GlUniform4f,
    uniform_4fv: GlUniform4fv,
    uniform_matrix_3fv: GlUniformMatrix3fv,
    uniform_matrix_4fv: GlUniformMatrix4fv,
    draw_arrays: GlDrawArrays,
    draw_arrays_instanced: Option<GlDrawArraysInstanced>,
    draw_elements: GlDrawElements,
    draw_range_elements: Option<GlDrawRangeElements>,
    draw_elements_instanced: Option<GlDrawElementsInstanced>,
    blend_color: Option<GlBlendColor>,
    blend_func: GlBlendFunc,
    blend_func_separate: Option<GlBlendFuncSeparate>,
    blend_equation: GlBlendEquationFn,
    blend_equation_separate: Option<GlBlendEquationSeparate>,
    gen_framebuffers: Option<GlGenFramebuffers>,
    bind_framebuffer: GlBindFramebuffer,
    delete_framebuffers: Option<GlDeleteFramebuffers>,
    framebuffer_texture_2d: Option<GlFramebufferTexture2D>,
    gen_renderbuffers: Option<GlGenRenderbuffers>,
    bind_renderbuffer: Option<GlBindRenderbuffer>,
    renderbuffer_storage: Option<GlRenderbufferStorage>,
    delete_renderbuffers: Option<GlDeleteRenderbuffers>,
    framebuffer_renderbuffer: Option<GlFramebufferRenderbuffer>,
    blit_framebuffer: Option<GlBlitFramebuffer>,
    get_error: Option<GlGetError>,
    check_framebuffer_status: Option<GlCheckFramebufferStatus>,
    invalidate_framebuffer: Option<GlInvalidateFramebuffer>,
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

fn query_gl_string_i(get_string_i: GlGetStringi, name: u32, index: u32) -> Option<String> {
    let raw = unsafe { get_string_i(name, index) };
    if raw.is_null() {
        return None;
    }
    Some(
        unsafe { CStr::from_ptr(raw.cast::<c_char>()) }
            .to_string_lossy()
            .into_owned(),
    )
}

fn query_gl_indexed_extensions(
    get_integer_v: GlGetIntegerv,
    get_string_i: Option<GlGetStringi>,
) -> String {
    let Some(get_string_i) = get_string_i else {
        return String::new();
    };
    let mut count = 0;
    unsafe { get_integer_v(GL_NUM_EXTENSIONS, &mut count) };
    let Ok(count) = u32::try_from(count) else {
        return String::new();
    };
    (0..count)
        .filter_map(|index| query_gl_string_i(get_string_i, GL_EXTENSIONS, index))
        .collect::<Vec<_>>()
        .join(" ")
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

fn glsizei_from_u32(value: u32, label: &str) -> Result<i32, String> {
    i32::try_from(value).map_err(|_| format!("{label} {value} exceeds GLsizei::MAX"))
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

    fn viewport_raw(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe { (self.viewport)(x, y, width, height) };
    }

    pub fn viewport(&self, rect: GlRect) -> Result<(), String> {
        let (x, y, width, height) = rect.as_gl_args("glViewport")?;
        self.viewport_raw(x, y, width, height);
        Ok(())
    }

    fn bind_framebuffer_raw(&self, target: GlFramebufferTarget, framebuffer: u32) {
        unsafe { (self.bind_framebuffer)(target.as_raw(), framebuffer) };
    }

    pub fn unbind_framebuffer(&self, target: GlFramebufferTarget) {
        self.bind_framebuffer_raw(target, 0);
    }

    fn bind_framebuffer_checked_raw(
        &self,
        target: GlFramebufferTarget,
        framebuffer: u32,
    ) -> Result<(), String> {
        self.bind_framebuffer_raw(target, framebuffer);
        self.check_no_error("glBindFramebuffer")?;
        if framebuffer != 0 {
            self.check_bound_framebuffer_complete(target)?;
        }
        Ok(())
    }

    pub fn bind_framebuffer(
        &self,
        target: GlFramebufferTarget,
        framebuffer: Option<GlFramebuffer>,
    ) -> Result<(), String> {
        self.bind_framebuffer_checked_raw(target, framebuffer.map_or(0, GlFramebuffer::as_raw))
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

    pub fn viewport(&self, rect: GlRect) -> Result<(), String> {
        self.clear.viewport(rect)
    }

    pub fn unbind_framebuffer(&self, target: GlFramebufferTarget) {
        self.clear.unbind_framebuffer(target);
    }

    pub fn bind_framebuffer(
        &self,
        target: GlFramebufferTarget,
        framebuffer: Option<GlFramebuffer>,
    ) -> Result<(), String> {
        self.clear.bind_framebuffer(target, framebuffer)
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

    pub fn build_program(
        &self,
        vertex_shader: &str,
        fragment_shader: &str,
    ) -> Result<GlProgram, String> {
        let vertex = self.compile_shader_source(GlShaderStage::Vertex, vertex_shader)?;
        let fragment = match self.compile_shader_source(GlShaderStage::Fragment, fragment_shader) {
            Ok(shader) => shader,
            Err(error) => {
                self.delete_shader(vertex);
                return Err(error);
            }
        };

        let program = match self.create_program() {
            Ok(program) => program,
            Err(error) => {
                self.delete_shader(vertex);
                self.delete_shader(fragment);
                return Err(error);
            }
        };

        self.attach_shader(program, vertex);
        self.attach_shader(program, fragment);
        let link_result = self.link_program(program);
        self.delete_shader(vertex);
        self.delete_shader(fragment);

        if let Err(error) = link_result {
            self.delete_program(program);
            return Err(error);
        }
        Ok(program)
    }

    pub fn compile_shader_source(
        &self,
        stage: GlShaderStage,
        source: &str,
    ) -> Result<GlShader, String> {
        let shader = GlShader::from_nonzero(self.create_shader_raw(stage.as_raw()), stage)?;
        if let Err(error) = self.shader_source_raw_handle(shader.as_raw(), source) {
            self.delete_shader(shader);
            return Err(error);
        }
        self.compile_shader_raw(shader.as_raw());

        let status = self.get_shader_iv(shader.as_raw(), GL_COMPILE_STATUS);
        if status == 0 {
            let log = self.get_shader_info_log(shader.as_raw());
            self.delete_shader(shader);
            return Err(format!("shader compile failed: {log}"));
        }

        if let Err(error) = self.check_no_error("CompatGl::compile_shader_source") {
            self.delete_shader(shader);
            return Err(error);
        }
        Ok(shader)
    }

    fn create_shader_raw(&self, shader_type: u32) -> u32 {
        unsafe { (self.create_shader)(shader_type) }
    }

    fn shader_source_raw_handle(&self, shader: u32, source: &str) -> Result<(), String> {
        let source = gl_string(source)?;
        let ptr = source.as_ptr();
        unsafe { (self.shader_source)(shader, 1, &ptr, std::ptr::null()) };
        Ok(())
    }

    fn compile_shader_raw(&self, shader: u32) {
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

    fn delete_shader_raw(&self, shader: u32) {
        unsafe { (self.delete_shader)(shader) };
    }

    pub fn delete_shader(&self, shader: GlShader) {
        self.delete_shader_raw(shader.as_raw());
    }

    fn create_program_raw(&self) -> u32 {
        unsafe { (self.create_program)() }
    }

    pub fn create_program(&self) -> Result<GlProgram, String> {
        GlProgram::from_nonzero(self.create_program_raw())
    }

    fn attach_shader_raw(&self, program: u32, shader: u32) {
        unsafe { (self.attach_shader)(program, shader) };
    }

    fn attach_shader(&self, program: GlProgram, shader: GlShader) {
        self.attach_shader_raw(program.as_raw(), shader.as_raw());
    }

    fn link_program_raw(&self, program: u32) {
        unsafe { (self.link_program)(program) };
    }

    pub fn link_program(&self, program: GlProgram) -> Result<(), String> {
        self.link_program_raw(program.as_raw());
        let status = self.get_program_iv(program.as_raw(), GL_LINK_STATUS);
        if status == 0 {
            let log = self.get_program_info_log(program.as_raw());
            return Err(format!("program link failed: {log}"));
        }
        self.check_no_error("CompatGl::link_program")
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

    fn delete_program_raw(&self, program: u32) {
        unsafe { (self.delete_program)(program) };
    }

    pub fn delete_program(&self, program: GlProgram) {
        self.delete_program_raw(program.as_raw());
    }

    fn use_program_raw(&self, program: u32) {
        unsafe { (self.use_program)(program) };
    }

    pub fn use_no_program(&self) {
        self.use_program_raw(0);
    }

    pub fn use_program(&self, program: Option<GlProgram>) {
        self.use_program_raw(program.map_or(0, GlProgram::as_raw));
    }

    fn gen_buffer_raw(&self) -> u32 {
        let mut id = 0;
        unsafe { (self.gen_buffers)(1, &mut id) };
        id
    }

    pub fn gen_buffer(&self) -> Result<GlBuffer, String> {
        GlBuffer::from_nonzero(self.gen_buffer_raw())
    }

    fn bind_buffer_raw(&self, target: GlBufferTarget, buffer: u32) {
        unsafe { (self.bind_buffer)(target.as_raw(), buffer) };
    }

    pub fn unbind_buffer(&self, target: GlBufferTarget) {
        self.bind_buffer_raw(target, 0);
    }

    pub fn bind_buffer(&self, target: GlBufferTarget, buffer: Option<GlBuffer>) {
        self.bind_buffer_raw(target, buffer.map_or(0, GlBuffer::as_raw));
    }

    pub fn buffer_data<T>(
        &self,
        target: GlBufferTarget,
        data: &[T],
        usage: GlBufferUsage,
    ) -> Result<(), String> {
        let byte_len = std::mem::size_of_val(data);
        let byte_len = GlBufferByteSize::from_bytes(byte_len).as_isize("GL buffer upload")?;
        unsafe {
            (self.buffer_data)(
                target.as_raw(),
                byte_len,
                data.as_ptr().cast::<c_void>(),
                usage.as_raw(),
            );
        }
        Ok(())
    }

    fn delete_buffer_raw(&self, id: u32) {
        unsafe { (self.delete_buffers)(1, &id) };
    }

    pub fn delete_buffer(&self, buffer: GlBuffer) {
        self.delete_buffer_raw(buffer.as_raw());
    }

    fn enable_vertex_attrib_array_raw(&self, index: u32) {
        unsafe { (self.enable_vertex_attrib_array)(index) };
    }

    pub fn enable_vertex_attrib(&self, location: GlVertexAttribLocation) {
        self.enable_vertex_attrib_array_raw(location.as_raw());
    }

    fn disable_vertex_attrib_array_raw(&self, index: u32) {
        unsafe { (self.disable_vertex_attrib_array)(index) };
    }

    pub fn disable_vertex_attrib(&self, location: GlVertexAttribLocation) {
        self.disable_vertex_attrib_array_raw(location.as_raw());
    }

    fn vertex_attrib_pointer_f32_raw(
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

    fn vertex_attrib_pointer_f32_at(
        &self,
        location: GlVertexAttribLocation,
        size: i32,
        normalized: bool,
        stride: i32,
        offset: usize,
    ) {
        self.vertex_attrib_pointer_f32_raw(location.as_raw(), size, normalized, stride, offset);
    }

    pub fn vertex_attrib_pointer_f32(
        &self,
        location: GlVertexAttribLocation,
        layout: GlVertexAttribF32Layout,
    ) {
        self.vertex_attrib_pointer_f32_at(
            location,
            layout.components.as_gl_size(),
            layout.normalized,
            layout.stride.as_i32(),
            layout.offset.as_bytes(),
        );
    }

    fn attrib_location(
        &self,
        program: u32,
        name: &str,
    ) -> Result<Option<GlVertexAttribLocation>, String> {
        let name = gl_string(name)?;
        let location = unsafe { (self.get_attrib_location)(program, name.as_ptr()) };
        Ok(GlVertexAttribLocation::from_raw(location))
    }

    fn required_attrib(&self, program: u32, name: &str) -> Result<GlVertexAttribLocation, String> {
        self.attrib_location(program, name)?
            .ok_or_else(|| format!("shader linked without required active attribute {name}"))
    }

    pub fn required_attrib_location(
        &self,
        program: GlProgram,
        name: &str,
    ) -> Result<GlVertexAttribLocation, String> {
        self.required_attrib(program.as_raw(), name)
    }

    fn uniform_location(
        &self,
        program: u32,
        name: &str,
    ) -> Result<Option<GlUniformLocation>, String> {
        let name = gl_string(name)?;
        let location = unsafe { (self.get_uniform_location)(program, name.as_ptr()) };
        Ok(GlUniformLocation::from_raw(location))
    }

    fn required_uniform(&self, program: u32, name: &str) -> Result<GlUniformLocation, String> {
        self.uniform_location(program, name)?
            .ok_or_else(|| format!("shader linked without required active uniform {name}"))
    }

    pub fn required_uniform_location(
        &self,
        program: GlProgram,
        name: &str,
    ) -> Result<GlUniformLocation, String> {
        self.required_uniform(program.as_raw(), name)
    }

    fn uniform_4fv_raw(&self, location: i32, values: &[f32; 4]) {
        unsafe { (self.uniform_4fv)(location, 1, values.as_ptr()) };
    }

    pub fn uniform_4fv(&self, location: GlUniformLocation, values: &[f32; 4]) {
        self.uniform_4fv_raw(location.as_raw(), values);
    }

    fn draw_arrays_raw(&self, mode: GlDrawMode, first: i32, count: i32) {
        unsafe { (self.draw_arrays)(mode.as_raw(), first, count) };
    }

    pub fn draw_arrays(&self, mode: GlDrawMode, range: GlDrawRange) -> Result<(), String> {
        let (first, count) = range.as_gl_args("glDrawArrays")?;
        self.draw_arrays_raw(mode, first, count);
        Ok(())
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

    fn gen_texture_raw(&self) -> u32 {
        let mut id = 0;
        unsafe { (self.gen_textures)(1, &mut id) };
        id
    }

    pub fn gen_texture(&self) -> Result<GlTexture, String> {
        GlTexture::from_nonzero(self.gen_texture_raw())
    }

    fn bind_texture_raw(&self, target: GlTextureTarget, texture: u32) {
        unsafe { (self.bind_texture)(target.as_raw(), texture) };
    }

    pub fn unbind_texture(&self, target: GlTextureTarget) {
        self.bind_texture_raw(target, 0);
    }

    pub fn bind_texture(&self, target: GlTextureTarget, texture: Option<GlTexture>) {
        self.bind_texture_raw(target, texture.map_or(0, GlTexture::as_raw));
    }

    pub fn active_texture(&self, unit: GlTextureUnit) -> Result<(), String> {
        unsafe { (self.active_texture)(unit.as_raw()?) };
        Ok(())
    }

    pub fn tex_min_filter(&self, target: GlTextureTarget, filter: GlTextureMinFilter) {
        unsafe {
            (self.tex_parameter_i)(
                target.as_raw(),
                GL_TEXTURE_MIN_FILTER,
                filter.as_raw() as i32,
            )
        };
    }

    pub fn tex_mag_filter(&self, target: GlTextureTarget, filter: GlTextureMagFilter) {
        unsafe {
            (self.tex_parameter_i)(
                target.as_raw(),
                GL_TEXTURE_MAG_FILTER,
                filter.as_raw() as i32,
            )
        };
    }

    pub fn tex_wrap_s(&self, target: GlTextureTarget, wrap: GlTextureWrap) {
        unsafe { (self.tex_parameter_i)(target.as_raw(), GL_TEXTURE_WRAP_S, wrap.as_raw() as i32) };
    }

    pub fn tex_wrap_t(&self, target: GlTextureTarget, wrap: GlTextureWrap) {
        unsafe { (self.tex_parameter_i)(target.as_raw(), GL_TEXTURE_WRAP_T, wrap.as_raw() as i32) };
    }

    pub fn tex_wrap_r(&self, target: GlTextureTarget, wrap: GlTextureWrap) {
        unsafe { (self.tex_parameter_i)(target.as_raw(), GL_TEXTURE_WRAP_R, wrap.as_raw() as i32) };
    }

    pub fn pixel_store_unpack_alignment(&self, alignment: GlPixelStoreAlignment) {
        unsafe { (self.pixel_store_i)(GL_UNPACK_ALIGNMENT, alignment.as_raw()) };
    }

    fn tex_image_2d_raw(
        &self,
        target: GlTextureTarget,
        internal_format: GlTextureInternalFormat,
        level: GlTextureLevel,
        size: GlTextureSize2D,
        format: GlTextureFormat,
        data_type: GlTextureDataType,
        bytes: Option<&[u8]>,
    ) -> Result<(), String> {
        let level = level.as_i32("glTexImage2D")?;
        let (width, height) = size.as_gl_args("glTexImage2D")?;
        let pixels = bytes
            .map(|bytes| bytes.as_ptr().cast::<c_void>())
            .unwrap_or(std::ptr::null());
        unsafe {
            (self.tex_image_2d)(
                target.as_raw(),
                level,
                internal_format.as_raw() as i32,
                width,
                height,
                0,
                format.as_raw(),
                data_type.as_raw(),
                pixels,
            );
        }
        Ok(())
    }

    pub fn tex_image_2d(
        &self,
        target: GlTextureTarget,
        internal_format: GlTextureInternalFormat,
        level: GlTextureLevel,
        size: GlTextureSize2D,
        format: GlTextureFormat,
        data_type: GlTextureDataType,
        bytes: Option<&[u8]>,
    ) -> Result<(), String> {
        self.tex_image_2d_raw(
            target,
            internal_format,
            level,
            size,
            format,
            data_type,
            bytes,
        )
    }

    fn delete_texture_raw(&self, id: u32) {
        unsafe { (self.delete_textures)(1, &id) };
    }

    pub fn delete_texture(&self, texture: GlTexture) {
        self.delete_texture_raw(texture.as_raw());
    }

    fn uniform_location(
        &self,
        program: u32,
        name: &str,
    ) -> Result<Option<GlUniformLocation>, String> {
        let name = gl_string(name)?;
        let location = unsafe { (self.get_uniform_location)(program, name.as_ptr()) };
        Ok(GlUniformLocation::from_raw(location))
    }

    fn required_uniform(&self, program: u32, name: &str) -> Result<GlUniformLocation, String> {
        self.uniform_location(program, name)?
            .ok_or_else(|| format!("shader linked without required active uniform {name}"))
    }

    pub fn required_uniform_location(
        &self,
        program: GlProgram,
        name: &str,
    ) -> Result<GlUniformLocation, String> {
        self.required_uniform(program.as_raw(), name)
    }

    fn uniform_1i_raw(&self, location: i32, value: i32) {
        unsafe { (self.uniform_1i)(location, value) };
    }

    pub fn uniform_1i(&self, location: GlUniformLocation, value: i32) {
        self.uniform_1i_raw(location.as_raw(), value);
    }

    fn uniform_4fv_raw(&self, location: i32, values: &[f32; 4]) {
        unsafe { (self.uniform_4fv)(location, 1, values.as_ptr()) };
    }

    pub fn uniform_4fv(&self, location: GlUniformLocation, values: &[f32; 4]) {
        self.uniform_4fv_raw(location.as_raw(), values);
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
        let get_string_i: Option<GlGetStringi> = load_optional_gl_symbol(runtime, "glGetStringi")?;
        let version_info = query_gl_version_info(get_string, context_type);
        let extensions_string = if version_info.is_gles || !version_info.version_at_least(3, 0) {
            query_gl_string(get_string, GL_EXTENSIONS)
        } else {
            // Desktop OpenGL 3+ core profiles require glGetStringi for
            // extensions; glGetString(GL_EXTENSIONS) is invalid and can leave a
            // sticky GL error that later shader/bootstrap validation reports.
            query_gl_indexed_extensions(get_integer_v, get_string_i)
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
            depth_func: load_optional_gl_symbol(runtime, "glDepthFunc")?,
            depth_mask: load_optional_gl_symbol(runtime, "glDepthMask")?,
            depth_range_f: load_optional_gl_symbol(runtime, "glDepthRangef")?,
            cull_face: load_optional_gl_symbol(runtime, "glCullFace")?,
            front_face: load_optional_gl_symbol(runtime, "glFrontFace")?,
            stencil_func: load_optional_gl_symbol(runtime, "glStencilFunc")?,
            stencil_mask: load_optional_gl_symbol(runtime, "glStencilMask")?,
            stencil_op: load_optional_gl_symbol(runtime, "glStencilOp")?,
            stencil_func_separate: load_optional_gl_symbol(runtime, "glStencilFuncSeparate")?,
            stencil_mask_separate: load_optional_gl_symbol(runtime, "glStencilMaskSeparate")?,
            stencil_op_separate: load_optional_gl_symbol(runtime, "glStencilOpSeparate")?,
            color_mask: load_optional_gl_symbol(runtime, "glColorMask")?,
            polygon_offset: load_optional_gl_symbol(runtime, "glPolygonOffset")?,
            gen_queries: load_optional_gl_symbol(runtime, "glGenQueries")?,
            delete_queries: load_optional_gl_symbol(runtime, "glDeleteQueries")?,
            begin_query: load_optional_gl_symbol(runtime, "glBeginQuery")?,
            end_query: load_optional_gl_symbol(runtime, "glEndQuery")?,
            get_query_object_uiv: load_optional_gl_symbol(runtime, "glGetQueryObjectuiv")?,
            fence_sync: load_optional_gl_symbol(runtime, "glFenceSync")?,
            client_wait_sync: load_optional_gl_symbol(runtime, "glClientWaitSync")?,
            wait_sync: load_optional_gl_symbol(runtime, "glWaitSync")?,
            delete_sync: load_optional_gl_symbol(runtime, "glDeleteSync")?,
            read_pixels: load_optional_gl_symbol(runtime, "glReadPixels")?,
            read_buffer: load_optional_gl_symbol(runtime, "glReadBuffer")?,
            draw_buffers: load_optional_gl_symbol(runtime, "glDrawBuffers")?,
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
            bind_buffer_base: load_optional_gl_symbol_aliases(runtime, &["glBindBufferBase"])?,
            bind_buffer_range: load_optional_gl_symbol_aliases(runtime, &["glBindBufferRange"])?,
            buffer_data: load_gl_symbol(runtime, "glBufferData")?,
            buffer_sub_data: load_gl_symbol(runtime, "glBufferSubData")?,
            copy_buffer_sub_data: load_optional_gl_symbol(runtime, "glCopyBufferSubData")?,
            delete_buffers: load_gl_symbol(runtime, "glDeleteBuffers")?,
            gen_textures: load_gl_symbol(runtime, "glGenTextures")?,
            bind_texture: load_gl_symbol(runtime, "glBindTexture")?,
            active_texture: load_gl_symbol(runtime, "glActiveTexture")?,
            tex_parameter_i: load_gl_symbol(runtime, "glTexParameteri")?,
            pixel_store_i: load_gl_symbol(runtime, "glPixelStorei")?,
            tex_image_2d: load_gl_symbol(runtime, "glTexImage2D")?,
            tex_sub_image_2d: load_gl_symbol(runtime, "glTexSubImage2D")?,
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
            vertex_attrib_divisor: load_optional_gl_symbol_aliases(
                runtime,
                &["glVertexAttribDivisor"],
            )?,
            get_uniform_location: load_gl_symbol(runtime, "glGetUniformLocation")?,
            get_attrib_location: load_gl_symbol(runtime, "glGetAttribLocation")?,
            bind_attrib_location: load_optional_gl_symbol(runtime, "glBindAttribLocation")?,
            uniform_1i: load_gl_symbol(runtime, "glUniform1i")?,
            uniform_1f: load_gl_symbol(runtime, "glUniform1f")?,
            uniform_2f: load_gl_symbol(runtime, "glUniform2f")?,
            uniform_3f: load_gl_symbol(runtime, "glUniform3f")?,
            uniform_4f: load_gl_symbol(runtime, "glUniform4f")?,
            uniform_4fv: load_gl_symbol(runtime, "glUniform4fv")?,
            uniform_matrix_3fv: load_gl_symbol(runtime, "glUniformMatrix3fv")?,
            uniform_matrix_4fv: load_gl_symbol(runtime, "glUniformMatrix4fv")?,
            draw_arrays: load_gl_symbol(runtime, "glDrawArrays")?,
            draw_arrays_instanced: load_optional_gl_symbol_aliases(
                runtime,
                &["glDrawArraysInstanced"],
            )?,
            draw_elements: load_gl_symbol(runtime, "glDrawElements")?,
            draw_range_elements: load_optional_gl_symbol(runtime, "glDrawRangeElements")?,
            draw_elements_instanced: load_optional_gl_symbol_aliases(
                runtime,
                &["glDrawElementsInstanced"],
            )?,
            blend_color: load_optional_gl_symbol(runtime, "glBlendColor")?,
            blend_func: load_gl_symbol(runtime, "glBlendFunc")?,
            blend_func_separate: load_optional_gl_symbol(runtime, "glBlendFuncSeparate")?,
            blend_equation: load_gl_symbol(runtime, "glBlendEquation")?,
            blend_equation_separate: load_optional_gl_symbol(runtime, "glBlendEquationSeparate")?,
            gen_framebuffers: load_optional_gl_symbol(runtime, "glGenFramebuffers")?,
            bind_framebuffer: load_gl_symbol(runtime, "glBindFramebuffer")?,
            delete_framebuffers: load_optional_gl_symbol(runtime, "glDeleteFramebuffers")?,
            framebuffer_texture_2d: load_optional_gl_symbol(runtime, "glFramebufferTexture2D")?,
            gen_renderbuffers: load_optional_gl_symbol(runtime, "glGenRenderbuffers")?,
            bind_renderbuffer: load_optional_gl_symbol(runtime, "glBindRenderbuffer")?,
            renderbuffer_storage: load_optional_gl_symbol(runtime, "glRenderbufferStorage")?,
            delete_renderbuffers: load_optional_gl_symbol(runtime, "glDeleteRenderbuffers")?,
            framebuffer_renderbuffer: load_optional_gl_symbol(
                runtime,
                "glFramebufferRenderbuffer",
            )?,
            blit_framebuffer: load_optional_gl_symbol_aliases(runtime, &["glBlitFramebuffer"])?,
            // Product rendering treats GL error/FBO checks as part of the
            // libretro shared-context contract. Narrow compatibility symbol
            // groups keep these optional, but the full renderer must not
            // silently disable them.
            get_error: Some(load_gl_symbol(runtime, "glGetError")?),
            check_framebuffer_status: Some(load_gl_symbol(runtime, "glCheckFramebufferStatus")?),
            invalidate_framebuffer: load_optional_gl_symbol(runtime, "glInvalidateFramebuffer")?,
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

    pub fn depth_func(&self, function: GlDepthFunction) -> Result<(), String> {
        let Some(depth_func) = self.depth_func else {
            return Err(
                "depth comparison function state is not available for this GL context".to_string(),
            );
        };
        unsafe { depth_func(function.as_raw()) };
        Ok(())
    }

    pub fn depth_mask(&self, enabled: bool) -> Result<(), String> {
        let Some(depth_mask) = self.depth_mask else {
            return Err("depth write mask state is not available for this GL context".to_string());
        };
        unsafe { depth_mask(if enabled { 1 } else { GL_FALSE }) };
        Ok(())
    }

    pub fn depth_range(&self, range: GlDepthRange) -> Result<(), String> {
        if let Some(depth_range_f) = self.depth_range_f {
            unsafe { depth_range_f(range.near as f32, range.far as f32) };
            Ok(())
        } else {
            Err("depth range state is not available for this GL context".to_string())
        }
    }

    pub fn cull_face(&self, mode: GlCullFaceMode) -> Result<(), String> {
        let Some(cull_face) = self.cull_face else {
            return Err("face culling mode state is not available for this GL context".to_string());
        };
        unsafe { cull_face(mode.as_raw()) };
        Ok(())
    }

    pub fn front_face(&self, winding: GlFrontFaceWinding) -> Result<(), String> {
        let Some(front_face) = self.front_face else {
            return Err(
                "front-face winding state is not available for this GL context".to_string(),
            );
        };
        unsafe { front_face(winding.as_raw()) };
        Ok(())
    }

    pub fn stencil_func(
        &self,
        function: GlStencilFunction,
        reference: GlStencilReference,
        mask: GlStencilMask,
    ) -> Result<(), String> {
        let Some(stencil_func) = self.stencil_func else {
            return Err(
                "stencil comparison state is not available for this GL context".to_string(),
            );
        };
        unsafe { stencil_func(function.as_raw(), reference.as_raw(), mask.as_raw()) };
        Ok(())
    }

    pub fn stencil_mask(&self, mask: GlStencilMask) -> Result<(), String> {
        let Some(stencil_mask) = self.stencil_mask else {
            return Err(
                "stencil write mask state is not available for this GL context".to_string(),
            );
        };
        unsafe { stencil_mask(mask.as_raw()) };
        Ok(())
    }

    pub fn stencil_op(
        &self,
        stencil_fail: GlStencilOperation,
        depth_fail: GlStencilOperation,
        depth_pass: GlStencilOperation,
    ) -> Result<(), String> {
        let Some(stencil_op) = self.stencil_op else {
            return Err("stencil operation state is not available for this GL context".to_string());
        };
        unsafe {
            stencil_op(
                stencil_fail.as_raw(),
                depth_fail.as_raw(),
                depth_pass.as_raw(),
            )
        };
        Ok(())
    }

    pub fn stencil_func_separate(
        &self,
        face: GlStencilFace,
        function: GlStencilFunction,
        reference: GlStencilReference,
        mask: GlStencilMask,
    ) -> Result<(), String> {
        let Some(stencil_func_separate) = self.stencil_func_separate else {
            return Err(
                "per-face stencil comparison state is not available for this GL context"
                    .to_string(),
            );
        };
        unsafe {
            stencil_func_separate(
                face.as_raw(),
                function.as_raw(),
                reference.as_raw(),
                mask.as_raw(),
            )
        };
        Ok(())
    }

    pub fn stencil_mask_separate(
        &self,
        face: GlStencilFace,
        mask: GlStencilMask,
    ) -> Result<(), String> {
        let Some(stencil_mask_separate) = self.stencil_mask_separate else {
            return Err(
                "per-face stencil write mask state is not available for this GL context"
                    .to_string(),
            );
        };
        unsafe { stencil_mask_separate(face.as_raw(), mask.as_raw()) };
        Ok(())
    }

    pub fn stencil_op_separate(
        &self,
        face: GlStencilFace,
        stencil_fail: GlStencilOperation,
        depth_fail: GlStencilOperation,
        depth_pass: GlStencilOperation,
    ) -> Result<(), String> {
        let Some(stencil_op_separate) = self.stencil_op_separate else {
            return Err(
                "per-face stencil operation state is not available for this GL context".to_string(),
            );
        };
        unsafe {
            stencil_op_separate(
                face.as_raw(),
                stencil_fail.as_raw(),
                depth_fail.as_raw(),
                depth_pass.as_raw(),
            )
        };
        Ok(())
    }

    pub fn color_mask(&self, mask: GlColorWriteMask) -> Result<(), String> {
        let Some(color_mask) = self.color_mask else {
            return Err("color write mask state is not available for this GL context".to_string());
        };
        let [red, green, blue, alpha] = mask.as_raw();
        unsafe { color_mask(red, green, blue, alpha) };
        Ok(())
    }

    pub fn polygon_offset(&self, offset: GlPolygonOffset) -> Result<(), String> {
        let Some(polygon_offset) = self.polygon_offset else {
            return Err("polygon offset state is not available for this GL context".to_string());
        };
        unsafe { polygon_offset(offset.factor, offset.units) };
        Ok(())
    }

    pub fn supports_queries(&self) -> bool {
        self.gen_queries.is_some()
            && self.delete_queries.is_some()
            && self.begin_query.is_some()
            && self.end_query.is_some()
            && self.get_query_object_uiv.is_some()
    }

    pub fn gen_query(&self) -> Result<GlQuery, String> {
        let Some(gen_queries) = self.gen_queries else {
            return Err("query objects are not available for this GL context".to_string());
        };
        let mut id = 0;
        unsafe { gen_queries(1, &mut id) };
        GlQuery::from_raw(id).ok_or_else(|| "glGenQueries returned 0".to_string())
    }

    pub fn delete_query(&self, query: GlQuery) -> Result<(), String> {
        let Some(delete_queries) = self.delete_queries else {
            return Err("query object deletion is not available for this GL context".to_string());
        };
        let id = query.as_raw();
        unsafe { delete_queries(1, &id) };
        Ok(())
    }

    pub fn begin_query(&self, target: GlQueryTarget, query: GlQuery) -> Result<(), String> {
        let Some(begin_query) = self.begin_query else {
            return Err("query begin is not available for this GL context".to_string());
        };
        unsafe { begin_query(target.as_raw(), query.as_raw()) };
        Ok(())
    }

    pub fn end_query(&self, target: GlQueryTarget) -> Result<(), String> {
        let Some(end_query) = self.end_query else {
            return Err("query end is not available for this GL context".to_string());
        };
        unsafe { end_query(target.as_raw()) };
        Ok(())
    }

    pub fn query_result_available(&self, query: GlQuery) -> Result<bool, String> {
        let value = self.query_object_u32(query, GL_QUERY_RESULT_AVAILABLE)?;
        Ok(value != 0)
    }

    pub fn query_result_u32(&self, query: GlQuery) -> Result<u32, String> {
        self.query_object_u32(query, GL_QUERY_RESULT)
    }

    fn query_object_u32(&self, query: GlQuery, property: u32) -> Result<u32, String> {
        let Some(get_query_object_uiv) = self.get_query_object_uiv else {
            return Err("query object results are not available for this GL context".to_string());
        };
        let mut value = 0;
        unsafe { get_query_object_uiv(query.as_raw(), property, &mut value) };
        Ok(value)
    }

    pub fn supports_sync_objects(&self) -> bool {
        self.fence_sync.is_some()
            && self.client_wait_sync.is_some()
            && self.wait_sync.is_some()
            && self.delete_sync.is_some()
    }

    pub fn fence_sync(&self) -> Result<GlSync, String> {
        let Some(fence_sync) = self.fence_sync else {
            return Err("sync fences are not available for this GL context".to_string());
        };
        let sync = unsafe { fence_sync(GL_SYNC_GPU_COMMANDS_COMPLETE, 0) };
        GlSync::from_raw(sync).ok_or_else(|| "glFenceSync returned null".to_string())
    }

    pub fn client_wait_sync(
        &self,
        sync: GlSync,
        flush_commands: bool,
        timeout: GlSyncTimeout,
    ) -> Result<GlSyncWaitResult, String> {
        let Some(client_wait_sync) = self.client_wait_sync else {
            return Err("client sync waits are not available for this GL context".to_string());
        };
        let flags = if flush_commands {
            GL_SYNC_FLUSH_COMMANDS_BIT
        } else {
            0
        };
        let result = unsafe { client_wait_sync(sync.as_raw(), flags, timeout.as_raw()) };
        GlSyncWaitResult::from_raw(result)
    }

    pub fn wait_sync(&self, sync: GlSync) -> Result<(), String> {
        let Some(wait_sync) = self.wait_sync else {
            return Err("server sync waits are not available for this GL context".to_string());
        };
        unsafe { wait_sync(sync.as_raw(), 0, GL_TIMEOUT_IGNORED) };
        Ok(())
    }

    pub fn delete_sync(&self, sync: GlSync) -> Result<(), String> {
        let Some(delete_sync) = self.delete_sync else {
            return Err("sync deletion is not available for this GL context".to_string());
        };
        unsafe { delete_sync(sync.as_raw()) };
        Ok(())
    }

    pub fn read_pixels(
        &self,
        rect: GlRect,
        format: GlTextureFormat,
        pixels: &mut [u8],
    ) -> Result<(), String> {
        let Some(read_pixels) = self.read_pixels else {
            return Err("pixel readback is not available for this GL context".to_string());
        };
        let (x, y, width, height) = rect.as_gl_args("glReadPixels")?;
        let byte_len = read_pixels_len(rect, format)?;
        if pixels.len() < byte_len {
            return Err(format!(
                "glReadPixels requires {byte_len} destination byte(s), got {}",
                pixels.len()
            ));
        }
        unsafe {
            read_pixels(
                x,
                y,
                width,
                height,
                format.as_raw(),
                GL_UNSIGNED_BYTE,
                pixels.as_mut_ptr().cast::<c_void>(),
            )
        };
        Ok(())
    }

    pub fn read_buffer(&self, buffer: GlFramebufferBuffer) -> Result<(), String> {
        let Some(read_buffer) = self.read_buffer else {
            return Err("read buffer selection is not available for this GL context".to_string());
        };
        unsafe { read_buffer(buffer.as_raw()?) };
        Ok(())
    }

    pub fn draw_buffers(&self, buffers: &[GlFramebufferBuffer]) -> Result<(), String> {
        let Some(draw_buffers) = self.draw_buffers else {
            return Err(
                "multiple draw buffer selection is not available for this GL context".to_string(),
            );
        };
        let (count, raw_buffers) = framebuffer_buffer_values(buffers, "draw buffer count")?;
        unsafe { draw_buffers(count, raw_buffers.as_ptr()) };
        Ok(())
    }

    pub fn blit_framebuffer(
        &self,
        source: GlRect,
        destination: GlRect,
        buffers: BitFlags<GlFramebufferBlitBuffer>,
        filter: GlFramebufferBlitFilter,
    ) -> Result<(), String> {
        let Some(blit_framebuffer) = self.blit_framebuffer else {
            return Err("framebuffer blit is not available for this GL context".to_string());
        };
        let ([src_x0, src_y0, src_x1, src_y1], [dst_x0, dst_y0, dst_x1, dst_y1], mask, filter) =
            framebuffer_blit_args(source, destination, buffers, filter)?;
        unsafe {
            blit_framebuffer(
                src_x0, src_y0, src_x1, src_y1, dst_x0, dst_y0, dst_x1, dst_y1, mask, filter,
            )
        };
        Ok(())
    }

    fn viewport_raw(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe { (self.viewport)(x, y, width, height) };
    }

    pub fn viewport(&self, rect: GlRect) -> Result<(), String> {
        let (x, y, width, height) = rect.as_gl_args("glViewport")?;
        self.viewport_raw(x, y, width, height);
        Ok(())
    }

    fn scissor_raw(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe { (self.scissor)(x, y, width, height) };
    }

    pub fn scissor(&self, rect: GlRect) -> Result<(), String> {
        let (x, y, width, height) = rect.as_gl_args("glScissor")?;
        self.scissor_raw(x, y, width, height);
        Ok(())
    }

    fn create_shader_raw(&self, shader_type: u32) -> u32 {
        unsafe { (self.create_shader)(shader_type) }
    }

    pub fn create_shader(&self, stage: GlShaderStage) -> Result<GlShader, String> {
        GlShader::from_nonzero(self.create_shader_raw(stage.as_raw()), stage)
    }

    pub fn compile_shader_source(
        &self,
        stage: GlShaderStage,
        source: &str,
    ) -> Result<GlShader, String> {
        let shader = self.create_shader(stage)?;
        if let Err(error) = self.shader_source(shader, source) {
            self.delete_shader(shader);
            return Err(error);
        }
        self.compile_shader(shader);

        let status = self.get_shader_iv(shader.as_raw(), GL_COMPILE_STATUS);
        if status == 0 {
            let log = self.get_shader_info_log(shader.as_raw());
            self.delete_shader(shader);
            return Err(format!("shader compile failed: {log}"));
        }

        if let Err(error) = self.check_no_error("glsym::compile_shader_source") {
            self.delete_shader(shader);
            return Err(error);
        }
        Ok(shader)
    }

    fn shader_source_raw_handle(&self, shader: u32, source: &str) -> Result<(), String> {
        let source = gl_string(source)?;
        self.shader_source_raw(shader, source.as_c_str());
        Ok(())
    }

    pub fn shader_source(&self, shader: GlShader, source: &str) -> Result<(), String> {
        self.shader_source_raw_handle(shader.as_raw(), source)
    }

    fn shader_source_raw(&self, shader: u32, source: &CStr) {
        let source_ptr = source.as_ptr();
        unsafe { (self.shader_source)(shader, 1, &source_ptr, std::ptr::null()) };
    }

    fn compile_shader_raw(&self, shader: u32) {
        unsafe { (self.compile_shader)(shader) };
    }

    pub fn compile_shader(&self, shader: GlShader) {
        self.compile_shader_raw(shader.as_raw());
    }

    fn get_shader_iv(&self, shader: u32, pname: u32) -> i32 {
        let mut value = 0;
        self.get_shader_iv_raw(shader, pname, &mut value);
        value
    }

    fn get_shader_iv_raw(&self, shader: u32, pname: u32, params: &mut i32) {
        unsafe { (self.get_shader_iv)(shader, pname, params) };
    }

    fn get_shader_info_log(&self, shader: u32) -> String {
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

    fn delete_shader_raw(&self, shader: u32) {
        unsafe { (self.delete_shader)(shader) };
    }

    pub fn delete_shader(&self, shader: GlShader) {
        self.delete_shader_raw(shader.as_raw());
    }

    fn create_program_raw(&self) -> u32 {
        unsafe { (self.create_program)() }
    }

    pub fn create_program(&self) -> Result<GlProgram, String> {
        GlProgram::from_nonzero(self.create_program_raw())
    }

    pub fn build_program(
        &self,
        vertex_source: &str,
        fragment_source: &str,
    ) -> Result<GlProgram, String> {
        let vertex_shader = self.compile_shader_source(GlShaderStage::Vertex, vertex_source)?;
        let fragment_shader =
            match self.compile_shader_source(GlShaderStage::Fragment, fragment_source) {
                Ok(shader) => shader,
                Err(error) => {
                    self.delete_shader(vertex_shader);
                    return Err(error);
                }
            };

        let program = match self.create_program() {
            Ok(program) => program,
            Err(error) => {
                self.delete_shader(vertex_shader);
                self.delete_shader(fragment_shader);
                return Err(error);
            }
        };
        self.attach_shader(program, vertex_shader);
        self.attach_shader(program, fragment_shader);
        let link_result = self.link_program(program);

        self.delete_shader(vertex_shader);
        self.delete_shader(fragment_shader);

        if let Err(error) = link_result {
            self.delete_program(program);
            return Err(error);
        }
        Ok(program)
    }

    fn attach_shader_raw(&self, program: u32, shader: u32) {
        unsafe { (self.attach_shader)(program, shader) };
    }

    pub fn attach_shader(&self, program: GlProgram, shader: GlShader) {
        self.attach_shader_raw(program.as_raw(), shader.as_raw());
    }

    fn link_program_raw(&self, program: u32) {
        unsafe { (self.link_program)(program) };
    }

    pub fn link_program(&self, program: GlProgram) -> Result<(), String> {
        self.link_program_raw(program.as_raw());
        let status = self.get_program_iv(program.as_raw(), GL_LINK_STATUS);
        if status == 0 {
            let log = self.get_program_info_log(program.as_raw());
            return Err(format!("program link failed: {log}"));
        }
        self.check_no_error("glsym::link_program")
    }

    fn get_program_iv(&self, program: u32, pname: u32) -> i32 {
        let mut value = 0;
        self.get_program_iv_raw(program, pname, &mut value);
        value
    }

    fn get_program_iv_raw(&self, program: u32, pname: u32, params: &mut i32) {
        unsafe { (self.get_program_iv)(program, pname, params) };
    }

    fn get_program_info_log(&self, program: u32) -> String {
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

    fn delete_program_raw(&self, program: u32) {
        unsafe { (self.delete_program)(program) };
    }

    pub fn delete_program(&self, program: GlProgram) {
        self.delete_program_raw(program.as_raw());
    }

    fn use_program_raw(&self, program: u32) {
        unsafe { (self.use_program)(program) };
    }

    pub fn use_no_program(&self) {
        self.use_program_raw(0);
    }

    pub fn use_program(&self, program: Option<GlProgram>) {
        self.use_program_raw(program.map_or(0, GlProgram::as_raw));
    }

    fn gen_buffer_raw(&self) -> u32 {
        let mut id = 0;
        unsafe { (self.gen_buffers)(1, &mut id) };
        id
    }

    pub fn gen_buffer(&self) -> Result<GlBuffer, String> {
        GlBuffer::from_nonzero(self.gen_buffer_raw())
    }

    fn bind_buffer_raw(&self, target: GlBufferTarget, buffer: u32) {
        unsafe { (self.bind_buffer)(target.as_raw(), buffer) };
    }

    pub fn unbind_buffer(&self, target: GlBufferTarget) {
        self.bind_buffer_raw(target, 0);
    }

    pub fn bind_buffer(&self, target: GlBufferTarget, buffer: Option<GlBuffer>) {
        self.bind_buffer_raw(target, buffer.map_or(0, GlBuffer::as_raw));
    }

    pub fn supports_indexed_buffer_bindings(&self) -> bool {
        self.bind_buffer_base.is_some() && self.bind_buffer_range.is_some()
    }

    pub fn bind_buffer_base(
        &self,
        target: GlIndexedBufferTarget,
        index: GlBufferBindingIndex,
        buffer: Option<GlBuffer>,
    ) -> Result<(), String> {
        let Some(bind_buffer_base) = self.bind_buffer_base else {
            return Err(
                "indexed buffer base binding is not available for this GL context".to_string(),
            );
        };
        unsafe {
            bind_buffer_base(
                target.as_raw(),
                index.as_raw(),
                buffer.map_or(0, GlBuffer::as_raw),
            );
        }
        Ok(())
    }

    pub fn bind_buffer_range(
        &self,
        target: GlIndexedBufferTarget,
        index: GlBufferBindingIndex,
        buffer: Option<GlBuffer>,
        range: GlBufferRange,
    ) -> Result<(), String> {
        let Some(bind_buffer_range) = self.bind_buffer_range else {
            return Err(
                "indexed buffer range binding is not available for this GL context".to_string(),
            );
        };
        let (offset, size) = range.as_gl_args("glBindBufferRange")?;
        unsafe {
            bind_buffer_range(
                target.as_raw(),
                index.as_raw(),
                buffer.map_or(0, GlBuffer::as_raw),
                offset,
                size,
            );
        }
        Ok(())
    }

    pub fn buffer_data<T>(
        &self,
        target: GlBufferTarget,
        data: &[T],
        usage: GlBufferUsage,
    ) -> Result<(), String> {
        let byte_len = GlBufferByteSize::from_bytes(std::mem::size_of_val(data))
            .as_isize("GL buffer upload")?;
        unsafe {
            self.buffer_data_raw(
                target.as_raw(),
                byte_len,
                data.as_ptr().cast::<c_void>(),
                usage.as_raw(),
            );
        }
        Ok(())
    }

    pub fn buffer_data_empty(
        &self,
        target: GlBufferTarget,
        byte_len: GlBufferByteSize,
        usage: GlBufferUsage,
    ) -> Result<(), String> {
        let byte_len = byte_len.as_isize("GL buffer allocation")?;
        unsafe {
            self.buffer_data_raw(target.as_raw(), byte_len, std::ptr::null(), usage.as_raw());
        }
        Ok(())
    }

    pub fn buffer_sub_data<T>(
        &self,
        target: GlBufferTarget,
        offset: GlBufferByteOffset,
        data: &[T],
    ) -> Result<(), String> {
        let byte_len = std::mem::size_of_val(data);
        self.buffer_sub_data_bytes_raw(target, offset, unsafe {
            std::slice::from_raw_parts(data.as_ptr().cast::<u8>(), byte_len)
        })
    }

    fn buffer_sub_data_bytes_raw(
        &self,
        target: GlBufferTarget,
        offset: GlBufferByteOffset,
        data: &[u8],
    ) -> Result<(), String> {
        let offset = offset.as_isize()?;
        let byte_len = isize::try_from(data.len()).map_err(|_| {
            format!(
                "GL buffer update byte length {} exceeds isize::MAX",
                data.len()
            )
        })?;
        unsafe {
            (self.buffer_sub_data)(
                target.as_raw(),
                offset,
                byte_len,
                data.as_ptr().cast::<c_void>(),
            );
        }
        Ok(())
    }

    pub fn copy_buffer_sub_data(
        &self,
        read_target: GlBufferTarget,
        write_target: GlBufferTarget,
        read_offset: GlBufferByteOffset,
        write_offset: GlBufferByteOffset,
        size: GlBufferByteSize,
    ) -> Result<(), String> {
        let Some(copy_buffer_sub_data) = self.copy_buffer_sub_data else {
            return Err("buffer copy is not available for this GL context".to_string());
        };
        let read_offset = read_offset.as_isize()?;
        let write_offset = write_offset.as_isize()?;
        let size = size.as_isize("glCopyBufferSubData")?;
        unsafe {
            copy_buffer_sub_data(
                read_target.as_raw(),
                write_target.as_raw(),
                read_offset,
                write_offset,
                size,
            );
        }
        Ok(())
    }

    unsafe fn buffer_data_raw(&self, target: u32, size: isize, data: *const c_void, usage: u32) {
        (self.buffer_data)(target, size, data, usage);
    }

    fn delete_buffer_raw(&self, id: u32) {
        unsafe { (self.delete_buffers)(1, &id) };
    }

    pub fn delete_buffer(&self, buffer: GlBuffer) {
        self.delete_buffer_raw(buffer.as_raw());
    }

    fn gen_texture_raw(&self) -> u32 {
        let mut id = 0;
        unsafe { (self.gen_textures)(1, &mut id) };
        id
    }

    pub fn gen_texture(&self) -> Result<GlTexture, String> {
        GlTexture::from_nonzero(self.gen_texture_raw())
    }

    fn bind_texture_raw(&self, target: GlTextureTarget, texture: u32) {
        unsafe { (self.bind_texture)(target.as_raw(), texture) };
    }

    pub fn unbind_texture(&self, target: GlTextureTarget) {
        self.bind_texture_raw(target, 0);
    }

    pub fn bind_texture(&self, target: GlTextureTarget, texture: Option<GlTexture>) {
        self.bind_texture_raw(target, texture.map_or(0, GlTexture::as_raw));
    }

    pub fn active_texture(&self, unit: GlTextureUnit) -> Result<(), String> {
        unsafe { (self.active_texture)(unit.as_raw()?) };
        Ok(())
    }

    pub fn tex_min_filter(&self, target: GlTextureTarget, filter: GlTextureMinFilter) {
        unsafe {
            (self.tex_parameter_i)(
                target.as_raw(),
                GL_TEXTURE_MIN_FILTER,
                filter.as_raw() as i32,
            )
        };
    }

    pub fn tex_mag_filter(&self, target: GlTextureTarget, filter: GlTextureMagFilter) {
        unsafe {
            (self.tex_parameter_i)(
                target.as_raw(),
                GL_TEXTURE_MAG_FILTER,
                filter.as_raw() as i32,
            )
        };
    }

    pub fn tex_wrap_s(&self, target: GlTextureTarget, wrap: GlTextureWrap) {
        unsafe { (self.tex_parameter_i)(target.as_raw(), GL_TEXTURE_WRAP_S, wrap.as_raw() as i32) };
    }

    pub fn tex_wrap_t(&self, target: GlTextureTarget, wrap: GlTextureWrap) {
        unsafe { (self.tex_parameter_i)(target.as_raw(), GL_TEXTURE_WRAP_T, wrap.as_raw() as i32) };
    }

    pub fn tex_wrap_r(&self, target: GlTextureTarget, wrap: GlTextureWrap) {
        unsafe { (self.tex_parameter_i)(target.as_raw(), GL_TEXTURE_WRAP_R, wrap.as_raw() as i32) };
    }

    pub fn pixel_store_unpack_alignment(&self, alignment: GlPixelStoreAlignment) {
        unsafe { (self.pixel_store_i)(GL_UNPACK_ALIGNMENT, alignment.as_raw()) };
    }

    pub fn pixel_store_pack_alignment(&self, alignment: GlPixelStoreAlignment) {
        unsafe { (self.pixel_store_i)(GL_PACK_ALIGNMENT, alignment.as_raw()) };
    }

    fn tex_image_2d_raw(
        &self,
        target: GlTextureTarget,
        internal_format: GlTextureInternalFormat,
        level: GlTextureLevel,
        size: GlTextureSize2D,
        format: GlTextureFormat,
        data_type: GlTextureDataType,
        bytes: Option<&[u8]>,
    ) -> Result<(), String> {
        let level = level.as_i32("glTexImage2D")?;
        let (width, height) = size.as_gl_args("glTexImage2D")?;
        let pixels = bytes
            .map(|bytes| bytes.as_ptr().cast::<c_void>())
            .unwrap_or(std::ptr::null());
        unsafe {
            (self.tex_image_2d)(
                target.as_raw(),
                level,
                internal_format.as_raw() as i32,
                width,
                height,
                0,
                format.as_raw(),
                data_type.as_raw(),
                pixels,
            );
        }
        Ok(())
    }

    pub fn tex_image_2d(
        &self,
        target: GlTextureTarget,
        internal_format: GlTextureInternalFormat,
        level: GlTextureLevel,
        size: GlTextureSize2D,
        format: GlTextureFormat,
        data_type: GlTextureDataType,
        bytes: Option<&[u8]>,
    ) -> Result<(), String> {
        self.tex_image_2d_raw(
            target,
            internal_format,
            level,
            size,
            format,
            data_type,
            bytes,
        )
    }

    pub fn tex_sub_image_2d(
        &self,
        target: GlTextureTarget,
        level: GlTextureLevel,
        offset: GlTextureOffset2D,
        size: GlTextureSize2D,
        format: GlTextureFormat,
        data_type: GlTextureDataType,
        bytes: &[u8],
    ) -> Result<(), String> {
        let level = level.as_i32("glTexSubImage2D")?;
        let (width, height) = size.as_gl_args("glTexSubImage2D")?;
        unsafe {
            (self.tex_sub_image_2d)(
                target.as_raw(),
                level,
                offset.x,
                offset.y,
                width,
                height,
                format.as_raw(),
                data_type.as_raw(),
                bytes.as_ptr().cast::<c_void>(),
            );
        }
        Ok(())
    }

    pub fn supports_texture_arrays(&self) -> bool {
        self.tex_image_3d.is_some()
            && self.tex_sub_image_3d.is_some()
            && supports_core_texture_arrays(self.context_type, self.version_info)
    }

    pub fn tex_image_3d(
        &self,
        target: GlTextureTarget,
        internal_format: GlTextureInternalFormat,
        level: GlTextureLevel,
        size: GlTextureSize3D,
        format: GlTextureFormat,
        data_type: GlTextureDataType,
        bytes: Option<&[u8]>,
    ) -> Result<(), String> {
        let Some(tex_image_3d) = self.tex_image_3d else {
            return Err("texture arrays are not available for this GL context".to_string());
        };
        let level = level.as_i32("glTexImage3D")?;
        let (width, height, depth) = size.as_gl_args("glTexImage3D")?;
        let pixels = bytes
            .map(|bytes| bytes.as_ptr().cast::<c_void>())
            .unwrap_or(std::ptr::null());
        unsafe {
            tex_image_3d(
                target.as_raw(),
                level,
                internal_format.as_raw() as i32,
                width,
                height,
                depth,
                0,
                format.as_raw(),
                data_type.as_raw(),
                pixels,
            );
        }
        Ok(())
    }

    pub fn tex_sub_image_3d(
        &self,
        target: GlTextureTarget,
        level: GlTextureLevel,
        offset: GlTextureOffset3D,
        size: GlTextureSize3D,
        format: GlTextureFormat,
        data_type: GlTextureDataType,
        bytes: &[u8],
    ) -> Result<(), String> {
        let Some(tex_sub_image_3d) = self.tex_sub_image_3d else {
            return Err("texture arrays are not available for this GL context".to_string());
        };
        let level = level.as_i32("glTexSubImage3D")?;
        let (width, height, depth) = size.as_gl_args("glTexSubImage3D")?;
        unsafe {
            tex_sub_image_3d(
                target.as_raw(),
                level,
                offset.x,
                offset.y,
                offset.z,
                width,
                height,
                depth,
                format.as_raw(),
                data_type.as_raw(),
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

    fn delete_texture_raw(&self, id: u32) {
        unsafe { (self.delete_textures)(1, &id) };
    }

    pub fn delete_texture(&self, texture: GlTexture) {
        self.delete_texture_raw(texture.as_raw());
    }

    pub fn supports_vertex_arrays(&self) -> bool {
        self.gen_vertex_arrays.is_some()
            && self.bind_vertex_array.is_some()
            && self.delete_vertex_arrays.is_some()
    }

    pub fn supports_instancing(&self) -> bool {
        self.vertex_attrib_divisor.is_some()
            && self.draw_arrays_instanced.is_some()
            && self.draw_elements_instanced.is_some()
            && supports_core_instancing(self.context_type, self.version_info)
    }

    fn gen_vertex_array_raw(&self) -> Result<u32, String> {
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

    pub fn gen_vertex_array(&self) -> Result<GlVertexArray, String> {
        GlVertexArray::from_nonzero(self.gen_vertex_array_raw()?)
    }

    fn bind_vertex_array_raw(&self, array: u32) -> Result<(), String> {
        let Some(bind_vertex_array) = self.bind_vertex_array else {
            return Err("vertex arrays are not available for this GL context".to_string());
        };

        unsafe { bind_vertex_array(array) };
        Ok(())
    }

    pub fn bind_vertex_array(&self, array: Option<GlVertexArray>) -> Result<(), String> {
        self.bind_vertex_array_raw(array.map_or(0, GlVertexArray::as_raw))
    }

    pub fn unbind_vertex_array(&self) -> Result<(), String> {
        self.bind_vertex_array(None)
    }

    fn delete_vertex_array_raw(&self, array: u32) -> Result<(), String> {
        let Some(delete_vertex_arrays) = self.delete_vertex_arrays else {
            return Err("vertex arrays are not available for this GL context".to_string());
        };

        unsafe { delete_vertex_arrays(1, &array) };
        Ok(())
    }

    pub fn delete_vertex_array(&self, array: GlVertexArray) -> Result<(), String> {
        self.delete_vertex_array_raw(array.as_raw())
    }

    fn enable_vertex_attrib_array_raw(&self, index: u32) {
        unsafe { (self.enable_vertex_attrib_array)(index) };
    }

    pub fn enable_vertex_attrib(&self, location: GlVertexAttribLocation) {
        self.enable_vertex_attrib_array_raw(location.as_raw());
    }

    fn disable_vertex_attrib_array_raw(&self, index: u32) {
        unsafe { (self.disable_vertex_attrib_array)(index) };
    }

    pub fn disable_vertex_attrib(&self, location: GlVertexAttribLocation) {
        self.disable_vertex_attrib_array_raw(location.as_raw());
    }

    fn vertex_attrib_pointer_f32_raw(
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

    fn vertex_attrib_pointer_f32_at(
        &self,
        location: GlVertexAttribLocation,
        size: i32,
        normalized: bool,
        stride: i32,
        offset: usize,
    ) {
        self.vertex_attrib_pointer_f32_raw(location.as_raw(), size, normalized, stride, offset);
    }

    pub fn vertex_attrib_pointer_f32(
        &self,
        location: GlVertexAttribLocation,
        layout: GlVertexAttribF32Layout,
    ) {
        self.vertex_attrib_pointer_f32_at(
            location,
            layout.components.as_gl_size(),
            layout.normalized,
            layout.stride.as_i32(),
            layout.offset.as_bytes(),
        );
    }

    fn vertex_attrib_divisor_raw(&self, index: u32, divisor: u32) -> Result<(), String> {
        let Some(vertex_attrib_divisor) = self.vertex_attrib_divisor else {
            return Err("instanced attributes are not available for this GL context".to_string());
        };

        unsafe { vertex_attrib_divisor(index, divisor) };
        Ok(())
    }

    pub fn vertex_attrib_divisor(
        &self,
        location: GlVertexAttribLocation,
        divisor: GlVertexAttribDivisor,
    ) -> Result<(), String> {
        self.vertex_attrib_divisor_raw(location.as_raw(), divisor.as_raw())
    }

    fn get_uniform_location(&self, program: u32, name: &str) -> Result<i32, String> {
        let name = gl_string(name)?;
        Ok(self.get_uniform_location_raw(program, name.as_c_str()))
    }

    fn uniform_location(
        &self,
        program: u32,
        name: &str,
    ) -> Result<Option<GlUniformLocation>, String> {
        self.get_uniform_location(program, name)
            .map(GlUniformLocation::from_raw)
    }

    fn required_uniform(&self, program: u32, name: &str) -> Result<GlUniformLocation, String> {
        self.uniform_location(program, name)?
            .ok_or_else(|| format!("shader linked without required active uniform {name}"))
    }

    pub fn required_uniform_location(
        &self,
        program: GlProgram,
        name: &str,
    ) -> Result<GlUniformLocation, String> {
        self.required_uniform(program.as_raw(), name)
    }

    fn get_uniform_location_raw(&self, program: u32, name: &CStr) -> i32 {
        unsafe { (self.get_uniform_location)(program, name.as_ptr()) }
    }

    fn get_attrib_location(&self, program: u32, name: &str) -> Result<i32, String> {
        let name = gl_string(name)?;
        Ok(self.get_attrib_location_raw(program, name.as_c_str()))
    }

    fn attrib_location(
        &self,
        program: u32,
        name: &str,
    ) -> Result<Option<GlVertexAttribLocation>, String> {
        self.get_attrib_location(program, name)
            .map(GlVertexAttribLocation::from_raw)
    }

    fn required_attrib(&self, program: u32, name: &str) -> Result<GlVertexAttribLocation, String> {
        self.attrib_location(program, name)?
            .ok_or_else(|| format!("shader linked without required active attribute {name}"))
    }

    pub fn required_attrib_location(
        &self,
        program: GlProgram,
        name: &str,
    ) -> Result<GlVertexAttribLocation, String> {
        self.required_attrib(program.as_raw(), name)
    }

    pub fn bind_attrib_location(
        &self,
        program: GlProgram,
        location: GlVertexAttribLocation,
        name: &str,
    ) -> Result<(), String> {
        let Some(bind_attrib_location) = self.bind_attrib_location else {
            return Err(
                "attribute location binding is not available for this GL context".to_string(),
            );
        };
        let name = gl_string(name)?;
        unsafe { bind_attrib_location(program.as_raw(), location.as_raw(), name.as_ptr()) };
        Ok(())
    }

    fn get_attrib_location_raw(&self, program: u32, name: &CStr) -> i32 {
        unsafe { (self.get_attrib_location)(program, name.as_ptr()) }
    }

    fn uniform_1f_raw(&self, location: i32, value: f32) {
        unsafe { (self.uniform_1f)(location, value) };
    }

    pub fn uniform_1f(&self, location: GlUniformLocation, value: f32) {
        self.uniform_1f_raw(location.as_raw(), value);
    }

    fn uniform_1i_raw(&self, location: i32, value: i32) {
        unsafe { (self.uniform_1i)(location, value) };
    }

    pub fn uniform_1i(&self, location: GlUniformLocation, value: i32) {
        self.uniform_1i_raw(location.as_raw(), value);
    }

    fn uniform_2f_raw(&self, location: i32, x: f32, y: f32) {
        unsafe { (self.uniform_2f)(location, x, y) };
    }

    pub fn uniform_2f(&self, location: GlUniformLocation, values: [f32; 2]) {
        self.uniform_2f_raw(location.as_raw(), values[0], values[1]);
    }

    fn uniform_3f_raw(&self, location: i32, values: [f32; 3]) {
        unsafe { (self.uniform_3f)(location, values[0], values[1], values[2]) };
    }

    pub fn uniform_3f(&self, location: GlUniformLocation, values: [f32; 3]) {
        self.uniform_3f_raw(location.as_raw(), values);
    }

    fn uniform_4f_raw(&self, location: i32, values: [f32; 4]) {
        unsafe { (self.uniform_4f)(location, values[0], values[1], values[2], values[3]) };
    }

    pub fn uniform_4f(&self, location: GlUniformLocation, values: [f32; 4]) {
        self.uniform_4f_raw(location.as_raw(), values);
    }

    fn uniform_4fv_raw(&self, location: i32, values: &[f32; 4]) {
        unsafe { (self.uniform_4fv)(location, 1, values.as_ptr()) };
    }

    pub fn uniform_4fv(&self, location: GlUniformLocation, values: &[f32; 4]) {
        self.uniform_4fv_raw(location.as_raw(), values);
    }

    fn uniform_matrix_3fv_raw(&self, location: i32, transpose: bool, values: &[f32; 9]) {
        unsafe {
            (self.uniform_matrix_3fv)(
                location,
                1,
                if transpose { 1 } else { GL_FALSE },
                values.as_ptr(),
            )
        };
    }

    pub fn uniform_matrix_3fv(
        &self,
        location: GlUniformLocation,
        transpose: bool,
        values: &[f32; 9],
    ) {
        self.uniform_matrix_3fv_raw(location.as_raw(), transpose, values);
    }

    fn uniform_matrix_4fv_raw(&self, location: i32, transpose: bool, values: &[f32; 16]) {
        unsafe {
            (self.uniform_matrix_4fv)(
                location,
                1,
                if transpose { 1 } else { GL_FALSE },
                values.as_ptr(),
            )
        };
    }

    pub fn uniform_matrix_4fv(
        &self,
        location: GlUniformLocation,
        transpose: bool,
        values: &[f32; 16],
    ) {
        self.uniform_matrix_4fv_raw(location.as_raw(), transpose, values);
    }

    fn draw_arrays_raw(&self, mode: GlDrawMode, first: i32, count: i32) {
        unsafe { (self.draw_arrays)(mode.as_raw(), first, count) };
    }

    pub fn draw_arrays(&self, mode: GlDrawMode, range: GlDrawRange) -> Result<(), String> {
        let (first, count) = range.as_gl_args("glDrawArrays")?;
        self.draw_arrays_raw(mode, first, count);
        Ok(())
    }

    fn draw_arrays_instanced_raw(
        &self,
        mode: GlDrawMode,
        first: i32,
        count: i32,
        instance_count: i32,
    ) -> Result<(), String> {
        let Some(draw_arrays_instanced) = self.draw_arrays_instanced else {
            return Err("instanced array draws are not available for this GL context".to_string());
        };
        unsafe {
            draw_arrays_instanced(mode.as_raw(), first, count, instance_count);
        }
        Ok(())
    }

    pub fn draw_arrays_instanced(
        &self,
        mode: GlDrawMode,
        range: GlDrawRange,
        instance_count: GlInstanceCount,
    ) -> Result<(), String> {
        let (first, count) = range.as_gl_args("glDrawArraysInstanced")?;
        let instance_count = instance_count.as_i32("glDrawArraysInstanced")?;
        self.draw_arrays_instanced_raw(mode, first, count, instance_count)
    }

    fn draw_elements_raw(
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

    pub fn draw_elements(
        &self,
        mode: GlDrawMode,
        index_type: GlIndexType,
        range: GlElementRange,
    ) -> Result<(), String> {
        let (count, offset) = range.as_gl_args("glDrawElements")?;
        self.draw_elements_raw(mode, count, index_type, offset);
        Ok(())
    }

    pub fn supports_draw_range_elements(&self) -> bool {
        self.draw_range_elements.is_some()
    }

    fn draw_range_elements_raw(
        &self,
        mode: GlDrawMode,
        vertex_range: GlElementVertexRange,
        count: i32,
        index_type: GlIndexType,
        offset: usize,
    ) -> Result<(), String> {
        let Some(draw_range_elements) = self.draw_range_elements else {
            return Err("bounded indexed draws are not available for this GL context".to_string());
        };
        let (start, end) = vertex_range.as_gl_args();
        unsafe {
            draw_range_elements(
                mode.as_raw(),
                start,
                end,
                count,
                index_type.as_raw(),
                offset as *const c_void,
            );
        }
        Ok(())
    }

    pub fn draw_range_elements(
        &self,
        mode: GlDrawMode,
        vertex_range: GlElementVertexRange,
        index_type: GlIndexType,
        range: GlElementRange,
    ) -> Result<(), String> {
        let (count, offset) = range.as_gl_args("glDrawRangeElements")?;
        self.draw_range_elements_raw(mode, vertex_range, count, index_type, offset)
    }

    fn draw_elements_instanced_raw(
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

    pub fn draw_elements_instanced(
        &self,
        mode: GlDrawMode,
        index_type: GlIndexType,
        range: GlElementRange,
        instance_count: GlInstanceCount,
    ) -> Result<(), String> {
        let (count, offset) = range.as_gl_args("glDrawElementsInstanced")?;
        let instance_count = instance_count.as_i32("glDrawElementsInstanced")?;
        self.draw_elements_instanced_raw(mode, count, index_type, offset, instance_count)
    }

    pub fn blend_func(&self, source: GlBlendFactor, destination: GlBlendFactor) {
        unsafe { (self.blend_func)(source.as_raw(), destination.as_raw()) };
    }

    pub fn blend_color(&self, r: f32, g: f32, b: f32, a: f32) -> Result<(), String> {
        let Some(blend_color) = self.blend_color else {
            return Err("constant blend color is not available for this GL context".to_string());
        };
        unsafe { blend_color(r, g, b, a) };
        Ok(())
    }

    pub fn blend_func_separate(
        &self,
        source_rgb: GlBlendFactor,
        destination_rgb: GlBlendFactor,
        source_alpha: GlBlendFactor,
        destination_alpha: GlBlendFactor,
    ) -> Result<(), String> {
        let Some(blend_func_separate) = self.blend_func_separate else {
            return Err("separate blend factors are not available for this GL context".to_string());
        };
        unsafe {
            blend_func_separate(
                source_rgb.as_raw(),
                destination_rgb.as_raw(),
                source_alpha.as_raw(),
                destination_alpha.as_raw(),
            )
        };
        Ok(())
    }

    pub fn blend_equation(&self, equation: GlBlendEquation) {
        unsafe { (self.blend_equation)(equation.as_raw()) };
    }

    pub fn blend_equation_separate(
        &self,
        rgb: GlBlendEquation,
        alpha: GlBlendEquation,
    ) -> Result<(), String> {
        let Some(blend_equation_separate) = self.blend_equation_separate else {
            return Err(
                "separate blend equations are not available for this GL context".to_string(),
            );
        };
        unsafe { blend_equation_separate(rgb.as_raw(), alpha.as_raw()) };
        Ok(())
    }

    pub fn supports_framebuffer_objects(&self) -> bool {
        self.gen_framebuffers.is_some()
            && self.delete_framebuffers.is_some()
            && self.framebuffer_texture_2d.is_some()
    }

    pub fn supports_renderbuffers(&self) -> bool {
        self.gen_renderbuffers.is_some()
            && self.bind_renderbuffer.is_some()
            && self.renderbuffer_storage.is_some()
            && self.delete_renderbuffers.is_some()
            && self.framebuffer_renderbuffer.is_some()
    }

    pub fn gen_framebuffer(&self) -> Result<GlFramebuffer, String> {
        let Some(gen_framebuffers) = self.gen_framebuffers else {
            return Err(
                "framebuffer object creation is not available for this GL context".to_string(),
            );
        };
        let mut id = 0;
        unsafe { gen_framebuffers(1, &mut id) };
        GlFramebuffer::from_nonzero(id)
    }

    pub fn delete_framebuffer(&self, framebuffer: GlFramebuffer) -> Result<(), String> {
        let Some(delete_framebuffers) = self.delete_framebuffers else {
            return Err(
                "framebuffer object deletion is not available for this GL context".to_string(),
            );
        };
        let id = framebuffer.as_raw();
        unsafe { delete_framebuffers(1, &id) };
        Ok(())
    }

    fn bind_framebuffer_raw(&self, target: GlFramebufferTarget, framebuffer: u32) {
        unsafe { (self.bind_framebuffer)(target.as_raw(), framebuffer) };
    }

    pub fn unbind_framebuffer(&self, target: GlFramebufferTarget) {
        self.bind_framebuffer_raw(target, 0);
    }

    fn bind_framebuffer_checked_raw(
        &self,
        target: GlFramebufferTarget,
        framebuffer: u32,
    ) -> Result<(), String> {
        self.bind_framebuffer_raw(target, framebuffer);
        self.check_no_error("glBindFramebuffer")?;
        if framebuffer != 0 {
            self.check_bound_framebuffer_complete(target)?;
        }
        Ok(())
    }

    pub fn bind_framebuffer(
        &self,
        target: GlFramebufferTarget,
        framebuffer: Option<GlFramebuffer>,
    ) -> Result<(), String> {
        self.bind_framebuffer_checked_raw(target, framebuffer.map_or(0, GlFramebuffer::as_raw))
    }

    pub fn framebuffer_texture_2d(
        &self,
        target: GlFramebufferTarget,
        attachment: GlFramebufferAttachment,
        texture_target: GlFramebufferTexture2DTarget,
        texture: Option<GlTexture>,
        level: GlTextureLevel,
    ) -> Result<(), String> {
        let Some(framebuffer_texture_2d) = self.framebuffer_texture_2d else {
            return Err(
                "2D framebuffer texture attachments are not available for this GL context"
                    .to_string(),
            );
        };
        unsafe {
            framebuffer_texture_2d(
                target.as_raw(),
                attachment.as_raw()?,
                texture_target.as_raw(),
                texture.map_or(0, GlTexture::as_raw),
                level.as_i32("glFramebufferTexture2D")?,
            );
        }
        Ok(())
    }

    pub fn gen_renderbuffer(&self) -> Result<GlRenderbuffer, String> {
        let Some(gen_renderbuffers) = self.gen_renderbuffers else {
            return Err("renderbuffer creation is not available for this GL context".to_string());
        };
        let mut id = 0;
        unsafe { gen_renderbuffers(1, &mut id) };
        GlRenderbuffer::from_nonzero(id)
    }

    pub fn bind_renderbuffer(
        &self,
        target: GlRenderbufferTarget,
        renderbuffer: Option<GlRenderbuffer>,
    ) -> Result<(), String> {
        let Some(bind_renderbuffer) = self.bind_renderbuffer else {
            return Err("renderbuffer binding is not available for this GL context".to_string());
        };
        unsafe {
            bind_renderbuffer(
                target.as_raw(),
                renderbuffer.map_or(0, GlRenderbuffer::as_raw),
            )
        };
        Ok(())
    }

    pub fn delete_renderbuffer(&self, renderbuffer: GlRenderbuffer) -> Result<(), String> {
        let Some(delete_renderbuffers) = self.delete_renderbuffers else {
            return Err("renderbuffer deletion is not available for this GL context".to_string());
        };
        let id = renderbuffer.as_raw();
        unsafe { delete_renderbuffers(1, &id) };
        Ok(())
    }

    pub fn renderbuffer_storage(
        &self,
        target: GlRenderbufferTarget,
        internal_format: GlRenderbufferInternalFormat,
        size: GlRenderbufferSize,
    ) -> Result<(), String> {
        let Some(renderbuffer_storage) = self.renderbuffer_storage else {
            return Err("renderbuffer storage is not available for this GL context".to_string());
        };
        let (width, height) = size.as_gl_args()?;
        unsafe { renderbuffer_storage(target.as_raw(), internal_format.as_raw(), width, height) };
        Ok(())
    }

    pub fn framebuffer_renderbuffer(
        &self,
        target: GlFramebufferTarget,
        attachment: GlFramebufferAttachment,
        renderbuffer_target: GlRenderbufferTarget,
        renderbuffer: Option<GlRenderbuffer>,
    ) -> Result<(), String> {
        let Some(framebuffer_renderbuffer) = self.framebuffer_renderbuffer else {
            return Err(
                "framebuffer renderbuffer attachments are not available for this GL context"
                    .to_string(),
            );
        };
        unsafe {
            framebuffer_renderbuffer(
                target.as_raw(),
                attachment.as_raw()?,
                renderbuffer_target.as_raw(),
                renderbuffer.map_or(0, GlRenderbuffer::as_raw),
            );
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
            depth_func: Some(fake_gl_depth_func),
            depth_mask: Some(fake_gl_depth_mask),
            depth_range_f: Some(fake_gl_depth_range_f),
            cull_face: Some(fake_gl_cull_face),
            front_face: Some(fake_gl_front_face),
            stencil_func: Some(fake_gl_stencil_func),
            stencil_mask: Some(fake_gl_stencil_mask),
            stencil_op: Some(fake_gl_stencil_op),
            stencil_func_separate: Some(fake_gl_stencil_func_separate),
            stencil_mask_separate: Some(fake_gl_stencil_mask_separate),
            stencil_op_separate: Some(fake_gl_stencil_op_separate),
            color_mask: Some(fake_gl_color_mask),
            polygon_offset: Some(fake_gl_polygon_offset),
            gen_queries: Some(fake_gl_gen_queries),
            delete_queries: Some(fake_gl_delete_queries),
            begin_query: Some(fake_gl_begin_query),
            end_query: Some(fake_gl_end_query),
            get_query_object_uiv: Some(fake_gl_get_query_object_uiv),
            fence_sync: Some(fake_gl_fence_sync),
            client_wait_sync: Some(fake_gl_client_wait_sync),
            wait_sync: Some(fake_gl_wait_sync),
            delete_sync: Some(fake_gl_delete_sync),
            read_pixels: Some(fake_gl_read_pixels),
            read_buffer: Some(fake_gl_read_buffer),
            draw_buffers: Some(fake_gl_draw_buffers),
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
            bind_buffer_base: Some(fake_gl_bind_buffer_base),
            bind_buffer_range: Some(fake_gl_bind_buffer_range),
            buffer_data: fake_gl_buffer_data,
            buffer_sub_data: fake_gl_buffer_sub_data,
            copy_buffer_sub_data: Some(fake_gl_copy_buffer_sub_data),
            delete_buffers: fake_gl_delete_buffers,
            gen_textures: fake_gl_gen_textures,
            bind_texture: fake_gl_bind_texture,
            active_texture: fake_gl_active_texture,
            tex_parameter_i: fake_gl_tex_parameter_i,
            pixel_store_i: fake_gl_pixel_store_i,
            tex_image_2d: fake_gl_tex_image_2d,
            tex_sub_image_2d: fake_gl_tex_sub_image_2d,
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
            bind_attrib_location: Some(fake_gl_bind_attrib_location),
            uniform_1i: fake_gl_uniform_1i,
            uniform_1f: fake_gl_uniform_1f,
            uniform_2f: fake_gl_uniform_2f,
            uniform_3f: fake_gl_uniform_3f,
            uniform_4f: fake_gl_uniform_4f,
            uniform_4fv: fake_gl_uniform_4fv,
            uniform_matrix_3fv: fake_gl_uniform_matrix_3fv,
            uniform_matrix_4fv: fake_gl_uniform_matrix_4fv,
            draw_arrays: fake_gl_draw_arrays,
            draw_arrays_instanced: state
                .config
                .supports_instancing
                .then_some(fake_gl_draw_arrays_instanced),
            draw_elements: fake_gl_draw_elements,
            draw_range_elements: Some(fake_gl_draw_range_elements),
            draw_elements_instanced: state
                .config
                .supports_instancing
                .then_some(fake_gl_draw_elements_instanced),
            blend_color: Some(fake_gl_blend_color),
            blend_func: fake_gl_blend_func,
            blend_func_separate: Some(fake_gl_blend_func_separate),
            blend_equation: fake_gl_blend_equation,
            blend_equation_separate: Some(fake_gl_blend_equation_separate),
            gen_framebuffers: Some(fake_gl_gen_framebuffers),
            bind_framebuffer: fake_gl_bind_framebuffer,
            delete_framebuffers: Some(fake_gl_delete_framebuffers),
            framebuffer_texture_2d: Some(fake_gl_framebuffer_texture_2d),
            gen_renderbuffers: Some(fake_gl_gen_renderbuffers),
            bind_renderbuffer: Some(fake_gl_bind_renderbuffer),
            renderbuffer_storage: Some(fake_gl_renderbuffer_storage),
            delete_renderbuffers: Some(fake_gl_delete_renderbuffers),
            framebuffer_renderbuffer: Some(fake_gl_framebuffer_renderbuffer),
            blit_framebuffer: Some(fake_gl_blit_framebuffer),
            get_error: Some(fake_gl_get_error),
            check_framebuffer_status: Some(fake_gl_check_framebuffer_status),
            invalidate_framebuffer: state
                .config
                .version_info
                .version_at_least(3, 0)
                .then_some(fake_gl_invalidate_framebuffer),
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

fn load_optional_gl_symbol_aliases<T>(
    runtime: &Runtime<'_>,
    names: &[&str],
) -> Result<Option<T>, String>
where
    T: Copy,
{
    for name in names {
        if let Some(symbol) = load_optional_gl_symbol(runtime, name)? {
            return Ok(Some(symbol));
        }
    }
    Ok(None)
}

fn gl_string(value: &str) -> Result<CString, String> {
    CString::new(value).map_err(|_| format!("GL strings cannot contain interior NULs: {value:?}"))
}

fn read_pixels_len(rect: GlRect, format: GlTextureFormat) -> Result<usize, String> {
    let channels = match format {
        GlTextureFormat::Red | GlTextureFormat::Luminance => 1usize,
        GlTextureFormat::Rgb => 3,
        GlTextureFormat::Rgba => 4,
    };
    let pixels = usize::try_from(rect.width)
        .ok()
        .and_then(|width| {
            usize::try_from(rect.height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .ok_or_else(|| "glReadPixels dimensions exceed usize::MAX".to_string())?;
    pixels
        .checked_mul(channels)
        .ok_or_else(|| "glReadPixels byte length exceeds usize::MAX".to_string())
}

fn framebuffer_buffer_values(
    buffers: &[GlFramebufferBuffer],
    count_label: &str,
) -> Result<(i32, Vec<u32>), String> {
    let count = i32::try_from(buffers.len())
        .map_err(|_| format!("{count_label} {} exceeds GLsizei::MAX", buffers.len()))?;
    let raw_buffers = buffers
        .iter()
        .copied()
        .map(GlFramebufferBuffer::as_raw)
        .collect::<Result<Vec<_>, _>>()?;
    Ok((count, raw_buffers))
}

fn framebuffer_blit_rect_endpoints(rect: GlRect, label: &str) -> Result<[i32; 4], String> {
    let width = glsizei_from_u32(rect.width, &format!("{label} width"))?;
    let height = glsizei_from_u32(rect.height, &format!("{label} height"))?;
    let x1 = rect
        .x
        .checked_add(width)
        .ok_or_else(|| format!("{label} x endpoint overflows GLint"))?;
    let y1 = rect
        .y
        .checked_add(height)
        .ok_or_else(|| format!("{label} y endpoint overflows GLint"))?;
    Ok([rect.x, rect.y, x1, y1])
}

fn framebuffer_blit_args(
    source: GlRect,
    destination: GlRect,
    buffers: BitFlags<GlFramebufferBlitBuffer>,
    filter: GlFramebufferBlitFilter,
) -> Result<([i32; 4], [i32; 4], u32, u32), String> {
    if buffers.bits() == 0 {
        return Err("framebuffer blit requires at least one buffer".to_string());
    }
    let depth_or_stencil = buffers.bits() & (GL_DEPTH_BUFFER_BIT | GL_STENCIL_BUFFER_BIT) != 0;
    if filter == GlFramebufferBlitFilter::Linear && depth_or_stencil {
        return Err("framebuffer depth or stencil blits require nearest filtering".to_string());
    }
    let source = framebuffer_blit_rect_endpoints(source, "source framebuffer blit rectangle")?;
    let destination =
        framebuffer_blit_rect_endpoints(destination, "destination framebuffer blit rectangle")?;
    Ok((source, destination, buffers.bits(), filter.as_raw()))
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeTextureSubImage2D {
    pub target: u32,
    pub level: i32,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub format: u32,
    pub type_: u32,
    pub has_pixels: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeBufferDataCall {
    pub target: u32,
    pub byte_len: usize,
    pub usage: u32,
    pub has_data: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeBufferSubDataCall {
    pub target: u32,
    pub offset: usize,
    pub byte_len: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeCopyBufferSubDataCall {
    pub read_target: u32,
    pub write_target: u32,
    pub read_offset: usize,
    pub write_offset: usize,
    pub byte_len: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeBindBufferBaseCall {
    pub target: u32,
    pub index: u32,
    pub buffer: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeBindBufferRangeCall {
    pub target: u32,
    pub index: u32,
    pub buffer: u32,
    pub offset: usize,
    pub size: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeFramebufferTexture2DCall {
    pub target: u32,
    pub attachment: u32,
    pub texture_target: u32,
    pub texture: u32,
    pub level: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeRenderbufferStorageCall {
    pub target: u32,
    pub internal_format: u32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeFramebufferRenderbufferCall {
    pub target: u32,
    pub attachment: u32,
    pub renderbuffer_target: u32,
    pub renderbuffer: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeBlitFramebufferCall {
    pub source: [i32; 4],
    pub destination: [i32; 4],
    pub mask: u32,
    pub filter: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeUniformFloatCall {
    pub location: i32,
    pub values: [u32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeUniformIntCall {
    pub location: i32,
    pub value: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeUniformMatrix3Call {
    pub location: i32,
    pub transpose: bool,
    pub values: [u32; 9],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeStencilFuncCall {
    pub function: u32,
    pub reference: i32,
    pub mask: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeStencilOpCall {
    pub stencil_fail: u32,
    pub depth_fail: u32,
    pub depth_pass: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeStencilFuncSeparateCall {
    pub face: u32,
    pub function: u32,
    pub reference: i32,
    pub mask: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeStencilMaskSeparateCall {
    pub face: u32,
    pub mask: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeStencilOpSeparateCall {
    pub face: u32,
    pub stencil_fail: u32,
    pub depth_fail: u32,
    pub depth_pass: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeQueryObjectCall {
    pub query: u32,
    pub property: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeClientWaitSyncCall {
    pub sync: usize,
    pub flags: u32,
    pub timeout_nanos: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeWaitSyncCall {
    pub sync: usize,
    pub flags: u32,
    pub timeout_nanos: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeCreateShaderCall {
    pub stage: u32,
    pub shader: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeAttachShaderCall {
    pub program: u32,
    pub shader: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FakeBindAttribLocationCall {
    pub program: u32,
    pub location: u32,
    pub name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeReadPixelsCall {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub format: u32,
    pub type_: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeDrawArraysCall {
    pub mode: u32,
    pub first: i32,
    pub count: i32,
    pub instance_count: Option<i32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeDrawElementsCall {
    pub mode: u32,
    pub vertex_range: Option<(u32, u32)>,
    pub count: i32,
    pub type_: u32,
    pub offset: usize,
    pub instance_count: Option<i32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeVertexAttribPointerCall {
    pub index: u32,
    pub size: i32,
    pub type_: u32,
    pub normalized: bool,
    pub stride: i32,
    pub offset: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeBlendFuncSeparateCall {
    pub source_rgb: u32,
    pub destination_rgb: u32,
    pub source_alpha: u32,
    pub destination_alpha: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeBlendEquationSeparateCall {
    pub rgb: u32,
    pub alpha: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeBlendColorCall {
    pub color: [u32; 4],
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FakeGlSnapshot {
    pub clear_calls: usize,
    pub current_clear_color: [u32; 4],
    pub clear_colors: Vec<[u32; 4]>,
    pub clear_scissor_enabled: Vec<bool>,
    pub draw_arrays_calls: usize,
    pub draw_arrays_call_args: Vec<FakeDrawArraysCall>,
    pub draw_elements_calls: usize,
    pub draw_elements_instanced_calls: usize,
    pub draw_elements_call_args: Vec<FakeDrawElementsCall>,
    pub blend_color_calls: Vec<FakeBlendColorCall>,
    pub blend_func_separate_calls: Vec<FakeBlendFuncSeparateCall>,
    pub blend_equation_separate_calls: Vec<FakeBlendEquationSeparateCall>,
    pub buffer_data_calls: usize,
    pub buffer_data_bytes: usize,
    pub buffer_data_uploads: Vec<FakeBufferDataCall>,
    pub buffer_sub_data_calls: Vec<FakeBufferSubDataCall>,
    pub copy_buffer_sub_data_calls: Vec<FakeCopyBufferSubDataCall>,
    pub bind_buffer_base_calls: Vec<FakeBindBufferBaseCall>,
    pub bind_buffer_range_calls: Vec<FakeBindBufferRangeCall>,
    pub created_shaders: Vec<FakeCreateShaderCall>,
    pub attached_shaders: Vec<FakeAttachShaderCall>,
    pub bind_attrib_location_calls: Vec<FakeBindAttribLocationCall>,
    pub linked_programs: Vec<u32>,
    pub deleted_buffers: Vec<u32>,
    pub deleted_shaders: Vec<u32>,
    pub deleted_programs: Vec<u32>,
    pub texture_parameter_calls: Vec<FakeTextureParameterCall>,
    pub texture_uploads_2d: Vec<FakeTextureUpload2D>,
    pub texture_sub_images_2d: Vec<FakeTextureSubImage2D>,
    pub texture_uploads_3d_calls: usize,
    pub generate_mipmap_calls: Vec<u32>,
    pub deleted_textures: Vec<u32>,
    pub current_program: u32,
    pub use_program_calls: Vec<u32>,
    pub uniform_1i_calls: Vec<FakeUniformIntCall>,
    pub uniform_3f_calls: Vec<FakeUniformFloatCall>,
    pub uniform_4f_calls: Vec<FakeUniformFloatCall>,
    pub uniform_4fv_calls: Vec<FakeUniformFloatCall>,
    pub uniform_matrix_3fv_calls: Vec<FakeUniformMatrix3Call>,
    pub bound_array_buffer: u32,
    pub bound_element_array_buffer: u32,
    pub bound_vertex_array: u32,
    pub vertex_array_bindings: Vec<u32>,
    pub deleted_vertex_arrays: Vec<u32>,
    pub bound_framebuffer: u32,
    pub generated_framebuffers: Vec<u32>,
    pub deleted_framebuffers: Vec<u32>,
    pub framebuffer_bindings: Vec<u32>,
    pub framebuffer_texture_2d_calls: Vec<FakeFramebufferTexture2DCall>,
    pub framebuffer_renderbuffer_calls: Vec<FakeFramebufferRenderbufferCall>,
    pub blit_framebuffer_calls: Vec<FakeBlitFramebufferCall>,
    pub framebuffer_invalidations: Vec<Vec<u32>>,
    pub bound_renderbuffer: u32,
    pub generated_renderbuffers: Vec<u32>,
    pub deleted_renderbuffers: Vec<u32>,
    pub renderbuffer_bindings: Vec<u32>,
    pub renderbuffer_storage_calls: Vec<FakeRenderbufferStorageCall>,
    pub viewport_calls: Vec<(i32, i32, i32, i32)>,
    pub pack_alignment: i32,
    pub pack_alignment_calls: Vec<i32>,
    pub unpack_alignment: i32,
    pub unpack_alignment_calls: Vec<i32>,
    pub active_texture: u32,
    pub bound_texture_2d: u32,
    pub bound_texture_2d_units: Vec<(u32, u32)>,
    pub bound_texture_2d_array: u32,
    pub bound_texture_2d_array_units: Vec<(u32, u32)>,
    pub blend_enabled: bool,
    pub scissor_enabled: bool,
    pub enabled_capabilities: Vec<u32>,
    pub depth_function: Option<u32>,
    pub depth_mask: bool,
    pub depth_range_f_calls: Vec<[u32; 2]>,
    pub cull_face_mode: Option<u32>,
    pub front_face_winding: Option<u32>,
    pub stencil_func: Option<FakeStencilFuncCall>,
    pub stencil_mask: u32,
    pub stencil_op: Option<FakeStencilOpCall>,
    pub stencil_func_separate_calls: Vec<FakeStencilFuncSeparateCall>,
    pub stencil_mask_separate_calls: Vec<FakeStencilMaskSeparateCall>,
    pub stencil_op_separate_calls: Vec<FakeStencilOpSeparateCall>,
    pub color_write_mask: [bool; 4],
    pub polygon_offset: Option<[u32; 2]>,
    pub generated_queries: Vec<u32>,
    pub deleted_queries: Vec<u32>,
    pub begin_query_calls: Vec<(u32, u32)>,
    pub end_query_calls: Vec<u32>,
    pub query_object_uiv_calls: Vec<FakeQueryObjectCall>,
    pub generated_syncs: Vec<usize>,
    pub client_wait_sync_calls: Vec<FakeClientWaitSyncCall>,
    pub wait_sync_calls: Vec<FakeWaitSyncCall>,
    pub deleted_syncs: Vec<usize>,
    pub read_pixels_calls: Vec<FakeReadPixelsCall>,
    pub read_buffer_calls: Vec<u32>,
    pub draw_buffers_calls: Vec<Vec<u32>>,
    pub scissor_calls: Vec<(i32, i32, i32, i32)>,
    pub enabled_vertex_attribs: Vec<u32>,
    pub vertex_attrib_pointer_calls: Vec<FakeVertexAttribPointerCall>,
}

struct FakeGlState {
    config: FakeGlConfig,
    vendor_string: CString,
    renderer_string: CString,
    version_string: CString,
    extensions_string: CString,
    indexed_extensions: Vec<CString>,
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
        let indexed_extensions = fake_indexed_extensions(&config.extensions_string);
        Self {
            config,
            vendor_string,
            renderer_string,
            version_string,
            extensions_string,
            indexed_extensions,
            next_id: 1,
            snapshot: FakeGlState::default_snapshot(),
        }
    }
}

impl FakeGlState {
    fn default_snapshot() -> FakeGlSnapshot {
        FakeGlSnapshot {
            pack_alignment: 4,
            unpack_alignment: 4,
            depth_mask: true,
            stencil_mask: u32::MAX,
            color_write_mask: [true; 4],
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
        self.indexed_extensions = fake_indexed_extensions(&config.extensions_string);
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

fn fake_indexed_extensions(extensions: &str) -> Vec<CString> {
    extensions
        .split_whitespace()
        .map(|extension| CString::new(extension).expect("fake GL extension string"))
        .collect()
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
        "glGetStringi" => Some(mem::transmute::<GlGetStringi, unsafe extern "C" fn()>(
            fake_gl_get_string_i,
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
        "glDepthFunc" => Some(mem::transmute::<GlDepthFunc, unsafe extern "C" fn()>(
            fake_gl_depth_func,
        )),
        "glDepthMask" => Some(mem::transmute::<GlDepthMask, unsafe extern "C" fn()>(
            fake_gl_depth_mask,
        )),
        "glDepthRangef" => Some(mem::transmute::<GlDepthRangef, unsafe extern "C" fn()>(
            fake_gl_depth_range_f,
        )),
        "glCullFace" => Some(mem::transmute::<GlCullFace, unsafe extern "C" fn()>(
            fake_gl_cull_face,
        )),
        "glFrontFace" => Some(mem::transmute::<GlFrontFace, unsafe extern "C" fn()>(
            fake_gl_front_face,
        )),
        "glStencilFunc" => Some(mem::transmute::<GlStencilFunc, unsafe extern "C" fn()>(
            fake_gl_stencil_func,
        )),
        "glStencilMask" => Some(mem::transmute::<GlStencilMaskFn, unsafe extern "C" fn()>(
            fake_gl_stencil_mask,
        )),
        "glStencilOp" => Some(mem::transmute::<GlStencilOp, unsafe extern "C" fn()>(
            fake_gl_stencil_op,
        )),
        "glStencilFuncSeparate" => Some(mem::transmute::<
            GlStencilFuncSeparate,
            unsafe extern "C" fn(),
        >(fake_gl_stencil_func_separate)),
        "glStencilMaskSeparate" => Some(mem::transmute::<
            GlStencilMaskSeparate,
            unsafe extern "C" fn(),
        >(fake_gl_stencil_mask_separate)),
        "glStencilOpSeparate" => Some(
            mem::transmute::<GlStencilOpSeparate, unsafe extern "C" fn()>(
                fake_gl_stencil_op_separate,
            ),
        ),
        "glColorMask" => Some(mem::transmute::<GlColorMaskFn, unsafe extern "C" fn()>(
            fake_gl_color_mask,
        )),
        "glPolygonOffset" => Some(mem::transmute::<GlPolygonOffsetFn, unsafe extern "C" fn()>(
            fake_gl_polygon_offset,
        )),
        "glGenQueries" => Some(mem::transmute::<GlGenQueries, unsafe extern "C" fn()>(
            fake_gl_gen_queries,
        )),
        "glDeleteQueries" => Some(mem::transmute::<GlDeleteQueries, unsafe extern "C" fn()>(
            fake_gl_delete_queries,
        )),
        "glBeginQuery" => Some(mem::transmute::<GlBeginQuery, unsafe extern "C" fn()>(
            fake_gl_begin_query,
        )),
        "glEndQuery" => Some(mem::transmute::<GlEndQuery, unsafe extern "C" fn()>(
            fake_gl_end_query,
        )),
        "glGetQueryObjectuiv" => Some(
            mem::transmute::<GlGetQueryObjectuiv, unsafe extern "C" fn()>(
                fake_gl_get_query_object_uiv,
            ),
        ),
        "glFenceSync" => Some(mem::transmute::<GlFenceSync, unsafe extern "C" fn()>(
            fake_gl_fence_sync,
        )),
        "glClientWaitSync" => Some(mem::transmute::<GlClientWaitSync, unsafe extern "C" fn()>(
            fake_gl_client_wait_sync,
        )),
        "glWaitSync" => Some(mem::transmute::<GlWaitSync, unsafe extern "C" fn()>(
            fake_gl_wait_sync,
        )),
        "glDeleteSync" => Some(mem::transmute::<GlDeleteSync, unsafe extern "C" fn()>(
            fake_gl_delete_sync,
        )),
        "glReadPixels" => Some(mem::transmute::<GlReadPixels, unsafe extern "C" fn()>(
            fake_gl_read_pixels,
        )),
        "glReadBuffer" => Some(mem::transmute::<GlReadBuffer, unsafe extern "C" fn()>(
            fake_gl_read_buffer,
        )),
        "glDrawBuffers" => Some(mem::transmute::<GlDrawBuffers, unsafe extern "C" fn()>(
            fake_gl_draw_buffers,
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
        "glBindBufferBase" => Some(mem::transmute::<GlBindBufferBase, unsafe extern "C" fn()>(
            fake_gl_bind_buffer_base,
        )),
        "glBindBufferRange" => Some(mem::transmute::<GlBindBufferRange, unsafe extern "C" fn()>(
            fake_gl_bind_buffer_range,
        )),
        "glBufferData" => Some(mem::transmute::<GlBufferData, unsafe extern "C" fn()>(
            fake_gl_buffer_data,
        )),
        "glBufferSubData" => Some(mem::transmute::<GlBufferSubData, unsafe extern "C" fn()>(
            fake_gl_buffer_sub_data,
        )),
        "glCopyBufferSubData" => Some(
            mem::transmute::<GlCopyBufferSubData, unsafe extern "C" fn()>(
                fake_gl_copy_buffer_sub_data,
            ),
        ),
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
        "glTexSubImage2D" => Some(mem::transmute::<GlTexSubImage2D, unsafe extern "C" fn()>(
            fake_gl_tex_sub_image_2d,
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
                GlVertexAttribDivisorFn,
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
        "glBindAttribLocation" => Some(mem::transmute::<
            GlBindAttribLocation,
            unsafe extern "C" fn(),
        >(fake_gl_bind_attrib_location)),
        "glUniform1i" => Some(mem::transmute::<GlUniform1i, unsafe extern "C" fn()>(
            fake_gl_uniform_1i,
        )),
        "glUniform1f" => Some(mem::transmute::<GlUniform1f, unsafe extern "C" fn()>(
            fake_gl_uniform_1f,
        )),
        "glUniform2f" => Some(mem::transmute::<GlUniform2f, unsafe extern "C" fn()>(
            fake_gl_uniform_2f,
        )),
        "glUniform3f" => Some(mem::transmute::<GlUniform3f, unsafe extern "C" fn()>(
            fake_gl_uniform_3f,
        )),
        "glUniform4f" => Some(mem::transmute::<GlUniform4f, unsafe extern "C" fn()>(
            fake_gl_uniform_4f,
        )),
        "glUniform4fv" => Some(mem::transmute::<GlUniform4fv, unsafe extern "C" fn()>(
            fake_gl_uniform_4fv,
        )),
        "glUniformMatrix3fv" => Some(
            mem::transmute::<GlUniformMatrix3fv, unsafe extern "C" fn()>(
                fake_gl_uniform_matrix_3fv,
            ),
        ),
        "glUniformMatrix4fv" => Some(
            mem::transmute::<GlUniformMatrix4fv, unsafe extern "C" fn()>(
                fake_gl_uniform_matrix_4fv,
            ),
        ),
        "glDrawArrays" => Some(mem::transmute::<GlDrawArrays, unsafe extern "C" fn()>(
            fake_gl_draw_arrays,
        )),
        "glDrawArraysInstanced" => fake_gl_state()
            .lock()
            .expect("fake GL state mutex poisoned")
            .config
            .supports_instancing
            .then_some(mem::transmute::<
                GlDrawArraysInstanced,
                unsafe extern "C" fn(),
            >(fake_gl_draw_arrays_instanced)),
        "glDrawElements" => Some(mem::transmute::<GlDrawElements, unsafe extern "C" fn()>(
            fake_gl_draw_elements,
        )),
        "glDrawRangeElements" => Some(
            mem::transmute::<GlDrawRangeElements, unsafe extern "C" fn()>(
                fake_gl_draw_range_elements,
            ),
        ),
        "glDrawElementsInstanced" => fake_gl_state()
            .lock()
            .expect("fake GL state mutex poisoned")
            .config
            .supports_instancing
            .then_some(mem::transmute::<
                GlDrawElementsInstanced,
                unsafe extern "C" fn(),
            >(fake_gl_draw_elements_instanced)),
        "glBlendColor" => Some(mem::transmute::<GlBlendColor, unsafe extern "C" fn()>(
            fake_gl_blend_color,
        )),
        "glBlendFunc" => Some(mem::transmute::<GlBlendFunc, unsafe extern "C" fn()>(
            fake_gl_blend_func,
        )),
        "glBlendFuncSeparate" => Some(
            mem::transmute::<GlBlendFuncSeparate, unsafe extern "C" fn()>(
                fake_gl_blend_func_separate,
            ),
        ),
        "glBlendEquation" => Some(mem::transmute::<GlBlendEquationFn, unsafe extern "C" fn()>(
            fake_gl_blend_equation,
        )),
        "glBlendEquationSeparate" => Some(mem::transmute::<
            GlBlendEquationSeparate,
            unsafe extern "C" fn(),
        >(fake_gl_blend_equation_separate)),
        "glGenFramebuffers" => Some(mem::transmute::<GlGenFramebuffers, unsafe extern "C" fn()>(
            fake_gl_gen_framebuffers,
        )),
        "glBindFramebuffer" => Some(mem::transmute::<GlBindFramebuffer, unsafe extern "C" fn()>(
            fake_gl_bind_framebuffer,
        )),
        "glDeleteFramebuffers" => Some(mem::transmute::<
            GlDeleteFramebuffers,
            unsafe extern "C" fn(),
        >(fake_gl_delete_framebuffers)),
        "glFramebufferTexture2D" => Some(mem::transmute::<
            GlFramebufferTexture2D,
            unsafe extern "C" fn(),
        >(fake_gl_framebuffer_texture_2d)),
        "glGenRenderbuffers" => Some(
            mem::transmute::<GlGenRenderbuffers, unsafe extern "C" fn()>(fake_gl_gen_renderbuffers),
        ),
        "glBindRenderbuffer" => Some(
            mem::transmute::<GlBindRenderbuffer, unsafe extern "C" fn()>(fake_gl_bind_renderbuffer),
        ),
        "glRenderbufferStorage" => Some(mem::transmute::<
            GlRenderbufferStorage,
            unsafe extern "C" fn(),
        >(fake_gl_renderbuffer_storage)),
        "glDeleteRenderbuffers" => Some(mem::transmute::<
            GlDeleteRenderbuffers,
            unsafe extern "C" fn(),
        >(fake_gl_delete_renderbuffers)),
        "glFramebufferRenderbuffer" => Some(mem::transmute::<
            GlFramebufferRenderbuffer,
            unsafe extern "C" fn(),
        >(fake_gl_framebuffer_renderbuffer)),
        "glBlitFramebuffer" => Some(mem::transmute::<GlBlitFramebuffer, unsafe extern "C" fn()>(
            fake_gl_blit_framebuffer,
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

unsafe extern "C" fn fake_gl_get_string_i(name: u32, index: u32) -> *const u8 {
    let state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    if name != GL_EXTENSIONS {
        return std::ptr::null();
    }
    state
        .indexed_extensions
        .get(index as usize)
        .map_or(std::ptr::null(), |extension| {
            extension.as_ptr().cast::<u8>()
        })
}

unsafe extern "C" fn fake_gl_get_integer_v(name: u32, value: *mut i32) {
    let state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    let result = match name {
        GL_MAX_TEXTURE_SIZE => state.config.max_texture_size,
        GL_MAX_TEXTURE_IMAGE_UNITS => state.config.max_texture_image_units,
        GL_MAX_VARYING_VECTORS => state.config.max_varying_vectors,
        GL_NUM_EXTENSIONS => u32::try_from(state.indexed_extensions.len()).ok(),
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
    if !state.snapshot.enabled_capabilities.contains(&cap) {
        state.snapshot.enabled_capabilities.push(cap);
        state.snapshot.enabled_capabilities.sort_unstable();
    }
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
    state
        .snapshot
        .enabled_capabilities
        .retain(|enabled| *enabled != cap);
    match cap {
        GL_BLEND => state.snapshot.blend_enabled = false,
        GL_SCISSOR_TEST => state.snapshot.scissor_enabled = false,
        _ => {}
    }
}

unsafe extern "C" fn fake_gl_depth_func(function: u32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .depth_function = Some(function);
}

unsafe extern "C" fn fake_gl_depth_mask(enabled: u8) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .depth_mask = enabled != GL_FALSE;
}

unsafe extern "C" fn fake_gl_depth_range_f(near: f32, far: f32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .depth_range_f_calls
        .push([near.to_bits(), far.to_bits()]);
}

unsafe extern "C" fn fake_gl_cull_face(mode: u32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .cull_face_mode = Some(mode);
}

unsafe extern "C" fn fake_gl_front_face(winding: u32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .front_face_winding = Some(winding);
}

unsafe extern "C" fn fake_gl_stencil_func(function: u32, reference: i32, mask: u32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .stencil_func = Some(FakeStencilFuncCall {
        function,
        reference,
        mask,
    });
}

unsafe extern "C" fn fake_gl_stencil_mask(mask: u32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .stencil_mask = mask;
}

unsafe extern "C" fn fake_gl_stencil_op(stencil_fail: u32, depth_fail: u32, depth_pass: u32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .stencil_op = Some(FakeStencilOpCall {
        stencil_fail,
        depth_fail,
        depth_pass,
    });
}

unsafe extern "C" fn fake_gl_stencil_func_separate(
    face: u32,
    function: u32,
    reference: i32,
    mask: u32,
) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .stencil_func_separate_calls
        .push(FakeStencilFuncSeparateCall {
            face,
            function,
            reference,
            mask,
        });
}

unsafe extern "C" fn fake_gl_stencil_mask_separate(face: u32, mask: u32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .stencil_mask_separate_calls
        .push(FakeStencilMaskSeparateCall { face, mask });
}

unsafe extern "C" fn fake_gl_stencil_op_separate(
    face: u32,
    stencil_fail: u32,
    depth_fail: u32,
    depth_pass: u32,
) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .stencil_op_separate_calls
        .push(FakeStencilOpSeparateCall {
            face,
            stencil_fail,
            depth_fail,
            depth_pass,
        });
}

unsafe extern "C" fn fake_gl_color_mask(red: u8, green: u8, blue: u8, alpha: u8) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .color_write_mask = [
        red != GL_FALSE,
        green != GL_FALSE,
        blue != GL_FALSE,
        alpha != GL_FALSE,
    ];
}

unsafe extern "C" fn fake_gl_polygon_offset(factor: f32, units: f32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .polygon_offset = Some([factor.to_bits(), units.to_bits()]);
}

unsafe extern "C" fn fake_gl_gen_queries(n: i32, queries: *mut u32) {
    if queries.is_null() {
        return;
    }
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    for index in 0..usize::try_from(n.max(0)).unwrap_or(0) {
        let id = state.next_id();
        unsafe { *queries.add(index) = id };
        state.snapshot.generated_queries.push(id);
    }
}

unsafe extern "C" fn fake_gl_delete_queries(n: i32, queries: *const u32) {
    if queries.is_null() {
        return;
    }
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    for index in 0..usize::try_from(n.max(0)).unwrap_or(0) {
        state
            .snapshot
            .deleted_queries
            .push(unsafe { *queries.add(index) });
    }
}

unsafe extern "C" fn fake_gl_begin_query(target: u32, query: u32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .begin_query_calls
        .push((target, query));
}

unsafe extern "C" fn fake_gl_end_query(target: u32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .end_query_calls
        .push(target);
}

unsafe extern "C" fn fake_gl_get_query_object_uiv(query: u32, property: u32, value: *mut u32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .query_object_uiv_calls
        .push(FakeQueryObjectCall { query, property });
    if value.is_null() {
        return;
    }
    unsafe {
        *value = if property == GL_QUERY_RESULT_AVAILABLE {
            1
        } else {
            77
        };
    }
}

unsafe extern "C" fn fake_gl_fence_sync(_condition: u32, _flags: u32) -> *const c_void {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    let id = state.next_id() as usize;
    state.snapshot.generated_syncs.push(id);
    id as *const c_void
}

unsafe extern "C" fn fake_gl_client_wait_sync(
    sync: *const c_void,
    flags: u32,
    timeout_nanos: u64,
) -> u32 {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .client_wait_sync_calls
        .push(FakeClientWaitSyncCall {
            sync: sync as usize,
            flags,
            timeout_nanos,
        });
    GL_ALREADY_SIGNALED
}

unsafe extern "C" fn fake_gl_wait_sync(sync: *const c_void, flags: u32, timeout_nanos: u64) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .wait_sync_calls
        .push(FakeWaitSyncCall {
            sync: sync as usize,
            flags,
            timeout_nanos,
        });
}

unsafe extern "C" fn fake_gl_delete_sync(sync: *const c_void) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .deleted_syncs
        .push(sync as usize);
}

unsafe extern "C" fn fake_gl_read_pixels(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    format: u32,
    type_: u32,
    pixels: *mut c_void,
) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .read_pixels_calls
        .push(FakeReadPixelsCall {
            x,
            y,
            width,
            height,
            format,
            type_,
        });
    if pixels.is_null() {
        return;
    }
    let channels = match format {
        GL_RED | GL_LUMINANCE => 1usize,
        GL_RGB => 3,
        GL_RGBA => 4,
        _ => 0,
    };
    let byte_len = usize::try_from(width.max(0))
        .unwrap_or(0)
        .saturating_mul(usize::try_from(height.max(0)).unwrap_or(0))
        .saturating_mul(channels);
    unsafe { std::ptr::write_bytes(pixels.cast::<u8>(), 0xA5, byte_len) };
}

unsafe extern "C" fn fake_gl_read_buffer(buffer: u32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .read_buffer_calls
        .push(buffer);
}

unsafe extern "C" fn fake_gl_draw_buffers(count: i32, buffers: *const u32) {
    let buffers = if count <= 0 || buffers.is_null() {
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(buffers, count as usize).to_vec() }
    };
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .draw_buffers_calls
        .push(buffers);
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

unsafe extern "C" fn fake_gl_create_shader(stage: u32) -> u32 {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    let shader = state.next_id();
    state
        .snapshot
        .created_shaders
        .push(FakeCreateShaderCall { stage, shader });
    shader
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

unsafe extern "C" fn fake_gl_attach_shader(program: u32, shader: u32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .attached_shaders
        .push(FakeAttachShaderCall { program, shader });
}

unsafe extern "C" fn fake_gl_link_program(program: u32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .linked_programs
        .push(program);
}

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

unsafe extern "C" fn fake_gl_bind_buffer_base(target: u32, index: u32, buffer: u32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .bind_buffer_base_calls
        .push(FakeBindBufferBaseCall {
            target,
            index,
            buffer,
        });
}

unsafe extern "C" fn fake_gl_bind_buffer_range(
    target: u32,
    index: u32,
    buffer: u32,
    offset: isize,
    size: isize,
) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .bind_buffer_range_calls
        .push(FakeBindBufferRangeCall {
            target,
            index,
            buffer,
            offset: usize::try_from(offset.max(0)).unwrap_or(0),
            size: usize::try_from(size.max(0)).unwrap_or(0),
        });
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
        state.snapshot.buffer_data_uploads.push(FakeBufferDataCall {
            target: _target,
            byte_len: size,
            usage: _usage,
            has_data: !_data.is_null(),
        });
    }
}

unsafe extern "C" fn fake_gl_buffer_sub_data(
    target: u32,
    offset: isize,
    size: isize,
    _data: *const c_void,
) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    state
        .snapshot
        .buffer_sub_data_calls
        .push(FakeBufferSubDataCall {
            target,
            offset: usize::try_from(offset.max(0)).unwrap_or(0),
            byte_len: usize::try_from(size.max(0)).unwrap_or(0),
        });
}

unsafe extern "C" fn fake_gl_copy_buffer_sub_data(
    read_target: u32,
    write_target: u32,
    read_offset: isize,
    write_offset: isize,
    size: isize,
) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .copy_buffer_sub_data_calls
        .push(FakeCopyBufferSubDataCall {
            read_target,
            write_target,
            read_offset: usize::try_from(read_offset.max(0)).unwrap_or(0),
            write_offset: usize::try_from(write_offset.max(0)).unwrap_or(0),
            byte_len: usize::try_from(size.max(0)).unwrap_or(0),
        });
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
    let unit = state.snapshot.active_texture;
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
        .active_texture = texture.saturating_sub(GL_TEXTURE0);
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
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    match parameter {
        GL_PACK_ALIGNMENT => {
            state.snapshot.pack_alignment = value;
            state.snapshot.pack_alignment_calls.push(value);
        }
        GL_UNPACK_ALIGNMENT => {
            state.snapshot.unpack_alignment = value;
            state.snapshot.unpack_alignment_calls.push(value);
        }
        _ => {}
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

unsafe extern "C" fn fake_gl_tex_sub_image_2d(
    target: u32,
    level: i32,
    xoffset: i32,
    yoffset: i32,
    width: i32,
    height: i32,
    format: u32,
    type_: u32,
    pixels: *const c_void,
) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .texture_sub_images_2d
        .push(FakeTextureSubImage2D {
            target,
            level,
            x: xoffset,
            y: yoffset,
            width,
            height,
            format,
            type_,
            has_pixels: !pixels.is_null(),
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
unsafe extern "C" fn fake_gl_delete_vertex_arrays(n: i32, arrays: *const u32) {
    if arrays.is_null() {
        return;
    }

    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    for index in 0..usize::try_from(n.max(0)).unwrap_or(0) {
        let array = unsafe { *arrays.add(index) };
        state.snapshot.deleted_vertex_arrays.push(array);
    }
}
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
    index: u32,
    size: i32,
    type_: u32,
    normalized: u8,
    stride: i32,
    pointer: *const c_void,
) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    state
        .snapshot
        .vertex_attrib_pointer_calls
        .push(FakeVertexAttribPointerCall {
            index,
            size,
            type_,
            normalized: normalized != GL_FALSE,
            stride,
            offset: pointer as usize,
        });
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

unsafe extern "C" fn fake_gl_bind_attrib_location(
    program: u32,
    location: u32,
    name: *const c_char,
) {
    let name = if name.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned()
    };
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .bind_attrib_location_calls
        .push(FakeBindAttribLocationCall {
            program,
            location,
            name,
        });
}

unsafe extern "C" fn fake_gl_uniform_1i(location: i32, value: i32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .uniform_1i_calls
        .push(FakeUniformIntCall { location, value });
}
unsafe extern "C" fn fake_gl_uniform_1f(_location: i32, _value: f32) {}
unsafe extern "C" fn fake_gl_uniform_2f(_location: i32, _x: f32, _y: f32) {}
unsafe extern "C" fn fake_gl_uniform_3f(location: i32, x: f32, y: f32, z: f32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .uniform_3f_calls
        .push(FakeUniformFloatCall {
            location,
            values: [x.to_bits(), y.to_bits(), z.to_bits(), 0],
        });
}

unsafe extern "C" fn fake_gl_uniform_4f(location: i32, x: f32, y: f32, z: f32, w: f32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .uniform_4f_calls
        .push(FakeUniformFloatCall {
            location,
            values: [x.to_bits(), y.to_bits(), z.to_bits(), w.to_bits()],
        });
}

unsafe extern "C" fn fake_gl_uniform_4fv(location: i32, count: i32, value: *const f32) {
    if value.is_null() || count <= 0 {
        return;
    }
    let values = unsafe { std::slice::from_raw_parts(value, 4) };
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .uniform_4fv_calls
        .push(FakeUniformFloatCall {
            location,
            values: [
                values[0].to_bits(),
                values[1].to_bits(),
                values[2].to_bits(),
                values[3].to_bits(),
            ],
        });
}

unsafe extern "C" fn fake_gl_uniform_matrix_3fv(
    location: i32,
    _count: i32,
    transpose: u8,
    value: *const f32,
) {
    if value.is_null() {
        return;
    }
    let values = unsafe { std::slice::from_raw_parts(value, 9) };
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .uniform_matrix_3fv_calls
        .push(FakeUniformMatrix3Call {
            location,
            transpose: transpose != GL_FALSE,
            values: [
                values[0].to_bits(),
                values[1].to_bits(),
                values[2].to_bits(),
                values[3].to_bits(),
                values[4].to_bits(),
                values[5].to_bits(),
                values[6].to_bits(),
                values[7].to_bits(),
                values[8].to_bits(),
            ],
        });
}

unsafe extern "C" fn fake_gl_uniform_matrix_4fv(
    _location: i32,
    _count: i32,
    _transpose: u8,
    _value: *const f32,
) {
}

unsafe extern "C" fn fake_gl_draw_arrays(mode: u32, first: i32, count: i32) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    state
        .snapshot
        .draw_arrays_call_args
        .push(FakeDrawArraysCall {
            mode,
            first,
            count,
            instance_count: None,
        });
    state.snapshot.draw_arrays_calls += 1;
}

unsafe extern "C" fn fake_gl_draw_arrays_instanced(
    mode: u32,
    first: i32,
    count: i32,
    instance_count: i32,
) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    state
        .snapshot
        .draw_arrays_call_args
        .push(FakeDrawArraysCall {
            mode,
            first,
            count,
            instance_count: Some(instance_count),
        });
    state.snapshot.draw_arrays_calls += 1;
}

unsafe extern "C" fn fake_gl_draw_elements(
    mode: u32,
    count: i32,
    type_: u32,
    indices: *const c_void,
) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    state.snapshot.draw_elements_calls += 1;
    state
        .snapshot
        .draw_elements_call_args
        .push(FakeDrawElementsCall {
            mode,
            vertex_range: None,
            count,
            type_,
            offset: indices as usize,
            instance_count: None,
        });
}

unsafe extern "C" fn fake_gl_draw_range_elements(
    mode: u32,
    start: u32,
    end: u32,
    count: i32,
    type_: u32,
    indices: *const c_void,
) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    state.snapshot.draw_elements_calls += 1;
    state
        .snapshot
        .draw_elements_call_args
        .push(FakeDrawElementsCall {
            mode,
            vertex_range: Some((start, end)),
            count,
            type_,
            offset: indices as usize,
            instance_count: None,
        });
}

unsafe extern "C" fn fake_gl_draw_elements_instanced(
    mode: u32,
    count: i32,
    type_: u32,
    indices: *const c_void,
    instance_count: i32,
) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    state.snapshot.draw_elements_instanced_calls += 1;
    state
        .snapshot
        .draw_elements_call_args
        .push(FakeDrawElementsCall {
            mode,
            vertex_range: None,
            count,
            type_,
            offset: indices as usize,
            instance_count: Some(instance_count),
        });
}

unsafe extern "C" fn fake_gl_blend_color(r: f32, g: f32, b: f32, a: f32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .blend_color_calls
        .push(FakeBlendColorCall {
            color: [r.to_bits(), g.to_bits(), b.to_bits(), a.to_bits()],
        });
}

unsafe extern "C" fn fake_gl_blend_func(_src: u32, _dst: u32) {}

unsafe extern "C" fn fake_gl_blend_func_separate(
    source_rgb: u32,
    destination_rgb: u32,
    source_alpha: u32,
    destination_alpha: u32,
) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .blend_func_separate_calls
        .push(FakeBlendFuncSeparateCall {
            source_rgb,
            destination_rgb,
            source_alpha,
            destination_alpha,
        });
}

unsafe extern "C" fn fake_gl_blend_equation(_mode: u32) {}

unsafe extern "C" fn fake_gl_blend_equation_separate(rgb: u32, alpha: u32) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .blend_equation_separate_calls
        .push(FakeBlendEquationSeparateCall { rgb, alpha });
}

unsafe extern "C" fn fake_gl_gen_framebuffers(n: i32, framebuffers: *mut u32) {
    if framebuffers.is_null() {
        return;
    }
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    for index in 0..usize::try_from(n.max(0)).unwrap_or(0) {
        let id = state.next_id();
        unsafe { *framebuffers.add(index) = id };
        state.snapshot.generated_framebuffers.push(id);
    }
}

unsafe extern "C" fn fake_gl_bind_framebuffer(_target: u32, framebuffer: u32) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    state.snapshot.bound_framebuffer = framebuffer;
    state.snapshot.framebuffer_bindings.push(framebuffer);
}

unsafe extern "C" fn fake_gl_delete_framebuffers(n: i32, framebuffers: *const u32) {
    if framebuffers.is_null() {
        return;
    }
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    for index in 0..usize::try_from(n.max(0)).unwrap_or(0) {
        state
            .snapshot
            .deleted_framebuffers
            .push(unsafe { *framebuffers.add(index) });
    }
}

unsafe extern "C" fn fake_gl_framebuffer_texture_2d(
    target: u32,
    attachment: u32,
    texture_target: u32,
    texture: u32,
    level: i32,
) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .framebuffer_texture_2d_calls
        .push(FakeFramebufferTexture2DCall {
            target,
            attachment,
            texture_target,
            texture,
            level,
        });
}

unsafe extern "C" fn fake_gl_gen_renderbuffers(n: i32, renderbuffers: *mut u32) {
    if renderbuffers.is_null() {
        return;
    }
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    for index in 0..usize::try_from(n.max(0)).unwrap_or(0) {
        let id = state.next_id();
        unsafe { *renderbuffers.add(index) = id };
        state.snapshot.generated_renderbuffers.push(id);
    }
}

unsafe extern "C" fn fake_gl_bind_renderbuffer(_target: u32, renderbuffer: u32) {
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    state.snapshot.bound_renderbuffer = renderbuffer;
    state.snapshot.renderbuffer_bindings.push(renderbuffer);
}

unsafe extern "C" fn fake_gl_renderbuffer_storage(
    target: u32,
    internal_format: u32,
    width: i32,
    height: i32,
) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .renderbuffer_storage_calls
        .push(FakeRenderbufferStorageCall {
            target,
            internal_format,
            width,
            height,
        });
}

unsafe extern "C" fn fake_gl_delete_renderbuffers(n: i32, renderbuffers: *const u32) {
    if renderbuffers.is_null() {
        return;
    }
    let mut state = fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned");
    for index in 0..usize::try_from(n.max(0)).unwrap_or(0) {
        state
            .snapshot
            .deleted_renderbuffers
            .push(unsafe { *renderbuffers.add(index) });
    }
}

unsafe extern "C" fn fake_gl_framebuffer_renderbuffer(
    target: u32,
    attachment: u32,
    renderbuffer_target: u32,
    renderbuffer: u32,
) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .framebuffer_renderbuffer_calls
        .push(FakeFramebufferRenderbufferCall {
            target,
            attachment,
            renderbuffer_target,
            renderbuffer,
        });
}

unsafe extern "C" fn fake_gl_blit_framebuffer(
    source_x0: i32,
    source_y0: i32,
    source_x1: i32,
    source_y1: i32,
    destination_x0: i32,
    destination_y0: i32,
    destination_x1: i32,
    destination_y1: i32,
    mask: u32,
    filter: u32,
) {
    fake_gl_state()
        .lock()
        .expect("fake GL state mutex poisoned")
        .snapshot
        .blit_framebuffer_calls
        .push(FakeBlitFramebufferCall {
            source: [source_x0, source_y0, source_x1, source_y1],
            destination: [
                destination_x0,
                destination_y0,
                destination_x1,
                destination_y1,
            ],
            mask,
            filter,
        });
}

unsafe extern "C" fn fake_gl_invalidate_framebuffer(
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
    use std::{ffi::c_void, mem, path::Path, process::Command};

    use enumflags2::BitFlags;

    use super::{
        CompatGl, FakeAttachShaderCall, FakeBindAttribLocationCall, FakeBindBufferBaseCall,
        FakeBindBufferRangeCall, FakeBlendColorCall, FakeBlendEquationSeparateCall,
        FakeBlendFuncSeparateCall, FakeBlitFramebufferCall, FakeBufferDataCall,
        FakeBufferSubDataCall, FakeCopyBufferSubDataCall, FakeCreateShaderCall, FakeDrawArraysCall,
        FakeDrawElementsCall, FakeGlConfig, FakeQueryObjectCall, FakeUniformIntCall,
        FakeVertexAttribPointerCall, GL_FLOAT, GlBlendEquation, GlBlendFactor, GlBuffer,
        GlBufferBindingIndex, GlBufferByteOffset, GlBufferByteSize, GlBufferRange, GlBufferTarget,
        GlBufferUsage, GlCapability, GlColorWriteMask, GlCullFaceMode, GlDepthFunction,
        GlDepthRange, GlDrawMode, GlDrawRange, GlElementByteOffset, GlElementRange,
        GlElementVertexRange, GlFramebuffer, GlFramebufferAttachment, GlFramebufferBlitBuffer,
        GlFramebufferBlitFilter, GlFramebufferBuffer, GlFramebufferTarget,
        GlFramebufferTexture2DTarget, GlFrontFaceWinding, GlIndexType, GlIndexedBufferTarget,
        GlInstanceCount, GlPixelStoreAlignment, GlPolygonOffset, GlProgram, GlQueryTarget, GlRect,
        GlRenderbufferInternalFormat, GlRenderbufferSize, GlRenderbufferTarget, GlShaderStage,
        GlStencilFace, GlStencilFunction, GlStencilMask, GlStencilOperation, GlStencilReference,
        GlSyncTimeout, GlSyncWaitResult, GlTexture, GlTextureDataType, GlTextureFormat,
        GlTextureInternalFormat, GlTextureLevel, GlTextureMagFilter, GlTextureMinFilter,
        GlTextureOffset2D, GlTextureSize2D, GlTextureTarget, GlTextureUnit, GlTextureWrap,
        GlUniformLocation, GlVersionInfo, GlVertexAttribF32Components, GlVertexAttribF32Layout,
        GlVertexAttribLocation, GlVertexAttribStride, HwContextType, fake_gl_test_guard,
        fallback_gl_version_info, glsym, normalize_positive_gl_limit, parse_gl_version_info,
        query_gl_indexed_extensions,
    };

    unsafe extern "C" fn fake_gl_get_string_i(_: u32, _: u32) -> *const u8 {
        std::ptr::null()
    }

    #[test]
    fn generated_raw_registry_covers_vendored_gl_xml() {
        assert_eq!(crate::glsym_raw::GL_XML_COMMAND_COUNT, 246);
        assert_eq!(crate::glsym_raw::GL_XML_TYPE_COUNT, 14);
        assert_eq!(crate::glsym_raw::GL_XML_ENUM_COUNT, 622);
        assert_eq!(
            crate::glsym_raw::GL_XML_COMMAND_NAMES.len(),
            crate::glsym_raw::GL_XML_COMMAND_COUNT
        );
        assert_eq!(
            crate::glsym_raw::GL_XML_TYPE_NAMES.len(),
            crate::glsym_raw::GL_XML_TYPE_COUNT
        );
        assert!(crate::glsym_raw::GL_XML_COMMAND_NAMES.contains(&"glGetStringi"));
        assert!(
            !crate::glsym_raw::GL_XML_COMMAND_NAMES.contains(&"glDebugMessageCallback"),
            "desktop/KHR debug commands are outside the default GLES <= 3.0 raw scope"
        );
        assert!(crate::glsym_raw::GL_XML_TYPE_NAMES.contains(&"GLenum"));
        assert!(
            !crate::glsym_raw::GL_XML_TYPE_NAMES.contains(&"GLDEBUGPROC"),
            "debug callback types are outside the default GLES <= 3.0 raw scope"
        );

        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let verifier = manifest_dir.join("../../scripts/verify_glsym_raw.py");
        let output = Command::new("python3")
            .arg(verifier)
            .output()
            .expect("run glsym raw verifier");
        assert!(
            output.status.success(),
            "glsym raw verifier failed: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn generated_raw_symbol_loader_exposes_registry_commands_optionally() {
        let symbols = crate::glsym_raw::GlRawSymbols::load_with(|name| {
            (name == "glGetStringi").then_some(fake_gl_get_string_i as *const () as *const c_void)
        });

        assert_eq!(symbols.available_count(), 1);
        assert!(symbols.is_available("glGetStringi"));
        assert!(!symbols.is_available("glDebugMessageCallback"));
        assert!(symbols.get_stringi.is_some());
        assert!(symbols.active_texture.is_none());
    }

    #[test]
    fn indexed_extension_query_uses_get_string_i_without_raw_public_api() {
        let _guard = fake_gl_test_guard();
        let _gl = glsym::fake_for_testing(FakeGlConfig {
            extensions_string: "GL_EXT_texture_filter_anisotropic GL_KHR_debug".to_string(),
            ..FakeGlConfig::default()
        });

        let extensions = query_gl_indexed_extensions(
            super::fake_gl_get_integer_v,
            Some(super::fake_gl_get_string_i),
        );

        assert_eq!(extensions, "GL_EXT_texture_filter_anisotropic GL_KHR_debug");
    }

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
    fn blend_separate_helpers_use_typed_factors_and_equations() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig::default());

        gl.blend_color(0.1, 0.2, 0.3, 0.4).expect("blend color");
        gl.blend_func_separate(
            GlBlendFactor::SourceAlpha,
            GlBlendFactor::OneMinusSourceAlpha,
            GlBlendFactor::One,
            GlBlendFactor::OneMinusSourceAlpha,
        )
        .expect("blend func separate");
        gl.blend_equation_separate(GlBlendEquation::Add, GlBlendEquation::ReverseSubtract)
            .expect("blend equation separate");

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(
            snapshot.blend_color_calls,
            vec![FakeBlendColorCall {
                color: [
                    0.1f32.to_bits(),
                    0.2f32.to_bits(),
                    0.3f32.to_bits(),
                    0.4f32.to_bits(),
                ],
            }]
        );
        assert_eq!(
            snapshot.blend_func_separate_calls,
            vec![FakeBlendFuncSeparateCall {
                source_rgb: GlBlendFactor::SourceAlpha.as_raw(),
                destination_rgb: GlBlendFactor::OneMinusSourceAlpha.as_raw(),
                source_alpha: GlBlendFactor::One.as_raw(),
                destination_alpha: GlBlendFactor::OneMinusSourceAlpha.as_raw(),
            }]
        );
        assert_eq!(
            snapshot.blend_equation_separate_calls,
            vec![FakeBlendEquationSeparateCall {
                rgb: GlBlendEquation::Add.as_raw(),
                alpha: GlBlendEquation::ReverseSubtract.as_raw(),
            }]
        );
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
        gl.bind_vertex_array(Some(vertex_array))
            .expect("fake VAO bind");
        gl.unbind_vertex_array().expect("fake VAO unbind");
        gl.delete_vertex_array(vertex_array)
            .expect("fake VAO delete");
        gl.clear_color(0.25, 0.5, 0.75, 1.0);
        gl.clear_color_buffer();
        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(snapshot.bound_vertex_array, 0);
        assert_eq!(
            snapshot.vertex_array_bindings,
            vec![vertex_array.as_raw(), 0]
        );
        assert_eq!(snapshot.deleted_vertex_arrays, vec![vertex_array.as_raw()]);
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
        let program = GlProgram::from_nonzero(1).unwrap();

        assert_eq!(
            gl.required_attrib_location(program, "position"),
            Ok(GlVertexAttribLocation::ZERO)
        );
        assert_eq!(
            gl.required_uniform_location(program, "projection"),
            Ok(GlUniformLocation::from_raw(0).unwrap())
        );
        assert!(
            gl.required_attrib_location(program, "definitely_missing")
                .unwrap_err()
                .contains("required active attribute")
        );
        assert!(
            gl.required_uniform_location(program, "definitely_missing")
                .unwrap_err()
                .contains("required active uniform")
        );
        assert_eq!(gl.uniform_location(1, "definitely_missing").unwrap(), None);
        assert_eq!(
            gl.required_uniform_location(program, "projection").unwrap(),
            GlUniformLocation::from_raw(0).unwrap()
        );
        gl.uniform_2f(GlUniformLocation::from_raw(0).unwrap(), [320.0, 240.0]);
    }

    #[test]
    fn typed_program_helpers_build_use_reflect_and_delete_programs() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig::default());
        let compat = CompatGl::from_glsym(gl.clone());

        let program = gl
            .build_program("attribute vec2 position; void main() {}", "void main() {}")
            .expect("program object");
        let compat_program = compat
            .build_program("attribute vec2 a_pos; void main() {}", "void main() {}")
            .expect("compat program object");

        assert_eq!(
            gl.required_attrib_location(program, "position").unwrap(),
            GlVertexAttribLocation::ZERO
        );
        assert_eq!(
            compat
                .required_uniform_location(compat_program, "projection")
                .unwrap(),
            GlUniformLocation::from_raw(0).unwrap()
        );

        gl.bind_attrib_location(program, GlVertexAttribLocation::from_index(2), "a_color")
            .expect("bind attribute location");
        assert!(
            gl.bind_attrib_location(program, GlVertexAttribLocation::ZERO, "bad\0name")
                .unwrap_err()
                .contains("interior NUL")
        );
        gl.use_program(Some(program));
        compat.use_program(Some(compat_program));
        compat.use_program(None);
        gl.delete_program(program);
        compat.delete_program(compat_program);

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(
            snapshot.use_program_calls,
            vec![program.as_raw(), compat_program.as_raw(), 0]
        );
        assert_eq!(
            snapshot.deleted_programs,
            vec![program.as_raw(), compat_program.as_raw()]
        );
        assert_eq!(
            snapshot.bind_attrib_location_calls,
            vec![FakeBindAttribLocationCall {
                program: program.as_raw(),
                location: 2,
                name: "a_color".to_string(),
            }]
        );
    }

    #[test]
    fn typed_uniform_setters_use_locations_and_fixed_size_values() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig::default());
        let color = gl.required_uniform(1, "projection").unwrap();

        gl.uniform_1i(color, 7);
        gl.uniform_4fv(color, &[1.0, 0.5, 0.25, 0.125]);
        gl.uniform_3f(color, [1.0, 0.5, 0.25]);
        gl.uniform_4f(color, [1.0, 0.5, 0.25, 0.125]);
        gl.uniform_matrix_3fv(color, true, &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 2.0, 3.0, 1.0]);

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(
            snapshot.uniform_1i_calls,
            vec![FakeUniformIntCall {
                location: color.as_raw(),
                value: 7,
            }]
        );
        assert_eq!(snapshot.uniform_4fv_calls.len(), 1);
        assert_eq!(
            snapshot.uniform_4fv_calls[0].values,
            [
                1.0_f32.to_bits(),
                0.5_f32.to_bits(),
                0.25_f32.to_bits(),
                0.125_f32.to_bits(),
            ]
        );
        assert_eq!(snapshot.uniform_3f_calls.len(), 1);
        assert_eq!(snapshot.uniform_3f_calls[0].location, color.as_raw());
        assert_eq!(
            snapshot.uniform_3f_calls[0].values,
            [1.0_f32.to_bits(), 0.5_f32.to_bits(), 0.25_f32.to_bits(), 0]
        );
        assert_eq!(snapshot.uniform_4f_calls.len(), 1);
        assert_eq!(
            snapshot.uniform_4f_calls[0].values,
            [
                1.0_f32.to_bits(),
                0.5_f32.to_bits(),
                0.25_f32.to_bits(),
                0.125_f32.to_bits(),
            ]
        );
        assert_eq!(snapshot.uniform_matrix_3fv_calls.len(), 1);
        assert!(snapshot.uniform_matrix_3fv_calls[0].transpose);
        assert_eq!(
            snapshot.uniform_matrix_3fv_calls[0].location,
            color.as_raw()
        );
    }

    #[test]
    fn compat_uniform_helpers_use_typed_locations() {
        let _guard = fake_gl_test_guard();
        let raw_gl = glsym::fake_for_testing(FakeGlConfig::default());
        let gl = CompatGl::from_glsym(raw_gl.clone());
        let text_gl = super::CompatTextureGl::from_glsym(raw_gl);

        let color = gl.required_uniform(1, "u_color").unwrap();
        let font = text_gl.required_uniform(1, "u_font").unwrap();
        gl.uniform_4fv(color, &[0.25, 0.5, 0.75, 1.0]);
        text_gl.uniform_1i(font, 0);
        text_gl.uniform_4fv(color, &[1.0, 0.75, 0.5, 0.25]);

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(
            snapshot.uniform_1i_calls,
            vec![FakeUniformIntCall {
                location: font.as_raw(),
                value: 0,
            }]
        );
        assert_eq!(snapshot.uniform_4fv_calls.len(), 2);
        assert_eq!(snapshot.uniform_4fv_calls[0].location, color.as_raw());
        assert_eq!(
            snapshot.uniform_4fv_calls[1].values,
            [
                1.0_f32.to_bits(),
                0.75_f32.to_bits(),
                0.5_f32.to_bits(),
                0.25_f32.to_bits(),
            ]
        );
    }

    #[test]
    fn typed_vertex_attrib_helpers_use_locations() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig::default());
        let compat = CompatGl::from_glsym(gl.clone());

        let position = gl.required_attrib(1, "a_pos").unwrap();
        assert_eq!(position, GlVertexAttribLocation::ZERO);
        let color = compat.required_attrib(1, "a_color").unwrap();
        assert_eq!(color.as_raw(), 3);

        gl.enable_vertex_attrib(position);
        gl.vertex_attrib_pointer_f32(
            position,
            GlVertexAttribF32Layout::interleaved(GlVertexAttribF32Components::Two, 5).unwrap(),
        );
        compat.enable_vertex_attrib(color);
        compat.vertex_attrib_pointer_f32(
            color,
            GlVertexAttribF32Layout::interleaved(GlVertexAttribF32Components::Three, 5)
                .unwrap()
                .with_offset_components(GlVertexAttribF32Components::Two),
        );
        compat.disable_vertex_attrib(color);

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(snapshot.enabled_vertex_attribs, vec![position.as_raw()]);
        assert_eq!(
            snapshot.vertex_attrib_pointer_calls,
            vec![
                FakeVertexAttribPointerCall {
                    index: position.as_raw(),
                    size: 2,
                    type_: GL_FLOAT,
                    normalized: false,
                    stride: 20,
                    offset: 0,
                },
                FakeVertexAttribPointerCall {
                    index: color.as_raw(),
                    size: 3,
                    type_: GL_FLOAT,
                    normalized: false,
                    stride: 20,
                    offset: 8,
                },
            ]
        );
        assert!(
            GlVertexAttribStride::from_f32_count((i32::MAX as usize / mem::size_of::<f32>()) + 1)
                .is_err()
        );
    }

    #[test]
    fn draw_range_checks_glsizei_limits() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig {
            supports_instancing: true,
            ..FakeGlConfig::default()
        });
        let compat = CompatGl::from_glsym(gl.clone());

        gl.draw_arrays(GlDrawMode::Triangles, GlDrawRange::from_start(3))
            .expect("draw arrays range");
        compat
            .draw_arrays(GlDrawMode::Triangles, GlDrawRange::new(1, 2))
            .expect("compat draw arrays range");
        gl.draw_arrays_instanced(
            GlDrawMode::Triangles,
            GlDrawRange::new(2, 3),
            GlInstanceCount::new(4),
        )
        .expect("instanced draw arrays range");
        gl.draw_elements(
            GlDrawMode::Triangles,
            GlIndexType::UnsignedShort,
            GlElementRange::new(
                6,
                GlElementByteOffset::from_indices(GlIndexType::UnsignedShort, 3).unwrap(),
            ),
        )
        .expect("draw elements range");
        assert!(gl.supports_draw_range_elements());
        gl.draw_range_elements(
            GlDrawMode::Triangles,
            GlElementVertexRange::new(2, 7).unwrap(),
            GlIndexType::UnsignedShort,
            GlElementRange::new(
                5,
                GlElementByteOffset::from_indices(GlIndexType::UnsignedShort, 4).unwrap(),
            ),
        )
        .expect("draw range elements range");
        gl.draw_elements_instanced(
            GlDrawMode::Triangles,
            GlIndexType::UnsignedShort,
            GlElementRange::from_start(6),
            GlInstanceCount::new(2),
        )
        .expect("instanced draw elements range");

        assert!(
            gl.draw_arrays(
                GlDrawMode::Triangles,
                GlDrawRange::new(0, i32::MAX as u32 + 1),
            )
            .unwrap_err()
            .contains("vertex count")
        );
        assert!(
            gl.draw_arrays_instanced(
                GlDrawMode::Triangles,
                GlDrawRange::from_start(1),
                GlInstanceCount::new(i32::MAX as u32 + 1),
            )
            .unwrap_err()
            .contains("instance count")
        );
        assert!(
            gl.draw_elements(
                GlDrawMode::Triangles,
                GlIndexType::UnsignedShort,
                GlElementRange::from_start(i32::MAX as u32 + 1),
            )
            .unwrap_err()
            .contains("index count")
        );
        assert!(
            gl.draw_elements_instanced(
                GlDrawMode::Triangles,
                GlIndexType::UnsignedShort,
                GlElementRange::from_start(1),
                GlInstanceCount::new(i32::MAX as u32 + 1),
            )
            .unwrap_err()
            .contains("instance count")
        );
        assert!(
            GlElementVertexRange::new(8, 2)
                .unwrap_err()
                .contains("exceeds end")
        );

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(snapshot.draw_arrays_calls, 3);
        assert_eq!(
            snapshot.draw_arrays_call_args,
            vec![
                FakeDrawArraysCall {
                    mode: GlDrawMode::Triangles.as_raw(),
                    first: 0,
                    count: 3,
                    instance_count: None,
                },
                FakeDrawArraysCall {
                    mode: GlDrawMode::Triangles.as_raw(),
                    first: 1,
                    count: 2,
                    instance_count: None,
                },
                FakeDrawArraysCall {
                    mode: GlDrawMode::Triangles.as_raw(),
                    first: 2,
                    count: 3,
                    instance_count: Some(4),
                },
            ]
        );
        assert_eq!(snapshot.draw_elements_calls, 2);
        assert_eq!(snapshot.draw_elements_instanced_calls, 1);
        assert_eq!(
            snapshot.draw_elements_call_args,
            vec![
                FakeDrawElementsCall {
                    mode: GlDrawMode::Triangles.as_raw(),
                    vertex_range: None,
                    count: 6,
                    type_: GlIndexType::UnsignedShort.as_raw(),
                    offset: 6,
                    instance_count: None,
                },
                FakeDrawElementsCall {
                    mode: GlDrawMode::Triangles.as_raw(),
                    vertex_range: Some((2, 7)),
                    count: 5,
                    type_: GlIndexType::UnsignedShort.as_raw(),
                    offset: 8,
                    instance_count: None,
                },
                FakeDrawElementsCall {
                    mode: GlDrawMode::Triangles.as_raw(),
                    vertex_range: None,
                    count: 6,
                    type_: GlIndexType::UnsignedShort.as_raw(),
                    offset: 0,
                    instance_count: Some(2),
                },
            ]
        );
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
            .bind_framebuffer(GlFramebufferTarget::Framebuffer, GlFramebuffer::from_raw(7))
            .unwrap_err();
        assert!(error.contains("incomplete framebuffer"));
        assert!(error.contains("GL_FRAMEBUFFER_INCOMPLETE_ATTACHMENT"));
    }

    #[test]
    fn shader_source_errors_delete_unowned_shader_objects() {
        let _guard = fake_gl_test_guard();
        let compat = CompatGl::fake_for_testing(FakeGlConfig::default());

        let error = compat
            .compile_shader_source(GlShaderStage::Vertex, "void\0main")
            .unwrap_err();

        assert!(error.contains("interior NUL"));
        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(snapshot.deleted_shaders, vec![1]);

        let gl = glsym::fake_for_testing(FakeGlConfig::default());
        let error = gl
            .compile_shader_source(GlShaderStage::Fragment, "void\0main")
            .unwrap_err();

        assert!(error.contains("interior NUL"));
        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(snapshot.deleted_shaders, vec![1]);
    }

    #[test]
    fn typed_shader_helpers_use_stage_enums_and_shader_handles() {
        let _guard = fake_gl_test_guard();
        let raw_gl = glsym::fake_for_testing(FakeGlConfig::default());
        let compat = CompatGl::from_glsym(raw_gl.clone());

        let vertex = compat
            .compile_shader_source(GlShaderStage::Vertex, "void main() {}")
            .expect("compat vertex shader");
        let fragment = raw_gl
            .compile_shader_source(GlShaderStage::Fragment, "void main() {}")
            .expect("fragment shader");
        let program = raw_gl.create_program().expect("program object");
        raw_gl.attach_shader(program, vertex);
        raw_gl.link_program(program).expect("link program");

        raw_gl.delete_shader(fragment);
        compat.delete_shader(vertex);
        raw_gl.delete_program(program);

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(
            snapshot.created_shaders,
            vec![
                FakeCreateShaderCall {
                    stage: GlShaderStage::Vertex.as_raw(),
                    shader: vertex.as_raw(),
                },
                FakeCreateShaderCall {
                    stage: GlShaderStage::Fragment.as_raw(),
                    shader: fragment.as_raw(),
                },
            ]
        );
        assert_eq!(
            snapshot.deleted_shaders,
            vec![fragment.as_raw(), vertex.as_raw()]
        );
        assert_eq!(
            snapshot.attached_shaders,
            vec![FakeAttachShaderCall {
                program: program.as_raw(),
                shader: vertex.as_raw(),
            }]
        );
        assert_eq!(snapshot.linked_programs, vec![program.as_raw()]);
        assert_eq!(snapshot.deleted_programs, vec![program.as_raw()]);
    }

    #[test]
    fn fake_gl_snapshot_tracks_texture_upload_formats() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig::default());
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
        gl.tex_image_2d(
            GlTextureTarget::Texture2D,
            GlTextureInternalFormat::Rgba,
            GlTextureLevel::ZERO,
            GlTextureSize2D::new(4, 2),
            GlTextureFormat::Rgba,
            GlTextureDataType::UnsignedByte,
            None,
        )
        .expect("sized texture upload");
        assert!(
            gl.tex_image_2d(
                GlTextureTarget::Texture2D,
                GlTextureInternalFormat::Rgba,
                GlTextureLevel::ZERO,
                GlTextureSize2D::new(u32::MAX, 2),
                GlTextureFormat::Rgba,
                GlTextureDataType::UnsignedByte,
                None,
            )
            .unwrap_err()
            .contains("glTexImage2D width")
        );

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(snapshot.texture_uploads_2d.len(), 2);
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
        assert_eq!(snapshot.texture_uploads_2d[1].width, 4);
        assert_eq!(snapshot.texture_uploads_2d[1].height, 2);
    }

    #[test]
    fn texture_parameter_helpers_use_typed_filters_and_wraps() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig::default());

        gl.tex_min_filter(
            GlTextureTarget::Texture2D,
            GlTextureMinFilter::LinearMipmapLinear,
        );
        gl.tex_mag_filter(GlTextureTarget::Texture2D, GlTextureMagFilter::Linear);
        gl.tex_wrap_s(GlTextureTarget::Texture2D, GlTextureWrap::ClampToEdge);
        gl.tex_wrap_t(GlTextureTarget::Texture2D, GlTextureWrap::Repeat);
        gl.tex_wrap_r(GlTextureTarget::Texture2DArray, GlTextureWrap::ClampToEdge);

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(
            snapshot.texture_parameter_calls,
            vec![
                super::FakeTextureParameterCall {
                    target: GlTextureTarget::Texture2D.as_raw(),
                    parameter: super::GL_TEXTURE_MIN_FILTER,
                    value: GlTextureMinFilter::LinearMipmapLinear.as_raw() as i32,
                },
                super::FakeTextureParameterCall {
                    target: GlTextureTarget::Texture2D.as_raw(),
                    parameter: super::GL_TEXTURE_MAG_FILTER,
                    value: GlTextureMagFilter::Linear.as_raw() as i32,
                },
                super::FakeTextureParameterCall {
                    target: GlTextureTarget::Texture2D.as_raw(),
                    parameter: super::GL_TEXTURE_WRAP_S,
                    value: GlTextureWrap::ClampToEdge.as_raw() as i32,
                },
                super::FakeTextureParameterCall {
                    target: GlTextureTarget::Texture2D.as_raw(),
                    parameter: super::GL_TEXTURE_WRAP_T,
                    value: GlTextureWrap::Repeat.as_raw() as i32,
                },
                super::FakeTextureParameterCall {
                    target: GlTextureTarget::Texture2DArray.as_raw(),
                    parameter: super::GL_TEXTURE_WRAP_R,
                    value: GlTextureWrap::ClampToEdge.as_raw() as i32,
                },
            ]
        );
    }

    #[test]
    fn texture_sub_image_2d_uses_typed_offset_size_and_formats() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig::default());
        let pixels = [255_u8; 16];

        gl.tex_sub_image_2d(
            GlTextureTarget::Texture2D,
            GlTextureLevel::ZERO,
            GlTextureOffset2D::new(4, 8),
            GlTextureSize2D::new(2, 2),
            GlTextureFormat::Rgba,
            GlTextureDataType::UnsignedByte,
            &pixels,
        )
        .expect("texture sub image");

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(snapshot.texture_sub_images_2d.len(), 1);
        let upload = snapshot.texture_sub_images_2d[0];
        assert_eq!(upload.target, GlTextureTarget::Texture2D.as_raw());
        assert_eq!(upload.level, 0);
        assert_eq!(upload.x, 4);
        assert_eq!(upload.y, 8);
        assert_eq!(upload.width, 2);
        assert_eq!(upload.height, 2);
        assert_eq!(upload.format, GlTextureFormat::Rgba.as_raw());
        assert!(upload.has_pixels);
    }

    #[test]
    fn read_pixels_uses_typed_rectangles_formats_and_mut_slices() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig::default());
        let mut pixels = [0_u8; 16];

        gl.read_buffer(GlFramebufferBuffer::ColorAttachment(1))
            .expect("read buffer");
        gl.draw_buffers(&[
            GlFramebufferBuffer::ColorAttachment(0),
            GlFramebufferBuffer::ColorAttachment(1),
        ])
        .expect("draw buffers");
        gl.read_pixels(GlRect::new(2, 4, 2, 2), GlTextureFormat::Rgba, &mut pixels)
            .expect("read pixels");
        assert!(pixels.iter().all(|byte| *byte == 0xA5));

        assert!(
            gl.read_pixels(GlRect::new(0, 0, 2, 2), GlTextureFormat::Rgb, &mut [0; 11])
                .unwrap_err()
                .contains("requires 12 destination byte")
        );

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(
            snapshot.read_buffer_calls,
            vec![GlFramebufferBuffer::ColorAttachment(1).as_raw().unwrap()]
        );
        assert_eq!(
            snapshot.draw_buffers_calls,
            vec![vec![
                GlFramebufferBuffer::ColorAttachment(0).as_raw().unwrap(),
                GlFramebufferBuffer::ColorAttachment(1).as_raw().unwrap(),
            ]]
        );
        assert_eq!(
            snapshot.read_pixels_calls,
            vec![super::FakeReadPixelsCall {
                x: 2,
                y: 4,
                width: 2,
                height: 2,
                format: GlTextureFormat::Rgba.as_raw(),
                type_: super::GL_UNSIGNED_BYTE,
            }]
        );
    }

    #[test]
    fn buffer_sub_data_uses_typed_byte_offsets_and_slice_lengths() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig::default());
        let compat = CompatGl::from_glsym(gl.clone());
        let vertices = [1.0_f32, 2.0, 3.0, 4.0];

        let buffer = gl.gen_buffer().expect("buffer object");
        gl.bind_buffer(GlBufferTarget::ArrayBuffer, Some(buffer));
        gl.buffer_data(
            GlBufferTarget::ArrayBuffer,
            &vertices,
            GlBufferUsage::StaticDraw,
        )
        .expect("buffer data");
        compat.bind_buffer(GlBufferTarget::ArrayBuffer, None);
        gl.buffer_data_empty(
            GlBufferTarget::ElementArrayBuffer,
            GlBufferByteSize::from_bytes(64),
            GlBufferUsage::DynamicDraw,
        )
        .expect("empty buffer allocation");
        gl.buffer_sub_data(
            GlBufferTarget::ArrayBuffer,
            GlBufferByteOffset::from_bytes(8),
            &vertices,
        )
        .expect("buffer sub data");
        compat.delete_buffer(buffer);

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(snapshot.bound_array_buffer, 0);
        assert_eq!(snapshot.deleted_buffers, vec![buffer.as_raw()]);
        assert_eq!(
            snapshot.buffer_data_uploads,
            vec![
                FakeBufferDataCall {
                    target: GlBufferTarget::ArrayBuffer.as_raw(),
                    byte_len: 16,
                    usage: GlBufferUsage::StaticDraw.as_raw(),
                    has_data: true,
                },
                FakeBufferDataCall {
                    target: GlBufferTarget::ElementArrayBuffer.as_raw(),
                    byte_len: 64,
                    usage: GlBufferUsage::DynamicDraw.as_raw(),
                    has_data: false,
                },
            ]
        );
        assert_eq!(
            snapshot.buffer_sub_data_calls,
            vec![FakeBufferSubDataCall {
                target: GlBufferTarget::ArrayBuffer.as_raw(),
                offset: 8,
                byte_len: 16,
            }]
        );
    }

    #[test]
    fn buffer_copy_uses_typed_targets_offsets_and_sizes() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig::default());

        gl.copy_buffer_sub_data(
            GlBufferTarget::CopyReadBuffer,
            GlBufferTarget::CopyWriteBuffer,
            GlBufferByteOffset::from_bytes(8),
            GlBufferByteOffset::from_bytes(24),
            GlBufferByteSize::from_bytes(32),
        )
        .expect("copy buffer sub data");

        assert!(
            gl.copy_buffer_sub_data(
                GlBufferTarget::CopyReadBuffer,
                GlBufferTarget::CopyWriteBuffer,
                GlBufferByteOffset::from_bytes(0),
                GlBufferByteOffset::from_bytes(0),
                GlBufferByteSize::from_bytes(usize::MAX),
            )
            .unwrap_err()
            .contains("glCopyBufferSubData byte length")
        );

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(
            snapshot.copy_buffer_sub_data_calls,
            vec![FakeCopyBufferSubDataCall {
                read_target: GlBufferTarget::CopyReadBuffer.as_raw(),
                write_target: GlBufferTarget::CopyWriteBuffer.as_raw(),
                read_offset: 8,
                write_offset: 24,
                byte_len: 32,
            }]
        );
    }

    #[test]
    fn indexed_buffer_bindings_use_typed_targets_indices_and_ranges() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig::default());
        assert!(gl.supports_indexed_buffer_bindings());

        let buffer = gl.gen_buffer().expect("buffer object");
        let index = GlBufferBindingIndex::from_index(2);
        gl.bind_buffer_base(GlIndexedBufferTarget::UniformBuffer, index, Some(buffer))
            .expect("bind buffer base");
        gl.bind_buffer_range(
            GlIndexedBufferTarget::TransformFeedbackBuffer,
            GlBufferBindingIndex::ZERO,
            Some(buffer),
            GlBufferRange::new(
                GlBufferByteOffset::from_bytes(16),
                GlBufferByteSize::from_bytes(32),
            ),
        )
        .expect("bind buffer range");
        gl.bind_buffer_base(GlIndexedBufferTarget::UniformBuffer, index, None)
            .expect("unbind buffer base");

        assert!(
            gl.bind_buffer_range(
                GlIndexedBufferTarget::TransformFeedbackBuffer,
                GlBufferBindingIndex::ZERO,
                Some(buffer),
                GlBufferRange::from_start(GlBufferByteSize::from_bytes(usize::MAX)),
            )
            .unwrap_err()
            .contains("glBindBufferRange byte length")
        );

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(
            snapshot.bind_buffer_base_calls,
            vec![
                FakeBindBufferBaseCall {
                    target: GlIndexedBufferTarget::UniformBuffer.as_raw(),
                    index: 2,
                    buffer: buffer.as_raw(),
                },
                FakeBindBufferBaseCall {
                    target: GlIndexedBufferTarget::UniformBuffer.as_raw(),
                    index: 2,
                    buffer: 0,
                },
            ]
        );
        assert_eq!(
            snapshot.bind_buffer_range_calls,
            vec![FakeBindBufferRangeCall {
                target: GlIndexedBufferTarget::TransformFeedbackBuffer.as_raw(),
                index: 0,
                buffer: buffer.as_raw(),
                offset: 16,
                size: 32,
            }]
        );
    }

    #[test]
    fn checked_buffer_allocation_rejects_lengths_that_exceed_gl_sizeiptr() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig::default());

        assert!(
            gl.buffer_data_empty(
                GlBufferTarget::ArrayBuffer,
                GlBufferByteSize::from_bytes(usize::MAX),
                GlBufferUsage::StreamDraw,
            )
            .unwrap_err()
            .contains("GL buffer allocation byte length")
        );

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert!(snapshot.buffer_data_uploads.is_empty());
    }

    #[test]
    fn framebuffer_and_renderbuffer_helpers_use_typed_targets_and_attachments() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig::default());

        assert!(gl.supports_framebuffer_objects());
        assert!(gl.supports_renderbuffers());

        let framebuffer = gl.gen_framebuffer().expect("framebuffer");
        let renderbuffer = gl.gen_renderbuffer().expect("renderbuffer");

        gl.bind_framebuffer(GlFramebufferTarget::Framebuffer, Some(framebuffer))
            .expect("bind framebuffer");
        gl.bind_renderbuffer(GlRenderbufferTarget::Renderbuffer, Some(renderbuffer))
            .expect("bind renderbuffer");
        gl.renderbuffer_storage(
            GlRenderbufferTarget::Renderbuffer,
            GlRenderbufferInternalFormat::DepthComponent16,
            GlRenderbufferSize::new(320, 240),
        )
        .expect("renderbuffer storage");
        gl.framebuffer_renderbuffer(
            GlFramebufferTarget::Framebuffer,
            GlFramebufferAttachment::Depth,
            GlRenderbufferTarget::Renderbuffer,
            Some(renderbuffer),
        )
        .expect("framebuffer renderbuffer");
        gl.framebuffer_texture_2d(
            GlFramebufferTarget::Framebuffer,
            GlFramebufferAttachment::Color(0),
            GlFramebufferTexture2DTarget::Texture2D,
            Some(GlTexture::from_nonzero(99).unwrap()),
            GlTextureLevel::ZERO,
        )
        .expect("framebuffer texture");
        gl.delete_framebuffer(framebuffer)
            .expect("delete framebuffer");
        gl.delete_renderbuffer(renderbuffer)
            .expect("delete renderbuffer");

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(snapshot.generated_framebuffers, vec![framebuffer.as_raw()]);
        assert_eq!(snapshot.deleted_framebuffers, vec![framebuffer.as_raw()]);
        assert_eq!(snapshot.framebuffer_bindings, vec![framebuffer.as_raw()]);
        assert_eq!(
            snapshot.generated_renderbuffers,
            vec![renderbuffer.as_raw()]
        );
        assert_eq!(snapshot.deleted_renderbuffers, vec![renderbuffer.as_raw()]);
        assert_eq!(snapshot.renderbuffer_bindings, vec![renderbuffer.as_raw()]);
        assert_eq!(
            snapshot.renderbuffer_storage_calls,
            vec![super::FakeRenderbufferStorageCall {
                target: GlRenderbufferTarget::Renderbuffer.as_raw(),
                internal_format: GlRenderbufferInternalFormat::DepthComponent16.as_raw(),
                width: 320,
                height: 240,
            }]
        );
        assert_eq!(
            snapshot.framebuffer_renderbuffer_calls,
            vec![super::FakeFramebufferRenderbufferCall {
                target: GlFramebufferTarget::Framebuffer.as_raw(),
                attachment: GlFramebufferAttachment::Depth.as_raw().unwrap(),
                renderbuffer_target: GlRenderbufferTarget::Renderbuffer.as_raw(),
                renderbuffer: renderbuffer.as_raw(),
            }]
        );
        assert_eq!(
            snapshot.framebuffer_texture_2d_calls,
            vec![super::FakeFramebufferTexture2DCall {
                target: GlFramebufferTarget::Framebuffer.as_raw(),
                attachment: GlFramebufferAttachment::Color(0).as_raw().unwrap(),
                texture_target: GlFramebufferTexture2DTarget::Texture2D.as_raw(),
                texture: 99,
                level: 0,
            }]
        );
    }

    #[test]
    fn framebuffer_blit_uses_typed_rectangles_buffers_and_filter() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig::default());

        gl.blit_framebuffer(
            GlRect::new(0, 1, 320, 240),
            GlRect::new(10, 20, 160, 120),
            GlFramebufferBlitBuffer::Color.into(),
            GlFramebufferBlitFilter::Linear,
        )
        .expect("color blit");
        gl.blit_framebuffer(
            GlRect::new(-5, -6, 32, 16),
            GlRect::new(0, 0, 32, 16),
            GlFramebufferBlitBuffer::Color
                | GlFramebufferBlitBuffer::Depth
                | GlFramebufferBlitBuffer::Stencil,
            GlFramebufferBlitFilter::Nearest,
        )
        .expect("depth/stencil blit");
        assert!(
            gl.blit_framebuffer(
                GlRect::new(0, 0, 1, 1),
                GlRect::new(0, 0, 1, 1),
                BitFlags::empty(),
                GlFramebufferBlitFilter::Nearest,
            )
            .unwrap_err()
            .contains("at least one buffer")
        );
        assert!(
            gl.blit_framebuffer(
                GlRect::new(0, 0, 1, 1),
                GlRect::new(0, 0, 1, 1),
                GlFramebufferBlitBuffer::Depth.into(),
                GlFramebufferBlitFilter::Linear,
            )
            .unwrap_err()
            .contains("nearest filtering")
        );
        assert!(
            gl.blit_framebuffer(
                GlRect::new(i32::MAX, 0, 1, 1),
                GlRect::new(0, 0, 1, 1),
                GlFramebufferBlitBuffer::Color.into(),
                GlFramebufferBlitFilter::Nearest,
            )
            .unwrap_err()
            .contains("x endpoint")
        );

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(
            snapshot.blit_framebuffer_calls,
            vec![
                FakeBlitFramebufferCall {
                    source: [0, 1, 320, 241],
                    destination: [10, 20, 170, 140],
                    mask: GlFramebufferBlitBuffer::Color as u32,
                    filter: GlFramebufferBlitFilter::Linear.as_raw(),
                },
                FakeBlitFramebufferCall {
                    source: [-5, -6, 27, 10],
                    destination: [0, 0, 32, 16],
                    mask: (GlFramebufferBlitBuffer::Color
                        | GlFramebufferBlitBuffer::Depth
                        | GlFramebufferBlitBuffer::Stencil)
                        .bits(),
                    filter: GlFramebufferBlitFilter::Nearest.as_raw(),
                },
            ]
        );
    }

    #[test]
    fn query_helpers_use_typed_targets_handles_and_return_values() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig::default());

        assert!(gl.supports_queries());
        let query = gl.gen_query().expect("query object");
        gl.begin_query(GlQueryTarget::AnySamplesPassed, query)
            .expect("begin query");
        gl.end_query(GlQueryTarget::AnySamplesPassed)
            .expect("end query");

        assert!(gl.query_result_available(query).expect("availability"));
        assert_eq!(gl.query_result_u32(query).expect("query result"), 77);
        gl.delete_query(query).expect("delete query");

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(snapshot.generated_queries, vec![query.as_raw()]);
        assert_eq!(snapshot.deleted_queries, vec![query.as_raw()]);
        assert_eq!(
            snapshot.begin_query_calls,
            vec![(GlQueryTarget::AnySamplesPassed.as_raw(), query.as_raw())]
        );
        assert_eq!(
            snapshot.end_query_calls,
            vec![GlQueryTarget::AnySamplesPassed.as_raw()]
        );
        assert_eq!(
            snapshot.query_object_uiv_calls,
            vec![
                FakeQueryObjectCall {
                    query: query.as_raw(),
                    property: super::GL_QUERY_RESULT_AVAILABLE,
                },
                FakeQueryObjectCall {
                    query: query.as_raw(),
                    property: super::GL_QUERY_RESULT,
                },
            ]
        );
    }

    #[test]
    fn sync_helpers_hide_raw_fence_pointers() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig::default());

        assert!(gl.supports_sync_objects());
        let sync = gl.fence_sync().expect("sync fence");
        assert_eq!(
            gl.client_wait_sync(sync, true, GlSyncTimeout::from_nanos(5))
                .expect("client wait"),
            GlSyncWaitResult::AlreadySignaled
        );
        gl.wait_sync(sync).expect("server wait");
        gl.delete_sync(sync).expect("delete sync");

        let sync_id = sync.as_raw() as usize;
        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(snapshot.generated_syncs, vec![sync_id]);
        assert_eq!(
            snapshot.client_wait_sync_calls,
            vec![super::FakeClientWaitSyncCall {
                sync: sync_id,
                flags: super::GL_SYNC_FLUSH_COMMANDS_BIT,
                timeout_nanos: 5,
            }]
        );
        assert_eq!(
            snapshot.wait_sync_calls,
            vec![super::FakeWaitSyncCall {
                sync: sync_id,
                flags: 0,
                timeout_nanos: super::GL_TIMEOUT_IGNORED,
            }]
        );
        assert_eq!(snapshot.deleted_syncs, vec![sync_id]);
    }

    #[test]
    fn fake_gl_snapshot_tracks_shared_context_state() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig::default());

        gl.enable(GlCapability::Blend);
        gl.enable(GlCapability::DepthTest);
        gl.enable(GlCapability::ScissorTest);
        gl.enable(GlCapability::CullFace);
        gl.depth_func(GlDepthFunction::LessOrEqual).unwrap();
        gl.depth_mask(false).unwrap();
        gl.cull_face(GlCullFaceMode::Back).unwrap();
        gl.front_face(GlFrontFaceWinding::CounterClockwise).unwrap();
        gl.stencil_func(
            GlStencilFunction::Always,
            GlStencilReference::new(3),
            GlStencilMask::new(0x0f),
        )
        .unwrap();
        gl.stencil_mask(GlStencilMask::new(0xf0)).unwrap();
        gl.stencil_op(
            GlStencilOperation::Keep,
            GlStencilOperation::Replace,
            GlStencilOperation::IncrementWrap,
        )
        .unwrap();
        gl.stencil_func_separate(
            GlStencilFace::Back,
            GlStencilFunction::LessOrEqual,
            GlStencilReference::new(2),
            GlStencilMask::ALL,
        )
        .unwrap();
        gl.stencil_mask_separate(GlStencilFace::Front, GlStencilMask::NONE)
            .unwrap();
        gl.stencil_op_separate(
            GlStencilFace::FrontAndBack,
            GlStencilOperation::DecrementClamp,
            GlStencilOperation::Invert,
            GlStencilOperation::Keep,
        )
        .unwrap();
        gl.color_mask(GlColorWriteMask::RGB).unwrap();
        gl.polygon_offset(GlPolygonOffset::new(1.25, -2.5)).unwrap();
        gl.bind_buffer(
            GlBufferTarget::ArrayBuffer,
            Some(GlBuffer::from_nonzero(11).unwrap()),
        );
        gl.bind_buffer(
            GlBufferTarget::ElementArrayBuffer,
            Some(GlBuffer::from_nonzero(12).unwrap()),
        );
        gl.bind_framebuffer(
            GlFramebufferTarget::Framebuffer,
            GlFramebuffer::from_raw(13),
        )
        .unwrap();
        gl.viewport(GlRect::new(1, 2, 320, 240)).unwrap();
        gl.scissor(GlRect::new(3, 4, 160, 120)).unwrap();
        gl.pixel_store_unpack_alignment(GlPixelStoreAlignment::One);
        gl.pixel_store_pack_alignment(GlPixelStoreAlignment::Eight);
        gl.active_texture(GlTextureUnit::from_index(1)).unwrap();
        let texture_2d_unit_1 = gl.gen_texture().expect("texture unit 1");
        let texture_array_unit_1 = gl.gen_texture().expect("texture array unit 1");
        let texture_2d_unit_0 = gl.gen_texture().expect("texture unit 0");
        let texture_array_unit_0 = gl.gen_texture().expect("texture array unit 0");
        gl.bind_texture(GlTextureTarget::Texture2D, Some(texture_2d_unit_1));
        gl.bind_texture(GlTextureTarget::Texture2DArray, Some(texture_array_unit_1));
        gl.active_texture(GlTextureUnit::ZERO).unwrap();
        gl.bind_texture(GlTextureTarget::Texture2D, Some(texture_2d_unit_0));
        gl.bind_texture(GlTextureTarget::Texture2DArray, Some(texture_array_unit_0));
        assert!(
            gl.active_texture(GlTextureUnit::from_index(u32::MAX))
                .unwrap_err()
                .contains("texture unit index")
        );
        gl.clear_color_buffer();

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(snapshot.clear_scissor_enabled, vec![true]);
        assert!(snapshot.blend_enabled);
        assert!(snapshot.scissor_enabled);
        assert_eq!(
            snapshot.enabled_capabilities,
            vec![
                GlCapability::CullFace.as_raw(),
                GlCapability::DepthTest.as_raw(),
                GlCapability::Blend.as_raw(),
                GlCapability::ScissorTest.as_raw(),
            ]
        );
        assert_eq!(
            snapshot.depth_function,
            Some(GlDepthFunction::LessOrEqual.as_raw())
        );
        assert!(!snapshot.depth_mask);
        gl.depth_range(GlDepthRange::new(0.25, 0.75)).unwrap();
        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(
            snapshot.depth_range_f_calls,
            vec![[0.25f32.to_bits(), 0.75f32.to_bits()]]
        );
        assert_eq!(snapshot.cull_face_mode, Some(GlCullFaceMode::Back.as_raw()));
        assert_eq!(
            snapshot.front_face_winding,
            Some(GlFrontFaceWinding::CounterClockwise.as_raw())
        );
        assert_eq!(
            snapshot.stencil_func,
            Some(super::FakeStencilFuncCall {
                function: GlStencilFunction::Always.as_raw(),
                reference: 3,
                mask: 0x0f,
            })
        );
        assert_eq!(snapshot.stencil_mask, 0xf0);
        assert_eq!(
            snapshot.stencil_op,
            Some(super::FakeStencilOpCall {
                stencil_fail: GlStencilOperation::Keep.as_raw(),
                depth_fail: GlStencilOperation::Replace.as_raw(),
                depth_pass: GlStencilOperation::IncrementWrap.as_raw(),
            })
        );
        assert_eq!(
            snapshot.stencil_func_separate_calls,
            vec![super::FakeStencilFuncSeparateCall {
                face: GlStencilFace::Back.as_raw(),
                function: GlStencilFunction::LessOrEqual.as_raw(),
                reference: 2,
                mask: u32::MAX,
            }]
        );
        assert_eq!(
            snapshot.stencil_mask_separate_calls,
            vec![super::FakeStencilMaskSeparateCall {
                face: GlStencilFace::Front.as_raw(),
                mask: 0,
            }]
        );
        assert_eq!(
            snapshot.stencil_op_separate_calls,
            vec![super::FakeStencilOpSeparateCall {
                face: GlStencilFace::FrontAndBack.as_raw(),
                stencil_fail: GlStencilOperation::DecrementClamp.as_raw(),
                depth_fail: GlStencilOperation::Invert.as_raw(),
                depth_pass: GlStencilOperation::Keep.as_raw(),
            }]
        );
        assert_eq!(snapshot.color_write_mask, [true, true, true, false]);
        assert_eq!(
            snapshot.polygon_offset,
            Some([1.25f32.to_bits(), (-2.5f32).to_bits()])
        );
        assert_eq!(snapshot.bound_array_buffer, 11);
        assert_eq!(snapshot.bound_element_array_buffer, 12);
        assert_eq!(snapshot.bound_framebuffer, 13);
        assert_eq!(snapshot.framebuffer_bindings, vec![13]);
        assert_eq!(snapshot.viewport_calls, vec![(1, 2, 320, 240)]);
        assert_eq!(snapshot.scissor_calls, vec![(3, 4, 160, 120)]);
        assert_eq!(snapshot.pack_alignment, 8);
        assert_eq!(snapshot.pack_alignment_calls, vec![8]);
        assert_eq!(snapshot.unpack_alignment, 1);
        assert_eq!(snapshot.unpack_alignment_calls, vec![1]);
        assert_eq!(snapshot.active_texture, 0);
        assert_eq!(snapshot.bound_texture_2d, texture_2d_unit_0.as_raw());
        assert_eq!(
            snapshot.bound_texture_2d_units,
            vec![
                (0, texture_2d_unit_0.as_raw()),
                (1, texture_2d_unit_1.as_raw())
            ]
        );
        assert_eq!(
            snapshot.bound_texture_2d_array,
            texture_array_unit_0.as_raw()
        );
        assert_eq!(
            snapshot.bound_texture_2d_array_units,
            vec![
                (0, texture_array_unit_0.as_raw()),
                (1, texture_array_unit_1.as_raw())
            ]
        );

        gl.disable(GlCapability::Blend);
        gl.disable(GlCapability::DepthTest);
        gl.disable(GlCapability::ScissorTest);
        gl.disable(GlCapability::CullFace);
        gl.unbind_buffer(GlBufferTarget::ArrayBuffer);
        gl.unbind_buffer(GlBufferTarget::ElementArrayBuffer);
        gl.unbind_framebuffer(GlFramebufferTarget::Framebuffer);
        gl.pixel_store_unpack_alignment(GlPixelStoreAlignment::Four);
        gl.pixel_store_pack_alignment(GlPixelStoreAlignment::Four);
        gl.active_texture(GlTextureUnit::from_index(1)).unwrap();
        gl.bind_texture(GlTextureTarget::Texture2D, None);
        gl.bind_texture(GlTextureTarget::Texture2DArray, None);
        gl.active_texture(GlTextureUnit::ZERO).unwrap();
        gl.bind_texture(GlTextureTarget::Texture2D, None);
        gl.bind_texture(GlTextureTarget::Texture2DArray, None);
        gl.delete_texture(texture_2d_unit_1);
        gl.delete_texture(texture_2d_unit_0);
        gl.delete_texture(texture_array_unit_1);
        gl.delete_texture(texture_array_unit_0);
        gl.clear_color_buffer();

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert_eq!(snapshot.clear_scissor_enabled, vec![true, false]);
        assert!(!snapshot.blend_enabled);
        assert!(!snapshot.scissor_enabled);
        assert!(snapshot.enabled_capabilities.is_empty());
        assert_eq!(snapshot.bound_array_buffer, 0);
        assert_eq!(snapshot.bound_element_array_buffer, 0);
        assert_eq!(snapshot.bound_framebuffer, 0);
        assert_eq!(snapshot.framebuffer_bindings, vec![13, 0]);
        assert_eq!(snapshot.pack_alignment, 4);
        assert_eq!(snapshot.pack_alignment_calls, vec![8, 4]);
        assert_eq!(snapshot.unpack_alignment, 4);
        assert_eq!(snapshot.unpack_alignment_calls, vec![1, 4]);
        assert_eq!(snapshot.active_texture, 0);
        assert_eq!(snapshot.bound_texture_2d, 0);
        assert_eq!(snapshot.bound_texture_2d_units, vec![(0, 0), (1, 0)]);
        assert_eq!(
            snapshot.deleted_textures,
            vec![
                texture_2d_unit_1.as_raw(),
                texture_2d_unit_0.as_raw(),
                texture_array_unit_1.as_raw(),
                texture_array_unit_0.as_raw(),
            ]
        );
        assert_eq!(snapshot.bound_texture_2d_array, 0);
        assert_eq!(snapshot.bound_texture_2d_array_units, vec![(0, 0), (1, 0)]);
    }

    #[test]
    fn typed_rectangles_reject_sizes_that_exceed_gl_sizei() {
        let _guard = fake_gl_test_guard();
        let gl = glsym::fake_for_testing(FakeGlConfig::default());

        assert!(
            gl.viewport(GlRect::new(0, 0, u32::MAX, 1))
                .unwrap_err()
                .contains("glViewport width")
        );
        assert!(
            gl.scissor(GlRect::new(0, 0, 1, u32::MAX))
                .unwrap_err()
                .contains("glScissor height")
        );

        let snapshot = glsym::snapshot_fake_state_for_testing();
        assert!(snapshot.viewport_calls.is_empty());
        assert!(snapshot.scissor_calls.is_empty());
    }
}
