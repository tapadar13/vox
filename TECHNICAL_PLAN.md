# Vox — Technical Plan of Action

Production-grade plan. Every choice below optimizes for: **speed felt by the user**,
**architectural soundness**, **zero running cost**, and **a codebase that stays small**.

---

## 1. Tech stack (all free, all open source)

| Layer | Choice | Why |
|---|---|---|
| App shell | **Tauri 2.x** | ~10MB binaries, native webview, Rust backend, first-class tray/global-shortcut/updater plugins, cross-platform later |
| Backend | **Rust** (stable) | Real-time audio without GC pauses, fearless concurrency for the pipeline, links whisper.cpp natively |
| STT engine | **whisper.cpp** via `whisper-rs` | Metal GPU acceleration on Apple Silicon, 99 languages incl. Hindi/Arabic, MIT license, model `large-v3-turbo` quantized (~600MB) |
| Alt engine | **Parakeet TDT via `sherpa-onnx`** (optional adapter) | Fastest English ASR available locally; proves the adapter layer is real |
| Audio in | `cpal` + `rubato` | Cross-platform capture; high-quality resample of device rate (48k) → 16k mono f32 that Whisper needs |
| Paste | `arboard` (clipboard) + CGEvent keystroke (⌘V) | The Wispr trick: preserve clipboard → set text → synthesize ⌘V → restore clipboard |
| Hotkeys | `tauri-plugin-global-shortcut` | Configurable combos; Esc registered *only while recording* so we never swallow Esc system-wide |
| Storage | **SQLite** via `rusqlite` (bundled) | Single-file local DB in app data dir; history + stats with plain SQL |
| Frontend | **React 19 + TypeScript + Vite + Tailwind** | Small surface (2 windows), fast iteration, easy glassmorphism |
| Glass | `window-vibrancy` crate (real NSVisualEffectView blur) + `backdrop-filter` CSS | Genuine macOS frosted glass, not fake |
| Pill window | `tauri-nspanel` (non-activating NSPanel) | **Critical**: the pill floats above everything but never steals focus from the app receiving the paste |
| Updates | `tauri-plugin-updater` + **GitHub Releases** | Free hosting; updates signed with our minisign key; static `latest.json` manifest |
| CI | GitHub Actions (free tier) | Tag → build DMG + updater artifacts → draft release |
| License | MIT | As requested |

Also: `tauri-plugin-autostart` (login item), `tauri-plugin-single-instance`,
`tokio` (async runtime), `thiserror`/`anyhow` (errors), `serde` (config), `tracing`
(structured logs to a local rotating file — the "no silent failures" backbone).

**Why not Swift**: agreed, excluded. Everything native we need (vibrancy, NSPanel,
CGEvent, permissions) is reachable from Rust via existing maintained crates.

**Model licensing**: Whisper weights MIT, whisper.cpp MIT, Parakeet CC-BY-4.0,
sherpa-onnx Apache-2.0 — all compatible with an MIT app.

---

## 2. Architecture — hexagonal core, thin adapters

One domain core that owns all logic; everything with a platform or vendor smell
lives behind a **port** (Rust trait). The frontend is a dumb view.

```
┌────────────────────────────  UI (webviews)  ───────────────────────────┐
│   Pill window (NSPanel)          Dashboard window        Tray menu     │
│   waveform / countdown           stats · history · settings            │
└───────────────▲────────────────────────▲───────────────────────────────┘
                │  Tauri events (state pushed down)   Tauri commands (user intent up)
┌───────────────┴────────────────────────┴───────────────────────────────┐
│                        DOMAIN CORE (pure Rust, no I/O)                 │
│                                                                        │
│   DictationController ── the state machine (§3)                        │
│   FormatterPipeline   ── deterministic text cleanup (§5)               │
│   StatsService        ── time-saved / WPM math                         │
│                                                                        │
│   Ports (traits): AudioInput · SttEngine · TextRefiner ·               │
│                   TranscriptStore · TextDelivery · SettingsStore       │
└───────────────▲────────────────────────────────────────────────────────┘
                │ implemented by
┌───────────────┴────────────────────────────────────────────────────────┐
│                              ADAPTERS                                  │
│  CpalAudioInput      (cpal + rubato ring buffer)                       │
│  WhisperEngine       (whisper-rs, Metal)      ┐ selected via           │
│  ParakeetEngine      (sherpa-onnx, optional)  ┘ EngineRegistry         │
│  RuleTextRefiner     (regex/casing pipeline; future: LocalLlmRefiner)  │
│  SqliteStore         (rusqlite)                                        │
│  MacTextDelivery     (arboard + CGEvent ⌘V, clipboard restore)         │
│  JsonSettings        (serde → settings.json in app dir)                │
└────────────────────────────────────────────────────────────────────────┘
```

