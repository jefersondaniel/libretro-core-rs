#!/usr/bin/env python3
"""Validate a core against installed Mesa EGL (not RetroArch).

Uses Python stdlib and installed system libraries; downloads nothing. Asserts
nonempty rendered output, audio delivery, framebuffer zero/nonzero, and matching
pixels after controlled destruction and context-loss recreation.
"""
import argparse
import ctypes as C
import ctypes.util
import hashlib
import json
import os
from pathlib import Path

U = C.c_uint
I = C.c_int
P = C.c_void_p
B = C.c_bool
S = C.c_size_t
F = C.c_float
D = C.c_double
CB = C.CFUNCTYPE


class Geometry(C.Structure):
    _fields_ = [("width", U), ("height", U), ("max_width", U),
                ("max_height", U), ("aspect", F)]


class Timing(C.Structure):
    _fields_ = [("fps", D), ("sample_rate", D)]


class AvInfo(C.Structure):
    _fields_ = [("geometry", Geometry), ("timing", Timing)]


class GameInfo(C.Structure):
    _fields_ = [("path", C.c_char_p), ("data", P), ("size", S), ("meta", C.c_char_p)]


class HwRender(C.Structure):
    _fields_ = [("context", I), ("reset", P), ("framebuffer", P), ("proc", P),
                ("depth", B), ("stencil", B), ("bottom_left", B),
                ("major", U), ("minor", U), ("cache", B), ("destroy", P), ("debug", B)]


class Message(C.Structure):
    _fields_ = [("text", C.c_char_p), ("frames", U)]


def bind(library, name, result, *arguments):
    function = getattr(library, name)
    function.restype = result
    function.argtypes = arguments
    return function


class Egl:
    def __init__(self, version, width, height):
        self.version, self.width, self.height = version, width, height
        self.lib = C.CDLL(ctypes_library("EGL"))
        self.get_proc = bind(self.lib, "eglGetProcAddress", P, C.c_char_p)
        platform_address = self.get_proc(b"eglGetPlatformDisplayEXT")
        assert platform_address, "EGL platform display extension unavailable"
        platform = CB(P, U, P, C.POINTER(I))(platform_address)
        self.display = platform(0x31DD, None, None)  # EGL_PLATFORM_SURFACELESS_MESA
        self.initialize = bind(self.lib, "eglInitialize", U, P, C.POINTER(I), C.POINTER(I))
        major, minor = I(), I()
        assert self.initialize(self.display, C.byref(major), C.byref(minor)), "eglInitialize failed"
        assert bind(self.lib, "eglBindAPI", U, U)(0x30A0), "eglBindAPI ES failed"
        attributes = (I * 15)(0x3033, 1, 0x3040, 4, 0x3024, 8, 0x3023, 8,
                              0x3022, 8, 0x3021, 8, 0x3025, 0, 0x3038)
        config, count = P(), I()
        choose = bind(self.lib, "eglChooseConfig", U, P, C.POINTER(I), C.POINTER(P), I, C.POINTER(I))
        assert choose(self.display, attributes, C.byref(config), 1, C.byref(count)) and count.value, "No EGL pbuffer ES config"
        self.config = config
        self.create_context = bind(self.lib, "eglCreateContext", P, P, P, P, C.POINTER(I))
        self.create_surface = bind(self.lib, "eglCreatePbufferSurface", P, P, P, C.POINTER(I))
        self.make_current = bind(self.lib, "eglMakeCurrent", U, P, P, P, P)
        self.destroy_context = bind(self.lib, "eglDestroyContext", U, P, P)
        self.destroy_surface = bind(self.lib, "eglDestroySurface", U, P, P)
        self.context = self.surface = None
        self.recreate()

    def recreate(self):
        if self.context:
            assert self.make_current(self.display, None, None, None)
            assert self.destroy_context(self.display, self.context)
            assert self.destroy_surface(self.display, self.surface)
        context_attributes = (I * 3)(0x3098, self.version, 0x3038)
        surface_attributes = (I * 5)(0x3057, self.width, 0x3056, self.height, 0x3038)
        self.context = self.create_context(self.display, self.config, None, context_attributes)
        self.surface = self.create_surface(self.display, self.config, surface_attributes)
        assert self.context and self.surface, "EGL context/pbuffer creation failed"
        assert self.make_current(self.display, self.surface, self.surface, self.context), "eglMakeCurrent failed"

    def gl(self, name, result, *arguments):
        address = self.get_proc(name.encode())
        assert address, f"Missing GL function {name}"
        return CB(result, *arguments)(address)


