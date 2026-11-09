//! A room's joined member list, grouped by admin/regular status.

use matrix_sdk::RoomMemberships;
use matrix_sdk::room::RoomMemberRole;
use matrix_sdk::ruma::RoomId;
use serde::Serialize;

use crate::client::MatrixClient;
use crate::error::ClientError;

/// A joined room member's identity, as of the last sync.
#[derive(Debug, Clone, Serialize)]
pub struct RoomMemberSummary {
    /// The member's fully-qualified user ID.
    pub user_id: String,
    /// The member's display name, falling back to their user ID's
    /// localpart if none is set (`matrix-sdk`'s own `RoomMember::name()`
    /// behavior).
    pub display_name: String,
    /// `true` for moderators, administrators, and creators (power level
    /// 50+), matching `design/index.html`'s two-group "Admins"/"Members"
    /// member-drawer split — finer-grained role distinctions aren't
    /// surfaced yet since the design only has two buckets.
    pub is_admin: bool,
}

impl MatrixClient {
    /// Returns the given room's joined members.
    pub async fn room_members(&self, room_id: &str) -> Result<Vec<RoomMemberSummary>, ClientError> {
        let room_id = RoomId::parse(room_id).map_err(|source| ClientError::InvalidRoomId {
            room_id: room_id.to_owned(),
            source,
        })?;
        let room = self
            .client
            .get_room(&room_id)
            .ok_or_else(|| ClientError::RoomNotFound(room_id.to_string()))?;
        let members = room.members(RoomMemberships::JOIN).await?;
        Ok(members
            .into_iter()
            .map(|member| RoomMemberSummary {
                user_id: member.user_id().to_string(),
                display_name: member.name().to_owned(),
                is_admin: !matches!(
                    member.suggested_role_for_power_level(),
                    RoomMemberRole::User
                ),
            })
            .collect())
    }
}
