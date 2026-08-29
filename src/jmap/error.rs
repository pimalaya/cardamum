//! # Set error rendering
//!
//! Renders the per-item errors of a JMAP write (RFC 8620 §5.3) as prose
//! a user can read, rather than as the `Debug` shape of the error value.
//!
//! Each rejected item carries its own typed error, with an optional
//! server description and, for `invalidProperties`, the property names
//! at fault. Mirrors himalaya's jmap/error.rs.

use io_jmap::rfc9610::{
    address_book::set::JmapAddressBookSetItemError,
    contact_card::{copy::JmapContactCardCopyItemError, set::JmapContactCardSetItemError},
};

/// The parts of a JMAP set error worth showing.
pub trait JmapSetError {
    /// The error type as JMAP spells it, e.g. `invalidProperties`.
    fn type_name(&self) -> &'static str;
    /// The optional human-readable detail the server sent along.
    fn description(&self) -> Option<&str>;
    /// The property names at fault, empty outside `invalidProperties`.
    fn properties(&self) -> &[String];
}

/// Renders a set error as the suffix of a rejection message.
pub fn format_set_error<E: JmapSetError>(err: &E) -> String {
    let mut msg = format!(": {}", err.type_name());

    if !err.properties().is_empty() {
        msg.push_str(" (`");
        msg.push_str(&err.properties().join("`, `"));
        msg.push_str("`)");
    }

    if let Some(description) = err.description() {
        msg.push_str(": ");
        msg.push_str(description.trim_end_matches(['.', '\n']));
    }

    msg
}

impl JmapSetError for JmapAddressBookSetItemError {
    fn type_name(&self) -> &'static str {
        match self {
            Self::AddressBookHasContents { .. } => "addressBookHasContents",
            Self::Forbidden { .. } => "forbidden",
            Self::NotFound { .. } => "notFound",
            Self::InvalidPatch { .. } => "invalidPatch",
            Self::WillDestroy { .. } => "willDestroy",
            Self::InvalidProperties { .. } => "invalidProperties",
            Self::Unknown => "unknown",
        }
    }

    fn description(&self) -> Option<&str> {
        match self {
            Self::AddressBookHasContents { description }
            | Self::Forbidden { description }
            | Self::NotFound { description }
            | Self::InvalidPatch { description }
            | Self::WillDestroy { description }
            | Self::InvalidProperties { description, .. } => description.as_deref(),
            Self::Unknown => None,
        }
    }

    fn properties(&self) -> &[String] {
        match self {
            Self::InvalidProperties { properties, .. } => properties,
            _ => &[],
        }
    }
}

impl JmapSetError for JmapContactCardSetItemError {
    fn type_name(&self) -> &'static str {
        match self {
            Self::BlobNotFound { .. } => "blobNotFound",
            Self::Forbidden { .. } => "forbidden",
            Self::NotFound { .. } => "notFound",
            Self::InvalidPatch { .. } => "invalidPatch",
            Self::WillDestroy { .. } => "willDestroy",
            Self::InvalidProperties { .. } => "invalidProperties",
            Self::Unknown => "unknown",
        }
    }

    fn description(&self) -> Option<&str> {
        match self {
            Self::BlobNotFound { description }
            | Self::Forbidden { description }
            | Self::NotFound { description }
            | Self::InvalidPatch { description }
            | Self::WillDestroy { description }
            | Self::InvalidProperties { description, .. } => description.as_deref(),
            Self::Unknown => None,
        }
    }

    fn properties(&self) -> &[String] {
        match self {
            Self::InvalidProperties { properties, .. } => properties,
            _ => &[],
        }
    }
}

impl JmapSetError for JmapContactCardCopyItemError {
    fn type_name(&self) -> &'static str {
        match self {
            Self::AlreadyExists { .. } => "alreadyExists",
            Self::NotFound { .. } => "notFound",
            Self::InvalidProperties { .. } => "invalidProperties",
            Self::Unknown => "unknown",
        }
    }

    fn description(&self) -> Option<&str> {
        match self {
            Self::AlreadyExists { description }
            | Self::NotFound { description }
            | Self::InvalidProperties { description, .. } => description.as_deref(),
            Self::Unknown => None,
        }
    }

    fn properties(&self) -> &[String] {
        match self {
            Self::InvalidProperties { properties, .. } => properties,
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_the_type_the_properties_and_the_description() {
        let err = JmapContactCardSetItemError::InvalidProperties {
            description: Some("unknown property.".into()),
            properties: vec!["vCardProps".into(), "links/1/@type".into()],
        };

        assert_eq!(
            format_set_error(&err),
            ": invalidProperties (`vCardProps`, `links/1/@type`): unknown property"
        );
    }

    #[test]
    fn renders_an_error_carrying_neither_properties_nor_description() {
        let err = JmapContactCardSetItemError::NotFound { description: None };

        assert_eq!(format_set_error(&err), ": notFound");
    }
}
