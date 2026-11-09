# Timeline stream resource leak fix (2026-11-09, fourth phase of the day)

## The bug, found live by the owner
After the member-drawer feature landed, the owner tested the running
app directly (not a code-review catch) and reported: clicking into a
room worked once, but clicking the same room again, or switching back
to Home, did nothing — no error banner, no crash, just stuck.

## Root cause
Yesterday's live-timeline-stream feature (`RoomTimeline::
subscribe_messages`, bridged as `timeline_messages_stream`) has a real
resource leak. `home_screen.dart`'s `_openRoom` does `await
_timelineSubscription?.cancel()` before opening the next room, but
`flutter_rust_bridge`'s `StreamSink` only discovers a cancelled Dart
subscription *the next time it tries to push a value* — confirmed by
reading the actual vendored `flutter_rust_bridge` 2.13.0 source
(`src/stream/stream_sink.rs`, `src/rust_async/io.rs`): `#[frb]` async
functions are plain `tokio::spawn`ed tasks with no cancellation wiring
back from a dropped Dart `StreamSubscription`. On a quiet room (no new
messages arriving), the Rust-side loop sits forever on `updates.next()
.await`, never reaching the `sink.add()` call that would discover the
Dart side is gone. That means the `Timeline` (and the background
room-update listener task `matrix-sdk-ui` keeps alive for it, per its
own `TimelineDropHandle`) never gets dropped — every room switch leaked
one more of these, and enough accumulated zombies eventually caused real
contention/hangs on new timeline opens.

## The fix
- `crates/matrix-client/src/timeline.rs`: `RoomTimeline` now owns a
  `tokio_util::sync::CancellationToken`. `subscribe_messages`'s loop is
  rewritten with `async_stream::stream!` + `tokio::select!`, racing the
  diff stream against `cancel.cancelled()` — whichever resolves first
  ends the loop. New `RoomTimeline::stop()` cancels the token.
- `flutter/rust/src/api/matrix.rs`: new `stop_timeline` bridge function;
  `timeline_messages_stream`'s consuming loop needed `futures_util::
  pin_mut!` too (the `async_stream`-built stream is `!Unpin`, same class
  of issue `rooms_stream` hit yesterday with `.then()`).
- `flutter/lib/screens/home_screen.dart`: new `_stopCurrentTimeline()`
  helper, called in `_openRoom` (before replacing `_timeline`),
  `_selectSpace` (same), and `dispose()`. Cancelling the Dart
  `StreamSubscription` alone was never enough — both sides need to agree
  the stream is over.

## A process note worth remembering
Partway through writing this fix, discovered the persistent Bash
session's working directory had silently drifted to `/home/lor1kos/
github` (the publish staging folder) during the earlier GitHub
cleanup/push work, and several `nix develop -c cargo check` commands
using relative `cd` were silently re-checking that *stale* copy instead
of the real `/home/lor1kos/matrix` working tree — a false-positive
"compiles clean" that would've shipped the stale copy's state, not
today's actual edits. Caught it by noticing the checked path in cargo's
own output. The `flutter build linux` checks were unaffected (always
invoked with the Flutter project's full absolute path), so the member
drawer feature itself was never actually mis-verified — but it's a real
process gap worth remembering: prefer absolute paths for `cd` in this
session's Bash tool, since the working directory persists across calls
and silently drifting is easy to miss.

## Verified
`cargo check -p matrix-client`, `cargo check` on `app_bridge` (both
re-run with explicit absolute paths after catching the above), `dart
analyze` (no issues), `flutter build linux` (succeeds). **Owner
confirmed live**: relaunched the fixed build, clicked through multiple
rooms/spaces repeatedly — no more hang. This is the first bug this
project shipped that was caught by the owner's own usage rather than
automated checks or my own review; automated checks had no way to catch
it since it's a runtime resource-lifetime issue, not a type error.
