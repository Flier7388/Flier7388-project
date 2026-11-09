#![forbid(unsafe_code)]
// `Timeline::send`'s future type nests deep enough (matrix-sdk-ui's own
// internal combinators) to blow the default query-depth limit during
// layout computation — raising it is the compiler's own suggested fix,
// verified locally 2026-11-06, not a sign of a real problem in our code.
#![recursion_limit = "256"]
//! Pure-Rust Matrix client logic, built on `matrix-sdk` and
//! `matrix-sdk-ui`.
//!
//! This crate owns session lifecycle, sync, and room/timeline/send APIs.
//! It deliberately has **no** dependency on `flutter_rust_bridge` or any
//! other FFI/bridge tooling — see `flutter/rust` for the thin bridge
//! crate that wraps this one for Dart consumption. Keeping the split
//! means this crate's logic stays exercisable with plain `cargo test`,
//! independent of whichever bridge tool is wired up on the Flutter side.

mod client;
mod error;
mod room;
mod session;
mod space;
mod sync;
mod timeline;

pub use client::MatrixClient;
pub use error::ClientError;
pub use room::RoomSummary;
pub use session::Session;
pub use space::SpaceSummary;
pub use timeline::{RoomTimeline, TimelineMessage};
