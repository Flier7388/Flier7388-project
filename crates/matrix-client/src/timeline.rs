//! Per-room timeline: reading recent messages and sending new ones.

use futures_util::{Stream, StreamExt};
use matrix_sdk::ruma::RoomId;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk_ui::timeline::{MsgLikeKind, RoomExt, Timeline, TimelineItemContent};
use serde::Serialize;

use crate::client::MatrixClient;
use crate::error::ClientError;

/// A single plain-text timeline message, flattened for serialization.
///
/// Non-text event types (reactions, state events, redactions, etc.) are
/// skipped when building a snapshot, not surfaced as an error — richer
/// event-type handling is a later phase's concern.
#[derive(Debug, Clone, Serialize)]
pub struct TimelineMessage {
    /// The sending user's ID.
    pub sender: String,
    /// The message's plain-text body.
    pub body: String,
}

/// A handle to one room's timeline: read recent messages, send new ones.
pub struct RoomTimeline {
    timeline: Timeline,
}

impl MatrixClient {
    /// Builds a timeline handle for the given room ID.
    pub async fn timeline_for(&self, room_id: &str) -> Result<RoomTimeline, ClientError> {
        let room_id = RoomId::parse(room_id).map_err(|source| ClientError::InvalidRoomId {
            room_id: room_id.to_owned(),
            source,
        })?;
        let room = self
            .client
            .get_room(&room_id)
            .ok_or_else(|| ClientError::RoomNotFound(room_id.to_string()))?;
        let timeline = room.timeline_builder().build().await?;
        Ok(RoomTimeline { timeline })
    }
}

/// Picks out a plain-text message's sender/body from a timeline item, or
/// `None` for anything else (reactions, state events, redactions, etc.) —
/// shared between the snapshot and live-subscription reads so both apply
/// the exact same filter.
fn as_text_message(item: &matrix_sdk_ui::timeline::TimelineItem) -> Option<TimelineMessage> {
    let event = item.as_event()?;
    let TimelineItemContent::MsgLike(msg_like) = event.content() else {
        return None;
    };
    let MsgLikeKind::Message(message) = &msg_like.kind else {
        return None;
    };
    Some(TimelineMessage {
        sender: event.sender().to_string(),
        body: message.body().to_owned(),
    })
}

impl RoomTimeline {
    /// Returns a snapshot of the timeline's current plain-text messages.
    ///
    /// Snapshot only — prefer [`RoomTimeline::subscribe_messages`] for a
    /// live-updating view.
    pub async fn messages(&self) -> Vec<TimelineMessage> {
        let (items, _updates) = self.timeline.subscribe().await;
        items.iter().filter_map(|item| as_text_message(item)).collect()
    }

    /// Returns the timeline's current plain-text messages, plus a stream
    /// that yields a fresh full snapshot every time the timeline changes
    /// (new message, edit, local echo settling, etc.).
    ///
    /// Built on `Timeline::subscribe_filter_map` rather than the raw
    /// `subscribe()` used by [`RoomTimeline::messages`]: it does the
    /// text-message filtering *before* diffing, so the diff stream only
    /// ever reports changes to messages the caller actually cares about.
    /// Each [`eyeball_im::VectorDiff`] batch element is applied to a
    /// local copy of the list and re-flattened to a plain `Vec` — a full
    /// snapshot per update, not an incremental diff, since the Flutter
    /// side has no equivalent of `imbl::Vector` to apply diffs against
    /// and a per-room timeline is small enough that this is cheap.
    pub async fn subscribe_messages(
        &self,
    ) -> (Vec<TimelineMessage>, impl Stream<Item = Vec<TimelineMessage>> + use<'_>) {
        let (items, diffs) = self
            .timeline
            .subscribe_filter_map(|item| as_text_message(&item))
            .await;
        let initial = items.iter().cloned().collect();
        let mut state = items;
        let snapshots = diffs.map(move |diff| {
            diff.apply(&mut state);
            state.iter().cloned().collect()
        });
        (initial, snapshots)
    }

    /// Sends a plain-text message to this room.
    pub async fn send_text(&self, body: &str) -> Result<(), ClientError> {
        self.timeline
            .send(RoomMessageEventContent::text_plain(body).into())
            .await?;
        Ok(())
    }
}
