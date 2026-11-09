# 2026-11-06 — Rust skeleton verified; Flutter side blocked on SDK install

Follow-up to `2026-11-06-initial-restructure.md`. Implements the plan at
the time (`rustling-giggling-snowglobe`): bridge = `flutter_rust_bridge`,
v1 platforms = Windows + Linux desktop only, Flutter SDK installed
independently of Nix.

## Done and verified
- `rust-toolchain.toml` / `Cargo.toml`: MSRV bumped 1.95.0 → 1.96.0 to
  match `matrix-rust-sdk`'s own workspace requirement.
- `crates/matrix-client`: new crate, `matrix-sdk` 0.19.1 (rustls via
  `rustls-aws-lc-rs` — the actual feature name; the hoped-for `rustls-tls`
  doesn't exist on this version) + `sqlite` + `e2e-encryption` features.
  Ships only `health_check()` for now, by design — real session/sync logic
  is the next phase, not this one.
- `flake.nix` (new, project root): devShell with the Rust toolchain
  (resolves to 1.99.0 off `nixos-unstable` right now — above the 1.96
  floor) plus native Linux-desktop/Flutter build deps. Fixed the
  deprecated `xorg.lib*` names to the flat `lib*` names nixpkgs now wants,
  and added `clippy`/`rustfmt` (not bundled with the bare `rustc`/`cargo`
  packages).
- Verified inside `nix develop`: `cargo check --workspace --all-targets`,
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `cargo test --workspace` (14 tests, all green), `cargo deny check` — all
  clean.
- `deny.toml`: two new exceptions, both transitive-only through
  `matrix-sdk` itself (not something we chose) — `BSL-1.0` license
  (`xxhash-rust` via `growable-bloom-filter` via `matrix-sdk-base`) and
  `RUSTSEC-2026-0319` (`anymap2`, unmaintained, no safe upgrade exists).
  Documented in both `deny.toml` and `docs/SECURITY.md`.

## Blocked
Flutter SDK still isn't installed on this machine (confirmed via
`which flutter`/`which dart` — neither resolves). Per the owner's own
decision, Flutter is installed independently of Nix, not something this
session does. Everything downstream of that — `flutter create`,
`flutter_rust_bridge_codegen create`, the `ping()` round-trip proof — is
blocked until it is.

## Housekeeping note
Hit `ENOSPC` once mid-session: `/` is at 98% used (5.8G free of 250G).
Not this session's problem to fix, but flagging since a `cargo`/Nix-heavy
project will keep growing `target/` and the Nix store.
