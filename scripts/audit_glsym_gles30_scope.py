#!/usr/bin/env python3
"""Audit hand-written glsym command names against the GLES <= 3.0 scope."""

from __future__ import annotations

import argparse
from collections import defaultdict
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
GL_XML = ROOT / "crates/libretro-core/opengl-registry/gl.xml"
GLSYM_RS = ROOT / "crates/libretro-core/src/glsym.rs"
SCOPE_API = "gles2"
SCOPE_MAX_VERSION = 3.0


def collect_scoped_commands(root: ET.Element) -> set[str]:
    commands: set[str] = set()
    for feature in root.findall("./feature"):
        if feature.get("api") != SCOPE_API:
            continue
        number = feature.get("number")
        if number is None or float(number) > SCOPE_MAX_VERSION:
            continue
        for require in feature.findall("require"):
            commands.update(
                command.get("name")
                for command in require.findall("command")
                if command.get("name")
            )
        for remove in feature.findall("remove"):
            commands.difference_update(
                command.get("name")
                for command in remove.findall("command")
                if command.get("name")
            )
    return commands


def find_line_number(lines: list[str], needle: str) -> int:
    for line_number, line in enumerate(lines, start=1):
        if needle in line:
            return line_number
    raise ValueError(f"could not find marker in glsym.rs: {needle}")


def section_for_line(line_number: int, *, fake_start: int, tests_start: int) -> str:
    if line_number >= tests_start:
        return "tests"
    if line_number >= fake_start:
        return "fake_gl"
    return "production"


def command_literal_occurrences(path: Path) -> dict[str, set[str]]:
    occurrences: dict[str, set[str]] = defaultdict(set)
    lines = path.read_text(encoding="utf-8").splitlines()
    fake_start = find_line_number(lines, "fn fake_indexed_extensions")
    tests_start = find_line_number(lines, "mod tests")
    for line_number, line in enumerate(lines, start=1):
        for command in re.findall(r'"(gl[A-Za-z0-9_]+)"', line):
            occurrences[command].add(
                section_for_line(
                    line_number,
                    fake_start=fake_start,
                    tests_start=tests_start,
                )
            )
    return occurrences


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--deny-out-of-scope",
        action="store_true",
        help="exit non-zero when glsym.rs references commands outside GLES <= 3.0",
    )
    args = parser.parse_args()

    root = ET.parse(GL_XML).getroot()
    scoped = collect_scoped_commands(root)
    occurrences = command_literal_occurrences(GLSYM_RS)
    mentioned = set(occurrences)
    in_scope = sorted(mentioned & scoped)
    out_of_scope = sorted(mentioned - scoped)
    section_counts = {
        section: sum(section in occurrences[command] for command in out_of_scope)
        for section in ("production", "fake_gl", "tests")
    }

    print("# glsym GLES <= 3.0 scope audit")
    print()
    print(f"source={GL_XML.relative_to(ROOT)}")
    print(f"audited={GLSYM_RS.relative_to(ROOT)}")
    print(f"api={SCOPE_API}")
    print(f"max_version={SCOPE_MAX_VERSION:.1f}")
    print(f"scoped_command_count={len(scoped)}")
    print(f"mentioned_command_literals={len(mentioned)}")
    print(f"in_scope_mentions={len(in_scope)}")
    print(f"out_of_scope_mentions={len(out_of_scope)}")
    for section, count in section_counts.items():
        print(f"out_of_scope_{section}_mentions={count}")
    print()
    print("[out_of_scope_command_literals]")
    for command in out_of_scope:
        sections = ",".join(sorted(occurrences[command]))
        print(f"{command}\t{sections}")

    if args.deny_out_of_scope and out_of_scope:
        print(
            f"found {len(out_of_scope)} out-of-scope glsym command literals",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