Rules that keep this from rotting:
- The core **never** imports cpal/whisper/tauri types. It sees traits and plain data.
- Adapters contain zero business logic. If an `if` is about *what should happen*,
  it belongs in the core; if it's about *how a vendor API works*, the adapter.
- UI state is a single serializable `AppState` broadcast over one Tauri event
  channel. The frontend renders it; it never derives its own truth.
- One crate, ~20 source files, no premature workspace splitting.

### The key port

```rust
#[async_trait]
pub trait SttEngine: Send + Sync {
    fn id(&self) -> EngineId;                     // "whisper-turbo", "parakeet-v2"
    fn capabilities(&self) -> EngineCaps;         // languages, streaming?, size
    async fn load(&self) -> Result<(), SttError>; // called at app start (pre-warm)
    async fn transcribe(&self, audio: AudioClip, lang: LangHint)
        -> Result<Transcript, SttError>;
}
```

`EngineRegistry` owns the instances; Settings picks the active one. Adding Apple
SpeechAnalyzer or a cloud engine later = one new file + one registry entry. A
`StreamingSttEngine` extension trait is reserved for the incremental-transcription
fast-follow — the architecture supports it, the MVP doesn't need it.

---

## 3. The dictation state machine (the heart of correctness)

Explicit enum, exhaustive transitions, unit-tested in isolation. No booleans-soup.

```
IDLE
 └─ hotkey ─────────────▶ RECORDING        (mic stream on, Esc shortcut registered)
RECORDING
 ├─ hotkey ─────────────▶ TRANSCRIBING     (stop capture → engine.transcribe)
 └─ Esc ────────────────▶ CANCEL_PENDING   (capture CONTINUES; 3s countdown)
CANCEL_PENDING
 ├─ Esc ────────────────▶ RECORDING        (countdown aborted, nothing lost)
 ├─ hotkey ─────────────▶ TRANSCRIBING     (cancel overridden, finish normally)
 └─ timer expires ──────▶ IDLE             (audio discarded, nothing saved)
TRANSCRIBING
 ├─ ok ─────────────────▶ DELIVERING       (format → paste → save to history)
 └─ error ──────────────▶ ERROR            (pill shows message; audio clip kept in
                                            memory + text of last good partial, if
                                            any, saved — user can retry from pill)
DELIVERING
 └─ done ───────────────▶ IDLE             (pill: ✓ then fade)
```

Details that make it feel engineered:
- Audio keeps flowing during `CANCEL_PENDING`, so double-Esc resume loses **zero** words.
- Esc is registered as a global shortcut only on entering `RECORDING` and
  unregistered on leaving the recording states — the rest of the OS keeps its Esc.
- Every transition emits one `AppState` event; the pill is a pure function of it.
- A hard cap (default 5 min) auto-finishes a forgotten recording instead of
  growing memory forever.

---

## 4. Latency engineering ("unbelievably fast")

Budget for the perceived-speed path, M-series Mac, `large-v3-turbo` q5:

| Step | Budget | How |
|---|---|---|
| Hotkey → pill visible + mic live | < 100ms | Model **pre-loaded at app start** (stays warm in RAM ~1.5GB); pill window pre-created and hidden, just shown; cpal stream opened on demand (~20ms) |
| Speaking | realtime | Lock-free ring buffer; rubato resamples on the audio thread's consumer side; waveform peaks streamed to pill at 30fps |
| Stop → text ready (10s utterance) | 300–900ms | whisper.cpp + Metal runs turbo at >10× realtime; audio is already 16k mono in memory — zero file I/O |
| Text → pasted | < 50ms | clipboard set + synthesized ⌘V |
| **Total stop→pasted** | **≲ 1s** | typically ~0.5s for conversational utterances |

