use io_people::v1::rest::people::PeoplePersonField;

/// The reduced person field mask `otherContacts` accepts: it exposes
/// only names, emails, phones and metadata (requesting more fails with
/// an invalid-read-mask error).
pub const OTHER_CONTACT_FIELDS: &[PeoplePersonField] = &[
    PeoplePersonField::Names,
    PeoplePersonField::EmailAddresses,
    PeoplePersonField::PhoneNumbers,
    PeoplePersonField::Metadata,
];
