# Member list drawer (2026-11-09, third phase of the day)

## What changed
Closed the first checklist item of `ROADMAP.md`'s 0.1.0-alpha milestone
beyond what was already done: a real member-list drawer for an open
room, matching `design/index.html`'s `#membersDrawer` mockup.

- `crates/matrix-client/src/member.rs` (new): `MatrixClient::
  room_members()` on `matrix_sdk::room::Room::members(RoomMemberships::
  JOIN)`, mapping each `RoomMember` to a flat `RoomMemberSummary`
  (`user_id`, `display_name` via `.name()`'s own localpart fallback,
  `is_admin` via `.suggested_role_for_power_level()` — true for
  Moderator/Administrator/Creator, since the mockup only has two groups
  rather than a finer role split).
- `flutter/rust/src/api/matrix.rs`: `room_members` bridge function +
  `RoomMemberSummary` mirror type, same pattern as every other
  bridge-crossing type in this file.
- `flutter/lib/widgets/member_drawer.dart` (new): the drawer itself —
  266px wide (matches `design/style.css`'s `--members-w`), grouped
  "Admins — N" / "Members — N" headers, same avatar-initial treatment
  (`colorForMxid`/`initials`) the rail nav already uses for spaces/user.
- `flutter/lib/widgets/room_view.dart`: room header gets a real "Show
  Members" icon button (was previously just avatar + name, no actions
  at all).
- `flutter/lib/screens/home_screen.dart`: member state
  (`_membersOpen`/`_membersBusy`/`_members`), fetched on toggle via a
  one-shot `roomMembers()` call (not a live stream — membership changes
  are rare enough that snapshot-on-open is fine, unlike timeline/room
  list). Stays open across a room switch, refetching for the newly
  opened room, rather than closing — matches how a persistent side
  panel should behave.

## Verified
`cargo check -p matrix-client`, `cargo check` on `app_bridge`,
`flutter_rust_bridge_codegen generate`, `dart analyze` (no issues),
`flutter build linux` (succeeds) — all clean, first try on the Rust
side (API verified against the real vendored `matrix-sdk` 0.19.1
source, not docs.rs, same standing practice). **Not runtime-tested this
round** — the owner flagged their laptop running hot after several
builds in a row this session, so skipped the usual launch+screenshot
pass. Same standing gap as every other feature this week: no `xdotool`/
`ydotool` here, so the actual "open a room, click Show Members, see the
real participant list" flow still needs the owner's own click-through
whenever convenient.

## Still open (0.1.0-alpha)
- Settings dialog: real profile display (not just logout).
- Direct Messages tab: real 1:1 room list.
- Windows desktop build actually attempted (no Windows machine
  available in this environment either).
