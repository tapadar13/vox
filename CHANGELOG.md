# Changelog

All notable changes to Vox are documented here. The project follows Semantic
Versioning once the first stable release is published.

## Unreleased

### Added

- Local Whisper transcription accelerated with Metal on Apple Silicon
- Incremental overlapping transcription with stable and provisional live text
- Final-tail processing and Unicode-aware overlap deduplication
- Continuous CPAL capture and streaming resampling to 16 kHz mono
- Global configurable dictation shortcut and reversible Esc cancellation
- Non-activating floating pill with waveform and state feedback
- Deterministic multilingual formatting with optional filler-word trimming
- Clipboard-preserving automatic paste and manual-paste fallback
- SQLite transcript history, search, deletion, and usage statistics
- Resumable checksum-verified model management for Turbo, Small, and Base
- Optional Parakeet transducer adapter behind a Cargo feature
- First-run microphone and Accessibility onboarding
- Login-item, tray, single-instance, structured log, and updater integration
- Signed GitHub Release workflow for universal macOS artifacts
- Frontend and Rust CI, unit tests, and dependency automation

### Privacy

- Audio remains in memory and is never persisted
- Transcription remains entirely local
- Network use is limited to explicit model downloads and application updates

## 0.1.0

Initial release is in preparation.
