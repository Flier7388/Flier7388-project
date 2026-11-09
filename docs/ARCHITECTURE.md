# Architecture

## Direction
A Matrix client with the client logic in **Rust** (on `matrix-rust-sdk`) and
the UI in **Flutter**, rendered via Flutter's own GPU pipeline (Impeller on
mobile/desktop, Skia fallback where Impeller isn't available). Rust is the
core product; Flutter is a UI shell that talks to it.

This repo's layout, going forward:
- `crates/` — the Rust workspace (this phase's content, below).
- `flutter/` — the Flutter application (not scaffolded yet).
- `examples/` — reference material, provided by the project owner or kept
  from retired spikes, not part of any build.
- `reports/` — status/decision reports from work sessions.
- `docs/` — this directory: architecture and security notes that apply to
  the whole project, kept in sync with what's actually built.

## Decided
1. **Rust↔Flutter bridge: `flutter_rust_bridge`** (stable 2.13.0), not
   `matrix-sdk-ffi`'s own UniFFI path. Rejected UniFFI because its only
   Dart bindgen, `uniffi-bindgen-dart`, is a single-maintainer tool created
   March 2026 with no confirmed production Matrix-client usage — not
   acceptable given this project's strict security review bar (see
   `docs/SECURITY.md`). `matrix-sdk-ffi` itself is solid (it's what Element
   X ships on Android/iOS); the Dart generator on top of it is the part
   that isn't mature enough yet.
2. **v1 target platforms: Windows desktop + Linux desktop only.** No
   Android/macOS/iOS for now.
3. **Flutter SDK is installed independently of Nix**, not via nixpkgs' own
   `flutter` package (version lag + known NixOS packaging friction). The
   project's `flake.nix` devShell provides only the pinned Rust toolchain
   and the native libraries a Flutter Linux desktop build needs.

## Current Rust workspace
- `matrix-core` — SDK-free domain types (Matrix identifier parsing:
  `@user:server`, `!room:server`, `#alias:server`). No network, no storage.
  `#![forbid(unsafe_code)]`.
- `matrix-storage` — disk media-cache **policy only** (LRU/retention
  decisions, pure logic, no filesystem I/O yet). Messages, room state, and
  E2EE storage are delegated entirely to `matrix-sdk-sqlite` once
  `matrix-rust-sdk` is integrated; this crate never reimplements that
  store. It only owns the separate on-disk blob cache for downloaded media.
  `#![forbid(unsafe_code)]`.
- `feedback-dto` — a serializable report type, no transport. **Needs
  rework**: it currently models an in-app end-user bug report
  (low/medium/critical severity), which is not the same thing as the
  owner-facing "score 0–10 with fix/add/remove notes, delivered as a
  Matrix message to a specific user" report feature described for this
  project — revisit its shape once that feature is actually scoped.
  `#![forbid(unsafe_code)]`.
