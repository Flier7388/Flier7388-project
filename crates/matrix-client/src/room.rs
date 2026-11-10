//! A flat, serializable snapshot of the joined room list.

use futures_util::{Stream, StreamExt};
use matrix_sdk::{Room, RoomState};
use matrix_sdk_ui::eyeball_im::Vector;
use serde::Serialize;

use crate::client::MatrixClient;
use crate::error::ClientError;

/// A joined room's identity and display name, as of the last sync.
///
/// Deliberately flat and `Serialize`-friendly: this is the shape that will
/// eventually cross the Flutter bridge, so it stays simple on purpose
/// rather than exposing `matrix-sdk`'s richer `Room` type directly.
#[derive(Debug, Clone, Serialize)]
pub struct RoomSummary {
    /// The room's ID, e.g. `!abc123:example.org`.
    pub room_id: String,
    /// The room's computed display name.
    pub display_name: String,
}

impl MatrixClient {
    /// Returns a snapshot of all currently-joined chat rooms.
    ///
    /// Excludes spaces (`m.space` rooms) — a space isn't a chat room, it's
    /// a container that groups other rooms, and has no timeline to read or
    /// send into. Listing it alongside real rooms here would let the UI
    /// open it like a chat. Space support (listing a space's own child
    /// rooms, nested navigation) is a separate, not-yet-built feature.
    ///
    /// This is a snapshot, not a live-updating list — the sync service
    /// (via [`MatrixClient::start_sync`]) keeps the underlying store
    /// current, but each call here re-reads it fresh rather than
    /// subscribing to updates. Reactive updates are deferred to whichever
    /// phase wires this into the Flutter bridge.
    pub async fn rooms(&self) -> Result<Vec<RoomSummary>, ClientError> {
        let mut summaries = Vec::new();
        for room in self.client.rooms() {
            if room.state() != RoomState::Joined || room.is_space() {
                continue;
            }
            let display_name = room.display_name().await?.to_string();
            summaries.push(RoomSummary {
                room_id: room.room_id().to_string(),
                display_name,
            });
        }
        Ok(summaries)
    }

    /// Returns the current joined-room list, plus a stream that yields a
    /// fresh full snapshot every time it changes (join, leave, rename,
    /// etc.) — the reactive counterpart to [`MatrixClient::rooms`].
    ///
    /// Built on `matrix_sdk::Client::rooms_stream()`, the same low-level
    /// primitive `matrix-sdk-ui`'s own `RoomListService::entries()` uses
    /// internally — `RoomListService`'s own `entries_with_dynamic_adapters`
    /// was deliberately not used here: it's built for Element X's sorted,
    /// paginated, multi-filter room list UI, far more machinery than this
    /// app's flat unsorted list needs.
    pub async fn subscribe_rooms(
        &self,
    ) -> (Vec<RoomSummary>, impl Stream<Item = Vec<RoomSummary>> + use<'_>) {
        let (items, diffs) = self.client.rooms_stream();
        let initial = summarize_rooms(items.clone()).await;
        let mut state = items;
        let snapshots = diffs.then(move |batch| {
            for diff in batch {
                diff.apply(&mut state);
            }
            let snapshot = state.clone();
            async move { summarize_rooms(snapshot).await }
        });
        (initial, snapshots)
    }

    /// Returns a snapshot of all currently-joined direct-message rooms:
    /// 1:1 rooms flagged via the `m.direct` account data (`Room::is_dm`),
    /// not just any 2-member room. This is the Direct Messages tab's data
    /// source, as opposed to [`MatrixClient::rooms`]'s full joined list
    /// (which includes DMs alongside everything else).
    pub async fn direct_rooms(&self) -> Result<Vec<RoomSummary>, ClientError> {
        let mut summaries = Vec::new();
        for room in self.client.rooms() {
            if room.state() != RoomState::Joined || !room.is_dm() {
                continue;
            }
            let display_name = room.display_name().await?.to_string();
            summaries.push(RoomSummary {
                room_id: room.room_id().to_string(),
                display_name,
            });
        }
        Ok(summaries)
    }

    /// The DM-filtered counterpart to [`MatrixClient::subscribe_rooms`] —
    /// same live-updating snapshot stream, restricted to `Room::is_dm`
    /// rooms.
    pub async fn subscribe_direct_rooms(
        &self,
    ) -> (Vec<RoomSummary>, impl Stream<Item = Vec<RoomSummary>> + use<'_>) {
        let (items, diffs) = self.client.rooms_stream();
        let initial = summarize_direct_rooms(items.clone()).await;
        let mut state = items;
        let snapshots = diffs.then(move |batch| {
            for diff in batch {
                diff.apply(&mut state);
            }
            let snapshot = state.clone();
            async move { summarize_direct_rooms(snapshot).await }
        });
        (initial, snapshots)
    }
}

