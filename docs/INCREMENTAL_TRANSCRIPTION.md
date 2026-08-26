# Incremental transcription

Incremental transcription is the default Vox recording path, not a post-MVP mode.
Its purpose is to move nearly all inference work into speaking time so the stop
action has only a short tail to finish.

## Window geometry

The default configuration uses:

| Parameter | Value |
|---|---:|
| Model sample rate | 16,000 Hz mono |
| Chunk duration | 4.5 seconds |
| Step duration | 3.0 seconds |
| Overlap | 1.5 seconds |
| Scheduler poll | 120 milliseconds |

The first chunk starts once 4.5 seconds of audio is available. Each following pass
advances by 3 seconds, retaining 1.5 seconds of shared context. Short recordings are
handled entirely by the final pass.

## Stable and provisional text

Whisper returns timestamped segments. After each background pass, segments that end
before the current stability boundary are committed. Newer words remain provisional
and may be replaced after Whisper sees more right-hand context. Both values live in
`AppState`; the pill renders committed words more strongly than provisional words.

The accumulator tracks committed and processed boundaries separately. That matters
when a segment crosses the stability boundary: the segment must remain eligible for
a later window even though earlier audio has already been processed.

## Stop path

When the user stops:

1. Vox records the stop timestamp immediately for honest latency measurement.
2. Capture is stopped and the incremental worker is joined.
3. The session calculates the uncommitted tail, including the necessary overlap.
4. Whisper processes that tail under the same inference lock used by background
   chunks.
5. Timestamped segments and text overlaps are merged and deduplicated.
6. Formatting and paste run once against the merged transcript.

If a background pass fails, the session retains the error and the stop path falls
back to one full-clip inference. This costs latency but preserves the dictation.

## Deduplication

Chunk merging combines timestamp ownership with Unicode-aware word overlap. It finds
the longest suffix of retained text that equals a prefix of incoming text after
case-folding and punctuation normalization, then appends only the novel words. This
works for non-ASCII scripts and avoids repeated phrases at window boundaries.

## Performance expectations

On a warm model and a suitable Apple Silicon Mac, only a few seconds remain at stop,
so stop-to-paste can approach 200–500 ms. This is a target range, not a guarantee.
Selected model, language detection, thermals, hardware, and phrase boundary quality
all affect inference time.

No chunk, partial transcript, or final transcript is sent over the network.

## Tests

The incremental module has unit coverage for configuration geometry, chunk planning,
stability boundaries, crossing segments, Unicode overlaps, language voting, and
final-tail behavior. Audio resampling and bounded live-buffer behavior are tested
without requiring a microphone device.
