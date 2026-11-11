//! Per-room timeline: reading recent messages and sending new ones.

use async_stream::stream;
use futures_util::{Stream, StreamExt};
use matrix_sdk::ruma::RoomId;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk_base::crypto::types::events::UtdCause;
use matrix_sdk_ui::timeline::{EncryptedMessage, MsgLikeKind, RoomExt, Timeline, TimelineItemContent};
use serde::Serialize;
use tokio_util::sync::CancellationToken;

use crate::client::MatrixClient;
use crate::error::ClientError;

/// A single timeline message, flattened for serialization.
///
/// Non-message event types (reactions, state events, redactions, etc.)
/// are skipped when building a snapshot, not surfaced as an error —
/// richer event-type handling is a later phase's concern. A message
/// that couldn't be decrypted *is* included, with `body` set to a
/// human-readable placeholder (see [`as_timeline_message`]) rather than
/// silently dropped — otherwise it just looks like missing history.
#[derive(Debug, Clone, Serialize)]
pub struct TimelineMessage {
    /// The sending user's ID.
    pub sender: String,
    /// The message's plain-text body, or a placeholder like "🔒 Unable
    /// to decrypt message (…)" if it couldn't be decrypted.
    pub body: String,
    /// When the event was sent, in milliseconds since the Unix epoch
    /// (the event's `origin_server_ts`) — flattened to a plain integer
    /// since Dart has no use for `ruma`'s `MilliSecondsSinceUnixEpoch`.
    pub timestamp_ms: u64,
}

/// One-line, human-readable reason a message couldn't be decrypted.
/// Mirrors (a simplified version of) the cause breakdown `git/robrix`'s
/// `event_preview.rs` renders from the same `UtdCause` — not every
/// variant's full explanation, just enough to tell "this will probably
/// never decrypt" apart from "this is likely transient".
fn utd_reason(cause: &UtdCause) -> &'static str {
    match cause {
        UtdCause::SentBeforeWeJoined => "sent before you joined this room",
        UtdCause::VerificationViolation => "sent by a since-unverified user",
        UtdCause::UnsignedDevice => "sending device wasn't signed by its owner",
        UtdCause::UnknownDevice => "sending device's signature wasn't found",
        UtdCause::HistoricalMessageAndBackupIsDisabled
        | UtdCause::HistoricalMessageAndDeviceIsUnverified => {
            "history isn't available on this device"
        }
        UtdCause::WithheldForUnverifiedOrInsecureDevice => {
            "withheld: this device doesn't meet the sender's security requirements"
        }
        UtdCause::WithheldBySender => "withheld by the sender",
        _ => "no key available",
    }
}

/// How many raw timeline events to request per backfill page when a
/// timeline is first opened. See [`MAX_BACKFILL_PAGES`].
const BACKFILL_PAGE_SIZE: u16 = 50;

/// How many backfill pages to request on open before giving up — bounds
/// the request count for a room with a very long history rather than
/// paginating all the way back on every open.
const MAX_BACKFILL_PAGES: usize = 10;

/// A handle to one room's timeline: read recent messages, send new ones.
pub struct RoomTimeline {
    timeline: Timeline,
    /// Lets [`RoomTimeline::stop`] interrupt a live
    /// [`RoomTimeline::subscribe_messages`] loop. See that method's doc
    /// comment for why this exists — without it, switching away from a
    /// quiet room leaks the `Timeline` and its background sync task.
    cancel: CancellationToken,
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
        // Without this, the timeline only shows whatever's already in
        // the local event cache from live sync — no backfill from the
        // server on open, so a room with history not yet seen by this
        // session showed only its single most recent message. A single
        // batch of 20 raw timeline events (state events, reactions, etc.
        // included) often reduced to only one or two actual text
        // messages after filtering, nowhere near "most of the history"
        // other clients show on open — loop until either the start of
        // the room's history is hit or a bounded number of events have
        // been pulled in, rather than stopping after one shallow batch.
        for _ in 0..MAX_BACKFILL_PAGES {
            let hit_start = timeline.paginate_backwards(BACKFILL_PAGE_SIZE).await?;
            if hit_start {
                break;
            }
        }
        Ok(RoomTimeline {
            timeline,
            cancel: CancellationToken::new(),
        })
    }
}

