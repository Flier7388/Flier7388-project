//! Parsing and validation for the `sigil + localpart + ":" + server_name`
//! family of Matrix identifiers (user IDs, room IDs, room aliases).
//!
//! Event IDs are deliberately out of scope: from room version 3 onwards
//! they are content hashes with no server name suffix, which is a
//! different parsing problem this crate does not yet need to solve.
//!
//! Reference: Matrix Specification, "Identifier Grammar"
//! <https://spec.matrix.org/latest/appendices/#identifier-grammar>

use std::fmt;

/// The kind of identifier, determined by its leading sigil character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// `@localpart:server_name` — a user.
    User,
    /// `!localpart:server_name` — a room's internal, non-human-readable ID.
    Room,
    /// `#localpart:server_name` — a human-readable room alias.
    RoomAlias,
}

impl Kind {
    /// The sigil character that prefixes an identifier of this kind.
    #[must_use]
    pub const fn sigil(self) -> char {
        match self {
            Self::User => '@',
            Self::Room => '!',
            Self::RoomAlias => '#',
        }
    }

    fn from_sigil(sigil: char) -> Option<Self> {
        match sigil {
            '@' => Some(Self::User),
            '!' => Some(Self::Room),
            '#' => Some(Self::RoomAlias),
            _ => None,
        }
    }
}

/// A validated `sigil:localpart:server_name` Matrix identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MatrixId {
    kind: Kind,
    localpart: String,
    server_name: String,
}

impl MatrixId {
    /// Parses `input` as a Matrix identifier.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] if `input` does not start with a recognised
    /// sigil, has an empty localpart or server name, or is missing the
    /// `:` separator required by the identifier grammar.
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let sigil = input.chars().next().ok_or(ParseError::Empty)?;
        let kind = Kind::from_sigil(sigil).ok_or(ParseError::UnknownSigil(sigil))?;

        let rest = &input[sigil.len_utf8()..];
        let (localpart, server_name) = rest.split_once(':').ok_or(ParseError::MissingServerName)?;

        if localpart.is_empty() {
            return Err(ParseError::EmptyLocalpart);
        }
        if server_name.is_empty() {
            return Err(ParseError::EmptyServerName);
        }

        Ok(Self {
            kind,
            localpart: localpart.to_owned(),
            server_name: server_name.to_owned(),
        })
    }

    /// The identifier's kind.
    #[must_use]
    pub const fn kind(&self) -> Kind {
        self.kind
    }

    /// The portion between the sigil and the `:` separator.
    #[must_use]
    pub fn localpart(&self) -> &str {
        &self.localpart
    }

    /// The portion after the `:` separator.
    #[must_use]
    pub fn server_name(&self) -> &str {
        &self.server_name
    }
}

impl fmt::Display for MatrixId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}:{}",
            self.kind.sigil(),
            self.localpart,
            self.server_name
        )
    }
}

/// Why [`MatrixId::parse`] rejected its input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    /// The input was empty.
    #[error("identifier is empty")]
    Empty,
    /// The leading character is not `@`, `!`, or `#`.
    #[error("unrecognised sigil {0:?}")]
    UnknownSigil(char),
    /// No `:` separator was found between the localpart and server name.
    #[error("missing ':' separator before the server name")]
    MissingServerName,
    /// The localpart (between the sigil and `:`) was empty.
    #[error("localpart is empty")]
    EmptyLocalpart,
    /// The server name (after the `:`) was empty.
    #[error("server name is empty")]
    EmptyServerName,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_well_formed_user_id() {
        let id = MatrixId::parse("@alice:matrix.org").expect("valid user id");
        assert_eq!(id.kind(), Kind::User);
        assert_eq!(id.localpart(), "alice");
        assert_eq!(id.server_name(), "matrix.org");
        assert_eq!(id.to_string(), "@alice:matrix.org");
    }

    #[test]
    fn parses_a_room_alias() {
        let id = MatrixId::parse("#general:tuwunel.example").expect("valid alias");
        assert_eq!(id.kind(), Kind::RoomAlias);
    }

    #[test]
    fn rejects_missing_server_name() {
        assert_eq!(
            MatrixId::parse("@alice"),
            Err(ParseError::MissingServerName)
        );
    }

    #[test]
    fn rejects_unknown_sigil() {
        assert_eq!(
            MatrixId::parse("$event:matrix.org"),
            Err(ParseError::UnknownSigil('$'))
        );
    }

    #[test]
    fn rejects_empty_localpart() {
        assert_eq!(
            MatrixId::parse("@:matrix.org"),
            Err(ParseError::EmptyLocalpart)
        );
    }

    #[test]
    fn rejects_empty_server_name() {
        assert_eq!(MatrixId::parse("!room:"), Err(ParseError::EmptyServerName));
    }
}