/// Builds a [`RoomSummary`] for every joined chat room in `rooms` (same
/// space-exclusion rule as [`MatrixClient::rooms`]). A display-name
/// lookup failure is logged and the room is skipped rather than failing
/// the whole snapshot — a transient per-room issue on a live subscription
/// shouldn't blank out every other room until the next change arrives.
async fn summarize_rooms(rooms: Vector<Room>) -> Vec<RoomSummary> {
    let mut summaries = Vec::new();
    for room in rooms.iter() {
        if room.state() != RoomState::Joined || room.is_space() {
            continue;
        }
        match room.display_name().await {
            Ok(display_name) => summaries.push(RoomSummary {
                room_id: room.room_id().to_string(),
                display_name: display_name.to_string(),
            }),
            Err(error) => {
                tracing::warn!(
                    room_id = %room.room_id(),
                    %error,
                    "failed to compute room display name, skipping"
                );
            }
        }
    }
    summaries
}

/// Builds a [`RoomSummary`] for every joined DM room in `rooms` (same
/// `Room::is_dm` filter as [`MatrixClient::direct_rooms`]). Same
/// skip-on-error behavior as [`summarize_rooms`].
async fn summarize_direct_rooms(rooms: Vector<Room>) -> Vec<RoomSummary> {
    let mut summaries = Vec::new();
    for room in rooms.iter() {
        if room.state() != RoomState::Joined || !room.is_dm() {
            continue;
        }
        match room.display_name().await {
            Ok(display_name) => summaries.push(RoomSummary {
                room_id: room.room_id().to_string(),
                display_name: display_name.to_string(),
            }),
            Err(error) => {
                tracing::warn!(
                    room_id = %room.room_id(),
                    %error,
                    "failed to compute DM room display name, skipping"
                );
            }
        }
    }
    summaries
}

#[cfg(test)]
mod tests {
    use matrix_sdk::ruma::{RoomVersionId, room_id, user_id};
    use matrix_sdk::test_utils::mocks::MatrixMockServer;
    use matrix_sdk_test::JoinedRoomBuilder;
    use matrix_sdk_test::event_factory::EventFactory;

    use crate::client::test_support::test_client;

    // Regression test for the bug the project owner found by hand: a space
    // showing up in the room list as if it were an openable chat (and
    // letting messages be sent into it).
    #[tokio::test]
    async fn rooms_excludes_spaces() {
        let server = MatrixMockServer::new().await;
        server.mock_room_state_encryption().plain().mount().await;

        let client = test_client(&server).await;
        let user_id = client.client.user_id().unwrap();
        let factory = EventFactory::new();

        let plain_room_id = room_id!("!plain:example.org");
        let space_room_id = room_id!("!space:example.org");

        server
            .sync_room(
                &client.client,
                JoinedRoomBuilder::new(plain_room_id)
                    .add_state_event(factory.create(user_id, RoomVersionId::V1)),
            )
            .await;
        server
            .sync_room(
                &client.client,
                JoinedRoomBuilder::new(space_room_id).add_state_event(
                    factory.create(user_id, RoomVersionId::V1).with_space_type(),
                ),
            )
            .await;

        let rooms = client.rooms().await.unwrap();
        let ids: Vec<_> = rooms.iter().map(|r| r.room_id.as_str()).collect();

        assert_eq!(ids, vec![plain_room_id.as_str()]);
    }

    // `direct_rooms` must only pick up rooms flagged via the `m.direct`
    // account data (what `Room::is_dm` checks), not every joined room.
    #[tokio::test]
    async fn direct_rooms_includes_only_dm_flagged_rooms() {
        let server = MatrixMockServer::new().await;
        server.mock_room_state_encryption().plain().mount().await;

        let client = test_client(&server).await;
        let user_id = client.client.user_id().unwrap();
        let factory = EventFactory::new();
        let other_user = user_id!("@other:example.org");

        let plain_room_id = room_id!("!plain:example.org");
        let dm_room_id = room_id!("!dm:example.org");

        server
            .mock_sync()
            .ok_and_run(&client.client, |builder| {
                builder
                    .add_joined_room(
                        JoinedRoomBuilder::new(plain_room_id)
                            .add_state_event(factory.create(user_id, RoomVersionId::V1)),
                    )
                    .add_joined_room(
                        JoinedRoomBuilder::new(dm_room_id)
                            .add_state_event(factory.create(user_id, RoomVersionId::V1)),
                    )
                    .add_global_account_data(
                        factory
                            .direct()
                            .add_user(other_user.to_owned().into(), dm_room_id),
                    );
            })
            .await;

        let rooms = client.direct_rooms().await.unwrap();
        let ids: Vec<_> = rooms.iter().map(|r| r.room_id.as_str()).collect();

        assert_eq!(ids, vec![dm_room_id.as_str()]);
    }
}
