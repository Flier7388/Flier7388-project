# Retired: `iced` UI spike

This crate was built by a previous AI session before the project direction was
confirmed. It is **not part of the Cargo workspace anymore** (removed from
`members` in the root `Cargo.toml`) and does not build as part of the project.

## Why it's here instead of deleted

The project now targets **Flutter** for the UI (GPU-rendered via Flutter's own
Impeller/Skia pipeline), not a pure-Rust toolkit. This crate's `iced`-based code
is not portable to Flutter in any form — different language, different widget
model — so there is nothing to reuse code-wise.

What *is* worth keeping is the list of concrete risk points it was built to
probe, which still apply once the Flutter UI is built:

- **Android IME/text input**: `ime_test.rs` tests Cyrillic and emoji ZWJ
  (zero-width joiner) sequence entry, counting bytes/scalars/graphemes
  separately to make silent mis-handling observable. Flutter's own Android
  text input has historically had its own IME edge cases — re-verify this
  concern against Flutter, don't assume it's solved by switching toolkits.
- **Large-list scroll performance**: `virtual_list.rs` windows a 10,000+ row
  list to stay off-screen-row-count independent. Flutter's
  `ListView.builder` solves this natively, but the *target* (smooth scroll
  at that row count, measured, not assumed) is the part worth keeping.
- **Android lifecycle**: `android.rs` notes that `android_main`-equivalent
  entry points can run more than once per process lifetime (Activity
  recreation on rotation/config change) and that silently keeping stale
  state is a real failure mode to test for, not just a theoretical one.

See `docs/ARCHITECTURE.md` for the current plan.
