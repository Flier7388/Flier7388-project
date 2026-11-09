# 2026-11-06 — Flutter ↔ Rust pipeline proven end-to-end

Closes out `rustling-giggling-snowglobe`'s plan. Done:

## What was built
- `flutter/`: `flutter create --platforms=linux,windows` (Linux + Windows
  desktop only, per the owner's decision — no Android/web/macOS/iOS).
- `flutter/rust` (`app_bridge`): scaffolded via
  `flutter_rust_bridge_codegen integrate --rust-crate-name app_bridge
  --rust-crate-dir rust`. Depends on `crates/matrix-client` as a path
  dependency. Only function: `ping()`, forwarding to
  `matrix_client::health_check()`.
- `flutter/lib/main.dart`: minimal screen calling `ping()` on startup and
  rendering the result — replaced the stock frb demo ("greet Tom") and the
  stock `flutter create` counter app entirely.
- Root `Cargo.toml`: added `workspace.exclude = ["flutter/rust"]` — needed
  because Cargo auto-detects the enclosing workspace by walking up
  directories and errors without an explicit exclusion.

## Verified
- `flutter build linux` succeeds: Cargokit builds `app_bridge` (pulling in
  `matrix-client` → `matrix-sdk` 0.19.1) as `libapp_bridge.so`, links it
  into the bundle. `✓ Built build/linux/x64/release/bundle/matrix_client`.
- Ran the built binary live (real `DISPLAY`/Wayland session) — stayed up,
  took a screenshot confirming the UI renders.
- `flutter test` (with `LD_LIBRARY_PATH` pointed at the release bundle's
  `lib/` — see below) passes: confirms Dart actually called into Rust and
  got `"matrix-client: ok"` back, not just that it compiled.

## Two real environment problems hit and fixed
1. **Cargokit needs `rustup`.** flutter_rust_bridge's default build
   backend shells out to `rustup` unconditionally for local builds, with
   no override for an already-working `cargo`/`rustc`. Added `rustup` to
   `flake.nix`'s devShell (manages its own toolchain, separate from the
   Nix-provided one used for everything else).
2. **`flutter test` doesn't bundle the native lib.** Unlike
   `build`/`run`, it needs `LD_LIBRARY_PATH` pointed at an existing build's
   `lib/` directory manually.

Both documented in `docs/ARCHITECTURE.md`'s new "Known environment
quirks" section so nobody re-discovers them from scratch.

## Disk space incident (own mistake, logged so it isn't repeated)
Added `sysprof` to `flake.nix` to satisfy a `glib-2.0` pkg-config
requirement (`sysprof-capture-4`) — the Linux CMake configure step fails
outright without it. Didn't check the package's closure first: `sysprof`
is nixpkgs' full GNOME profiler *application*, whose `buildInputs` pull
`gtk4`, `libadwaita`, `polkit`, `libpanel`, and transitively a large
GStreamer/codec stack. Cost ~60GiB of Nix store space in one `nix develop`
and ran the disk to **0 bytes free**, breaking Nix's own SQLite caches
md-flight. Fixed by running `nix-collect-garbage -d` (freed 61.2GiB,
standard safe maintenance, didn't touch project files). Tried `sysprof.dev`
hoping for a smaller closure — confirmed via a second `nix develop` that it
does **not** help on this nixpkgs revision (dev output's own propagated
inputs pull the identical GTK4/libadwaita chain). Reverted to plain
`sysprof`, documented not to retry that "fix" blindly. Net cost is already
paid and cached locally; current disk headroom ~45GiB, not urgent.

## Still open
Same as before: target homeserver for real login testing (owner's own
server, not yet named), and the scored report/feedback-to-Matrix feature
(deferred, not designed). Real session/room/timeline/send logic in
`matrix-client` is the next phase — `ping()` was always meant to be
throwaway scaffolding, not a feature.
