# Release Guide

## Overview

Releases are tag-driven. Pushing a `v*` tag to GitHub triggers the full release pipeline: lint, tests, version bump, cross-platform builds, and a GitHub Release with downloadable binaries.

## Branching Strategy

```
feature/* ──┐
bugfix/*  ──┤──► develop ──► main ──► (tag triggers release)
ci/*      ──┘
```

- `develop` — integration branch; all PRs target here first
- `main` — always reflects the latest release; only updated via PR from `develop` or direct CI commits
- Feature/bugfix/ci branches are merged to `develop` via PR

## Pre-Release Checklist

Before tagging:

- [ ] All target features/fixes are merged to `develop`
- [ ] `develop` branch CI is green (PR workflow: clippy + tests)
- [ ] `CHANGELOG.md` updated with the new version and release notes
- [ ] Open a PR from `develop` → `main` and get it merged

## Tagging a Release

After `develop` is merged into `main`:

```bash
# Fetch latest main
git fetch origin
git checkout main
git pull origin main

# Create an annotated tag (replace X.Y.Z with the new version)
git tag -a vX.Y.Z -m "Release vX.Y.Z"

# Push the tag — this triggers the release pipeline
git push origin vX.Y.Z
```

> **Version format:** Semantic Versioning — `vMAJOR.MINOR.PATCH`
>
> - PATCH — bug fixes, no new features
> - MINOR — new backwards-compatible features
> - MAJOR — breaking changes

## What the Release Pipeline Does

Triggered by any tag matching `v*` (`.github/workflows/release.yml`):

| Job | Description |
|-----|-------------|
| `check` | Runs `cargo clippy` and `cargo test` |
| `bump-version` | Updates `version` in `Cargo.toml` + `Cargo.lock`, commits to `main` |
| `build` | Cross-compiles for 4 targets (see below) |
| `release` | Creates a GitHub Release and uploads all binaries |

### Build Targets

| Target | OS | Output |
|--------|----|--------|
| `x86_64-unknown-linux-gnu` | Ubuntu | `.tar.gz` |
| `x86_64-apple-darwin` | macOS (Intel) | `.tar.gz` |
| `aarch64-apple-darwin` | macOS (Apple Silicon) | `.tar.gz` |
| `x86_64-pc-windows-msvc` | Windows | `.zip` |

## Updating the Changelog

Edit `CHANGELOG.md` before merging to `main`. Follow the existing format:

```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added
- New feature description

### Fixed
- Bug fix description

### Changed
- Behavioral change description
```

## Hotfix Flow

For urgent fixes that can't wait for the next planned release:

```bash
# Branch from main (not develop)
git checkout main
git pull origin main
git checkout -b bugfix/short-description

# ... make the fix, commit ...

# PR into main directly, then tag
```

After the hotfix PR merges to `main`, also cherry-pick or merge `main` back into `develop` to keep branches in sync:

```bash
git checkout develop
git merge main
git push origin develop
```

## Verifying a Release

After the pipeline completes (~5–10 min):

1. Check the [GitHub Releases page](https://github.com/IgnacioToledoDev/ninja-linter/releases) — all 4 binary artifacts should be attached
2. Confirm `Cargo.toml` on `main` shows the new version
3. Smoke-test by downloading one binary and running `ninja-linter --version`
