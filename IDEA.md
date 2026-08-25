# Vox — Local, Instant Voice-to-Text

> **Vox** is a free, open-source (MIT), fully local
> alternative to Wispr Flow. Press a hotkey, speak, press it again — your words are
> formatted, punctuated, and pasted at your cursor before you can blink.

---

## The Idea

Typing is the bottleneck. Especially now that people talk to AI agents all day, the
fastest input device you own is your mouth (~150 WPM spoken vs ~40 WPM typed).

Vox is a menubar-resident macOS app:

1. You press a global hotkey **anywhere** — in Cursor, Slack, a browser, a terminal.
2. A tiny frosted-glass **pill** appears with a live waveform. You speak.
3. You press the hotkey again. The pill flashes "transcribing" for a fraction of a
   second, and the finished, punctuated, correctly-cased text is **pasted directly
   at your cursor**. No app switching. No clicking.

Everything runs on-device. No account, no subscription, no usage limits, no audio or
text ever leaves the machine. The only network calls the app ever makes are (a) a
one-time model download on first launch and (b) checking GitHub for app updates.

## Why it wins

| | Wispr Flow | Apple Dictation | **Vox** |
|---|---|---|---|
| Price | $15/mo | Free | **Free forever, MIT** |
| Runs locally | No (cloud) | Partially | **100%** |
| Usage limits | Yes | No | **None** |
| Multilingual | Yes | Yes | **Yes (99 languages)** |
| Punctuation/grammar | Yes | Weak | **Yes** |
| History + stats | Yes | No | **Yes, stored locally** |
| Open source | No | No | **Yes** |

## Core UX flows

### Dictation (the whole product in 5 seconds)
- **Hotkey ↓** → mic on instantly (<100ms), pill appears with waveform.
- Speak naturally, any supported language.
- **Hotkey ↓ again** → mic off, pill shows a brief "processing" shimmer,
  text is pasted at the cursor and saved to history. Pill fades out.
- If auto-paste isn't possible (no accessibility permission, secure input field),
  the text is on the clipboard and a subtle pill state says "Copied — ⌘V to paste."

### Escape / cancel (exactly as specced)
- While recording, press **Esc** → pill switches to a **cancel countdown** (3s ring
  animation): "Cancelling… press Esc to keep."
- Countdown **expires** → recording is discarded, nothing saved, pill fades.
- Press **Esc again** before it expires → cancel is aborted, recording **resumes
  seamlessly** (audio was never actually stopped, so zero words are lost).
- Pressing the main hotkey during the countdown → overrides the cancel and
  finishes/transcribes normally.

### History as a safety net
Every completed transcription is stored locally (SQLite). If a paste ever fails —
wrong window focused, app crashed, clipboard clobbered — the text is one click away
in the dashboard: search it, copy it, delete it. Nothing is ever silently lost.

## The two UI surfaces

Both are heavy frosted-glass (real macOS vibrancy/blur, not fake CSS), rounded,
minimal — it should feel like Apple shipped it.

1. **The Pill** — a small, floating, always-on-top capsule near the bottom of the
   screen. Appears only while dictating. Shows: live waveform → processing shimmer →
   success tick / cancel countdown. Never steals focus from the app you're typing
   into (this is critical and is engineered for, not hoped for).
2. **The Dashboard** — a compact window (not a big desktop app; more like a widget)
   opened from the menubar icon or a hotkey. Three sections:
   - **Home / Stats** — total transcriptions, total words, estimated **time saved**
     vs typing, your effective speaking **WPM**, weekly activity chart, streak.
   - **History** — searchable list of past transcriptions, copy / delete.
   - **Settings** — change hotkey, language (auto-detect or pinned), model/engine
     picker, auto-paste toggle, launch-at-login, appearance.

Plus a **menubar (tray) icon**: start dictation, open dashboard, quit.

## Feature list

### Must-have (MVP — what gets built now)
- [x] Global hotkey toggle dictation (configurable in Settings)
- [x] Instant mic capture with live waveform pill
- [x] Fully local transcription, Metal (Apple GPU) accelerated
- [x] Multilingual: English, Hindi, Spanish, Italian, French, Arabic, German,
      Portuguese, Japanese, Chinese… (99 languages via Whisper), auto-detect or pinned
- [x] Proper punctuation, casing, and formatting (model-native + deterministic
      post-processing pipeline)
- [x] Auto-paste at cursor (with clipboard preserve/restore); clipboard-only fallback
- [x] Esc cancel-with-countdown / double-Esc resume (state machine, exactly as above)
- [x] Local history (SQLite) with search, copy, delete
- [x] Stats dashboard: transcription count, words, time saved, speaking WPM, activity
- [x] Settings: hotkey recorder, language, model manager, auto-paste, login item
- [x] Menubar app, no Dock icon, runs quietly in background
- [x] First-run onboarding: mic + accessibility permissions, model download with progress
- [x] Auto-updates from GitHub Releases (free infra, cryptographically signed updates)
- [x] Engine adapter layer — STT engines are plugins behind one interface (see below)

### Good-to-have (fast follows, architecture already accommodates them)
- Streaming/incremental transcription while you speak → perceived latency ~0
- Bare-modifier hotkeys (tap Fn or Right-⌥ alone, like Wispr) via event tap
- Custom vocabulary / proper-noun dictionary (names, jargon, brand words)
- Spoken commands: "new line", "comma", "scratch that"
- Optional local-LLM polish pass (rewrite filler words, tone) — off by default
- Per-app language/profile rules ("in Slack, casual; in Mail, formal")
- Push-to-talk mode (hold to record) in addition to toggle
- Export history (Markdown/JSON), auto-prune retention policy
- Windows/Linux builds (Tauri makes this a port, not a rewrite)

### Explicitly out of scope
- Accounts, sign-ups, servers, telemetry, analytics — none, ever
- Cloud transcription of any kind in the core product (the adapter layer means a
  user *could* opt into a cloud engine someday; it will never be a default)

## The engine adapter layer (future-proofing)

Speech-to-text engines are hidden behind a single `SttEngine` interface. Shipping
with:

- **Whisper (whisper.cpp)** — default. Multilingual (incl. Hindi/Arabic), MIT,
  Metal-accelerated, `large-v3-turbo` quantized ≈ 600MB, near-instant on M-series.
- **NVIDIA Parakeet (ONNX)** — optional toggle for English-heavy users; extremely
  fast on CPU. *Note: Parakeet v2 is English-only, v3 is 25 European languages —
  no Hindi/Arabic, which is why it cannot be the default.*

Future engines drop in behind the same interface without touching the rest of the
app: Apple SpeechAnalyzer, faster-whisper, any future NVIDIA multilingual model, or
(user-opt-in) cloud APIs. Same pattern (`TextRefiner`) for grammar/LLM polish.

## Success criteria

- Hotkey → recording pill visible: **< 100ms**
- Stop → pasted text (10s utterance, M-series): **< 1s**, typically ~0.5s
- Zero data leaves the machine during dictation. Verifiable — it's open source.
- One `.dmg`, drag to Applications, works. Updates arrive automatically.
- The codebase stays small enough that one person can hold it in their head.
