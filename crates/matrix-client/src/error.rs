//! Unified error type for everything `matrix-client` returns.

/// Everything that can go wrong in this crate's public API.
///
/// Wraps the several distinct error types `matrix-sdk` and `matrix-sdk-ui`
/// actually return (login, sync, room lookup, timeline, send) behind one
/// type so callers don't need to match on four different error enums.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    /// A `matrix-sdk` client-level error (login, restore, sending a
    /// message).
    #[error("matrix-sdk error: {0}")]
    Sdk(#[from] matrix_sdk::Error),
    /// Failed to build the `matrix-sdk` `Client` itself (homeserver
    /// discovery, opening the sqlite store, etc.).
    #[error("failed to build client: {0}")]
    ClientBuild(#[from] matrix_sdk::ClientBuildError),
    /// Failed to read from the local `matrix-sdk-sqlite` store.
    #[error("store error: {0}")]
    Store(#[from] matrix_sdk::StoreError),
    /// Failed to parse a room ID string into a real `RoomId`.
    #[error("invalid room id {room_id:?}: {source}")]
    InvalidRoomId {
        /// The string that failed to parse.
        room_id: String,
        /// The underlying parse error.
        source: matrix_sdk::ruma::IdParseError,
    },
    /// The given room ID is not in this client's known room list.
    #[error("room {0} not found")]
    RoomNotFound(String),
    /// A `matrix-sdk-ui` sync-service-level error.
    #[error("sync service error: {0}")]
    SyncService(#[from] matrix_sdk_ui::sync_service::Error),
    /// A `matrix-sdk-ui` timeline-level error (building or paginating).
    #[error("timeline error: {0}")]
    Timeline(#[from] matrix_sdk_ui::timeline::Error),
    /// The client's current session isn't a plain username/password
    /// (`AuthSession::Matrix`) session — e.g. it's an OAuth session. This
    /// crate only ever performs username/password login itself, so this
    /// should not occur against a session this crate created.
    #[error("session is not a plain username/password session")]
    UnsupportedSession,
}
