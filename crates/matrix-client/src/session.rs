//! Session persistence: a flat, serializable snapshot of a logged-in
//! session, independent of `matrix-sdk`'s own [`AuthSession`] shape.

use matrix_sdk::SessionMeta;
use matrix_sdk::authentication::matrix::MatrixSession;
use matrix_sdk::authentication::{AuthSession, SessionTokens};
use matrix_sdk::ruma::{OwnedDeviceId, OwnedUserId};
use serde::{Deserialize, Serialize};

use crate::error::ClientError;

/// A logged-in session's identity and tokens, flattened for serialization.
///
/// This crate never decides *where* this gets written to disk, or how it's
/// protected — it's a bearer access token, so that's the caller's
/// responsibility (see `docs/ARCHITECTURE.md`). Mirrors the field shape of
/// `matrix-sdk`'s own [`MatrixSession`], confirmed against real production
/// usage in `iamb`'s and `robrix`'s session-persistence code (both store
/// the same flat fields rather than serializing `AuthSession` directly).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// The homeserver URL this session was established against.
    pub homeserver_url: String,
    /// The fully-qualified user ID, e.g. `@test:example.org`.
    pub user_id: OwnedUserId,
    /// This device's ID, as assigned by the homeserver at login.
    pub device_id: OwnedDeviceId,
    /// The current access token.
    pub access_token: String,
    /// The current refresh token, if the homeserver issued one.
    pub refresh_token: Option<String>,
}

impl Session {
    pub(crate) fn from_auth_session(
        homeserver_url: String,
        auth_session: AuthSession,
    ) -> Result<Self, ClientError> {
        let AuthSession::Matrix(matrix_session) = auth_session else {
            return Err(ClientError::UnsupportedSession);
        };
        let MatrixSession { meta, tokens } = matrix_session;
        Ok(Self {
            homeserver_url,
            user_id: meta.user_id,
            device_id: meta.device_id,
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
        })
    }

    pub(crate) fn into_auth_session(self) -> AuthSession {
        AuthSession::Matrix(MatrixSession {
            meta: SessionMeta {
                user_id: self.user_id,
                device_id: self.device_id,
            },
            tokens: SessionTokens {
                access_token: self.access_token,
                refresh_token: self.refresh_token,
            },
        })
    }
}