def ctypes_library(name):
    path = C.util.find_library(name)
    assert path, f"System lib{name} is unavailable"
    return path


class Frontend:
    def __init__(self, core, version):
        self.core = C.CDLL(str(Path(core).resolve()))
        av = AvInfo()
        bind(self.core, "retro_get_system_av_info", None, C.POINTER(AvInfo))(C.byref(av))
        self.width, self.height = av.geometry.width, av.geometry.height
        assert 0 < self.width <= 4096 and 0 < self.height <= 4096
        self.egl = Egl(version, self.width, self.height)
        self.version = version
        self.actual_version = self.egl.gl("glGetString", C.c_char_p, U)(0x1f02).decode()
        assert self.actual_version.startswith(f"OpenGL ES {version}.0"), self.actual_version
        self.callbacks, self.errors, self.messages = [], [], []
        self.reset = self.destroy = None
        self.framebuffer = self.frames = self.audio = self.polls = 0
        self.capture = None
        self.install()
        bind(self.core, "retro_init", None)()
        game = GameInfo(b"smoke.bin", None, 0, None)
        assert bind(self.core, "retro_load_game", B, C.POINTER(GameInfo))(C.byref(game))
        assert self.reset and self.destroy, "No hardware callbacks negotiated"
        self.targets()
        self.reset()
        self.check()

    def callback(self, result, arguments, function, fallback=None):
        def guarded(*args):
            try:
                return function(*args)
            except BaseException as error:
                self.errors.append(str(error))
                return fallback
        value = CB(result, *arguments)(guarded)
        self.callbacks.append(value)
        return value

    def install(self):
        self.proc = self.callback(P, [C.c_char_p], self.egl.get_proc)
        self.fbo = self.callback(S, [], lambda: self.framebuffer)
        callbacks = [
            ("environment", B, [U, P], self.environment, False),
            ("video_refresh", None, [P, U, U, S], self.video, None),
            ("audio_sample_batch", S, [P, S], self.audio_batch, 0),
            ("audio_sample", None, [C.c_int16, C.c_int16], lambda *_: None, None),
            ("input_poll", None, [], self.poll, None),
            ("input_state", C.c_int16, [U, U, U, U], lambda *_: 0, 0),
        ]
        for name, result, args, function, fallback in callbacks:
            callback = self.callback(result, args, function, fallback)
            bind(self.core, "retro_set_" + name, None, type(callback))(callback)

    def environment(self, command, data):
        command &= 0xffff
        if command == 56:  # preferred HW context
            C.cast(data, C.POINTER(U))[0] = 2 if self.version == 2 else 4
            return True
        if command == 14:  # SET_HW_RENDER
            hw = C.cast(data, C.POINTER(HwRender)).contents
            if hw.context != (2 if self.version == 2 else 4):
                return False
            self.reset, self.destroy = CB(None)(hw.reset), CB(None)(hw.destroy)
            hw.framebuffer, hw.proc = C.cast(self.fbo, P).value, C.cast(self.proc, P).value
            return True
        if command == 6:
            self.messages.append(C.cast(data, C.POINTER(Message)).contents.text.decode())
            return True
        if command in (10, 11, 18, 35):
            return True
        return False

    def gl(self, name, result, args, *values):
        return self.egl.gl(name, result, *args)(*values)

    def targets(self):
        texture, framebuffer = U(), U()
        self.gl("glGenTextures", None, [I, C.POINTER(U)], 1, C.byref(texture))
        self.gl("glBindTexture", None, [U, U], 0x0de1, texture.value)
        self.gl("glTexParameteri", None, [U, U, I], 0x0de1, 0x2801, 0x2600)
        self.gl("glTexImage2D", None, [U, I, I, I, I, I, U, U, P],
                0x0de1, 0, 0x1908, self.width, self.height, 0, 0x1908, 0x1401, None)
        self.gl("glGenFramebuffers", None, [I, C.POINTER(U)], 1, C.byref(framebuffer))
        self.gl("glBindFramebuffer", None, [U, U], 0x8d40, framebuffer.value)
        self.gl("glFramebufferTexture2D", None, [U, U, U, U, I], 0x8d40, 0x8ce0, 0x0de1, texture.value, 0)
        assert self.gl("glCheckFramebufferStatus", U, [U], 0x8d40) == 0x8cd5
        self.target = framebuffer.value
        self.gl("glBindFramebuffer", None, [U, U], 0x8d40, 0)
        self.gl("glBindTexture", None, [U, U], 0x0de1, 0)
        self.framebuffer = 0

    def video(self, data, width, height, pitch):
        assert data == C.c_void_p(-1).value, "Expected hardware frame"
        assert (width, height) == (self.width, self.height)
        self.frames += 1
        self.gl("glBindFramebuffer", None, [U, U], 0x8d40, self.framebuffer)
        self.gl("glPixelStorei", None, [U, I], 0x0d05, 1)
        pixels = (C.c_ubyte * (width * height * 4))()
        self.gl("glReadPixels", None, [I, I, I, I, U, U, P], 0, 0, width, height, 0x1908, 0x1401, pixels)
        self.capture = bytes(pixels)
        self.gl("glBindFramebuffer", None, [U, U], 0x8d40, 0)

    def audio_batch(self, data, frames):
        self.audio += frames
        return frames

    def poll(self):
        self.polls += 1

    def check(self):
        assert not self.errors, self.errors
        assert not self.messages, self.messages
        assert self.gl("glGetError", U, []) == 0, "GL error"

    def run_frames(self):
        baseline = self.frames
        for index in range(6):
            self.framebuffer = self.target if index % 2 else 0
            bind(self.core, "retro_run", None)()
            self.check()
        assert self.frames == baseline + 6
        assert self.capture and any(self.capture[i] for i in range(len(self.capture)) if i % 4 != 3)
        return hashlib.sha256(self.capture).hexdigest()

    def run(self):
        initial = self.run_frames()
        for lost in (False, True):
            if not lost:
                self.destroy()
                self.check()
            self.egl.recreate()
            self.targets()
            self.reset()
            self.check()
            assert self.run_frames() == initial, "Reset did not restore rendered output"
        self.destroy()
        self.check()
        bind(self.core, "retro_unload_game", None)()
        bind(self.core, "retro_deinit", None)()
        assert self.audio == self.frames * 800
        assert self.polls == self.frames
        print(json.dumps({"gles": self.version, "actual_version": self.actual_version, "frames": self.frames, "audio_frames": self.audio,
                          "framebuffers": [0, self.target], "reset_pixels_sha256": initial}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--core", required=True)
    parser.add_argument("--gles", type=int, choices=(2, 3), required=True)
    args = parser.parse_args()
    os.environ["MESA_GLES_VERSION_OVERRIDE"] = f"{args.gles}.0"
    os.environ.setdefault("LIBGL_ALWAYS_SOFTWARE", "1")
    vendor = Path("/usr/share/glvnd/egl_vendor.d/50_mesa.json")
    if vendor.exists():
        os.environ.setdefault("__EGL_VENDOR_LIBRARY_FILENAMES", str(vendor))
    Frontend(args.core, args.gles).run()


if __name__ == "__main__":
    main()
