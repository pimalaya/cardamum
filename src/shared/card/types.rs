//! Card types shared across every backend.

use serde::{Deserialize, Serialize};

/// A single card inside an addressbook.
///
/// Strict least-common-denominator shape: contents stay raw vCard
/// bytes. Backends without a native vCard representation (JMAP,
/// Microsoft Graph, Google People) project their wire contact onto a
/// vCard document.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Card {
    /// Card identifier: file stem (vdir), last URL segment (CardDAV),
    /// ContactCard id (JMAP), contact id (Graph) or person id
    /// (Google).
    pub id: String,

    /// Parent addressbook identifier.
    pub addressbook_id: String,

    /// Entity tag (RFC 9110 §8.8.3, without surrounding quotes) when
    /// the backend exposes one: the CardDAV ETag, the Graph changeKey,
    /// the Google person etag or a JSON hash for JMAP; vdir surfaces
    /// `None`.
    #[serde(default)]
    pub etag: Option<String>,

    /// Raw vCard bytes.
    pub contents: Vec<u8>,
}
