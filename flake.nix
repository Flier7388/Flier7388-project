{
  description = "Matrix client — Rust backend (matrix-rust-sdk) + Flutter UI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        # Flutter/Dart: `pkgs.flutter` (this flake's own pinned `nixpkgs`,
        # 3.47.4 as of this pin — already past 3.47's Impeller-by-default
        # threshold). Originally left out in favor of the system-wide
        # `flutter` install, but that turned out to cause a real bug, not
        # just a style preference: the system install's `dart`/`flutter`
        # binaries are linked against whatever glibc the *system's own*
        # nixpkgs channel happens to pin — a different generation than
        # this flake's own pinned `nixpkgs`, which is what actually builds
        # `libapp_bridge.so` (via Cargokit, inside this shell). `flutter
        # build linux`'s bundle step keeps its own lib directory
        # self-consistent, so the built app runs fine either way — but
        # plain `flutter test`/`flutter test integration_test` dlopen the
        # bridge `.so` straight into the system-glibc `dart` process,
        # which then fails with `GLIBC_2.44' not found` (verified
        # 2026-11-08, `ldd` on the built `.so` showed it linked against
        # this flake's glibc while the system `dart` process had an older
        # one already resident). Pulling `flutter` from this same pinned
        # `nixpkgs` puts every native piece on one glibc.
        #
        # Two devShells, not one — deliberately split 2026-11-06 after this
        # cost two separate multi-hour disk-space incidents (~60GiB each)
        # in one session. The Flutter Linux desktop build needs
        # `sysprof` (nixpkgs' full GNOME profiler *application*, pulled in
        # only to satisfy `glib-2.0`'s pkg-config `Requires:
        # sysprof-capture-4` — see the comment below) which drags in
        # `gtk4`/`libadwaita`/`polkit`/`libpanel` and a large
        # GStreamer/codec closure transitively. Every plain `cargo
        # check`/`test`/`run --example` session for `matrix-client` was
        # going through this *same* shell, so cleaning the Nix store
        # between sessions (routine, safe maintenance) meant the next
        # `nix develop` — even just to run a Rust example — re-pulled that
        # entire ~60GiB closure from the binary cache before anything else
        # could run. Pure-Rust work now uses `devShells.default`, which
        # never touches GTK/GStreamer at all; only `devShells.flutter`
        # pays that cost, and only when actually building the Flutter app.
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # --- Rust (matches rust-toolchain.toml's channel) ---
            rustc
            cargo
            rust-analyzer
            clippy
            rustfmt

            # --- matrix-sdk (rustls-based TLS; see crates/matrix-client) ---
            openssl
            sqlite
          ];
        };

        devShells.flutter = pkgs.mkShell {
          buildInputs = with pkgs; [
            # --- Flutter/Dart (see the glibc-consistency comment above) ---
            flutter

            # --- Rust (matches rust-toolchain.toml's channel) ---
            rustc
            cargo
            # Cargokit (flutter_rust_bridge's default build backend for the
            # `flutter/rust` bridge crate) unconditionally shells out to a
            # `rustup`-managed toolchain to build locally — it errors out
            # ("rustup not found in PATH") even with a perfectly working
            # `cargo`/`rustc` already on PATH, no bypass available (verified
            # against cargokit's own `build_tool` Dart source,
            # flutter/rust_builder/cargokit/build_tool/lib/src/*.dart, on
            # 2026-11-06). This `rustup` manages its own toolchain,
            # independent of the `rustc`/`cargo` above — only Cargokit calls
            # it.
            rustup

            # --- Flutter Linux desktop build dependencies ---
            pkg-config
            cmake
            ninja
            clang
            gtk3
            # `glib-2.0`'s own `.pc` file requires `sysprof-capture-4` at
            # pkg-config resolution time, even though it's unused at
            # runtime here — without it, GTK's CMake probe fails outright
            # (verified via `flutter build linux`'s own error, 2026-11-06).
            # `sysprof` is the full GNOME profiler *application* — its
            # `buildInputs` pull `gtk4`, `libadwaita`, `polkit`, `libpanel`
            # and, transitively, a large GStreamer/codec closure (~60GiB in
            # one `nix develop`, verified locally 2026-11-06). Tried
            # `sysprof.dev` hoping for a smaller closure (the usual nixpkgs
            # pattern for "I just need the headers/.pc file") — it does
            # NOT help here: `.dev`'s own propagated inputs still pull the
            # same GTK4/libadwaita chain on this nixpkgs revision (verified
            # by watching `nix develop`'s own copy-path log; don't retry
            # this "fix" without re-checking nixpkgs first). This cost is
            # now isolated to this shell alone — see the split rationale
            # above.
            sysprof

            # --- GPU stack (Impeller on Linux targets Vulkan) ---
            vulkan-loader
            libGL
            libxkbcommon
            wayland
            # Flat `lib*` names, not the deprecated `xorg.lib*` set (nixpkgs
            # warns on `xorg.*` as of this flake's nixos-unstable pin,
            # verified locally via `nix develop`'s own eval warnings).
            libx11
            libxcursor
            libxi
            libxrandr
            libxrender
            libxext
            libxcb

            # --- matrix-sdk (rustls-based TLS; see crates/matrix-client) ---
            openssl
            sqlite
          ];
        };
      });
}
