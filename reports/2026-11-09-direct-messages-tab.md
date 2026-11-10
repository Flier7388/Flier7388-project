# Direct Messages tab (2026-11-09, seventh phase of the day)

## What changed
The last open 0.1.0-alpha item that wasn't blocked on real hardware:
the Direct Messages rail tab was just the generic "Not implemented
yet" placeholder. It now shows a real, live-updating list of the
account's DM rooms, same list/room-view/member-drawer layout as the
Home tab.

- `crates/matrix-client/src/room.rs`: `MatrixClient::direct_rooms()`
  and `subscribe_direct_rooms()`, siblings of the existing
  `rooms()`/`subscribe_rooms()` but filtered on `matrix_sdk::Room::
  is_dm()` instead of "not a space". `is_dm()` checks the room's
  `m.direct` account-data flag (via `direct_targets_length() == 1`
  under this client's default `DmRoomDefinition::MatrixSpec`) — not a
  "has at most 2 members" heuristic, which would wrongly catch any
  small group room nobody ever flagged as a DM.
- `flutter/rust/src/api/matrix.rs`: `direct_rooms`/`direct_rooms_stream`
  bridge functions, same shape as `rooms`/`rooms_stream`.
- `flutter/lib/screens/home_screen.dart`: new `_directRooms` state +
  `_directRoomsSubscription`, wired into the existing `RailTab.direct`
  branch with the same `RoomList`/`RoomView`/`MemberDrawer` trio the
  Home tab and space view already use (title: "Direct Messages").

## A real test, not just a compile check
`direct_rooms_includes_only_dm_flagged_rooms` in `room.rs`: builds two
joined rooms via the mock homeserver, flags one of them in `m.direct`
global account data (`EventFactory::direct().add_user(...)`,
`SyncResponseBuilder::add_global_account_data`), and asserts
`direct_rooms()` returns only the flagged one. Confirms the filter is
actually keyed on the DM flag, not room shape.

## Disk ran out mid-session
`flutter_rust_bridge_codegen generate` died with "No space left on
device" — `/` was at 100% (1.3M free). `nix store gc --dry-run` showed
70 collectible paths; asked the owner before running it live (not a
routine pass, an actually-blocked one). Confirmed, ran it: 65.9 GiB
freed, back to 73% used. Codegen, `dart analyze`, and
`flutter build linux` all succeeded afterward. `matrix-save/` (a 20G
sibling directory that looks like an old project backup) was flagged
but left untouched — never got a read of what's actually in it, and
the owner didn't pick that option.

## Verified
`cargo check -p matrix-client`, `cargo check` on `app_bridge`,
`cargo test -p matrix-client` (new test passes),
`flutter_rust_bridge_codegen generate`, `dart analyze` (no issues),
`flutter build linux` (succeeds) — all clean. Launched the built
binary and raised it via the KWin-script trick (still no `xdotool`/
`ydotool`); screenshotted while the owner appeared to already be
poking at it live — the Direct Messages panel rendered correctly
(title, refresh button, clean empty list), but the account checked
against has no DM-flagged rooms right now, so the "shows the other
user's name" half of the roadmap's acceptance criterion is still
unconfirmed in practice, only in the unit test. Noted as a standing
gap in `ROADMAP.md`/`ROADMAP.ru.md` rather than claimed as fully done.

## Also added: a CI job to actually attempt the Windows build
0.1.0-alpha's one other open item — `flutter build windows` — can't be
run in this environment (no Windows machine, and this working copy
isn't a git repo, so nothing here can push to trigger GitHub Actions).
Added `flutter-windows-build` to `.github/workflows/ci.yml`: a
`windows-latest` job (Rust toolchain + `subosito/flutter-action`
pinned to 3.47.4, matching the local Nix devShell's Flutter version)
that runs `flutter pub get` then `flutter build windows` in `flutter/`.
`flutter build windows` drives Cargokit itself, so it also
cross-compiles `app_bridge` for the Windows target — no separate
`cargo build --target x86_64-pc-windows-msvc` step needed. Unverified
until the owner pushes this and a CI run actually succeeds (or someone
runs it on a real Windows box) — left open in both roadmap files with
that caveat spelled out, not checked off.

## 0.1.0-alpha status
Every item in this milestone is now done except the one thing this
environment genuinely cannot provide:
- [x] 3-pane layout, custom theme, live navigation, member drawer,
  settings profile, Direct Messages tab — all done.
- [ ] Windows build actually attempted — blocked, no Windows machine
  available here. The only thing left before calling 0.1.0-alpha
  closed.
