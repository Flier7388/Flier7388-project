# Live-updating timeline (2026-11-08, second phase of the day)

## What changed
Replaced the `home_screen.dart` 2-second polling hack (`Timer.periodic`
re-calling `timelineMessages` every 2s while a room was open — explicitly
flagged as a hack in its own comment) with a real push stream, closing
the "live-updating room list / timeline streams" item from the deferred
list.

- `crates/matrix-client/src/timeline.rs`: new `RoomTimeline::
  subscribe_messages()` returns `(Vec<TimelineMessage>, impl Stream<Item
  = Vec<TimelineMessage>>)`. Built on `matrix_sdk_ui::timeline::Timeline::
  subscribe_filter_map` (not the raw `subscribe()` the existing snapshot
  `messages()` uses) — it filters to text messages *before* diffing, so
  the diff stream only reports changes to messages the UI actually cares
  about. Each `VectorDiff` from the stream is applied to a local
  `imbl::Vector` copy and re-flattened to a plain `Vec` per update (full
  snapshot, not incremental diff) — the Dart side has nothing to apply an
  `imbl::Vector` diff against, and a single room's timeline is small
  enough that this is cheap. Shared the event-filtering logic between
  `messages()` and `subscribe_messages()` via a new `as_text_message`
  free function.
- `flutter/rust/src/api/matrix.rs`: new `timeline_messages_stream(timeline,
  sink: StreamSink<Vec<TimelineMessage>>)` — pushes the initial snapshot,
  then every subsequent one, stopping if `sink.add` errors (Dart side
  cancelled its subscription). Codegen turns this into `Stream<List<
  TimelineMessage>> timelineMessagesStream({...})` on the Dart side.
- `flutter/lib/screens/home_screen.dart`: `_openRoom` now subscribes to
  `timelineMessagesStream` instead of polling; `_sendMessage` no longer
  manually re-fetches afterward (the live subscription picks up the
  local echo and its confirmation on its own). `Timer? _timelinePoll`
  replaced with `StreamSubscription<List<TimelineMessage>>?
  _timelineSubscription`, cancelled on room switch/space switch/dispose.

Deliberately **not** done this phase: room-list reactivity (`rooms()`/
`spaces()` are still snapshot-plus-manual-refresh). Scoped to the
timeline only — it was the one with an actual hack sitting in the code
calling for a fix; the room list's refresh-button UX isn't broken, just
less automatic. Worth a follow-up if it becomes the actual complaint.

## Verified against real compiled source, not docs.rs
Per this project's standing practice (docs.rs misled previous phases
twice): confirmed `StreamSink<T>`'s real shape and the `sink: StreamSink<T>`
parameter pattern against flutter_rust_bridge_codegen's own vendored
`ui_related.rs` boilerplate generator (`~/.cargo/registry/.../
flutter_rust_bridge_codegen-2.13.0/src/library/codegen/parser/
early_generator/ui_related.rs`), and `Timeline::subscribe_filter_map`'s
exact signature plus the `VectorDiff::apply(self, vec: &mut Vector<T>)`
diff-application pattern against both matrix-sdk-ui's own vendored source
and `git/robrix`'s real usage (`sliding_sync.rs`'s pinned-messages
watcher does the exact same `diff.apply(&mut items)` loop). Two real bugs
this caught before ever reaching the compiler:
- `StreamSink` isn't exported from the `flutter_rust_bridge` crate root
  — it's a type alias flutter_rust_bridge_codegen generates per-codec in
  `frb_generated.rs`. `use flutter_rust_bridge::StreamSink;` fails;
  `use crate::frb_generated::StreamSink;` is correct (rustc's own
  suggestion matched the pattern found in the vendored boilerplate).
- The returned `impl Stream<...>` needed `+ use<'_>` (Rust's precise
  capturing syntax) since it borrows `&self` through `diffs` —
  `matrix_sdk_ui::Timeline::subscribe()` sidesteps this by cloning a drop
  handle into an owned wrapper type, but `subscribe_filter_map` doesn't,
  so the lifetime genuinely has to be captured here.

## Verified
`cargo check -p matrix-client`, `cargo check` on `app_bridge` (both via
the lightweight `devShells.default`), `flutter_rust_bridge_codegen
generate`, `dart analyze` (no issues), `flutter build linux` — all clean.
Launched the built binary: login screen renders correctly (screenshotted,
partially obscured by the terminal window but the form is visibly
intact). **Not click-tested live** — same `xdotool`/`ydotool` gap as
every prior phase, so the actual "open a room, watch a message arrive
without polling" behavior still needs the owner's own click-through.

## `flutter_rust_bridge_codegen` isn't on the flutter devShell's PATH
It's a `cargo install`-managed binary living in `~/.cargo/bin`, not part
of `flake.nix`'s `devShells.flutter` `buildInputs` — `nix develop
.#flutter -c flutter_rust_bridge_codegen generate` fails with "not
found" because `nix develop -c` doesn't inherit the interactive shell's
`~/.cargo/bin` PATH addition. Worked around this time by calling
`~/.cargo/bin/flutter_rust_bridge_codegen` directly. Worth fixing
properly later — either add it to the devShell's `buildInputs` (as a
`rustPlatform.buildRustPackage` pin, or via `cargo install` inside a
shellHook) or just always invoke it by absolute path.

## Disk space, a third and fourth time today
Ran into "No space left on device" twice more mid-phase — once before
`cargo check` even started (`nix develop` itself failed fetching its
flake input), once mid-way through app_bridge's check. Investigated
whether this project's own flake-source copy (this repo isn't a git
repo, so Nix can't apply git-aware `.gitignore` filtering to the
self-referencing `path:` input, meaning every `nix develop` should in
theory copy the *entire* directory tree verbatim) was the actual root
cause — checked `du -sh` on `git/`'s four reference-client clones (only
~280MB total) and the whole project tree minus build dirs (~300MB):
**too small to explain 50-70GiB GC frees**, so that theory doesn't hold.
The dominant cost is still what `docs/ARCHITECTURE.md` already
documents: the Flutter+GTK closure (~50-60GiB) isn't pinned as a GC root
by a plain `nix develop` shell, so it gets evicted and fully refetched
basically every session. `nix-collect-garbage -d` run twice more
(genuinely blocked both times, not preemptively) — 53.7GiB then 70.5GiB
freed. No new mitigation found this session beyond what's already
written down; a real fix (e.g. a persistent GC root for the flutter
devShell, `nix develop --profile`) is worth a dedicated look if this
keeps costing full sessions' worth of time.

## Still open
- Owner click-through of both the session-persistence round-trip (from
  this morning's phase) and this phase's live-timeline-stream, together
  in one real session.
- Room-list (`rooms()`/`spaces()`) reactivity, if it ends up mattering —
  currently snapshot + manual refresh, deliberately not touched this
  phase.
- `feedback-dto` rework — still blocked on the owner describing the
  feature.
- A real persistent-GC-root fix for the Nix disk-thrash pattern, if it
  keeps recurring.
