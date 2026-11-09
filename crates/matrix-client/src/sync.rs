//! Starting and stopping sync for a [`MatrixClient`].

use crate::client::MatrixClient;

impl MatrixClient {
    /// Starts the sync service. Safe to call more than once — per
    /// `matrix-sdk-ui`'s own docs, calling it while already running is a
    /// no-op.
    pub async fn start_sync(&self) {
        self.sync_service.start().await;
    }

    /// Stops the sync service. Call this on a clean shutdown.
    pub async fn stop_sync(&self) {
        self.sync_service.stop().await;
    }
}
