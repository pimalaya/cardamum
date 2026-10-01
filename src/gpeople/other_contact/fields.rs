//! # Other contact field mask
//!
//! The reduced person field mask the `otherContacts` endpoints accept.

use io_gpeople::v1::rest::people::GpeoplePersonField;

/// The reduced person field mask `otherContacts` accepts.
///
/// Only names, emails, phones and metadata are exposed: asking for more
/// fails with an invalid-read-mask error.
pub const OTHER_CONTACT_FIELDS: &[GpeoplePersonField] = &[
    GpeoplePersonField::Names,
    GpeoplePersonField::EmailAddresses,
    GpeoplePersonField::PhoneNumbers,
    GpeoplePersonField::Metadata,
];
