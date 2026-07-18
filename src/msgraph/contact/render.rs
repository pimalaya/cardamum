use core::fmt;

use comfy_table::{Cell, Color, Row, Table};
use io_msgraph::v1::rest::users::contacts::MsgraphContact;
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

/// A page of contacts. The table shows ID / NAME / EMAIL / PHONE;
/// `--json` emits the raw Graph contact objects plus any next-page link.
#[derive(Clone, Debug, Serialize)]
pub struct ContactsReport {
    #[serde(skip)]
    pub preset: String,
    #[serde(skip)]
    pub id_color: Color,
    #[serde(rename = "contacts")]
    pub contacts: Vec<MsgraphContact>,
    #[serde(rename = "@odata.nextLink", skip_serializing_if = "Option::is_none")]
    pub next_link: Option<String>,
}

impl fmt::Display for ContactsReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_preset(&self.preset)
            .set_header(Row::from([
                Cell::new("ID"),
                Cell::new("NAME"),
                Cell::new("EMAIL"),
                Cell::new("PHONE"),
            ]))
            .add_rows(self.contacts.iter().map(|contact| {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(&contact.id).fg(self.id_color))
                    .add_cell(Cell::new(contact_name(contact)))
                    .add_cell(Cell::new(contact_email(contact)))
                    .add_cell(Cell::new(contact_phone(contact)));
                row
            }));

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        if self.next_link.is_some() {
            writeln!(f, "(more contacts available: raise --top)")?;
        }
        Ok(())
    }
}

/// A single contact; `--json` emits the raw Graph object.
#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct ContactReport(pub MsgraphContact);

impl fmt::Display for ContactReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let contact = &self.0;
        writeln!(f, "id: {}", contact.id)?;
        writeln!(f, "display-name: {}", contact_name(contact))?;
        writeln!(f, "email: {}", contact_email(contact))?;
        writeln!(f, "mobile-phone: {}", contact_phone(contact))
    }
}
