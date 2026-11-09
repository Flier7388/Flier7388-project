//! The [`FeedbackReport`] type and its validation rules.

use serde::{Deserialize, Serialize};

/// Upper bound on [`FeedbackReport::message`] length, in UTF-8 bytes.
///
/// Chosen to comfortably fit a detailed bug report while bounding the
/// size of a single submission against a future backend.
pub const MAX_MESSAGE_BYTES: usize = 8_192;

/// How severe the user judged the issue they are reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Cosmetic issue; does not block usage.
    Low,
    /// Impacts usage but has a workaround.
    Medium,
    /// Blocks a core workflow (sending/receiving messages, login, E2EE).
    Critical,
}

/// A validated, serializable in-app feedback report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedbackReport {
    severity: Severity,
    message: String,
    client_version: String,
}

impl FeedbackReport {
    /// Validates and builds a [`FeedbackReport`].
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError`] if `message` is empty (or
    /// whitespace-only) or exceeds [`MAX_MESSAGE_BYTES`].
    pub fn new(
        severity: Severity,
        message: impl Into<String>,
        client_version: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        let message = message.into();
        if message.trim().is_empty() {
            return Err(ValidationError::EmptyMessage);
        }
        if message.len() > MAX_MESSAGE_BYTES {
            return Err(ValidationError::MessageTooLong {
                len: message.len(),
                max: MAX_MESSAGE_BYTES,
            });
        }

        Ok(Self {
            severity,
            message,
            client_version: client_version.into(),
        })
    }

    /// The reported severity.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        self.severity
    }

    /// The free-text report body.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The reporting client's version string.
    #[must_use]
    pub fn client_version(&self) -> &str {
        &self.client_version
    }
}

/// Why [`FeedbackReport::new`] rejected its input.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ValidationError {
    /// `message` was empty or whitespace-only.
    #[error("feedback message is empty")]
    EmptyMessage,
    /// `message` exceeded [`MAX_MESSAGE_BYTES`].
    #[error("feedback message is {len} bytes, exceeding the {max}-byte limit")]
    MessageTooLong {
        /// The actual encoded length of the rejected message, in bytes.
        len: usize,
        /// The limit that was exceeded, i.e. [`MAX_MESSAGE_BYTES`].
        max: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_json() {
        let report = FeedbackReport::new(
            Severity::Critical,
            "E2EE verification never completes",
            "0.1.0",
        )
        .expect("valid report");

        let json = serde_json::to_string(&report).expect("serializable");
        let decoded: FeedbackReport = serde_json::from_str(&json).expect("deserializable");

        assert_eq!(report, decoded);
        assert!(json.contains("\"critical\""));
    }

    #[test]
    fn rejects_empty_message() {
        assert_eq!(
            FeedbackReport::new(Severity::Low, "   ", "0.1.0"),
            Err(ValidationError::EmptyMessage)
        );
    }

    #[test]
    fn rejects_oversized_message() {
        let too_long = "x".repeat(MAX_MESSAGE_BYTES + 1);
        let err = FeedbackReport::new(Severity::Low, too_long, "0.1.0").unwrap_err();
        assert!(matches!(err, ValidationError::MessageTooLong { .. }));
    }
}