Fast-follow (architecture ready): incremental transcription — transcribe rolling
windows *while speaking*, so stop only processes the final 2–3s → perceived
~200ms regardless of utterance length.

Smaller `small`/`base` quantized models offered in the model manager for older or
RAM-tight machines (speed/accuracy slider, honest labels).

---

## 5. Text quality pipeline

1. **Model-native**: Whisper already emits punctuation, casing, and native-script
   output (Devanagari for Hindi, Arabic script, accents for es/fr/it).
2. **FormatterPipeline** (deterministic, ordered, individually unit-tested):
   whitespace normalization → sentence-capitalization → spacing around punctuation
   → strip leading/trailing artifacts → (config) filler-word trim off by default.
3. **`TextRefiner` port**: MVP ships `RuleTextRefiner` (the pipeline above).
   Future `LocalLlmRefiner` (llama.cpp + a small instruct model) slots in behind
   the same trait as an opt-in "polish" toggle — never default, never cloud.

Language: `auto` (Whisper detects per-utterance) or pinned in Settings (pinning
skips detection → slightly faster and more accurate for monolingual users).

---

## 6. Data & storage

App data dir: `~/Library/Application Support/com.vox.app/`

```
settings.json          # serde-validated, versioned schema, atomic writes
vox.db                 # SQLite
models/                # downloaded GGUF/ONNX models + sha256 manifest
logs/vox.log           # tracing rolling log (local only, for debugging)
```

```sql
CREATE TABLE transcriptions (
  id           INTEGER PRIMARY KEY,
  created_at   TEXT NOT NULL,            -- ISO-8601 UTC
  text         TEXT NOT NULL,            -- final delivered text
  raw_text     TEXT NOT NULL,            -- engine output pre-formatting
  language     TEXT NOT NULL,            -- detected or pinned (BCP-47)
  duration_ms  INTEGER NOT NULL,         -- speaking time
  latency_ms   INTEGER NOT NULL,         -- stop→delivered (we show off with this)
  word_count   INTEGER NOT NULL,
  engine_id    TEXT NOT NULL,
  delivered    INTEGER NOT NULL          -- 1 pasted, 0 clipboard-only/failed
);
CREATE INDEX idx_t_created ON transcriptions(created_at);
-- schema_version table + tiny forward-only migration runner from day one
```

Stats are SQL over this table (no second bookkeeping source of truth):
- **Time saved** = Σ `word_count`/40wpm − Σ speaking time (40 WPM average typing;
  constant adjustable in Settings)
- **Speaking WPM** = Σ words / Σ duration
- Activity chart = words per day, last 12 weeks; streak = consecutive active days.

Audio is **never persisted** — it lives in RAM for the life of one dictation. Text
history is the safety net (per spec: recovery when paste fails), searchable
(SQLite `LIKE`; FTS5 if it ever feels slow), copy/delete per row, clear-all.

---

## 7. Windows, focus, and permissions (macOS craft)

- **Pill**: `tauri-nspanel` non-activating panel — floats above full-screen apps,
  never takes key focus, so the target app keeps its cursor. Transparent window +
  vibrancy blur; positioned bottom-center; click-through except its buttons.
- **Dashboard**: standard window, hidden-title-bar, `NSVisualEffectMaterial`
  vibrancy, compact (~720×480), lives in menubar (`ActivationPolicy::Accessory`,
  no Dock icon).
- **Permissions onboarding** (first run, honest and skippable):
  1. Microphone — standard prompt on first capture (Info.plist usage string).
  2. Accessibility — required only for auto-⌘V; polite explainer with a
     deep-link to System Settings; until granted, app works clipboard-only.
- **Delivery fallback chain**: paste succeeds → done; accessibility missing or
  secure-input active → clipboard + pill hint "Copied — press ⌘V"; even that
  failing → text is still in history. Three layers, nothing lost, ever.

