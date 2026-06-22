---
name: bump-version
description: Bump the version of every gribberish crate and package (the three Rust crates, the Python extension, and the npm package) in lockstep to a requested value, and regenerate Cargo.lock. Use when asked to bump, set, or change the version, cut a release, or prepare a new version of gribberish.
---

# bump-version

gribberish is a Cargo workspace whose five crates plus its npm package all share
**one** version, kept in lockstep. A version bump is not a single edit: each
manifest carries a `[package] version`, internal path-dependencies pin that same
version, and `Cargo.lock` records it for every crate. Miss one and `cargo` /
`napi` / `maturin` disagree at publish time.

The driver `bump_version.py` does all of it. **Always use it** — don't hand-edit
manifests.

All paths below are relative to the repo root (the directory with the workspace
`Cargo.toml`). Run from there.

## What carries the version

- `types/Cargo.toml`, `macros/Cargo.toml`, `gribberish/Cargo.toml`,
  `python/Cargo.toml`, `js/Cargo.toml` — each `[package] version`, plus every
  internal `gribberish*` path-dependency's `version = "..."`.
- `js/package.json` — top-level `"version"`.
- `Cargo.lock` — the five workspace crate entries (regenerated, not edited).
- `python/pyproject.toml` does **not** carry a version — it's `dynamic` and
  maturin reads it from `python/Cargo.toml`. Leave it alone.

## Run (agent path)

```bash
# What's the current version?
python3 .claude/skills/bump-version/bump_version.py --current

# Dry run — show which files/lines would change, write nothing.
python3 .claude/skills/bump-version/bump_version.py 1.2.0 --check

# Apply the bump (edits manifests + regenerates Cargo.lock).
python3 .claude/skills/bump-version/bump_version.py 1.2.0

# Review, then commit.
git diff
```

A clean bump prints the per-file line counts (e.g. `gribberish/Cargo.toml: 3
line(s)`) and ends with `Cargo.lock updated.` The expected footprint is **7
files, 17 insertions / 17 deletions** — if `git diff --stat` shows more or
fewer files, something is off.

The version argument must be valid semver (`1.2.0`, `2.0.0-rc.1`). The script
refuses anything else, refuses to run outside the workspace root, and is
idempotent (re-running at the current version is a no-op).

## Gotchas

- **External deps are never touched.** The script only rewrites internal
  `version` fields that sit behind a `path = "..."` (the `gribberish*` crates).
  `napi = "3.0.0"`, `pyo3 = "0.27.0"`, `numpy`, etc. have no `path`, so they're
  safe — that's why a bare find-and-replace of the version string is *not* what
  this does.
- **Consistency is enforced up front.** If the manifests don't already agree on
  one current version, the script aborts and lists the disagreement instead of
  bumping. Resolve the drift first.
- **`Cargo.lock` is regenerated via `cargo update --workspace`**, not text-edited.
  It tries `--offline` first and falls back to online if the lock needs a
  registry fetch. If you have no network and the lock is stale, that fallback
  can fail — run `cargo update --workspace` once with network, then re-run.
- **This bumps versions only.** It does not tag, publish, or push. Publishing
  the crates / npm package / wheels is a separate step.

## Troubleshooting

- `ERROR: run this from the repo root` — you're in a subdirectory. `cd` to the
  workspace root (where the `[workspace]` `Cargo.toml` lives).
- `ERROR: manifests disagree on current version` — a previous partial edit left
  things inconsistent. The error lists each file's version; reconcile them
  (usually `git checkout` the stray file) and re-run.
- `ERROR: cargo update failed` — `cargo` couldn't refresh the lock (often
  offline + stale lock). Run `cargo update --workspace` manually to see the real
  error.
