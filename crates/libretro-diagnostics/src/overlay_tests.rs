//! Driver-boundary regressions using typed ABI stubs. These tests model inherited
//! attribute state and failed allocations; real driver coverage lives in smoke_glow.py.
use super::*;
use std::{
    cell::RefCell,
    ffi::{c_char, c_void},
    num::NonZeroU32,
};
#[derive(Default)]
struct Driver {
    divisors: [u32; 2],
    draws: Vec<[u32; 2]>,
    error: u32,
    fail_upload: bool,
}
thread_local! {static DRIVER:RefCell<Driver> = RefCell::new(Driver::default());}
unsafe extern "system" fn get_string(name: u32) -> *const u8 {
    match name {
        glow::VERSION => c"OpenGL ES 2.0 stub".as_ptr().cast(),
        glow::EXTENSIONS => c"GL_ANGLE_instanced_arrays".as_ptr().cast(),
        _ => c"".as_ptr().cast(),
    }
}
unsafe extern "system" fn get_integer(_: u32, out: *mut i32) {
    unsafe {
        *out = 0;
    }
}
unsafe extern "system" fn get_error() -> u32 {
    DRIVER.with(|d| std::mem::take(&mut d.borrow_mut().error))
}
unsafe extern "system" fn uniform_location(_: u32, _: *const c_char) -> i32 {
    0
}
unsafe extern "system" fn one_u32(_: u32) {}
unsafe extern "system" fn two_u32(_: u32, _: u32) {}
unsafe extern "system" fn uniform4(_: i32, _: f32, _: f32, _: f32, _: f32) {}
unsafe extern "system" fn uniform4v(_: i32, _: i32, _: *const f32) {}
unsafe extern "system" fn uniform1(_: i32, _: i32) {}
unsafe extern "system" fn attribute(_: u32, _: i32, _: u32, _: u8, _: i32, _: *const c_void) {}
unsafe extern "system" fn divisor(index: u32, value: u32) {
    DRIVER.with(|d| d.borrow_mut().divisors[index as usize] = value);
}
unsafe extern "system" fn draw(_: u32, _: i32, _: i32) {
    DRIVER.with(|d| {
        let mut d = d.borrow_mut();
        let state = d.divisors;
        d.draws.push(state);
    });
}
unsafe extern "system" fn buffer_data(_: u32, _: isize, _: *const c_void, _: u32) {
    DRIVER.with(|d| {
        let mut d = d.borrow_mut();
        if d.fail_upload {
            d.error = glow::OUT_OF_MEMORY;
        }
    });
}
fn context() -> glow::Context {
    unsafe {
        glow::Context::from_loader_function(|name| match name {
            "glGetString" => get_string as *const c_void,
            "glGetIntegerv" => get_integer as *const c_void,
            "glGetError" => get_error as *const c_void,
            "glGetUniformLocation" => uniform_location as *const c_void,
            "glUseProgram"
            | "glActiveTexture"
            | "glEnable"
            | "glDisable"
            | "glEnableVertexAttribArray"
            | "glDisableVertexAttribArray"
            | "glBlendEquation" => one_u32 as *const c_void,
            "glBindTexture" | "glBindBuffer" | "glBlendFunc" => two_u32 as *const c_void,
            "glUniform4f" => uniform4 as *const c_void,
            "glUniform4fv" => uniform4v as *const c_void,
            "glUniform1i" => uniform1 as *const c_void,
            "glVertexAttribPointer" => attribute as *const c_void,
            "glVertexAttribDivisor" => divisor as *const c_void,
            "glDrawArrays" => draw as *const c_void,
            "glBufferData" => buffer_data as *const c_void,
            _ => std::ptr::null(),
        })
    }
}
fn overlay() -> DiagnosticTextOverlay {
    DiagnosticTextOverlay {
        font: DiagnosticFont::from_fnt_v1(DIAGNOSTIC_FONT_BYTES).unwrap(),
        layout: DiagnosticTextLayout::DEFAULT,
        program: glow::NativeProgram(NonZeroU32::new(1).unwrap()),
        buffer: glow::NativeBuffer(NonZeroU32::new(2).unwrap()),
        texture: glow::NativeTexture(NonZeroU32::new(3).unwrap()),
        vao: None,
        count: 6,
        reset_divisors: true,
        uniforms: TextUniforms {
            viewport: glow::NativeUniformLocation(0),
            color: glow::NativeUniformLocation(1),
            font: glow::NativeUniformLocation(2),
        },
    }
}
#[test]
fn standalone_text_resets_inherited_gles2_divisors() {
    DRIVER.with(|d| {
        *d.borrow_mut() = Driver {
            divisors: [3, 4],
            ..Driver::default()
        }
    });
    let gl = context();
    let text = overlay();
    unsafe {
        text.draw(&gl, 320, 240, [1.0; 4]).unwrap();
    }
    DRIVER.with(|d| assert_eq!(d.borrow().draws, vec![[0, 0]]));
}
#[test]
fn failed_upload_does_not_publish_a_larger_draw_count() {
    DRIVER.with(|d| {
        *d.borrow_mut() = Driver {
            fail_upload: true,
            ..Driver::default()
        }
    });
    let gl = context();
    let mut text = overlay();
    let result = unsafe { text.update_lines(&gl, &["a much longer string"]) };
    assert!(
        result.is_err(),
        "failed GPU allocation must reach the caller"
    );
    assert_eq!(
        text.count, 0,
        "failed upload must disable drawing stale/undefined buffer contents"
    );
    unsafe {
        text.draw(&gl, 320, 240, [1.0; 4]).unwrap();
    }
    DRIVER.with(|d| {
        assert!(d.borrow().draws.is_empty());
        d.borrow_mut().fail_upload = false;
    });
    unsafe {
        text.update_lines(&gl, &["recovered"]).unwrap();
        text.draw(&gl, 320, 240, [1.0; 4]).unwrap();
    }
    DRIVER.with(|d| assert_eq!(d.borrow().draws.len(), 1));
}
