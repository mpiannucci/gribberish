#!/usr/bin/env python3
"""Bump the version of every gribberish crate / package in lockstep.

All five crates (gribberish-types, gribberish-macros, gribberish, gribberishpy,
gribberish_js) and the npm package (@mattnucc/gribberish) share ONE version.
Internal path-dependencies pin that same version, so a bump touches both the
`[package] version` line and every internal `{ path = ..., version = ... }` ref.

Usage:
    python3 bump_version.py <new-version>          # apply the bump
    python3 bump_version.py <new-version> --check   # dry run, show planned edits
    python3 bump_version.py --current               # print current version and exit

Run from the repo root (the directory containing the workspace Cargo.toml).
After editing manifests this regenerates Cargo.lock via `cargo update`.
"""
import re
import subprocess
import sys
from pathlib import Path

# Manifests carrying a version. Paths are relative to the repo root.
CARGO_TOMLS = [
    "types/Cargo.toml",
    "macros/Cargo.toml",
    "gribberish/Cargo.toml",
    "python/Cargo.toml",
    "js/Cargo.toml",
]
PACKAGE_JSONS = ["js/package.json"]

# `[package] version = "X"` — line-anchored, so it never matches the
# version field inside an inline `{ ... }` dependency table.
PKG_VERSION_RE = re.compile(r'(?m)^version = "[^"]+"')
# Internal path-dependency version: `... path = "..", version = "X" ..`.
# Keyed on a preceding `path = "..."`, so external deps (napi, pyo3, numpy,
# which have no `path`) are left untouched.
PATH_DEP_RE = re.compile(r'(path\s*=\s*"[^"]*",\s*version\s*=\s*")[^"]+(")')
# Top-level npm `"version": "X"`.
JSON_VERSION_RE = re.compile(r'("version":\s*")[^"]+(")')

SEMVER_RE = re.compile(r"^\d+\.\d+\.\d+([-+].+)?$")


def read_current(root: Path) -> str:
    """Return the version every manifest agrees on, or die if they disagree."""
    found = {}
    for rel in CARGO_TOMLS:
        text = (root / rel).read_text()
        m = PKG_VERSION_RE.search(text)
        if not m:
            sys.exit(f"ERROR: no [package] version in {rel}")
        found[rel] = m.group(0).split('"')[1]
    for rel in PACKAGE_JSONS:
        text = (root / rel).read_text()
        m = JSON_VERSION_RE.search(text)
        if not m:
            sys.exit(f"ERROR: no version field in {rel}")
        found[rel] = m.group(0).split('"')[3]
    versions = set(found.values())
    if len(versions) != 1:
        lines = "\n".join(f"  {k}: {v}" for k, v in found.items())
        sys.exit(f"ERROR: manifests disagree on current version:\n{lines}")
    return versions.pop()


def plan_edits(root: Path, new: str) -> dict:
    """Compute new file contents. Returns {relpath: (old_text, new_text)}."""
    edits = {}
    for rel in CARGO_TOMLS:
        text = (root / rel).read_text()
        out = PKG_VERSION_RE.sub(f'version = "{new}"', text, count=1)
        out = PATH_DEP_RE.sub(rf'\g<1>{new}\g<2>', out)
        if out != text:
            edits[rel] = (text, out)
    for rel in PACKAGE_JSONS:
        text = (root / rel).read_text()
        out = JSON_VERSION_RE.sub(rf'\g<1>{new}\g<2>', text, count=1)
        if out != text:
            edits[rel] = (text, out)
    return edits


def main() -> None:
    args = sys.argv[1:]
    root = Path.cwd()
    if not (root / "Cargo.toml").exists() or "[workspace]" not in (root / "Cargo.toml").read_text():
        sys.exit("ERROR: run this from the repo root (the workspace Cargo.toml dir).")

    if args == ["--current"]:
        print(read_current(root))
        return

    check = "--check" in args
    positional = [a for a in args if not a.startswith("--")]
    if len(positional) != 1:
        sys.exit("Usage: bump_version.py <new-version> [--check] | --current")
    new = positional[0]
    if not SEMVER_RE.match(new):
        sys.exit(f"ERROR: '{new}' is not a valid semver (e.g. 1.2.0, 1.0.0-rc.1).")

    current = read_current(root)
    print(f"Current version: {current}")
    print(f"Target  version: {new}")
    if current == new:
        print("Already at target version; nothing to do.")
        return

    edits = plan_edits(root, new)
    for rel in sorted(edits):
        changed = sum(
            1 for a, b in zip(edits[rel][0].splitlines(), edits[rel][1].splitlines()) if a != b
        )
        print(f"  {rel}: {changed} line(s)")

    if check:
        print("\n--check: no files written.")
        return

    for rel, (_, out) in edits.items():
        (root / rel).write_text(out)
    print(f"\nWrote {len(edits)} manifest(s). Regenerating Cargo.lock...")

    # Refresh the workspace crate versions in Cargo.lock without touching deps.
    r = subprocess.run(
        ["cargo", "update", "--workspace", "--offline"],
        cwd=root, capture_output=True, text=True,
    )
    if r.returncode != 0:
        # --offline can fail if the lock needs a registry fetch; retry online.
        r = subprocess.run(
            ["cargo", "update", "--workspace"],
            cwd=root, capture_output=True, text=True,
        )
    if r.returncode != 0:
        print(r.stderr, file=sys.stderr)
        sys.exit("ERROR: `cargo update` failed; Cargo.lock not regenerated.")

    locked = read_current(root)  # re-validate manifests are consistent
    print(f"Done. All manifests now at {locked}. Cargo.lock updated.")
    print("Review with: git diff")


if __name__ == "__main__":
    main()