- `matrix-client` — pure-Rust client logic on `matrix-sdk` 0.19.1 (the same
  release `matrix-sdk-ffi` is built from) plus `matrix-sdk-ui` 0.19.1 (same
  release line) for `SyncService`/`RoomListService`/`Timeline` — the same
  stack Element X runs on top of `matrix-sdk`, chosen over hand-rolling
  sliding sync for the same "prefer upstream-maintained, battle-tested
  paths" reasoning as the storage delegation below. Owns session lifecycle
  (`login`/`restore`/`session`, `src/session.rs` + `src/client.rs`), sync
  (`start_sync`/`stop_sync`, `src/sync.rs`), the joined-room snapshot
  (`rooms()`, `src/room.rs`), and per-room timeline read/send
  (`timeline_for()`, `src/timeline.rs`). Carries **no**
  `flutter_rust_bridge` macros, so it stays testable with plain `cargo
  test` independent of the bridge tooling. `#![forbid(unsafe_code)]`.
  Verified end-to-end against the project owner's real homeserver
  (domain intentionally not published) test account 2026-11-06 (login, session round-trip through `serde_json`, sync start,
  room-list snapshot, timeline read, and `send_text()` — sent a real
  message and read it back with the correct sender/body — see
  `crates/matrix-client/examples/sync_smoke_test.rs`). Built against real
  usage in `git/robrix`
  (pins the identical `matrix-sdk-ui` 0.19.1) and `git/iamb`, not just
  `docs.rs` — its rendered API signatures were subtly wrong twice this
  session (`login_username`'s real signature, and `Timeline`'s builder
  method name, `timeline_builder()` via the `RoomExt` trait, not a
  free-standing `Timeline::builder(room)`).
  - Deliberately deferred to a later phase: the live-updating half of
    `RoomListService`/`Timeline`'s `subscribe()` APIs (snapshot only for
    now — reactive FFI callbacks are a bridge-layer design question);
    where/how `Session` gets persisted to disk (this crate only makes it
    serializable, see `src/session.rs`'s own doc comment); E2EE
    decryption of pre-existing encrypted rooms for a fresh device (no
    existing room keys on first login).

## Flutter + bridge layer
`flutter/` is the Flutter app; `flutter/rust` (`app_bridge`) is the bridge
crate scaffolded by `flutter_rust_bridge_codegen integrate`. It depends on
`matrix-client` as a path dependency and carries the only
`#[frb]`-annotated code in the repo — `flutter/rust/src/api/matrix.rs`
now exposes the real surface: `login`, `restore_session`,
`client_session`, `start_sync`, `stop_sync`, `rooms`, `timeline_for`,
`timeline_messages`, `send_text`. `flutter/rust` is **excluded** from the
root Cargo workspace (`Cargo.toml`'s `workspace.exclude`), not a member:
without the exclusion, Cargo errors with "believes it's in a workspace
when it's not" (it auto-detects the enclosing workspace by walking up
directories) — exclusion keeps frb's own manifest and Cargokit build
integration unconflicted, per the plan.

### Bridge data boundary: local mirrors, not `matrix-client`'s own types
`app_bridge` defines its own small plain-`String`-field mirror structs
(`Session`, `RoomSummary`, `TimelineMessage`) with explicit
`From`/`TryFrom` conversions, rather than exposing `matrix-client`'s real
types directly across the FFI boundary. Two reasons: (1) `matrix_client::
Session` carries real `ruma` ID types (`OwnedUserId`/`OwnedDeviceId`)
that Dart has no use for — a display string is the right shape on that
side anyway; (2) avoided betting on exactly how `flutter_rust_bridge`'s
codegen introspects a *plain data* type defined in an external crate
(its own docs site was inconsistently fetchable while researching this —
several 404s — so this was deliberately not gambled on). `MatrixClient`
and `RoomTimeline` themselves *do* cross directly as opaque handles
(confirmed working for external-crate types with async methods) — every
operation on them is a free function in `app_bridge` taking the handle as
a parameter (`pub use matrix_client::{MatrixClient, RoomTimeline};` must
be a `pub use`, not a plain `use` — a plain `use` compiles fine locally
but silently isn't visible to `frb_generated.rs`'s `use crate::api::
matrix::*;` glob import, which fails with `cannot find type MatrixClient`
only at the generated-code layer; verified locally 2026-11-07). Errors
cross as a plain `String` (`ClientError`'s `Display` impl via
`.to_string()`), not a mirrored enum — Dart sees a message, not
structured error *kinds*, a deliberate simplification.

The throwaway Flutter screen (`flutter/lib/main.dart`) exercises all of
it: a login form → room list → per-room timeline + send box. Plain
default Material widgets, not a designed UI — real UI work stays gated on
the owner's own visual references, as agreed. No session persistence to
disk yet (logs in fresh each launch).

### Known environment quirks (Nix + Cargokit)
- **Cargokit requires `rustup`.** flutter_rust_bridge's default build
  backend (Cargokit) unconditionally shells out to a `rustup`-managed
  toolchain for local builds — it errors ("rustup not found in PATH") even
  with a working `cargo`/`rustc` already on `PATH`, no bypass found in its
  own source (`flutter/rust_builder/cargokit/build_tool/lib/src/*.dart`).
  `flake.nix`'s devShell includes `rustup` solely for this; it manages its
  own toolchain, independent of the Nix-provided `rustc`/`cargo` used for
  everything else.
- **`glib-2.0`'s pkg-config file requires `sysprof-capture-4`.** Without
  it, the Linux desktop CMake configure step fails outright. The only
  source for it in nixpkgs right now is the full `sysprof` GNOME profiler
  *application* package — pulling `gtk4`, `libadwaita`, `polkit`,
  `libpanel`, and transitively a large GStreamer/codec closure (~60GiB of
  Nix store space; `.dev` output does **not** avoid this — its own
  propagated inputs pull the same chain, verified 2026-11-06). **This is
  why `flake.nix` has two separate `devShells`** (`default` for everyday
  Rust work, `flutter` for the Flutter Linux build) instead of one: with
  a single shared shell, routine `nix-collect-garbage` between sessions
  meant the *next* `nix develop` — even just for a plain `cargo` command —
  re-pulled this entire chain from the binary cache before anything else
  could run, causing two separate full-disk incidents in one session
  (2026-11-06). Use `nix develop .#flutter` for Flutter work, plain `nix
  develop` for everything else.
- **Don't run `nix-collect-garbage` preemptively.** Nix doesn't delete
  anything on its own — store paths only vanish when GC actually runs.
  Running it "just in case" between commands guarantees the *next* `nix
  develop` (even the lightweight shell) re-fetches whatever it just
  deleted, since nothing pins a `nix develop` shell's closure as a GC
  root once the shell exits. Hit this exact thrash loop repeatedly
  2026-11-06/07 — GC, next command refetches ~50GiB, repeat. Only GC when
  actually blocked (disk genuinely full), not preventively.
- **`flutter test` can't find `libapp_bridge.so` on its own.** Unlike
  `flutter run`/`flutter build`, plain `flutter test` doesn't bundle the
  native library onto a loadable path. Run it with
  `LD_LIBRARY_PATH=flutter/build/linux/x64/release/bundle/lib` (or the
  `debug` bundle path) pointed at a build that already exists — `flutter
  build linux`/`flutter run -d linux` first.

## Retired: `iced` UI spike
An earlier AI session built a Rust-native UI spike on the `iced` toolkit
(GPU-rendered via `wgpu`, single Rust codebase across platforms) before the
Flutter direction was confirmed. It has been removed from the Cargo
workspace and its source moved to `examples/legacy-iced-ui-spike/` as a
historical reference — see that directory's `README.md` for what, if
anything, is still worth remembering from it (mainly: concrete Android
IME/virtualization risk points to re-verify under Flutter, not any reusable
code).

## Matrix-sdk / server compatibility (forward-looking, not yet integrated)
Before wiring `matrix-rust-sdk` in a later phase, verify against whichever
homeserver the project actually targets:
- sliding sync / MSC support the target homeserver ships vs. what the
  pinned `matrix-sdk` version assumes;
- `matrix-sdk-sqlite`'s on-disk schema version is forward-compatible across
  the `matrix-sdk` version this project pins, since it's never our code to
  patch;
- E2EE device-verification flows (SAS, QR) against the target homeserver's
  actual key-backup endpoints, not just a generic one.
