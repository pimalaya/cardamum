//! # Contact rendering
//!
//! Text and JSON shapes of the contact commands.

use core::fmt;

use io_msgraph::v1::rest::users::contacts::MsgraphContact;
use pimalaya_cli::{
    fields::Fields,
    records::{Record, Records},
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;

/// Display name of a contact, or the empty string.
pub fn contact_name(contact: &MsgraphContact) -> &str {
    contact.display_name.as_deref().unwrap_or("")
}

/// Every email address of a contact, in Graph's order.
pub fn contact_emails(contact: &MsgraphContact) -> Vec<&str> {
    let addresses = contact.email_addresses.as_option().into_iter().flatten();
    let addresses = addresses.filter_map(|address| address.address.as_deref());
    addresses.filter(|address| !address.is_empty()).collect()
}

/// Every phone number of a contact: the mobile one, then the business and
/// the home ones, as Graph splits them.
pub fn contact_phones(contact: &MsgraphContact) -> Vec<&str> {
    let mobile = contact.mobile_phone.as_option().map(String::as_str);
    let business = contact.business_phones.as_option().into_iter().flatten();
    let home = contact.home_phones.as_option().into_iter().flatten();
    let phones = business.chain(home).map(String::as_str);

    mobile
        .into_iter()
        .chain(phones)
        .filter(|phone| !phone.is_empty())
        .collect()
}

/// A page of contacts.
///
/// The text prints each contact by its id with its name, emails and phones,
/// while `--json` emits the raw Graph contact objects. It prints through
/// [`Paginated`], Graph's `@odata.nextLink` lifted into `nextPage`.
///
/// [`Paginated`]: pimalaya_cli::output::Paginated
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphContactsOutput {
    /// The raw Graph contact objects.
    #[serde(rename = "contacts")]
    #[schemars(with = "Vec<serde_json::Value>")]
    pub contacts: Vec<MsgraphContact>,
}

impl fmt::Display for MsgraphContactsOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut records = Records::default();

        for contact in &self.contacts {
            records.push(Record {
                title: sanitize::text(&contact.id).into_owned(),
                state: Vec::new(),
                fields: contact_fields(contact),
                body: String::new(),
            });
        }

        write!(f, "{records}")
    }
}

/// A single contact, emitted verbatim by `--json`.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct MsgraphContactOutput(
    #[schemars(with = "serde_json::Map<String, serde_json::Value>")] pub MsgraphContact,
);

impl fmt::Display for MsgraphContactOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let contact = &self.0;
        let mut fields = Fields::default();
        fields.push("Id", sanitize::text(&contact.id));
        fields.rows.extend(contact_fields(contact).rows);
        write!(f, "{fields}")
    }
}

/// The name, emails and phones of a contact, sanitized, a field Graph
/// sent no value for printing no line.
fn contact_fields(contact: &MsgraphContact) -> Fields<'static> {
    let mut fields = Fields::default();
    let name = contact_name(contact);

    if !name.is_empty() {
        fields.push("Name", sanitize::text(name));
    }

    for (label, values) in [
        ("Email", contact_emails(contact)),
        ("Phone", contact_phones(contact)),
    ] {
        if !values.is_empty() {
            fields.push(label, sanitize::text(&values.join(", ")));
        }
    }

    fields
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn contact(value: serde_json::Value) -> MsgraphContact {
        serde_json::from_value(value).unwrap()
    }

    /// A contact prints every email and every phone Graph holds, the
    /// business and home ones beside the mobile one, and no line for a field
    /// it holds no value for.
    #[test]
    fn a_contact_prints_the_emails_and_phones_it_has() {
        let with_values = MsgraphContactOutput(contact(json!({
            "id": "AAMk",
            "displayName": "Anna Mueller",
            "emailAddresses": [
                {"address": "anna@example.org"},
                {"address": "a.mueller@example.org"},
            ],
            "mobilePhone": "+49 170 0",
            "businessPhones": ["+49 30 23125 042"],
            "homePhones": ["+49 30 1"],
        })));
        let expected = "\
Id:     AAMk
Name:   Anna Mueller
Email:  anna@example.org, a.mueller@example.org
Phone:  +49 170 0, +49 30 23125 042, +49 30 1
";
        assert_eq!(with_values.to_string(), expected);

        let business_only = MsgraphContactOutput(contact(json!({
            "id": "AAMk",
            "mobilePhone": null,
            "emailAddresses": [],
            "businessPhones": ["+33 6 00 00 99 45"],
        })));
        let expected = "Id:     AAMk\nPhone:  +33 6 00 00 99 45\n";
        assert_eq!(business_only.to_string(), expected);
    }
}
