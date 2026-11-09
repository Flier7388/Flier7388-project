# Room/space list reactivity (2026-11-09)

## What changed
Closed the other deferred reactivity item from yesterday: `rooms()`/
`spaces()` were snapshot + manual refresh (room list had a literal
"Refresh rooms" button and a fixed 5s post-sync delay before the first
read). Added live push streams, same shape as yesterday's timeline work.

- `crates/matrix-client/src/room.rs`: new `MatrixClient::subscribe_rooms()`
  built on `matrix_sdk::Client::rooms_stream()` — the same low-level
  primitive `matrix-sdk-ui`'s own `RoomListService::entries()` uses
  internally (verified against its vendored source; deliberately did
  *not* reach for `RoomListService::entries_with_dynamic_adapters`,
  which is Element X's sorted/paginated/multi-filter machinery, far more
  than this app's flat list needs). Diffs are applied to an
  `imbl::Vector<Room>` and re-summarized via `.then()` since each room's
  display name is an async lookup; a failed per-room display-name lookup
  is now logged and skipped rather than aborting the whole batch (a new,
  slightly more resilient helper `summarize_rooms`, used only by the new
  method — `rooms()`'s existing snapshot behavior was left untouched).
- `crates/matrix-client/src/space.rs`: new `MatrixClient::
  subscribe_top_level_joined_spaces()` on `SpaceService::
  subscribe_to_top_level_joined_spaces()` — a direct reactive API
  `matrix-sdk-ui` already exposes, no extra plumbing needed. Refactored
  `spaces()` to share the new `to_summary` mapping helper.
- `flutter/rust/src/api/matrix.rs`: `rooms_stream`/`spaces_stream`
  bridge functions, same push-until-cancelled `StreamSink` pattern as
  yesterday's `timeline_messages_stream`.
- `flutter/lib/screens/home_screen.dart`: subscribes to both streams in
  `initState`, dropped the fixed 5-second post-sync delay entirely (the
  list now just fills in whenever the store actually has data). Kept the
  room list's manual refresh button as a harmless one-shot nudge on top
  of the live subscription, since `RoomList`'s widget API still requires
  an `onRefresh` callback and removing it wasn't this task's scope.

Deliberately not touched: `space_rooms()` (listing a specific space's
child rooms) stays snapshot + manual refresh — its pagination logic
(`SpaceRoomList`/`MAX_HIERARCHY_PAGES`) would need its own look at a
reactive equivalent, out of scope for this pass.

## A real compile error caught by the type system, not speculation
`subscribe_rooms`'s stream (built with `StreamExt::then`, since each
update needs an async per-room display-name lookup) turned out to be
`!Unpin` — `app_bridge`'s consuming `while let Some(x) = stream.next().
await` loop failed to compile with `the trait Unpin is not implemented
for {async block}`. Fixed by pinning on the stack with `futures_util::
pin_mut!(updates)` before the loop. `spaces_stream`'s loop (built on
`.map()`, no async captured per item) didn't need this — the diagnostic
difference between `.then()` and `.map()`'s Unpin-ness is worth
remembering if more streaming bridge functions get added later.

## Verified
`cargo check -p matrix-client`, `cargo check` on `app_bridge`,
`flutter_rust_bridge_codegen generate`, `dart analyze` (no issues),
`flutter build linux` — all clean. Launched the built binary twice;
confirmed via its own stdout log (not just process-alive) that it starts
cleanly on the Impeller/OpenGLESSDF backend with no crash, only a benign
`Gdk-Message: Unable to load from the cursor theme` warning. Could not
get a clean full-window screenshot this round — the owner had several
other windows (including an open GitHub tab, mid-repo-setup) stacked over
the app window, and there's still no `xdotool`/`ydotool` to raise/focus
it. Left the process running on the owner's desktop for them to check
directly instead of fighting for a screenshot.

## Side task this session: exported sources for the owner's own GitHub push
Owner asked for project sources copied into a separate folder to upload
to GitHub themselves. Copied to a separate local export folder (19MB)
via `rsync`, excluding `target/`, `flutter/build`, `.dart_tool`, `.idea`,
`*.iml`, and — importantly — the `git/` directory of reference-client
clones (robrix/iamb/rumatui/element-x-android): those are other people's
code pulled in for research, not this project's own source, and
shouldn't end up in the owner's repository. Extended the exported copy's
`.gitignore` (the original only had `/target`) to also cover `flutter/
rust/target/`, `.idea/`, `*.iml`, `.flutter-shell-root`. Scanned the copy
for accidentally-included secrets/session files before handing it off —
clean (only found `session_store.dart`, the source file, not actual
session data).

## Still open
- Owner click-through of all three of this week's reactive features
  (session persistence, live timeline, live room/space lists) together.
- `space_rooms()` reactivity, if it ends up mattering.
- `feedback-dto` rework — still blocked on the owner describing the
  feature.
- `flutter_rust_bridge_codegen` still not on `devShells.flutter`'s PATH
  (noted yesterday, not fixed) — called by absolute path again.
