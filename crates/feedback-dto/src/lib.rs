#![forbid(unsafe_code)]
//! Wire format for in-app user feedback reports.
//!
//! This crate defines the data-transfer object shared between the
//! client UI and whatever feedback-collection backend is chosen later
//! (e.g. a GitHub issue webhook). It owns serialization and
//! input-validation only — no HTTP client, no storage.

pub mod report;

pub use report::{FeedbackReport, MAX_MESSAGE_BYTES, Severity, ValidationError};
