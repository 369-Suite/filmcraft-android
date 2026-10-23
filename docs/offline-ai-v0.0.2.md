# FilmCraft v0.0.2 — Offline AI implementation plan

All processing must remain on-device. No API keys, remote inference, or mandatory cloud services.

## 1. Cinematic colour grading
- Reuse the existing Rust filter/effects pipeline and Kotlin filter UI.
- Add deterministic cinematic presets: Monsoon Forest, Soft Film, Teal/Orange.
- Add intensity and preserve natural skin tones.
- Implement automatic exposure/white balance only after verifying frame sampling APIs.
- Test export parity with preview, HDR/SDR input, and performance.

## 2. Malayalam speech-to-subtitle
- Reuse existing on-device Whisper caption pipeline.
- Expose Malayalam (`ml`) explicitly and ensure multilingual (not English-only) model selection.
- Download models only on user request; run inference fully offline.
- Preserve Unicode Malayalam in editable captions, timeline and exported video.
- Test Malayalam speech, mixed Malayalam/English, timing, punctuation, and fonts.

## 3. AI-assisted editing
- First implement deterministic offline silence detection using decoded PCM and configurable thresholds.
- Add scene-change detection from sampled frames.
- Show non-destructive cut suggestions; require user confirmation before timeline edits.
- Keep undo/redo and original media intact.

## Acceptance criteria
- App runs with airplane mode after optional model download.
- Existing editing/export workflows remain functional.
- No network requests for inference.
- Each feature has tests, and a GitHub Actions APK build passes.
- Do not claim model accuracy without real-device validation.

## Status
Planning scaffold committed. Feature implementations and APK build remain pending.
