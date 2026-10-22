# FilmCraft Android: Offline AI Script Bridge (prototype)

This development branch is NOT the released FilmCraft app.
It adds an `AI` button to the Android app. A user can copy a JSON script created in ChatGPT, paste it into the app, tap **Review**, and then **Apply** to execute allowlisted editing actions. Android's text-sharing menu is also supported.

This is **not** an MCP server, an OpenAI API integration, autonomous ChatGPT control, or a source of new paid API tokens.

## Supported commands

- `project.create_blank`: requires `{"name":"..."}`; makes a new vertical 1080x1920 project
- `editor.add_blank`: adds FilmCraft's default blank clip
- `editor.add_text`: requires `{"text":"..."}`; adds text with FilmCraft defaults

Maximum 10 commands per script and 10,000 input characters. There is no arbitrary file access or command execution. Actions run only after an on-device review and explicit confirmation. Prior steps may remain saved if a later step fails.

Example: see `examples/ai_bridge_students_haven.json`.

## Build (separate test installation)

GitHub Actions workflow: `.github/workflows/build-ai-bridge.yml`. It runs for pushes to branch `ai-bridge-v1` or via manual dispatch where available and builds an **arm64 debug APK** using Rust + Gradle. Go to GitHub → Actions → `Build FilmCraft AI Bridge APK` → newest run → `filmcraft-ai-debug-apk` artifact. If the run fails, do NOT assume an APK exists. The debug applicationId suffix `.debug` allows the modified app to install alongside original FilmCraft.

## Caveats

- Kotlin/Rust compilation and installation on a physical phone remain unverified until the GitHub build succeeds and the resulting APK is tested.
- The first version only handles three basic edits; no layout positioning, transitions, speech, image creation, export, or duration control.
- Back up original projects. Test with dummy media before trusting this software.
- You must not expose an unauthenticated command endpoint to the internet. This build has no network listener.
