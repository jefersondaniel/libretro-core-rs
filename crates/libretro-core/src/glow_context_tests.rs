//! Procedure-loader regression tests with typed ABI stubs; no live GL required.
use super::*;
use crate::CoreState;

#[test]
fn rejects_calls_outside_reset_before_looking_up_symbols() {
    let mut state = CoreState::default();
    assert!(
        Runtime { state: &mut state }
            .create_glow_context()
            .err()
            .unwrap()
            .contains("hw_context_reset")
    );
}

#[test]
fn missing_initialization_symbol_is_an_error() {
    assert!(
        load_context(|_| std::ptr::null())
            .err()
            .unwrap()
            .contains("glGetString")
    );
}

unsafe extern "system" fn get_string(name: u32) -> *const u8 {
    match name {
        glow::VERSION => c"OpenGL ES 2.0 test".as_ptr().cast(),
        glow::EXTENSIONS => c"".as_ptr().cast(),
        _ => std::ptr::null(),
    }
}
unsafe extern "system" fn get_integer(_: u32, value: *mut i32) {
    unsafe {
        *value = 0;
    }
}
#[test]
fn creates_a_standard_gles2_glow_context() {
    let gl = load_context(|name| match name {
        "glGetString" => get_string as *const c_void,
        "glGetIntegerv" => get_integer as *const c_void,
        _ => std::ptr::null(),
    })
    .unwrap();
    assert!(gl.version().is_embedded);
    assert_eq!(gl.version().major, 2);
}
unsafe extern "system" fn null_string(_: u32) -> *const u8 {
    std::ptr::null()
}
#[test]
fn invalid_current_context_becomes_an_error() {
    let result = load_context(|name| match name {
        "glGetString" => null_string as *const c_void,
        "glGetIntegerv" => get_integer as *const c_void,
        _ => std::ptr::null(),
    });
    assert!(result.is_err());
}
unsafe extern "system" fn modern_string(name: u32) -> *const u8 {
    if name == glow::VERSION {
        c"OpenGL ES 3.0 test".as_ptr().cast()
    } else {
        c"".as_ptr().cast()
    }
}
#[test]
fn initializes_gles3_with_indexed_extension_enumeration() {
    let gl = load_context(|name| match name {
        "glGetString" => modern_string as *const c_void,
        "glGetIntegerv" => get_integer as *const c_void,
        _ => std::ptr::null(),
    })
    .unwrap();
    assert_eq!(gl.version().major, 3);
}
thread_local! { static DIVISOR: std::cell::Cell<u32> = const { std::cell::Cell::new(0) }; }
unsafe extern "system" fn extension_string(name: u32) -> *const u8 {
    if name == glow::EXTENSIONS {
        c"GL_ANGLE_instanced_arrays".as_ptr().cast()
    } else {
        unsafe { get_string(name) }
    }
}
unsafe extern "system" fn alias_divisor(_: u32, value: u32) {
    DIVISOR.set(value);
}
unsafe extern "system" fn unsupported_core_divisor(_: u32, _: u32) {
    DIVISOR.set(999);
}
#[test]
fn advertised_gles2_alias_overrides_nonnull_unsupported_core_symbol() {
    DIVISOR.set(0);
    let gl = load_context(|name| match name {
        "glGetString" => extension_string as *const c_void,
        "glGetIntegerv" => get_integer as *const c_void,
        "glVertexAttribDivisor" => unsupported_core_divisor as *const c_void,
        "glVertexAttribDivisorANGLE" => alias_divisor as *const c_void,
        _ => std::ptr::null(),
    })
    .unwrap();
    unsafe {
        gl.vertex_attrib_divisor(0, 7);
    }
    assert_eq!(DIVISOR.get(), 7);
}
#[test]
fn does_not_replace_core_dispatch_with_an_unadvertised_alias() {
    DIVISOR.set(0);
    let gl = load_context(|name| match name {
        "glGetString" => get_string as *const c_void,
        "glGetIntegerv" => get_integer as *const c_void,
        "glVertexAttribDivisor" => unsupported_core_divisor as *const c_void,
        "glVertexAttribDivisorANGLE" | "glVertexAttribDivisorEXT" | "glVertexAttribDivisorARB" => {
            alias_divisor as *const c_void
        }
        _ => std::ptr::null(),
    })
    .unwrap();
    // Mock dispatch only; a real GLES2 caller must not invoke this unsupported operation.
    unsafe {
        gl.vertex_attrib_divisor(0, 7);
    }
    assert_eq!(DIVISOR.get(), 999);
}
unsafe extern "system" fn two_extensions(name: u32) -> *const u8 {
    if name == glow::EXTENSIONS {
        c"GL_EXT_instanced_arrays GL_ANGLE_instanced_arrays"
            .as_ptr()
            .cast()
    } else {
        unsafe { get_string(name) }
    }
}
#[test]
fn tries_later_advertised_alias_when_first_is_missing() {
    DIVISOR.set(0);
    let gl = load_context(|name| match name {
        "glGetString" => two_extensions as *const c_void,
        "glGetIntegerv" => get_integer as *const c_void,
        "glVertexAttribDivisor" => unsupported_core_divisor as *const c_void,
        "glVertexAttribDivisorANGLE" => alias_divisor as *const c_void,
        _ => std::ptr::null(),
    })
    .unwrap();
    unsafe {
        gl.vertex_attrib_divisor(0, 7);
    }
    assert_eq!(DIVISOR.get(), 7);
}

struct ResetCore {
    panic: bool,
}
impl crate::Core for ResetCore {
    fn system_info(&self) -> crate::SystemInfo {
        crate::SystemInfo::new("reset-test", "1")
    }
    fn av_info(&self) -> crate::SystemAvInfo {
        crate::fixed_system_av_info(1, 1, 60.0, 48000.0)
    }
    fn run(&mut self, _: &mut Runtime<'_>) {}
    fn hw_context_reset(&mut self, rt: &mut Runtime<'_>) {
        assert!(rt.state.creating_glow_context_allowed);
        if self.panic {
            panic!("reset failure");
        }
    }
}
#[test]
fn reset_permission_is_scoped_even_when_core_panics() {
    for panic in [false, true] {
        let mut state = CoreState {
            core: Some(Box::new(ResetCore { panic })),
            ..CoreState::default()
        };
        crate::dispatch_hw_context_reset(&mut state);
        assert!(!state.creating_glow_context_allowed);
        assert!(state.core.is_some(), "core must survive callback unwinding");
        assert!(
            Runtime { state: &mut state }
                .create_glow_context()
                .err()
                .unwrap()
                .contains("hw_context_reset")
        );
    }
}
