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

/// The copy mask `copyOtherContactToMyContactsGroup` accepts.
///
/// Names, emails and phones only: `metadata`, valid in the read mask, is
/// refused there with `INVALID_ARGUMENT`.
pub const OTHER_CONTACT_COPY_FIELDS: &[GpeoplePersonField] = &[
    GpeoplePersonField::Names,
    GpeoplePersonField::EmailAddresses,
    GpeoplePersonField::PhoneNumbers,
];

#[cfg(test)]
mod tests {
    use io_gpeople::v1::rest::people::GpeoplePersonField;

    use super::OTHER_CONTACT_COPY_FIELDS;

    /// The copy mask holds only the three fields People copies: Google
    /// answered 400 to a mask holding `metadata` (live, 2026-11-24).
    #[test]
    fn the_copy_mask_holds_only_names_emails_and_phones() {
        for field in OTHER_CONTACT_COPY_FIELDS {
            assert!(
                matches!(
                    field,
                    GpeoplePersonField::Names
                        | GpeoplePersonField::EmailAddresses
                        | GpeoplePersonField::PhoneNumbers
                ),
                "{field:?}"
            );
        }
    }
}
