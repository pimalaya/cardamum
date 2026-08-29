//! # Other contact field mask
//!
//! The reduced person field mask the `otherContacts` endpoints accept.

use io_people::v1::rest::people::PeoplePersonField;

/// The reduced person field mask `otherContacts` accepts.
///
/// Only names, emails, phones and metadata are exposed: asking for more
/// fails with an invalid-read-mask error.
pub const OTHER_CONTACT_FIELDS: &[PeoplePersonField] = &[
    PeoplePersonField::Names,
    PeoplePersonField::EmailAddresses,
    PeoplePersonField::PhoneNumbers,
    PeoplePersonField::Metadata,
];
