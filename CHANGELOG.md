# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-11-22

### Added

- `CHANGELOG.md` following Keep a Changelog.
- Warning in the README against deleting models through LM Studio's UI, with the reason and recovery steps.

### Fixed

- Apply `--hf` and `--lms` overrides when a config file already exists; previously the flags were only honoured on the first run.

## [0.1.0] - 2026-11-22

### Added

- Interactive TUI for browsing and multi-selecting GGUF models from a Hugging Face cache.
- Scan `<hf_cache>/models--<org>--<model>/snapshots/*`, resolve symlinks to the real blobs, pick the revision from `refs/main`, and skip empty files.
- Hard-link selected models into `<lms_dir>/<org>/<model>/<filename>.gguf` without copying weights.
- Detect already-linked files and report conflicting targets, with `--force` to overwrite them.
- CLI options `-h/--help`, `-f/--force`, `--hf <PATH>` and `--lms <PATH>`.
- First-run configuration prompts with the config stored in the user config directory; `HF2LMS_CONFIG` selects an explicit path.
- README with usage, controls and current limitations.

[unreleased]: https://github.com/valsaven/hf2lms/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/valsaven/hf2lms/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/valsaven/hf2lms/releases/tag/v0.1.0
