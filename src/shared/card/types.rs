//! # Card types
//!
//! The card shape shared across every backend, and what an update could
//! not carry out.

use serde::{Deserialize, Serialize};

/// A single card inside an addressbook.
///
/// Strict least common denominator: contents stay raw vCard bytes, so a
/// backend with no native vCard representation projects its wire contact
/// onto a vCard document.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Card {
    /// Backend-specific identifier of the card.
    pub id: String,
    /// Parent addressbook identifier.
    pub addressbook_id: String,
    /// Entity tag (RFC 9110 section 8.8.3, unquoted), when the backend
    /// exposes one.
    #[serde(default)]
    pub etag: Option<String>,
    /// Raw vCard bytes.
    pub contents: Vec<u8>,
}

/// What an update could not carry out.
///
/// A backend stashing the vCard properties its wire model cannot hold
/// depends on the provider letting that stash be rewritten. Where it
/// cannot be emptied, a property dropped from the vCard stays on the
/// server and the update lands minus that. Other backends return this
/// empty.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct CardUpdateOutcome {
    /// Names of the vCard properties the update dropped and the server
    /// kept anyway.
    #[serde(default)]
    pub kept_properties: Vec<String>,
}
