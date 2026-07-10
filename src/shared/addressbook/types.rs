//! Addressbook types shared across every backend.

use serde::{Deserialize, Serialize};

/// An addressbook collection.
///
/// Strict least-common-denominator shape across the backends the CLI
/// targets (vdir, CardDAV, JMAP, Microsoft Graph, Google People).
/// Partial-coverage fields (description, color) remain optional and
/// are populated by the backends that know them.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Addressbook {
    /// Backend-specific identifier: collection directory name (vdir),
    /// last URL segment (CardDAV), AddressBook id (JMAP), contact
    /// folder id (Graph) or contact group id (Google).
    pub id: String,

    /// Human-readable display name.
    pub name: String,

    /// Free-form description, when the backend exposes it.
    #[serde(default)]
    pub description: Option<String>,

    /// ASCII `#RRGGBB` color marker, when the backend exposes it.
    #[serde(default)]
    pub color: Option<String>,
}

/// Partial update applied to an [`Addressbook`].
///
/// Every field is optional: `None` means "leave untouched", `Some`
/// means "replace with this value" (including `Some(None)` to clear an
/// optional field).
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct AddressbookDiff {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<Option<String>>,
    #[serde(default)]
    pub color: Option<Option<String>>,
}
