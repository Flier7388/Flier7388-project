# 2026-11-06 — real session/sync/room/timeline/send in `matrix-client`

Phase 2 of `rustling-giggling-snowglobe`'s plan. `matrix-client` grows from
a throwaway `health_check()` marker to a real feature surface, verified
end-to-end against the owner's own homeserver (domain intentionally not
published).

## What was built
- `crates/matrix-client/src/client.rs` — `MatrixClient`: `login()`,
  `restore()`, `session()`.
- `src/session.rs` — `Session`: a flat, serializable snapshot of
  `AuthSession::Matrix` (user ID, device ID, access/refresh token,
  homeserver URL). This crate only makes it serializable — *where* it's
  written to disk, and how it's protected (it's a bearer token), is
  explicitly left to the caller.
- `src/sync.rs` — `start_sync()`/`stop_sync()`, driving a
  `matrix-sdk-ui::sync_service::SyncService` built at login/restore time.
- `src/room.rs` — `rooms()`: a snapshot of joined rooms (room ID, display
  name). Deliberately reads `client.rooms()` directly rather than
  `RoomListService`'s dynamic-entries/diff-stream API — that machinery
  exists for live reactive UI updates, which this phase explicitly defers,
  so it would've added real complexity for no benefit right now.
- `src/timeline.rs` — `timeline_for()` → `RoomTimeline`: `messages()` (a
  snapshot of plain-text messages) and `send_text()`.
- `src/error.rs` — `ClientError`, unifying the ~5 distinct error types
  these APIs actually return.
- `crates/matrix-client/examples/sync_smoke_test.rs` replaces
  `login_smoke_test.rs` (deleted).

## Why `matrix-sdk-ui`
Same stack Element X runs on top of `matrix-sdk` — consistent with this
project's existing bar (prefer upstream-maintained, battle-tested paths
over hand-rolled protocol logic, especially where a security review will
focus).

## How this avoided repeating today's `docs.rs`-was-wrong mistake
`docs.rs`'s rendered text already proved subtly wrong once today
(`login_username`'s real signature). Rather than hand-write the rest of
this against docs.rs summaries and find out the hard way again, the owner
had cloned several reference Rust Matrix clients into `git/` — `robrix`
(pins the *exact same* `matrix-sdk-ui` 0.19.1 this project uses) and
`iamb` (plain `matrix-sdk`, no `matrix-sdk-ui`) turned out to be the
useful two. Two parallel research passes over their real source extracted
exact, compiler-verified method chains for building the client, logging
in, persisting/restoring a session, driving sync, reading the room list,
building a per-room timeline, and sending a message — then `cargo check`
caught the handful of remaining small misses (two missing `ClientError`
`From` impls, a missing `RoomExt` trait import for `timeline_builder()`,
one `#![recursion_limit]` bump `Timeline::send`'s future type needed).

## Verified
- `cargo check --workspace --all-targets`, `cargo test --workspace`,
  `cargo fmt --check`, `cargo clippy -D warnings`, `cargo deny check` all
  pass.
- `cargo run --example sync_smoke_test -p matrix-client` against the
  owner's real `test` account: login succeeds, `Session` round-trips
  through `serde_json`, sync starts, room list returns successfully
  (empty — the account isn't in any rooms yet).
- **Timeline read + `send_text()` fully verified** (owner invited the
  `test` account into two existing rooms, "Слуги Донбасса" and
  "Основной", then explicitly confirmed sending a real test message to
  the first one): `send_text()` sent
  `"matrix-client sync_smoke_test: send_text() + timeline read check"`,
  and `messages()` correctly round-tripped both the sender
  (`@test:<homeserver>`) and the exact body back out of the timeline. This
  closes out the phase's last unverified item.

## A second disk-space incident, and the actual fix (not just a workaround)
Hit the same full-disk failure twice more in this session (`write of 65536
bytes: No space left on device` from `nix develop`, then a `ConnectionReset`
mid-request that was arguably the first symptom of disk pressure rather
than a real network issue). Root cause, this time found and fixed
properly rather than just cleaned up: `sysprof`'s ~60GiB GTK4/libadwaita/
GStreamer closure (added last session to satisfy a Flutter-only
pkg-config requirement) lived in the *same* `devShells.default` used for
all plain Rust work. Routine `nix-collect-garbage -d` between sessions
deleted it along with everything else-and the next `nix develop` for even
a trivial `cargo run` re-pulled the whole ~60GiB chain before anything
else could happen, refilling the disk immediately. Split `flake.nix` into
`devShells.default` (Rust only, used for everything in this report) and
`devShells.flutter` (the old combined shell, used only for the Flutter
Linux build) — verified structurally via `nix flake check`, but **not**
re-verified live (building `devShells.flutter` again would currently
re-trigger the same ~60GiB pull, and disk headroom is limited enough
right now that doing so just to confirm a structural refactor isn't worth
the risk). Spot-check it next time real Flutter work happens.

## Still open
- Timeline read/send against a real room (blocked on the test account
  having one).
- `devShells.flutter` is unverified live (see above).
- Same long-standing deferred items: the owner-facing score/notes-via-
  Matrix-message report feature (not yet scoped), and wiring any of this
  phase's new API through `flutter_rust_bridge` to the UI (explicitly the
  next phase, not touched here).
