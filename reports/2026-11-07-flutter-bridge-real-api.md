# 2026-11-07 — real matrix-client API wired through flutter_rust_bridge

Phase 3 of `rustling-giggling-snowglobe`'s plan. `app_bridge` (`flutter/
rust`) now exposes `matrix-client`'s real session/sync/room/timeline/send
API to Dart, replacing the phase-1 `ping()` pipeline-proof marker.

## What was built
- `flutter/rust/src/api/matrix.rs` (new, replaces `simple.rs`): local
  `Session`/`RoomSummary`/`TimelineMessage` mirror structs (plain
  `String` fields, `From`/`TryFrom` conversions against `matrix-client`'s
  real types) plus free async functions — `login`, `restore_session`,
  `client_session`, `start_sync`, `stop_sync`, `rooms`, `timeline_for`,
  `timeline_messages`, `send_text` — each a thin wrapper converting
  `matrix_client::ClientError` to a plain `String`.
- `crates/matrix-client/src/lib.rs`: removed `health_check()` — the real
  API fully replaces it now that the bridge is wired up.
- `flutter/lib/main.dart`: replaced the `ping()` screen with a throwaway
  login-form → room-list → timeline+send screen (plain Material widgets,
  no design — real UI work stays gated on the owner's visual references).
- Updated `test/widget_test.dart` and `integration_test/simple_test.dart`
  to match (checked for the login form instead of the old `ping()` text).

## Two real bugs found and fixed
1. **`use matrix_client::{MatrixClient, RoomTimeline};` needed to be `pub
   use`.** A plain `use` compiles fine in `matrix.rs` itself, but is
   invisible to `frb_generated.rs`'s own `use crate::api::matrix::*;`
   glob import — codegen silently produced `cannot find type
   MatrixClient` errors only in the *generated* code, not in `matrix.rs`.
2. **Both `app_bridge` and (already, from phase 2) `matrix-client` need
   `#![recursion_limit = "256"]`** — `Timeline::send`'s future type nests
   deep enough to blow the default query-depth limit during layout
   computation in *any* crate that calls through it, not just the one
   that defines it.

Design decision made during planning, confirmed correct by the result:
exposing `matrix-client`'s plain data types (`RoomSummary`,
`TimelineMessage`, `Session`) directly across the FFI boundary was
avoided in favor of small local mirrors in `app_bridge` — partly because
`Session` carries real `ruma` ID types Dart has no use for, partly
because `flutter_rust_bridge`'s own docs site was inconsistently
fetchable while researching exactly how it introspects external-crate
plain structs (several 404s). `MatrixClient`/`RoomTimeline` themselves
*do* cross directly as opaque handles — confirmed working once the `pub
use` fix landed.

## Verified
- `cargo check` passes for both the root workspace and `app_bridge`
  standalone (lightweight Rust-only devShell, no GTK4 needed).
- `flutter_rust_bridge_codegen generate` runs clean; `dart analyze`
  reports "No issues found!" for both the generated bindings and the
  whole Flutter project.
- `flutter build linux` (via `devShells.flutter`) succeeds — took ~219s,
  worth knowing since a shorter timeout on the first attempt made it look
  like a build failure when it was actually just a tool-timeout cutoff.
- Ran the built binary live, screenshotted it: the real login screen
  renders (homeserver pre-filled with the owner's real homeserver,
  username/password fields, Log in button).
- **Manual click-through by the owner, since no `xdotool`/`ydotool` was
  available to drive the GUI myself**: found and fixed two real bugs this
  surfaced that pure backend testing hadn't:
  1. `_login()` called `rooms()` immediately after `startSync()` with no
     delay — the snapshot read raced the sync service's first fetch and
     came back empty every time (not a crash, just a permanently-empty
     room list). Fixed with a 5s delay (matching
     `sync_smoke_test.rs`'s own wait) plus a standalone "Refresh rooms"
     list entry so it's not purely timing-dependent.
  2. The hardcoded store path (`/tmp/matrix-client-flutter-proof`,
     reused across every run since there's no real persistence yet) hit
     a real `matrix-sdk` crypto-store identity conflict once — turned out
     to be from two app instances accidentally running at once against
     the same store path (an earlier `kill` had only killed a wrapper
     process, not the actual binary). Resolved by clearing the store
     directory and ensuring a single clean process; not a bug in
     `matrix-client` itself, but worth remembering that this throwaway
     screen's fixed store path doesn't tolerate concurrent instances.
  Full flow now confirmed live by the owner, start to finish: login →
  sync → real room list → opened a room → sent a real message to the
  owner's real homeserver from the actual running GUI, via `send_text()`
  through the bridge. Every piece of this phase's API surface has now been
  exercised through the real UI, not just the Rust-side smoke test.

## The disk-space story this session, finally diagnosed properly
Hit the disk-full wall five more times across this phase (once badly
enough to break the harness's own temp filesystem, needing the owner to
run `nix-collect-garbage` directly since the tool-call pipe itself
couldn't execute). Each `nix-collect-garbage -d` run freed a suspiciously
consistent ~50-57GiB, which was the actual clue: Nix never deletes
anything on its own, so that number being nearly identical every time
means it wasn't "new garbage piling up from legitimate builds" — it was
the *same* closure being deleted and immediately needing to be re-fetched
by the next `nix develop` command, because nothing pins a plain `nix
develop` shell's closure as a GC root once the shell process exits.
Stopped GC'ing preemptively partway through this phase; the remaining
work proceeded without further incidents. Documented in
`docs/ARCHITECTURE.md`'s Nix quirks section and as a standalone memory
([[feedback-dont-preempt-nix-gc]]) so this isn't rediscovered the hard
way again. Separately: one of the disk spikes (four ~12GiB `-source`
store paths, ~49GiB total) turned out to be completely unrelated to this
project — leftover from other activity on this machine (it's the owner's
main daily-driver box, not a dedicated sandbox; `ghidra`/
`chromium-unwrapped`/`wine`/`libreoffice` also live in the same
`/nix/store`).

## Still open
- Manual click-through of the login/rooms/timeline/send flow in the
  actual running GUI (owner's call whether this is worth doing — backend
  logic already proven).
- Session persistence to disk from the Dart side (currently logs in
  fresh every launch) — explicitly deferred, not this phase's scope.
- Real UI design — gated on the owner's visual references, per standing
  agreement.
- Live-updating room list / timeline streams (reactive, not snapshot) —
  deferred to whenever that becomes the actual blocker.
- Same long-standing deferred items: the owner-facing score/notes-via-
  Matrix-message report feature (not yet scoped).
