# 2026-11-06 — Initial restructure

## What was found
The repo already had a Rust workspace built by a prior AI session: `matrix-core`
(identifier parsing), `matrix-storage` (media-cache retention policy), `feedback-dto`
(a report DTO), and `app-ui` — a UI spike on the `iced` toolkit. None of it depended
on `matrix-rust-sdk`. This predates the current direction (Rust + matrix-rust-sdk
backend, Flutter UI) and `iced`/Flutter are mutually exclusive UI choices.

## What was kept
`matrix-core`, `matrix-storage`, `feedback-dto`: pure logic, no UI-toolkit
dependency, well-tested (`cargo test` passes), accurately documented. Useful
regardless of which UI framework sits on top. `feedback-dto`'s shape (severity
enum) doesn't match the "0–10 score + fix/add/remove notes, sent as a Matrix
message" report feature described for this project — flagged in
`docs/ARCHITECTURE.md` as needing rework once that feature is scoped, not removed.

## What was retired
`app-ui` (the `iced` spike): removed from the Cargo workspace, source moved to
`examples/legacy-iced-ui-spike/` with a README explaining why (no code is
portable to Flutter) and what's still worth remembering from it (Android IME
and large-list-scroll risk points to re-verify under Flutter).

## Cleanup that followed from removing `iced`
- Root `Cargo.toml`: dropped the `iced` workspace dependency and the `app-ui`
  workspace member.
- `.github/workflows/ci.yml`: removed the Android cross-check job (it only
  existed to type-check `app-ui`'s Android target).
- `deny.toml`: removed the RUSTSEC advisory ignores and `allow-git` entries
  that existed solely because of `iced`'s unreleased Android git branch;
  `bans.wildcards` reverted to `deny` (no more legitimately version-less git
  deps); `[graph].targets` dropped `aarch64-linux-android` for now.
- `rust-toolchain.toml`: dropped the `aarch64-linux-android` target.
- `docs/ARCHITECTURE.md` / `docs/SECURITY.md`: rewritten to describe the
  current state and the actual open decisions, instead of the `iced`-specific
  Android go/no-go plan.

Verified `cargo check --workspace --all-targets` passes after all of the above.

## New directories
- `examples/` — for reference material (user-provided or retired spikes).
- `reports/` — this directory.

## Still open (need the project owner's decision, not mine to assume)
1. Rust↔Flutter bridge mechanism: UniFFI/`matrix-sdk-ffi` vs.
   `flutter_rust_bridge` vs. a separate process over IPC.
2. First-milestone target platforms (Linux desktop? Android? Windows? more?).
3. The actual shape of the owner-facing "score 0–10 + notes, sent as a Matrix
   message" report feature — not designed yet, `feedback-dto` as it stands
   doesn't match it.
