#!/usr/bin/env python3
"""Generate raw GLES <= 3.0 symbol coverage from the vendored Khronos gl.xml."""

from __future__ import annotations

import keyword
import re
import sys
import textwrap
import xml.etree.ElementTree as ET
from collections import Counter
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
GL_XML = ROOT / "crates/libretro-core/opengl-registry/gl.xml"
OUT_RS = ROOT / "crates/libretro-core/src/glsym_raw.rs"
MANIFEST = ROOT / "crates/libretro-core/opengl-registry/glsym_coverage_manifest.txt"
SCOPE_API = "gles2"
SCOPE_MAX_VERSION = 3.0
SCOPE_LABEL = "OpenGL ES 2.0/3.0 core"

RUST_KEYWORDS = {
    "as",
    "break",
    "const",
    "continue",
    "crate",
    "else",
    "enum",
    "extern",
    "false",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "match",
    "mod",
    "move",
    "mut",
    "pub",
    "ref",
    "return",
    "self",
    "Self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "type",
    "unsafe",
    "use",
    "where",
    "while",
    "async",
    "await",
    "dyn",
    "abstract",
    "become",
    "box",
    "do",
    "final",
    "macro",
    "override",
    "priv",
    "typeof",
    "unsized",
    "virtual",
    "yield",
    "try",
}

TYPE_ALIASES = {
    "GLenum": "u32",
    "GLboolean": "u8",
    "GLbitfield": "u32",
    "GLvoid": "c_void",
    "GLbyte": "i8",
    "GLshort": "i16",
    "GLint": "i32",
    "GLclampx": "i32",
    "GLubyte": "u8",
    "GLushort": "u16",
    "GLuint": "u32",
    "GLsizei": "i32",
    "GLfloat": "f32",
    "GLclampf": "f32",
    "GLdouble": "f64",
    "GLclampd": "f64",
    "GLchar": "c_char",
    "GLcharARB": "c_char",
    "GLfixed": "i32",
    "GLhalf": "u16",
    "GLhalfARB": "u16",
    "GLhalfNV": "u16",
    "GLintptr": "isize",
    "GLsizeiptr": "isize",
    "GLintptrARB": "isize",
    "GLsizeiptrARB": "isize",
    "GLint64": "i64",
    "GLint64EXT": "i64",
    "GLuint64": "u64",
    "GLuint64EXT": "u64",
    "GLsync": "*mut c_void",
    "GLeglClientBufferEXT": "*mut c_void",
    "GLeglImageOES": "*mut c_void",
    "GLvdpauSurfaceNV": "isize",
}

CALLBACK_ALIASES = {
    "GLDEBUGPROC": "Option<unsafe extern \"C\" fn(GLenum, GLenum, GLuint, GLenum, GLsizei, *const GLchar, *const c_void)>",
    "GLDEBUGPROCARB": "Option<unsafe extern \"C\" fn(GLenum, GLenum, GLuint, GLenum, GLsizei, *const GLchar, *const c_void)>",
    "GLDEBUGPROCKHR": "Option<unsafe extern \"C\" fn(GLenum, GLenum, GLuint, GLenum, GLsizei, *const GLchar, *const c_void)>",
    "GLDEBUGPROCAMD": "Option<unsafe extern \"C\" fn(GLuint, GLenum, GLenum, GLsizei, *const GLchar, *mut c_void)>",
    "GLVULKANPROCNV": "Option<unsafe extern \"C\" fn()>",
}


def rust_ident(value: str) -> str:
    value = re.sub(r"[^0-9A-Za-z_]", "_", value)
    if value in keyword.kwlist or value in RUST_KEYWORDS:
        value += "_"
    if value and value[0].isdigit():
        value = "_" + value
    return value


def snake_gl_name(name: str) -> str:
    if name.startswith("gl"):
        name = name[2:]
    out: list[str] = []
    for index, char in enumerate(name):
        prev = name[index - 1] if index else ""
        next_ = name[index + 1] if index + 1 < len(name) else ""
        if (
            index
            and char.isupper()
            and (prev.islower() or prev.isdigit() or (prev.isupper() and next_.islower()))
        ):
            out.append("_")
        out.append(char.lower())
    return rust_ident("".join(out))


def fn_type_name(command_name: str) -> str:
    stem = command_name[2:] if command_name.startswith("gl") else command_name
    return rust_ident(f"Gl{stem}Fn")


