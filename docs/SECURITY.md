# Security

The project owner has flagged that this project's final work will get a
strict, dedicated security review. The notes below are the policy that
review will be checked against — keep this file in sync with what's
actually true in the code, not what's aspirational.

## E2EE and storage: fully delegated, not reimplemented
No cryptography or message/session store exists in this repo. When
`matrix-rust-sdk` integration lands, Olm/Megolm, device verification, and
the encrypted SQLite message/crypto store are all `matrix-sdk-sqlite`'s
responsibility, unmodified. This repo's own storage surface stays limited
to `matrix-storage`'s media-cache retention policy
(`crates/matrix-storage/src/retention.rs`) — never message content or key
material.

## `unsafe` policy
Workspace-wide `unsafe_code = "deny"` (see root `Cargo.toml`). `matrix-core`,
`matrix-storage`, `feedback-dto`, and `matrix-client` additionally carry
`#![forbid(unsafe_code)]` — no exception is possible in those crates
without editing the crate root itself. The day the `flutter_rust_bridge`
bridge crate (`flutter/rust`, outside this workspace) is scaffolded, expect
its FFI-facing surface to need a narrow, reviewed `#[allow(unsafe_code)]`
with a comment justifying it — not a blanket opt-out, and not anywhere in
the crates listed above.

## Rust↔Flutter bridge: why `flutter_rust_bridge` over `matrix-sdk-ffi`/UniFFI
Decided 2026-11-06. `matrix-sdk-ffi` (UniFFI-based, what Element X ships on
Android/iOS) was the first candidate, since reusing upstream bindings means
less custom glue code to audit. Rejected because its only maintained Dart
bindgen, `uniffi-bindgen-dart` (github.com/nchapman/uniffi-bindgen-dart),
is a single-maintainer project created March 2026 (latest release `0.2.0`,
2026-10-15) with no confirmed production Matrix-client usage — not an
acceptable foundation given this project's strict review bar. We instead
write our own thin Rust wrapper (`crates/matrix-client`, directly on
`matrix-sdk`) and bridge it with `flutter_rust_bridge` (stable, widely
adopted). This means **we own and must audit the bridge crate's surface
ourselves** — it is not upstream code, unlike the UniFFI path would have
been. Keep that crate (`flutter/rust`/`app_bridge`) thin and mechanical for
exactly this reason: the thinner it is, the less of it there is to get
wrong.

## Dependency supply chain
`cargo deny check` (licenses/bans/sources/advisories) and `cargo audit`
both run in CI (`.github/workflows/ci.yml`) against the committed
`Cargo.lock`. `deny.toml` currently allows **no** git dependencies
(`sources.allow-git = []`) and denies unpinned wildcard version
requirements (`bans.wildcards = "deny"`) — the moment either of those needs
to change for a real reason, that change must come with a comment in
`deny.toml` explaining exactly why, the same way the two exceptions below
are documented.

Adding `matrix-sdk` 0.19.1 (`crates/matrix-client`) introduced three
transitive, upstream-only exceptions, verified via `cargo deny check`'s own
dependency trace — none is a dependency we chose:
- **License**: `xxhash-rust` (BSL-1.0, via `growable-bloom-filter` via
  `matrix-sdk-base`) — added to `licenses.allow` (2026-11-06).
- **Advisory**: `RUSTSEC-2026-0319` (`anymap2` unmaintained, pulled
  directly by `matrix-sdk`, no safe upgrade available) — added to
  `advisories.ignore` (2026-11-06).
- **Advisory**: `RUSTSEC-2026-0459` / CVE-2026-119375 (uninitialized
  page-cache memory disclosure in SQLite versions bundled by
  `libsqlite3-sys` <0.39.0, requiring a DB + sibling `-wal` pair with
  mismatched page sizes). Pulled in by switching `matrix-sdk`'s sqlite
  feature from `sqlite` (links the system lib) to `bundled-sqlite`
  (compiles SQLite from source) — needed because Windows has no system
  `sqlite3.lib` to link against (`flutter build windows` failed with
  `LNK1181` otherwise). `matrix-sdk-sqlite` 0.19.1 hard-pins
  `rusqlite = "0.40.2"`, which bundles the vulnerable `libsqlite3-sys`
  0.38.2; no upgrade is possible without an upstream bump. Added to
  `advisories.ignore` (2026-11-10) — the exposure is low for this app
  specifically, since the only sqlite files it ever opens are its own
  local `matrix-sdk-sqlite` store at a fixed, user-owned path, never an
  externally supplied/untrusted DB+WAL pair (the advisory's actual
  attack vector).

Re-verify all three on every `matrix-sdk` version bump.

## Matrix-sdk / server compatibility (forward-looking, not yet integrated)
See `docs/ARCHITECTURE.md`'s matching section — the compatibility checks
listed there (sliding sync/MSC support, `matrix-sdk-sqlite` schema
forward-compatibility, E2EE device-verification flows) are security-
relevant, not just functional: a homeserver mismatch in any of them can
silently degrade E2EE guarantees rather than just break a feature visibly.

## Owner-facing report/feedback feature
Not designed yet. Per the project owner: a report (scored 0–10, with notes
on what to fix/add/remove) delivered as a Matrix message to a specific
user, triggered from a button in the app. Whatever this becomes, treat the
"which Matrix account/room it sends to" configuration as sensitive
(credentials or a hardcoded recipient ID are both review-worthy) and do not
let it reuse or leak the active user's own E2EE session state.
