# Vox

Vox is a private, local-first voice-to-text app for macOS. Hold a global shortcut,
speak naturally, and Vox formats the transcript before pasting it at the active
cursor. Audio and inference stay on the Mac.

## What works

- Continuous microphone capture and 16 kHz resampling with no audio files
- Incremental Whisper transcription over overlapping windows while you speak
- Stable/provisional live transcript in a non-activating floating pill
- Final-tail transcription, overlap deduplication, and deterministic formatting
- Configurable global shortcut with an Esc cancellation grace period
- Clipboard-preserving paste with a clipboard-only fallback
- Local SQLite history, activity, streaks, WPM, and time-saved statistics
- Resumable, SHA-256-verified model downloads
- Whisper with Metal acceleration and an optional Parakeet adapter
- Signed in-app updater artifacts published through GitHub Releases

## Requirements

- macOS 11 or newer
- Rust stable, Node.js 22+, and npm
- Xcode Command Line Tools
- A microphone permission grant
- Accessibility permission for automatic paste (clipboard-only mode works without it)

## Development

```bash
npm ci
npm run tauri dev
```

Vox downloads no model automatically. Open Settings → Local model and install one
of the curated Whisper models before dictating. The default Turbo model is about
574 MB; Small and Base are available for lower-memory Macs.

Useful checks:

```bash
npm test
npm run build
cd src-tauri
cargo fmt --check
cargo test --no-default-features
cargo clippy --no-default-features --all-targets -- -D warnings
cargo check
```

The normal build enables Whisper and Metal. The optional English Parakeet adapter
can be compiled independently:

```bash
cd src-tauri
cargo check --no-default-features --features parakeet
```

## How incremental transcription works

Vox continuously resamples captured audio and lets Whisper process 4.5-second
windows every 3 seconds. The 1.5-second overlap gives the model enough context at
chunk boundaries. Segments older than the stability boundary are retained in
memory; provisional text can still change. At stop, only the uncommitted tail is
processed, then every segment is merged and deduplicated before formatting.

This design targets roughly 200–500 ms from stop to paste in favorable conditions.
Actual latency depends on the Mac, model, language, and recording length.

## Local data and network access

Vox stores settings, downloaded models, transcript history, and rotating diagnostic
logs under the standard macOS application-support directory for
`com.tapadar.vox`. Audio exists only in RAM for the current dictation and is never
written to disk.

Network access is limited to user-initiated model downloads from Hugging Face and
update checks/downloads from GitHub Releases. Transcription never sends audio to a
server. See [PRIVACY.md](PRIVACY.md) for the complete policy.

## Distribution

Pushing a semantic-version tag such as `v0.1.0` runs the release workflow, builds a
universal macOS app and DMG, signs the updater bundle, and creates a draft GitHub
release. Apple notarization requires a paid Developer ID; without it, users may
need to right-click → Open on first launch.

See [docs/RELEASING.md](docs/RELEASING.md) for secret setup and the release
checklist.

## License

[MIT](LICENSE)