def declaration_before_name(element: ET.Element) -> str:
    chunks: list[str] = []
    if element.text:
        chunks.append(element.text)
    for child in element:
        if child.tag == "name":
            break
        if child.text:
            chunks.append(child.text)
        if child.tail:
            chunks.append(child.tail)
    return normalize_decl("".join(chunks))


def normalize_decl(value: str) -> str:
    value = value.replace("\n", " ")
    value = re.sub(r"\s+", " ", value)
    value = value.replace(" *", "*").replace("* ", "*")
    value = value.replace("const*", "const *")
    return value.strip()


def base_type_to_rust(base: str) -> str:
    if base == "void":
        return "c_void"
    if base == "struct _cl_context":
        return "_cl_context"
    if base == "struct _cl_event":
        return "_cl_event"
    if base == "GLhandleARB":
        return "GLhandleARB"
    if base in TYPE_ALIASES:
        return base
    if base in CALLBACK_ALIASES:
        return base
    raise ValueError(f"unmapped GL type: {base!r}")


def decl_to_rust(decl: str, *, is_return: bool = False) -> str:
    decl = normalize_decl(decl)
    if is_return and decl in ("void", "void*"):
        return "()" if decl == "void" else "*mut c_void"
    if decl == "const void*":
        return "*const c_void"
    if decl == "void*":
        return "*mut c_void"
    if decl == "void**":
        return "*mut *mut c_void"
    if decl == "const void**":
        return "*const *const c_void"
    if decl == "const void*const*":
        return "*const *const c_void"

    pointer_count = decl.count("*")
    is_const = bool(re.search(r"\bconst\b", decl))
    base = decl.replace("*", "").replace("const", "").strip()

    rust_base = base_type_to_rust(base)
    if pointer_count == 0:
        return rust_base

    pointer = "*const" if is_const else "*mut"
    rust_type = rust_base
    for _ in range(pointer_count):
        rust_type = f"{pointer} {rust_type}"
    return rust_type


def command_signature(command: ET.Element) -> tuple[str, str, list[str]]:
    proto = command.find("proto")
    if proto is None:
        raise ValueError("command missing proto")
    name = proto.findtext("name")
    if not name:
        raise ValueError("command missing name")
    return_type = decl_to_rust(declaration_before_name(proto), is_return=True)
    params = [
        decl_to_rust(declaration_before_name(param))
        for param in command.findall("param")
    ]
    return name, return_type, params


def enum_value(value: str) -> str:
    return value.rstrip("uUlL")


def enum_rust_type(value: str, explicit_type: str | None) -> str:
    clean = enum_value(value)
    if explicit_type == "ull":
        return "GLuint64"
    if explicit_type == "u":
        return "GLuint"
    if clean.startswith("-"):
        return "GLint"
    if int(clean, 0) > 0xFFFF_FFFF:
        return "GLuint64"
    return "GLenum"


def registry_type_names(root: ET.Element) -> set[str]:
    names: set[str] = set()
    for type_entry in root.findall("./types/type"):
        name = type_entry.get("name") or type_entry.findtext("name")
        if name and name != "khrplatform":
            names.add(name)
    return names


def collect_scoped_names(root: ET.Element, tag: str) -> list[str]:
    names: list[str] = []
    included: set[str] = set()
    for feature in root.findall("./feature"):
        if feature.get("api") != SCOPE_API:
            continue
        number = feature.get("number")
        if number is None or float(number) > SCOPE_MAX_VERSION:
            continue
        for require in feature.findall("require"):
            for entry in require.findall(tag):
                name = entry.get("name")
                if name and name not in included:
                    included.add(name)
                    names.append(name)
        for remove in feature.findall("remove"):
            for entry in remove.findall(tag):
                name = entry.get("name")
                if name in included:
                    included.remove(name)
                    names = [existing for existing in names if existing != name]
    return names


def collect_used_type_names(root: ET.Element, commands: list[ET.Element]) -> list[str]:
    registry_names = registry_type_names(root)
    names: set[str] = set()
    for command in commands:
        elements = [command.find("proto"), *command.findall("param")]
        for element in elements:
            if element is None:
                continue
            declaration = declaration_before_name(element)
            base = declaration.replace("*", "").replace("const", "").strip()
            if base in registry_names:
                names.add(base)
    return sorted(names)


