# Vox Privacy

Vox is designed so dictation remains on your Mac.

## Audio and transcription

Microphone audio is held in memory only while a dictation is active or recoverable
after an inference error. Vox does not write audio recordings to disk. Whisper or
the optional Parakeet engine transcribes the audio locally; Vox does not transmit
audio or transcript text to an inference service.

## Data stored on the Mac

Vox stores the following in its macOS application-support directory:

- `settings.json`, containing your app preferences
- `vox.db`, containing transcript history and data used to calculate statistics
- `models/`, containing models you explicitly download
- rotating diagnostic logs, containing operational errors and lifecycle messages

Logs are local and do not intentionally include microphone audio or full transcript
content. Transcript history can be deleted per item or cleared from the app.
Removing the Vox application-support directory removes all locally stored Vox data.

## Network requests

Vox makes network requests only for:

- model downloads that you initiate, hosted by Hugging Face; and
- update checks and update downloads, hosted by GitHub Releases.

These services may receive normal request metadata such as your IP address and user
agent under their respective policies. Dictation and formatting continue to work
offline after a model is installed.

## Clipboard and Accessibility

When automatic paste is enabled, Vox temporarily replaces the clipboard, emits a
Command-V keystroke, and restores the previous clipboard text when possible. macOS
Accessibility permission is used only for this synthesized paste action. Without
that permission, Vox leaves the transcript on the clipboard for manual paste.

## Telemetry

Vox contains no analytics, advertising SDK, crash-reporting service, account
system, or cloud synchronization.

## Scope

This document describes the open-source Vox application in this repository. A
redistributor who adds network services or telemetry is responsible for disclosing
those changes.
