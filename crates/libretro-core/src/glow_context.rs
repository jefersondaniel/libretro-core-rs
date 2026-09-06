//! Frontend procedure loading for glow. Context ownership remains with the
//! frontend; this module only constructs dispatch tables during context reset.

use crate::{HwContextType, Runtime};
use glow::HasContext;
use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};

impl Runtime<'_> {
    /// Loads a standard [`glow::Context`] for the current frontend GL context.
    ///
    /// Call only inside [`crate::Core::hw_context_reset`], after successful
    /// hardware negotiation. Other callbacks return an error without calling GL.
    /// Store the result and use it only while that same context is current (in
    /// `run`, reset, and destroy callbacks). Discard it on context destruction;
    /// a subsequent reset requires a fresh value and fresh GPU resources.
    ///
    /// This does not create a window/context, bind a framebuffer, reset GL state,
    /// or make glow's unsafe commands safe. Query [`Self::current_framebuffer`]
    /// every frame. Zero is a valid default framebuffer, represented by `None`
    /// in glow's `bind_framebuffer` API.
    ///
    /// Missing initialization functions and glow initialization panics become
    /// errors on unwind builds. As with the libretro ABI generally, the frontend
    /// must provide valid procedure addresses and a current context in reset.
    /// Do not register glow debug callbacks that outlive the current context:
    /// dropping a context with a debug callback may itself call GL.
    ///
    /// ```no_run
    /// use libretro_core::{Core, Runtime, glow};
    /// #[derive(Default)]
    /// struct Graphics { gl: Option<glow::Context> }
    /// impl Core for Graphics {
    ///     fn system_info(&self) -> libretro_core::SystemInfo {
    ///         libretro_core::SystemInfo::new("graphics", "1.0.0")
    ///     }
    ///     fn av_info(&self) -> libretro_core::SystemAvInfo {
    ///         libretro_core::fixed_system_av_info(320, 240, 60.0, 48_000.0)
    ///     }
    ///     fn run(&mut self, _: &mut Runtime<'_>) { /* render with self.gl */ }
    ///     fn hw_context_reset(&mut self, rt: &mut Runtime<'_>) {
    ///         self.gl = rt.create_glow_context().ok();
    ///     }
    ///     fn hw_context_destroy(&mut self, _: &mut Runtime<'_>) {
    ///         self.gl = None;
    ///     }
    /// }
    /// ```
    pub fn create_glow_context(&self) -> Result<glow::Context, String> {
        if !self.state.creating_glow_context_allowed {
            return Err("create_glow_context is only available inside hw_context_reset".into());
        }
        match self.hw_context_type() {
            Some(
                HwContextType::OpenGl
                | HwContextType::OpenGlCore
                | HwContextType::OpenGlEs2
                | HwContextType::OpenGlEs3
                | HwContextType::OpenGlEsVersion,
            ) => {}
            _ => return Err("the negotiated hardware context is not OpenGL/OpenGL ES".into()),
        }
        load_context(|name| self.hw_proc_address(name).unwrap_or(std::ptr::null()))
    }
}

fn load_context(mut loader: impl FnMut(&str) -> *const c_void) -> Result<glow::Context, String> {
    // glow invokes these during initialization. Cache addresses so preflight
    // and construction use the same entries, even with a stateful loader.
    let mut addresses = std::collections::HashMap::new();
    for name in ["glGetString", "glGetIntegerv"] {
        let address = loader(name);
        if address.is_null() {
            return Err(format!("cannot initialize glow: missing {name}"));
        }
        addresses.insert(name.to_owned(), address);
    }
    let context = catch_unwind(AssertUnwindSafe(|| unsafe {
        // SAFETY: called only in the frontend's context-reset callback. The
        // frontend ABI promises valid addresses and a current negotiated context.
        glow::Context::from_loader_function(|name| {
            *addresses
                .entry(name.to_owned())
                .or_insert_with(|| loader(name))
        })
    }))
    .map_err(|_| {
        "glow initialization failed: check the current GL context and frontend procedure lookup"
            .to_owned()
    })?;

    // Some GLES2 frontends expose only extension spellings. A non-null core
    // address from global lookup is not proof the operation is legal. Only use
    // an alias when the live context advertises that extension.
    let version = context.version();
    let core_instancing = if version.is_embedded {
        version.major >= 3
    } else {
        (version.major, version.minor) >= (3, 3)
    };
    let extension = if core_instancing {
        None
    } else {
        [
            ("GL_ARB_instanced_arrays", "ARB"),
            ("GL_EXT_instanced_arrays", "EXT"),
            ("GL_ANGLE_instanced_arrays", "ANGLE"),
        ]
        .into_iter()
        .find(|(name, _)| context.supported_extensions().contains(*name))
    };
    if let Some((_, suffix)) = extension {
        let alias = loader(&format!("glVertexAttribDivisor{suffix}"));
        if !alias.is_null() {
            addresses.insert("glVertexAttribDivisor".into(), alias);
            return catch_unwind(AssertUnwindSafe(|| unsafe {
                glow::Context::from_loader_function(|name| {
                    *addresses
                        .entry(name.to_owned())
                        .or_insert_with(|| loader(name))
                })
            }))
            .map_err(|_| "glow extension dispatch initialization failed".into());
        }
    }
    Ok(context)
}

#[cfg(test)]
mod tests {
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
            "glVertexAttribDivisorANGLE"
            | "glVertexAttribDivisorEXT"
            | "glVertexAttribDivisorARB" => alias_divisor as *const c_void,
            _ => std::ptr::null(),
        })
        .unwrap();
        // Mock dispatch only; a real GLES2 caller must not invoke this unsupported operation.
        unsafe {
            gl.vertex_attrib_divisor(0, 7);
        }
        assert_eq!(DIVISOR.get(), 999);
    }
}
