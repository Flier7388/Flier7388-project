//! Spaces: joined top-level spaces, and the chat rooms nested inside them.
//!
//! Built on `matrix-sdk-ui`'s own `SpaceService` (the same upstream path
//! Element X runs on — see `element-x-android`'s `HomeSpacesView`/
//! `HomeSpacesState`, built on this same Rust SDK underneath) rather than
//! walking `m.space.parent`/`m.space.child` state events by hand. Same
//! "prefer upstream-maintained paths" reasoning as `docs/ARCHITECTURE.md`'s
//! `SyncService`/`Timeline` choice.

use futures_util::{Stream, StreamExt};
use matrix_sdk::RoomState;
use matrix_sdk::ruma::RoomId;
use matrix_sdk_ui::spaces::SpaceRoom;
use matrix_sdk_ui::spaces::room_list::SpaceRoomListPaginationState;
use serde::Serialize;

use crate::client::MatrixClient;
use crate::error::ClientError;
use crate::room::RoomSummary;

/// A joined space's identity, as of the last sync.
#[derive(Debug, Clone, Serialize)]
pub struct SpaceSummary {
    /// The space's room ID, e.g. `!abc123:example.org`.
    pub room_id: String,
    /// The space's computed display name.
    pub display_name: String,
}

/// How many `/hierarchy` pages to fetch before giving up on a single
/// `space_rooms` call — bounds the request count for a pathologically
/// large space rather than paginating forever.
const MAX_HIERARCHY_PAGES: usize = 10;

/// Shared between [`MatrixClient::spaces`] and [`MatrixClient::
/// subscribe_top_level_joined_spaces`] so both apply the same mapping.
fn to_summary(space: &SpaceRoom) -> SpaceSummary {
    SpaceSummary {
        room_id: space.room_id.to_string(),
        display_name: space.display_name.clone(),
    }
}

impl MatrixClient {
    /// Returns a snapshot of the user's top-level joined spaces (spaces
    /// that aren't themselves nested inside another joined space — a
    /// subspace shows up under its parent's [`MatrixClient::space_rooms`]
    /// instead, once that's built out).
    ///
    /// Snapshot only — prefer [`MatrixClient::subscribe_top_level_joined_spaces`]
    /// for a live-updating view.
    pub async fn spaces(&self) -> Vec<SpaceSummary> {
        self.space_service
            .top_level_joined_spaces()
            .await
            .iter()
            .map(to_summary)
            .collect()
    }

    /// Returns the current top-level joined spaces, plus a stream that
    /// yields a fresh full snapshot every time the list changes (space
    /// joined/left) — the reactive counterpart to [`MatrixClient::spaces`].
    pub async fn subscribe_top_level_joined_spaces(
        &self,
    ) -> (Vec<SpaceSummary>, impl Stream<Item = Vec<SpaceSummary>> + use<'_>) {
        let (items, diffs) = self.space_service.subscribe_to_top_level_joined_spaces().await;
        let initial = items.iter().map(to_summary).collect();
        let mut state = items;
        let snapshots = diffs.map(move |batch| {
            for diff in batch {
                diff.apply(&mut state);
            }
            state.iter().map(to_summary).collect()
        });
        (initial, snapshots)
    }

    /// Returns the chat rooms directly inside the given space.
    ///
    /// Only rooms the user has actually joined are included — a space's
    /// `/hierarchy` can list rooms the user hasn't joined yet, but
    /// [`MatrixClient::timeline_for`] can't open those. Nested subspaces
    /// are excluded too: they aren't chat rooms, and subspace navigation
    /// isn't built yet (same reasoning as [`MatrixClient::rooms`]
    /// excluding top-level spaces).
    pub async fn space_rooms(&self, space_id: &str) -> Result<Vec<RoomSummary>, ClientError> {
        let space_id = RoomId::parse(space_id).map_err(|source| ClientError::InvalidRoomId {
            room_id: space_id.to_owned(),
            source,
        })?;
        let room_list = self.space_service.space_room_list(space_id).await;

        for _ in 0..MAX_HIERARCHY_PAGES {
            room_list.paginate().await?;
            if let SpaceRoomListPaginationState::Idle { end_reached: true } =
                room_list.pagination_state()
            {
                break;
            }
        }

        Ok(room_list
            .rooms()
            .await
            .into_iter()
            .filter(|room| room.room_type.is_none() && room.state == Some(RoomState::Joined))
            .map(|room| RoomSummary {
                room_id: room.room_id.to_string(),
                display_name: room.display_name,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use matrix_sdk::ruma::{RoomVersionId, room_id};
    use matrix_sdk::test_utils::mocks::MatrixMockServer;
    use matrix_sdk_test::JoinedRoomBuilder;
    use matrix_sdk_test::event_factory::EventFactory;

    use crate::client::test_support::test_client;

    #[tokio::test]
    async fn spaces_lists_top_level_joined_spaces() {
        let server = MatrixMockServer::new().await;
        server.mock_room_state_encryption().plain().mount().await;

        let client = test_client(&server).await;
        let user_id = client.client.user_id().unwrap();
        let factory = EventFactory::new();

        let space_room_id = room_id!("!space:example.org");
        server
            .sync_room(
                &client.client,
                JoinedRoomBuilder::new(space_room_id).add_state_event(
                    factory.create(user_id, RoomVersionId::V1).with_space_type(),
                ),
            )
            .await;

        let spaces = client.spaces().await;

        assert_eq!(spaces.len(), 1);
        assert_eq!(spaces[0].room_id, space_room_id.to_string());
    }

    // Regression test matching `space_rooms`'s own doc comment: a room
    // listed in a space's `/hierarchy` that the user hasn't joined must
    // not show up, since there's no joined room behind it to open a
    // timeline on.
    #[tokio::test]
    async fn space_rooms_excludes_unjoined_children() {
        let server = MatrixMockServer::new().await;
        server.mock_room_state_encryption().plain().mount().await;

        let client = test_client(&server).await;
        let user_id = client.client.user_id().unwrap();
        let factory = EventFactory::new();

        let space_id = room_id!("!space:example.org");
        let joined_child_id = room_id!("!joined_child:example.org");
        let unjoined_child_id = room_id!("!unjoined_child:example.org");

        server
            .sync_room(
                &client.client,
                JoinedRoomBuilder::new(space_id).add_state_event(
                    factory.create(user_id, RoomVersionId::V1).with_space_type(),
                ),
            )
            .await;
        server
            .sync_room(
                &client.client,
                JoinedRoomBuilder::new(joined_child_id)
                    .add_state_event(factory.create(user_id, RoomVersionId::V1)),
            )
            .await;

        server
            .mock_get_hierarchy()
            .ok_with_room_ids(vec![space_id, joined_child_id, unjoined_child_id])
            .mount()
            .await;

        let rooms = client.space_rooms(space_id.as_str()).await.unwrap();
        let ids: Vec<_> = rooms.iter().map(|r| r.room_id.as_str()).collect();

        assert_eq!(ids, vec![joined_child_id.as_str()]);
    }
}
