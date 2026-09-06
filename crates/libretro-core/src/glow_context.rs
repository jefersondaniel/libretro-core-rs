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
    if !core_instancing {
        for (extension, suffix) in [
            ("GL_ARB_instanced_arrays", "ARB"),
            ("GL_EXT_instanced_arrays", "EXT"),
            ("GL_ANGLE_instanced_arrays", "ANGLE"),
        ] {
            if !context.supported_extensions().contains(extension) {
                continue;
            }
            let symbol = format!("glVertexAttribDivisor{suffix}");
            let alias = *addresses
                .entry(symbol.clone())
                .or_insert_with(|| loader(&symbol));
            if alias.is_null() {
                continue;
            }
            addresses.insert("glVertexAttribDivisor".into(), alias);
            // Reuse the original addresses; only this supported alias changes.
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
#[path = "glow_context_tests.rs"]
mod tests;
