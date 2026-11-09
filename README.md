# Matrix Client

*[Читать на русском](README.ru.md)*

A [Matrix](https://matrix.org) chat client. Client logic lives in
**Rust** on [`matrix-rust-sdk`](https://github.com/matrix-org/matrix-rust-sdk);
the UI is **Flutter**, talking to the Rust core through
[`flutter_rust_bridge`](https://github.com/fzyzcjy/flutter_rust_bridge).

Status: **early alpha**, see [`ROADMAP.md`](ROADMAP.md) for what's done
and what's next. Not ready for daily use yet.

## About

This is a personal experiment: I'm using it to find out what current AI
models are actually capable of building, end to end, with real
oversight rather than a toy demo. It's built mostly for my own daily
use, not as a product aimed at a wide audience.

Being a personal/experimental project doesn't mean security gets
shortchanged — see [`docs/SECURITY.md`](docs/SECURITY.md) for the
actual policy this codebase is held to.

## Scope

v1 targets **desktop only: Windows and Linux**. No Android/iOS/macOS — a
phone client is planned as a separate project later, not a feature of
this one.

## Repository layout

- `crates/` — the Rust workspace:
  - `matrix-client` — session, sync, rooms, spaces, timeline; wraps
    `matrix-sdk`/`matrix-sdk-ui`. No bridge-tooling dependency, testable
    with plain `cargo test`.
  - `matrix-core` — SDK-free Matrix identifier parsing.
  - `matrix-storage` — media-cache retention policy.
  - `feedback-dto` — in-app feedback report shape (not finalized yet).
- `flutter/` — the Flutter app. `flutter/rust` is the
  `flutter_rust_bridge` bridge crate, excluded from the root Cargo
  workspace on purpose (see `docs/ARCHITECTURE.md`).
- `design/` — the visual reference this UI is built against.
- `docs/` — architecture and security notes, kept in sync with what's
  actually built.
- `reports/` — dated session reports from development.

## Building

Nix is the supported way to get a working toolchain; see `flake.nix`.

```sh
# Rust-only work (cargo check/test on the matrix-client crate etc.)
nix develop

# Building/running the Flutter desktop app (pulls in GTK/Flutter —
# a much heavier shell, kept separate on purpose, see flake.nix)
nix develop .#flutter
cd flutter
flutter build linux   # or: flutter run -d linux
```

Windows: the Rust workspace is checked in CI on Windows, but the actual
Flutter Windows build hasn't been attempted yet — treat it as
unverified until someone runs `flutter build windows` for real.

## License

[GPL-3.0-only](LICENSE).

## Contributing

This is currently a one-person project built with AI assistance.
Issues and commits in either Russian or English are both fine.
