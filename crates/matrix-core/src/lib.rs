#![forbid(unsafe_code)]
//! Core, SDK-independent domain types for the Matrix client.
//!
//! This crate intentionally has **no** dependency on `matrix-sdk`. It is
//! the home for small, pure domain logic (identifier parsing, protocol
//! value objects) that every other crate in the workspace — including
//! the real SDK integration landing in a later phase — can share
//! without pulling in networking, storage, or cryptography code.
//!
//! Phase 1 ships a single module, [`identifier`], as a minimal but real
//! piece of behavior to validate the crate's lint and test setup.

pub mod identifier;

pub use identifier::{Kind, MatrixId, ParseError};