---

## 8. Model management (first-run, zero cost to us)

The app binary stays ~15MB. On first run (and from Settings → Models):
- Curated registry (name, size, languages, speed rating, sha256, Hugging Face URL).
- Default: `whisper-large-v3-turbo` q5 (~600MB). Download with progress bar,
  resume support, checksum verify, atomic move into `models/`.
- This is the **only** non-update network access in the app, it's user-visible,
  and it's one-time. All inference is offline thereafter.

---

## 9. Updates & distribution (free infra)

- **GitHub Releases** hosts DMGs + updater bundles. `tauri-plugin-updater` checks a
  static `latest.json` on the release; updates are verified against our
  **minisign public key** baked into the app (key generated once, private half
  kept locally/in repo secrets).
- **CI**: GitHub Actions — on tag push: build universal DMG, sign updater
  artifacts, draft the release with generated notes. `cargo test` + `cargo clippy
  -D warnings` + `tsc --noEmit` gate every push.
- **Honest caveat (the only $ tradeoff in this plan)**: without a $99/yr Apple
  Developer ID, macOS Gatekeeper flags downloaded apps — users right-click → Open
  (or `xattr -cr`) once on first install. In-app auto-updates work fine after
  that regardless. If distribution ever gets serious, notarization is a drop-in
  later; nothing else changes.

---

## 10. Codebase layout (small on purpose)

```
vox/
├── IDEA.md · TECHNICAL_PLAN.md · README.md · LICENSE
├── src/                          # frontend (React + TS + Tailwind)
│   ├── windows/pill/             # Pill.tsx + waveform canvas + countdown ring
│   ├── windows/dashboard/        # Stats.tsx · History.tsx · Settings.tsx
│   ├── lib/                      # tauri bridge (events/commands), types (mirrors Rust)
│   └── ui/                       # Glass primitives: Panel, Stat, Button…
└── src-tauri/
    ├── tauri.conf.json · Cargo.toml · capabilities/
    └── src/
        ├── main.rs · app.rs      # bootstrap, DI wiring, tray, windows
        ├── domain/               # controller.rs (state machine) · state.rs · events.rs
        ├── ports.rs              # ALL trait definitions in one readable file
        ├── audio/                # capture.rs · ring.rs · resample.rs
        ├── stt/                  # registry.rs · whisper.rs · parakeet.rs
        ├── text/                 # formatter.rs (+ tests)
        ├── store/                # db.rs · migrations.rs · stats.rs
        ├── delivery.rs           # clipboard + ⌘V synthesis + restore
        ├── hotkeys.rs            # register/unregister lifecycle
        ├── models_mgr.rs         # registry, downloader, checksums
        └── settings.rs · error.rs
```

Testing philosophy: the state machine, formatter, and stats math are pure and get
dense unit tests (that's where correctness bugs would hide). Adapters get thin
smoke tests. No mocking theater around vendor APIs.

---

## 11. Build order (each phase ends runnable)

| Phase | Deliverable |
|---|---|
| **P0** | Tauri scaffold: tray app, hidden dashboard, empty pill panel, CI skeleton, settings load/save |
| **P1** | Happy path e2e: hotkey → record → whisper transcribe → format → paste. Ugly but *fast*. Latency logged from day one |
| **P2** | Full state machine + pill UI: waveform, shimmer, Esc countdown/resume, error states |
| **P3** | SQLite history + stats service + Dashboard (Stats, History) in glass design |
| **P4** | Settings UI: hotkey recorder, language, model manager (download/switch), engine picker, autostart; onboarding flow |
| **P5** | Parakeet adapter (proves the port), updater wiring, DMG packaging, README, tag v0.1.0 |

Risks & mitigations: whisper-rs Metal build quirks (pin known-good version, feature-flagged) ·
NSPanel focus edge cases in full-screen apps (tested in P2, fallback to plain
always-on-top window) · secure-input fields block synthetic ⌘V (detected, clipboard
fallback path) · large-model RAM on 8GB Macs (model manager defaults honest,
`small` offered).
