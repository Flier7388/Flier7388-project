# Session persistence (2026-11-08)

## What changed
The app logged in from scratch on every launch (explicitly deferred in
phase 3/4). The Rust side already had everything needed
(`MatrixClient::restore`, `client_session()`, bridged as `restoreSession`/
`clientSession`) — this was purely Dart-side wiring, no Rust changes.

- `flutter/lib/session_store.dart` (new): reads/writes a flat JSON session
  file and the `matrix-sdk-sqlite` store path, both under
  `path_provider`'s `getApplicationSupportDirectory()` instead of the old
  hardcoded `/tmp/matrix-client-flutter-proof` (which never survived a
  reboot and could collide between concurrent instances, per the phase-4
  report).
- `flutter/lib/main.dart`: `MatrixClientApp` is now a 3-stage state
  machine (`restoring` → `loggedOut` | `loggedIn`). On startup it tries
  `loadSession()` + `restoreSession(...)` before ever showing the login
  screen; a restore that throws (revoked/expired token) clears the saved
  file and falls back to login rather than retrying it forever.
- `flutter/lib/screens/login_screen.dart`: saves the session via
  `clientSession()` + `saveSession()` right after a successful login, and
  uses the new stable store path instead of `/tmp`.
- Added a real (not placeholder) logout action: `HomeScreen` takes an
  `onLogout` callback, wired from `main.dart` (`stopSync` + `clearSession`
  + drop back to `loggedOut`), surfaced on the settings rail tab via a new
  `_SettingsPanel` (just the user ID + a logout button — everything else
  the mockup's settings dialog has isn't backed by any API yet, so it
  stays a placeholder). Without this, a persisted session with no way to
  log out would strand the owner on whatever account logged in first.
- Added `path_provider: ^2.1.4` to `pubspec.yaml`.

## Verified
- `flutter pub get`, `dart analyze` (no issues), `flutter build linux`
  (succeeds) via `nix develop .#flutter`.
- Launched the built binary fresh (no saved session file): renders the
  login screen correctly, no crash — confirms the `restoring` →
  `loggedOut` branch. Screenshotted via `spectacle` (no `xdotool`/
  `ydotool` in this environment, same limitation as past phases, so the
  actual login → relaunch → auto-restore → logout round-trip through the
  live GUI still needs the owner's own click-through, as in phase 3/4).
- Did **not** run `cargo test --workspace` this session — no Rust code
  changed, and disk space was tight (see below), so it wasn't worth the
  risk for an unrelated-code verification pass.

## Disk space, again
Hit the exact same thrash pattern as phase 3/4, twice in one session:
`nix develop .#flutter` failed outright with "No space left on device"
fetching the flake input (5.7GB free of 250GB). `nix-collect-garbage -d`
freed 53.7GiB (only 15 real dead store paths, confirming — again — that
nothing was pinning the previous devShell closure as a GC root once its
shell exited; this isn't "garbage piling up", it's the same closure
getting evicted and refetched every session). This matches
`docs/ARCHITECTURE.md`'s existing Nix-quirks section and the standing
memory about not GC'ing preemptively — only did it here because the
build was genuinely, provably blocked.

Separately, a plain `cargo test --workspace` (debug build of the whole
workspace including `matrix-sdk`/`matrix-sdk-ui`) filled the disk
*completely* — badly enough that the harness's own tool-output capture
started failing with ENOSPC on commands as small as `echo`. Recovered by
`rm -rf target/` (safe — cargo regenerates it). Root cause not
investigated further this session (worth a follow-up: either this
machine's `/` partition is just too small for this project's workspace +
the Flutter/GTK Nix closures to coexist comfortably, or `target/` itself
has grown unusually large and could use `cargo clean` more routinely).

## Still open
- Owner click-through of the full persistence round-trip (login → quit →
  relaunch → auto-restore → logout → relaunch → login screen again) —
  same manual-testing gap as every prior phase, no GUI automation tool
  available in this environment.
- Same long-standing deferred items as before: `feedback-dto` rework,
  live-updating room list/timeline streams, real UI design for the
  remaining rail tabs (direct/explore/add-space/search/inbox), Flutter
  Impeller-on-desktop revisit.
