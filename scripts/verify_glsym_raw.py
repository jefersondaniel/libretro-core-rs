#!/usr/bin/env python3
"""Verify generated raw glsym coverage against the GLES <= 3.0 gl.xml scope."""

from __future__ import annotations

import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
GL_XML = ROOT / "crates/libretro-core/opengl-registry/gl.xml"
GENERATED = ROOT / "crates/libretro-core/src/glsym_raw.rs"
MANIFEST = ROOT / "crates/libretro-core/opengl-registry/glsym_coverage_manifest.txt"
SCOPE_API = "gles2"
SCOPE_MAX_VERSION = 3.0


def normalize_decl(value: str) -> str:
    value = value.replace("\n", " ")
    value = re.sub(r"\s+", " ", value)
    value = value.replace(" *", "*").replace("* ", "*")
    value = value.replace("const*", "const *")
    return value.strip()


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


def main() -> int:
    root = ET.parse(GL_XML).getroot()
    scoped_command_names = set(collect_scoped_names(root, "command"))
    scoped_enum_names = set(collect_scoped_names(root, "enum"))
    scoped_commands = [
        command
        for command in root.findall("./commands/command")
        if command.findtext("proto/name") in scoped_command_names
    ]
    commands = [command.findtext("proto/name") for command in scoped_commands]
    commands = [command for command in commands if command]
    types = collect_used_type_names(root, scoped_commands)
    enum_names: list[str] = []
    seen_enums = set()
    for enum in root.findall(".//enums/enum"):
        name = enum.get("name")
        if (
            name
            and enum.get("value") is not None
            and name in scoped_enum_names
            and name not in seen_enums
        ):
            seen_enums.add(name)
            enum_names.append(name)

    generated = GENERATED.read_text(encoding="utf-8")
    manifest = MANIFEST.read_text(encoding="utf-8")

    generated_checks = [
        (
            rf"pub(?:\(crate\))?\s+const\s+GL_XML_COMMAND_COUNT:\s+usize\s+=\s+{len(commands)};",
            "command count",
        ),
        (
            rf"pub(?:\(crate\))?\s+const\s+GL_XML_TYPE_COUNT:\s+usize\s+=\s+{len(types)};",
            "type count",
        ),
        (
            rf"pub(?:\(crate\))?\s+const\s+GL_XML_ENUM_COUNT:\s+usize\s+=\s+{len(enum_names)};",
            "enum count",
        ),
    ]
    missing_generated = [
        label for pattern, label in generated_checks if not re.search(pattern, generated)
    ]
    manifest_checks = [
        (f"api={SCOPE_API}", "manifest API scope"),
        (f"max_version={SCOPE_MAX_VERSION:.1f}", "manifest max version"),
        (f"command_count={len(commands)}", "manifest command count"),
        (f"type_count={len(types)}", "manifest type count"),
        (f"enum_count={len(enum_names)}", "manifest enum count"),
    ]
    missing_manifest = [label for needle, label in manifest_checks if needle not in manifest]
    if missing_generated or missing_manifest:
        missing = missing_generated + missing_manifest
        print("generated metadata mismatch: " + ", ".join(missing), file=sys.stderr)
        return 1

    generated_command_names = re.findall(r'^\s+"(gl[A-Za-z0-9_]+)",$', generated, re.MULTILINE)
    command_set = set(commands)
    generated_command_set = set(generated_command_names)
    if command_set != generated_command_set:
        missing_commands = sorted(command_set - generated_command_set)
        extra_commands = sorted(generated_command_set - command_set)
        print(f"command coverage mismatch: missing={missing_commands[:10]} extra={extra_commands[:10]}", file=sys.stderr)
        return 1

    for type_name in types:
        if f'"{type_name}",' not in generated:
            print(f"missing generated GL type name: {type_name}", file=sys.stderr)
            return 1

    for command in commands:
        if not re.search(rf"^{re.escape(command)}$", manifest, re.MULTILINE):
            print(f"manifest missing command: {command}", file=sys.stderr)
            return 1

    print(
        f"verified GLES <= 3.0 glsym raw coverage: {len(commands)} commands, "
        f"{len(types)} types, {len(enum_names)} enums"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
