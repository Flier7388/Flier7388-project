//! The core [`MatrixClient`] type: owns a logged-in `matrix-sdk` [`Client`]
//! and the `matrix-sdk-ui` [`SyncService`] that drives sync for it.

use std::path::Path;

use matrix_sdk::Client;
use matrix_sdk_ui::spaces::SpaceService;
use matrix_sdk_ui::sync_service::SyncService;

use crate::error::ClientError;
use crate::session::Session;

/// A logged-in Matrix client, wrapping `matrix-sdk`'s [`Client`] and the
/// `matrix-sdk-ui` [`SyncService`]/[`SpaceService`] used to drive sync,
/// room list, space, and timeline access.
pub struct MatrixClient {
    pub(crate) client: Client,
    pub(crate) sync_service: SyncService,
    pub(crate) space_service: SpaceService,
    pub(crate) homeserver_url: String,
}

async fn build_client(homeserver_url: &str, store_path: &Path) -> Result<Client, ClientError> {
    Ok(Client::builder()
        .homeserver_url(homeserver_url)
        .sqlite_store(store_path, None)
        .build()
        .await?)
}

impl MatrixClient {
    /// Logs in with a username and password against `homeserver_url`,
    /// using `store_path` as the on-disk `matrix-sdk-sqlite` store
    /// directory (room state, E2EE keys, etc. — see `matrix-sdk-sqlite`'s
    /// own docs for what lives there).
    pub async fn login(
        homeserver_url: &str,
        user: &str,
        password: &str,
        store_path: &Path,
    ) -> Result<Self, ClientError> {
        let client = build_client(homeserver_url, store_path).await?;
        client
            .matrix_auth()
            .login_username(user, password)
            .initial_device_display_name("matrix-client")
            .send()
            .await?;
        let sync_service = SyncService::builder(client.clone()).build().await?;
        let space_service = SpaceService::new(client.clone()).await;
        Ok(Self {
            client,
            sync_service,
            space_service,
            homeserver_url: homeserver_url.to_owned(),
        })
    }

    /// Rebuilds a client from a previously-saved [`Session`], without
    /// logging in again.
    pub async fn restore(store_path: &Path, session: Session) -> Result<Self, ClientError> {
        let homeserver_url = session.homeserver_url.clone();
        let client = build_client(&homeserver_url, store_path).await?;
        client.restore_session(session.into_auth_session()).await?;
        let sync_service = SyncService::builder(client.clone()).build().await?;
        let space_service = SpaceService::new(client.clone()).await;
        Ok(Self {
            client,
            sync_service,
            space_service,
            homeserver_url,
        })
    }

    /// Returns the current session for persistence, or `None` if somehow
    /// not logged in (shouldn't happen for a `MatrixClient` that exists at
    /// all, since both constructors require a successful login/restore).
    #[must_use]
    pub fn session(&self) -> Option<Session> {
        let auth_session = self.client.session()?;
        Session::from_auth_session(self.homeserver_url.clone(), auth_session).ok()
    }

    /// Returns the logged-in account's own display name, or `None` if
    /// none is set (the UI falls back to the user ID in that case, same
    /// as everywhere else display names are shown).
    pub async fn account_display_name(&self) -> Result<Option<String>, ClientError> {
        Ok(self.client.account().get_display_name().await?)
    }
}

/// Test-only helper shared by `room.rs`/`space.rs`'s mock-server tests.
///
/// Builds a `MatrixClient` straight from a `matrix-sdk` test client rather
/// than through [`MatrixClient::login`] — [`MatrixMockServer`]'s client is
/// already "logged in" to a synthetic test user, so there's no real
/// homeserver login flow to exercise here. Lives in this module (not a
/// `tests/` integration test) because `MatrixClient`'s fields are
/// `pub(crate)`, not public.
#[cfg(test)]
pub(crate) mod test_support {
    use matrix_sdk::test_utils::mocks::MatrixMockServer;
    use matrix_sdk_ui::spaces::SpaceService;
    use matrix_sdk_ui::sync_service::SyncService;

    use super::MatrixClient;

    pub(crate) async fn test_client(server: &MatrixMockServer) -> MatrixClient {
        let client = server.client_builder().build().await;
        let sync_service = SyncService::builder(client.clone())
            .build()
            .await
            .expect("building a SyncService against a mock client never fails");
        let space_service = SpaceService::new(client.clone()).await;
        MatrixClient {
            client,
            sync_service,
            space_service,
            homeserver_url: server.uri(),
        }
    }
}