def collect_enums(root: ET.Element, scoped_names: set[str]) -> list[tuple[str, str, str]]:
    enums: list[tuple[str, str, str]] = []
    seen: set[str] = set()
    for enum in root.findall(".//enums/enum"):
        name = enum.get("name")
        value = enum.get("value")
        if not name or value is None or name in seen or name not in scoped_names:
            continue
        seen.add(name)
        enums.append((name, enum_rust_type(value, enum.get("type")), enum_value(value)))
    return enums


def write_generated(root: ET.Element) -> None:
    scoped_command_names = set(collect_scoped_names(root, "command"))
    scoped_enum_names = set(collect_scoped_names(root, "enum"))
    scoped_commands = [
        command
        for command in root.findall("./commands/command")
        if command.findtext("proto/name") in scoped_command_names
    ]
    command_specs = [command_signature(command) for command in scoped_commands]
    field_names = [snake_gl_name(name) for name, _, _ in command_specs]
    duplicates = [name for name, count in Counter(field_names).items() if count > 1]
    if duplicates:
        raise ValueError(f"duplicate generated field names: {duplicates[:10]}")

    type_names = collect_used_type_names(root, scoped_commands)
    enums = collect_enums(root, scoped_enum_names)

    lines: list[str] = [
        "// @generated by scripts/generate_glsym_raw.py from opengl-registry/gl.xml.",
        "// Do not edit by hand.",
        "//! Low-level OpenGL ES 2.0/3.0 registry bindings generated from Khronos `gl.xml`.",
        "//!",
        "//! This module exists so the crate can audit and load the GLES core",
        "//! command surface up to `GL_ES_VERSION_3_0` without hand-maintaining ABI",
        "//! declarations. It is internal registry data; normal core-author workflows",
        "//! should prefer the safe, typed `glsym`, `CompatGl`, and `CompatTextureGl`",
        "//! APIs in the crate root.",
        "",
        "#![allow(dead_code)]",
        "#![allow(non_camel_case_types)]",
        "#![allow(non_snake_case)]",
        "#![allow(non_upper_case_globals)]",
        "",
        "use std::ffi::{c_char, c_void};",
        "use std::mem;",
        "",
        f"pub(crate) const GL_XML_COMMAND_COUNT: usize = {len(command_specs)};",
        f"pub(crate) const GL_XML_TYPE_COUNT: usize = {len(type_names)};",
        f"pub(crate) const GL_XML_ENUM_COUNT: usize = {len(enums)};",
        "",
    ]

    for name in sorted(TYPE_ALIASES):
        if name in type_names:
            lines.append(f"pub(crate) type {name} = {TYPE_ALIASES[name]};")
    if "GLhandleARB" in type_names:
        lines.extend(
            [
                "#[cfg(target_os = \"macos\")]",
                "pub(crate) type GLhandleARB = *mut c_void;",
                "#[cfg(not(target_os = \"macos\"))]",
                "pub(crate) type GLhandleARB = u32;",
            ]
        )
    if "_cl_context" in type_names:
        lines.extend(
            [
                "#[repr(C)]",
                "pub(crate) struct _cl_context {",
                "    _private: [u8; 0],",
                "}",
            ]
        )
    if "_cl_event" in type_names:
        lines.extend(
            [
                "#[repr(C)]",
                "pub(crate) struct _cl_event {",
                "    _private: [u8; 0],",
                "}",
            ]
        )
    for name in sorted(CALLBACK_ALIASES):
        if name in type_names:
            lines.append(f"pub(crate) type {name} = {CALLBACK_ALIASES[name]};")
    lines.append("")

    for name, rust_type, value in enums:
        lines.append(f"pub(crate) const {name}: {rust_type} = {value};")
    lines.append("")

    lines.append("pub(crate) const GL_XML_COMMAND_NAMES: &[&str] = &[")
    for name, _, _ in command_specs:
        lines.append(f"    \"{name}\",")
    lines.append("];")
    lines.append("")
    lines.append("pub(crate) const GL_XML_TYPE_NAMES: &[&str] = &[")
    for name in type_names:
        lines.append(f"    \"{name}\",")
    lines.append("];")
    lines.append("")

    for name, return_type, params in command_specs:
        fn_name = fn_type_name(name)
        params_text = ", ".join(params)
        if return_type == "()":
            lines.append(f"pub(crate) type {fn_name} = unsafe extern \"C\" fn({params_text});")
        else:
            lines.append(
                f"pub(crate) type {fn_name} = unsafe extern \"C\" fn({params_text}) -> {return_type};"
            )
    lines.append("")

    lines.extend(
        [
            "/// Optional raw OpenGL ES 2.0/3.0 symbol table generated from Khronos `gl.xml`.",
            "///",
            "/// Every GLES core command up to 3.0 has a matching field. Fields are `None`",
            "/// when the active OpenGL context or frontend proc loader does not expose",
            "/// that command.",
            "///",
            "/// This type deliberately preserves the upstream C ABI. Calling loaded",
            "/// function pointers is unsafe and should stay behind small wrappers;",
            "/// regular core rendering code should use the crate's typed safe helpers.",
            "#[derive(Clone, Default)]",
            "pub(crate) struct GlRawSymbols {",
        ]
    )
    for field, (name, _, _) in zip(field_names, command_specs):
        lines.append(f"    pub(crate) {field}: Option<{fn_type_name(name)}>,")
    lines.append("}")
    lines.append("")

    lines.extend(
        [
            "impl GlRawSymbols {",
            "    pub(crate) fn load_with<F>(mut get_proc_address: F) -> Self",
            "    where",
            "        F: FnMut(&str) -> Option<*const c_void>,",
            "    {",
            "        Self {",
        ]
    )
    for field, (name, _, _) in zip(field_names, command_specs):
        lines.append(
            f"            {field}: load_symbol(&mut get_proc_address, \"{name}\"),"
        )
    lines.extend(
        [
            "        }",
            "    }",
            "",
            "    pub(crate) fn try_load_with<F, E>(mut get_proc_address: F) -> Result<Self, E>",
            "    where",
            "        F: FnMut(&str) -> Result<Option<*const c_void>, E>,",
            "    {",
            "        Ok(Self {",
        ]
    )
    for field, (name, _, _) in zip(field_names, command_specs):
        lines.append(
            f"            {field}: try_load_symbol(&mut get_proc_address, \"{name}\")?,"
        )
    lines.extend(
        [
            "        })",
            "    }",
            "",
            "    pub(crate) fn available_count(&self) -> usize {",
            "        let mut count = 0;",
        ]
    )
    for field in field_names:
        lines.append(f"        if self.{field}.is_some() {{ count += 1; }}")
    lines.extend(
        [
            "        count",
            "    }",
            "",
            "    pub(crate) fn is_available(&self, command_name: &str) -> bool {",
            "        match command_name {",
        ]
    )
    for field, (name, _, _) in zip(field_names, command_specs):
        lines.append(f"            \"{name}\" => self.{field}.is_some(),")
    lines.extend(
        [
            "            _ => false,",
            "        }",
            "    }",
            "}",
            "",
            "fn load_symbol<T, F>(get_proc_address: &mut F, name: &str) -> Option<T>",
            "where",
            "    T: Copy,",
            "    F: FnMut(&str) -> Option<*const c_void>,",
            "{",
            "    let raw = get_proc_address(name)?;",
            "    if raw.is_null() {",
            "        None",
            "    } else {",
            "        Some(unsafe { mem::transmute_copy(&raw) })",
            "    }",
            "}",
            "",
            "fn try_load_symbol<T, F, E>(get_proc_address: &mut F, name: &str) -> Result<Option<T>, E>",
            "where",
            "    T: Copy,",
            "    F: FnMut(&str) -> Result<Option<*const c_void>, E>,",
            "{",
            "    Ok(get_proc_address(name)?.and_then(|raw| {",
            "        if raw.is_null() {",
            "            None",
            "        } else {",
            "            Some(unsafe { mem::transmute_copy(&raw) })",
            "        }",
            "    }))",
            "}",
            "",
        ]
    )

    OUT_RS.write_text("\n".join(lines), encoding="utf-8")

    manifest = [
        "# glsym raw coverage manifest",
        "",
        f"source={GL_XML.relative_to(ROOT)}",
        f"generated={OUT_RS.relative_to(ROOT)}",
        f"scope={SCOPE_LABEL}",
        f"api={SCOPE_API}",
        f"max_version={SCOPE_MAX_VERSION:.1f}",
        f"command_count={len(command_specs)}",
        f"type_count={len(type_names)}",
        f"enum_count={len(enums)}",
        "",
        "[commands]",
        *[name for name, _, _ in command_specs],
        "",
        "[types]",
        *type_names,
    ]
    MANIFEST.write_text("\n".join(manifest) + "\n", encoding="utf-8")


def main() -> int:
    root = ET.parse(GL_XML).getroot()
    write_generated(root)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
