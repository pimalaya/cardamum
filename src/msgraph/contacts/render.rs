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

/// First email address of a contact, or the empty string.
pub fn contact_email(contact: &MsgraphContact) -> &str {
    contact
        .email_addresses
        .as_option()
        .and_then(|addresses| addresses.first())
        .and_then(|address| address.address.as_deref())
        .unwrap_or("")
}

/// Mobile phone of a contact, or the empty string.
pub fn contact_phone(contact: &MsgraphContact) -> &str {
    contact.mobile_phone.as_deref().unwrap_or("")
}

/// A page of contacts.
///
/// The text prints each contact by its id with its name, email and phone,
/// while `--json` emits the raw Graph contact objects plus any next-page
/// link.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphContactsOutput {
    /// The raw Graph contact objects.
    #[serde(rename = "contacts")]
    #[schemars(with = "Vec<serde_json::Value>")]
    pub contacts: Vec<MsgraphContact>,
    /// The link to the next page, when the page was truncated.
    #[serde(rename = "@odata.nextLink", skip_serializing_if = "Option::is_none")]
    pub next_link: Option<String>,
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

        write!(f, "{records}")?;

        if self.next_link.is_some() {
            if !records.items.is_empty() {
                writeln!(f)?;
            }

            writeln!(f, "(more contacts available: raise --top)")?;
        }

        Ok(())
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

/// The name, first email and mobile phone of a contact, sanitized.
fn contact_fields(contact: &MsgraphContact) -> Fields<'static> {
    let mut fields = Fields::default();
    fields.push("Name", sanitize::text(contact_name(contact)));
    fields.push("Email", sanitize::text(contact_email(contact)));
    fields.push("Phone", sanitize::text(contact_phone(contact)));
    fields
}
