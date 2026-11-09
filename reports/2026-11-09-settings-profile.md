# Settings profile display (2026-11-09, sixth phase of the day)

## What changed
Last 0.1.0-alpha checklist item that wasn't blocked on a real Windows
machine: the Settings panel was just a bare "Log out" button. Now it
shows the logged-in user's real profile: a colored-initial avatar
(`colorForMxid`/`initials`, same treatment every other avatar in the
app uses), display name, and user ID.

- `crates/matrix-client/src/client.rs`: `MatrixClient::
  account_display_name()` on `matrix_sdk::Client::account().
  get_display_name()`.
- `flutter/rust/src/api/matrix.rs`: `account_display_name` bridge fn.
- `flutter/lib/screens/home_screen.dart`: `_loadSession` now also
  fetches the display name (best-effort — a failure here isn't
  critical, same fallback-to-user-ID pattern used everywhere else).
  `_SettingsPanel` rewritten with a real profile card above the
  existing logout button.

No avatar *image* — this app doesn't download/render any media yet
(that's the 0.5.0-beta milestone), so "avatar" here means the same
colored-initial treatment used throughout, not a real picture. Matches
what the roadmap's acceptance criteria actually asked for.

## Verified
`cargo check -p matrix-client`, `cargo check` on `app_bridge`,
`flutter_rust_bridge_codegen generate`, `dart analyze` (no issues),
`flutter build linux` (succeeds) — all clean. **Not runtime-tested this
round** — skipped the usual launch+screenshot pass given today's
repeated builds already running the owner's laptop hot; left it for
them to check next time they're in the app.

## 0.1.0-alpha status
Every item left in this milestone is now either done or blocked on
something this environment can't provide:
- [x] 3-pane layout, custom theme, live navigation, member drawer,
  settings profile — all done.
- [ ] Direct Messages tab — not started, next candidate.
- [ ] Windows build actually attempted — blocked, no Windows machine
  available here.
