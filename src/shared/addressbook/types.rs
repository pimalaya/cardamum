//! # Addressbook types
//!
//! The addressbook shape shared across every backend, and the partial
//! update applied to it.

use serde::{Deserialize, Serialize};

/// A collection of cards.
///
/// Strict least common denominator across the backends the CLI targets:
/// a field only some of them expose stays optional, and is filled in by
/// the ones that know it.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Addressbook {
    /// Backend-specific identifier of the collection.
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
/// `None` leaves the field untouched, `Some` replaces it, and
/// `Some(None)` clears an optional field.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct AddressbookDiff {
    /// New display name.
    #[serde(default)]
    pub name: Option<String>,
    /// New description, `Some(None)` to clear it.
    #[serde(default)]
    pub description: Option<Option<String>>,
    /// New color marker, `Some(None)` to clear it.
    #[serde(default)]
    pub color: Option<Option<String>>,
}
