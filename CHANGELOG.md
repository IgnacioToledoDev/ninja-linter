# Changelog

All notable changes to ninja-linter are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/). Versions follow [Semantic Versioning](https://semver.org/).

---

## [0.7.3] - 2026-06-27

### Fixed

- Auto-close TUI dashboard when all parallel workers finish. Previously the dashboard stayed open after workers completed, requiring manual exit (`q`/`Esc`).

---

## [0.7.2] - 2026-04-20

### Fixed

- Silent mode output suppressed correctly during parallel process execution.

---

## [0.7.1] - 2026-04-16

### Fixed

- Silent mode flag not respected when running in parallel mode.

---

## [0.7.0] - 2026-04-16

### Added

- Retrieve committed (staged) files in addition to unstaged modified files.

---

## [0.6.2] - 2026-03-17

### Fixed

- Corrected path resolution for `.ninja-linter.json` config file.

---

## [0.6.1] - 2026-03-17

### Fixed

- Config file path lookup on initial setup.

---

[0.7.3]: https://github.com/IgnacioToledoDev/ninja-linter/compare/v0.7.2...v0.7.3
[0.7.2]: https://github.com/IgnacioToledoDev/ninja-linter/compare/v0.7.1...v0.7.2
[0.7.1]: https://github.com/IgnacioToledoDev/ninja-linter/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/IgnacioToledoDev/ninja-linter/compare/v0.6.2...v0.7.0
[0.6.2]: https://github.com/IgnacioToledoDev/ninja-linter/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/IgnacioToledoDev/ninja-linter/releases/tag/v0.6.1
