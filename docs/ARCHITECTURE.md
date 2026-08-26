# Architecture

Vox is a Tauri application with a React view layer and a Rust runtime. The Rust
domain owns state transitions; webviews send intent through commands and render the
single `AppState` emitted by the backend.

## Runtime boundaries

```text
Dashboard / pill / tray
        ↕ commands and vox://state events
VoxRuntime + DictationController
        ↕ plain Rust ports
audio · STT · formatter · SQLite · delivery · settings
```

`DictationController` is a pure state machine. It maps events to a new state and a
list of effects without importing Tauri, CPAL, Whisper, or SQLite types. `VoxRuntime`
executes those effects and pushes the resulting state to both webviews.

External behavior sits behind traits in `src-tauri/src/ports.rs`:

- `AudioInput` captures and snapshots 16 kHz mono audio.
- `SttEngine` loads a local model and returns timestamped segments.
- `TextRefiner` formats raw engine output.
- `TranscriptStore` and `StatsStore` persist/query local data.
- `TextDelivery` pastes or falls back to the clipboard.
- `SettingsStore` persists validated settings atomically.

## Dictation lifecycle

1. The global shortcut dispatches `ToggleRequested`.
2. The controller enters recording and requests capture, pill display, and temporary
   Esc registration.
3. CPAL writes short raw samples into a bounded ring. A named worker continuously
   resamples them and appends them to a bounded live buffer.
4. The incremental task snapshots that buffer and transcribes overlapping chunks.
5. A second shortcut stops capture. Vox waits for the active inference, processes
   only the final uncommitted tail, and merges all retained segments.
6. The rule formatter normalizes the transcript. Delivery preserves the clipboard,
   attempts Command-V, and reports whether it pasted or copied.
7. SQLite records the outcome and the state returns to idle after pill feedback.

Esc enters a grace period rather than immediately discarding audio. A second Esc
resumes recording with no gap; expiration discards the in-memory clip.

## Concurrency invariants

- The CPAL callback performs only bounded ring-buffer writes.
- Resampling and inference never run on the callback thread.
- A Whisper inference mutex serializes background and final-tail calls because one
  loaded context is shared.
- Recording and cancellation epochs make delayed async work harmless after a new
  lifecycle begins.
- Only one incremental session/task is owned by the runtime at a time.
- Audio buffers have fixed duration caps derived from the recording limit.

## Storage

SQLite migrations are forward-only and transactional. Statistics are computed from
transcription rows rather than duplicated counters. Settings use a temporary file
and atomic rename. Model downloads use resumable `.part` files, checksum validation,
and an atomic final rename.

## Feature flags

The default feature is `whisper-metal`. `whisper` provides Whisper without forcing
Metal, and `parakeet` compiles the optional sherpa-rs transducer adapter. The engine
registry rejects duplicate IDs and presents one uniform interface to the runtime.
