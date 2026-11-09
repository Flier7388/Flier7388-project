#![forbid(unsafe_code)]
//! Disk-backed media cache policy for the Matrix client.
//!
//! This crate owns exactly one responsibility: deciding which cached
//! media blobs to evict to respect a disk budget. It does **not**
//! duplicate `matrix-sdk`'s own message/state/E2EE store — that remains
//! entirely delegated to `matrix-sdk-sqlite`. This crate is only for the
//! separate on-disk cache of downloaded media (images, avatars, files),
//! which has no equivalent in the SDK's own storage.
//!
//! Phase 1 ships only the pure eviction-decision logic in
//! [`retention`]; the filesystem-backed cache implementation that calls
//! it lands once the SDK integration phase defines how media is fetched
//! and keyed.

pub mod retention;

pub use retention::{CacheEntry, RetentionPolicy};