/// Picks out a message's sender/body from a timeline item, or `None` for
/// anything else (reactions, state events, redactions, etc.) — shared
/// between the snapshot and live-subscription reads so both apply the
/// exact same filter.
///
/// A plain-text message and an undecryptable one both produce a
/// `Some(..)` here — the UI shouldn't have to tell "silently skipped"
/// apart from "real content", and a dropped encrypted message looked
/// exactly like missing history before this (found via a real complaint
/// after the fact, not caught by any review).
fn as_timeline_message(item: &matrix_sdk_ui::timeline::TimelineItem) -> Option<TimelineMessage> {
    let event = item.as_event()?;
    let TimelineItemContent::MsgLike(msg_like) = event.content() else {
        return None;
    };
    let body = match &msg_like.kind {
        MsgLikeKind::Message(message) => message.body().to_owned(),
        MsgLikeKind::UnableToDecrypt(EncryptedMessage::MegolmV1AesSha2 { cause, .. }) => {
            format!("🔒 Unable to decrypt message ({})", utd_reason(cause))
        }
        MsgLikeKind::UnableToDecrypt(_) => "🔒 Unable to decrypt message".to_owned(),
        _ => return None,
    };
    Some(TimelineMessage {
        sender: event.sender().to_string(),
        body,
        timestamp_ms: event.timestamp().0.into(),
    })
}

impl RoomTimeline {
    /// Returns a snapshot of the timeline's current plain-text messages.
    ///
    /// Snapshot only — prefer [`RoomTimeline::subscribe_messages`] for a
    /// live-updating view.
    pub async fn messages(&self) -> Vec<TimelineMessage> {
        let (items, _updates) = self.timeline.subscribe().await;
        items.iter().filter_map(|item| as_timeline_message(item)).collect()
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
    ///
    /// The returned stream ends as soon as [`RoomTimeline::stop`] is
    /// called, not just whenever the next diff happens to arrive —
    /// `.subscribe_filter_map`'s own diff stream only reveals a dropped
    /// listener on the *next* item it tries to produce, which on a quiet
    /// room could be never, leaking this `Timeline` (and the background
    /// sync task it holds) for as long as the process runs (found via a
    /// real repro: switching rooms repeatedly eventually made every
    /// further room-open hang). Racing against an explicit
    /// `CancellationToken` instead means the caller controls the
    /// lifetime directly.
    pub async fn subscribe_messages(
        &self,
    ) -> (Vec<TimelineMessage>, impl Stream<Item = Vec<TimelineMessage>> + use<'_>) {
        let (items, diffs) = self
            .timeline
            .subscribe_filter_map(|item| as_timeline_message(&item))
            .await;
        let initial = items.iter().cloned().collect();
        let mut state = items;
        let cancel = self.cancel.clone();
        let snapshots = stream! {
            tokio::pin!(diffs);
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => break,
                    diff = diffs.next() => {
                        let Some(diff) = diff else { break };
                        diff.apply(&mut state);
                        yield state.iter().cloned().collect();
                    }
                }
            }
        };
        (initial, snapshots)
    }

    /// Stops any live [`RoomTimeline::subscribe_messages`] stream for
    /// this handle. Call this before discarding a `RoomTimeline` the
    /// caller is done with (switching rooms, closing the app) — see that
    /// method's doc comment for why this matters.
    pub fn stop(&self) {
        self.cancel.cancel();
    }

    /// Sends a plain-text message to this room.
    pub async fn send_text(&self, body: &str) -> Result<(), ClientError> {
        self.timeline
            .send(RoomMessageEventContent::text_plain(body).into())
            .await?;
        Ok(())
    }
}
