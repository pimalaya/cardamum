//! # Set error rendering
//!
//! Renders the per-item errors of a JMAP write (RFC 8620 §5.3) as prose
//! a user can read, rather than as the `Debug` shape of the error value.
//!
//! Each rejected item carries its own typed error, with an optional
//! server description and, for `invalidProperties`, the property names
//! at fault. Mirrors himalaya's jmap/error.rs.

use anyhow::{Error, anyhow};
use io_jmap::rfc9610::{
    address_book::set::JmapAddressBookSetItemError,
    contact_card::{copy::JmapContactCardCopyItemError, set::JmapContactCardSetItemError},
};
use pimalaya_cli::error::CodedError;

use crate::error::ErrorCode;

/// The parts of a JMAP set error worth showing.
pub trait JmapSetError {
    /// The error type as JMAP spells it, e.g. `invalidProperties`.
    fn type_name(&self) -> &str;
    /// The optional human-readable detail the server sent along.
    fn description(&self) -> Option<&str>;
    /// The property names at fault, empty outside `invalidProperties`.
    fn properties(&self) -> &[String];
    /// The code of a `notFound`, naming what the set found missing.
    fn not_found(&self) -> ErrorCode;

    /// The code of the error, read from its type (RFC 8620 §5.3): none
    /// for a type no caller acts on, its type kept in the message.
    fn code(&self) -> Option<ErrorCode> {
        match self.type_name() {
            "forbidden" => Some(ErrorCode::PermissionDenied),
            "notFound" => Some(self.not_found()),
            "overQuota" => Some(ErrorCode::QuotaExceeded),
            "rateLimit" => Some(ErrorCode::RateLimited),
            _ => None,
        }
    }
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

/// The failure of a write the server refused, `prefix` naming the write,
/// followed by the rendered error ([`format_set_error`]) and coded by its
/// type.
pub fn set_error<E: JmapSetError>(prefix: &str, err: &E) -> Error {
    set_error_message(format!("{prefix}{}", format_set_error(err)), err)
}

/// The failure of a write the server refused, worded `message`, coded by
/// the type of `err`.
pub fn set_error_message<E: JmapSetError>(message: String, err: &E) -> Error {
    match err.code() {
        Some(code) => CodedError::new(code, message).into(),
        None => anyhow!(message),
    }
}

impl JmapSetError for JmapAddressBookSetItemError {
    fn not_found(&self) -> ErrorCode {
        ErrorCode::AddressbookNotFound
    }

    fn type_name(&self) -> &str {
        match self {
            Self::AddressBookHasContents { .. } => "addressBookHasContents",
            Self::Forbidden { .. } => "forbidden",
            Self::NotFound { .. } => "notFound",
            Self::InvalidPatch { .. } => "invalidPatch",
            Self::WillDestroy { .. } => "willDestroy",
            Self::InvalidProperties { .. } => "invalidProperties",
            Self::Other { type_name, .. } if type_name.is_empty() => "unknown",
            Self::Other { type_name, .. } => type_name,
        }
    }

    fn description(&self) -> Option<&str> {
        match self {
            Self::AddressBookHasContents { description }
            | Self::Forbidden { description }
            | Self::NotFound { description }
            | Self::InvalidPatch { description }
            | Self::WillDestroy { description }
            | Self::InvalidProperties { description, .. }
            | Self::Other { description, .. } => description.as_deref(),
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
    fn not_found(&self) -> ErrorCode {
        ErrorCode::CardNotFound
    }

    fn type_name(&self) -> &str {
        match self {
            Self::BlobNotFound { .. } => "blobNotFound",
            Self::Forbidden { .. } => "forbidden",
            Self::NotFound { .. } => "notFound",
            Self::InvalidPatch { .. } => "invalidPatch",
            Self::WillDestroy { .. } => "willDestroy",
            Self::InvalidProperties { .. } => "invalidProperties",
            Self::Other { type_name, .. } if type_name.is_empty() => "unknown",
            Self::Other { type_name, .. } => type_name,
        }
    }

    fn description(&self) -> Option<&str> {
        match self {
            Self::BlobNotFound { description }
            | Self::Forbidden { description }
            | Self::NotFound { description }
            | Self::InvalidPatch { description }
            | Self::WillDestroy { description }
            | Self::InvalidProperties { description, .. }
            | Self::Other { description, .. } => description.as_deref(),
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
    fn not_found(&self) -> ErrorCode {
        ErrorCode::CardNotFound
    }

    fn type_name(&self) -> &str {
        match self {
            Self::AlreadyExists { .. } => "alreadyExists",
            Self::NotFound { .. } => "notFound",
            Self::InvalidProperties { .. } => "invalidProperties",
            Self::Other { type_name, .. } if type_name.is_empty() => "unknown",
            Self::Other { type_name, .. } => type_name,
        }
    }

    fn description(&self) -> Option<&str> {
        match self {
            Self::AlreadyExists { description, .. }
            | Self::NotFound { description }
            | Self::InvalidProperties { description, .. }
            | Self::Other { description, .. } => description.as_deref(),
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

    /// The code a refused write of a card answered `set_error` fails with,
    /// the error read as io-jmap reads it off the wire.
    fn card_code(set_error: serde_json::Value) -> Option<ErrorCode> {
        let err: JmapContactCardSetItemError = serde_json::from_value(set_error).unwrap();
        let err = super::set_error("ContactCard destroy rejected", &err);
        pimalaya_cli::error::code_of(&err)
    }

    #[test]
    fn a_set_error_is_coded_by_its_type() {
        use serde_json::json;

        let code = |kind: &str| card_code(json!({"type": kind}));

        assert_eq!(code("forbidden"), Some(ErrorCode::PermissionDenied));
        assert_eq!(code("notFound"), Some(ErrorCode::CardNotFound));
        assert_eq!(code("overQuota"), Some(ErrorCode::QuotaExceeded));
        assert_eq!(code("rateLimit"), Some(ErrorCode::RateLimited));
        assert_eq!(code("invalidPatch"), None);
        assert_eq!(code("tooLarge"), None);
    }

    /// An address book a set does not find is no card.
    #[test]
    fn an_address_book_not_found_is_coded_so() {
        let err: JmapAddressBookSetItemError =
            serde_json::from_value(serde_json::json!({"type": "notFound"})).unwrap();
        let err = set_error("AddressBook destroy rejected", &err);

        assert_eq!(
            pimalaya_cli::error::code_of(&err),
            Some(ErrorCode::AddressbookNotFound)
        );
        assert_eq!(err.to_string(), "AddressBook destroy rejected: notFound");
    }
}
