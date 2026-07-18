use core::fmt;

use comfy_table::{Cell, Color, Row, Table};
use io_people::v1::rest::{contact_groups::PeopleContactGroup, people::PeoplePerson};
use serde::Serialize;

use crate::google::project;

/// Display name of a person (display name, else unstructured name).
pub fn person_name(person: &PeoplePerson) -> &str {
    person
        .names
        .first()
        .and_then(|name| {
            name.display_name
                .as_deref()
                .or(name.unstructured_name.as_deref())
        })
        .unwrap_or("")
}

/// First email of a person, or the empty string.
pub fn person_email(person: &PeoplePerson) -> &str {
    person
        .email_addresses
        .first()
        .and_then(|email| email.value.as_deref())
        .unwrap_or("")
}

/// First phone number of a person, or the empty string.
pub fn person_phone(person: &PeoplePerson) -> &str {
    person
        .phone_numbers
        .first()
        .and_then(|phone| phone.value.as_deref())
        .unwrap_or("")
}

/// A list of people (connections, other contacts, search results). The
/// table shows ID / NAME / EMAIL / PHONE; `--json` emits the raw People
/// person objects, plus any page and sync tokens.
#[derive(Clone, Debug, Serialize)]
pub struct PersonsReport {
    #[serde(skip)]
    pub preset: String,
    #[serde(skip)]
    pub id_color: Color,
    #[serde(rename = "people")]
    pub people: Vec<PeoplePerson>,
    #[serde(rename = "nextPageToken", skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
    #[serde(rename = "nextSyncToken", skip_serializing_if = "Option::is_none")]
    pub next_sync_token: Option<String>,
}

impl fmt::Display for PersonsReport {
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
            .add_rows(self.people.iter().map(|person| {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(
                        Cell::new(project::person_id(&person.resource_name)).fg(self.id_color),
                    )
                    .add_cell(Cell::new(person_name(person)))
                    .add_cell(Cell::new(person_email(person)))
                    .add_cell(Cell::new(person_phone(person)));
                row
            }));

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        if let Some(token) = &self.next_sync_token {
            writeln!(f, "sync-token: {token}")?;
        }
        if self.next_page_token.is_some() {
            writeln!(f, "(more people available: follow nextPageToken)")?;
        }
        Ok(())
    }
}

/// A single person; `--json` emits the raw People object.
#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct PersonReport(pub PeoplePerson);

impl fmt::Display for PersonReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let person = &self.0;
        writeln!(f, "id: {}", project::person_id(&person.resource_name))?;
        writeln!(f, "name: {}", person_name(person))?;
        writeln!(f, "email: {}", person_email(person))?;
        writeln!(f, "phone: {}", person_phone(person))
    }
}

/// Bare id behind a `contactGroups/<id>` resource name.
pub fn group_id(group: &PeopleContactGroup) -> &str {
    group
        .resource_name
        .strip_prefix("contactGroups/")
        .unwrap_or(&group.resource_name)
}

/// User-defined (or locale-formatted) name of a group.
pub fn group_name(group: &PeopleContactGroup) -> &str {
    group
        .name
        .as_deref()
        .or(group.formatted_name.as_deref())
        .unwrap_or(&group.resource_name)
}

/// A list of contact groups. The table shows ID / NAME / TYPE / MEMBERS;
/// `--json` emits the raw People group objects, plus any tokens.
#[derive(Clone, Debug, Serialize)]
pub struct GroupsReport {
    #[serde(skip)]
    pub preset: String,
    #[serde(skip)]
    pub id_color: Color,
    #[serde(rename = "contactGroups")]
    pub groups: Vec<PeopleContactGroup>,
    #[serde(rename = "nextPageToken", skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
    #[serde(rename = "nextSyncToken", skip_serializing_if = "Option::is_none")]
    pub next_sync_token: Option<String>,
}

impl fmt::Display for GroupsReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_preset(&self.preset)
            .set_header(Row::from([
                Cell::new("ID"),
                Cell::new("NAME"),
                Cell::new("TYPE"),
                Cell::new("MEMBERS"),
            ]))
            .add_rows(self.groups.iter().map(|group| {
                let group_type = group
                    .group_type
                    .as_ref()
                    .map(|kind| format!("{kind:?}"))
                    .unwrap_or_default();
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(group_id(group)).fg(self.id_color))
                    .add_cell(Cell::new(group_name(group)))
                    .add_cell(Cell::new(group_type))
                    .add_cell(Cell::new(group.member_resource_names.len()));
                row
            }));

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        if self.next_page_token.is_some() {
            writeln!(f, "(more groups available: follow nextPageToken)")?;
        }
        Ok(())
    }
}

/// A single contact group; `--json` emits the raw People object.
#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct GroupReport(pub PeopleContactGroup);

impl fmt::Display for GroupReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let group = &self.0;
        writeln!(f, "id: {}", group_id(group))?;
        writeln!(f, "name: {}", group_name(group))?;
        writeln!(f, "resource-name: {}", group.resource_name)?;
        writeln!(f, "members: {}", group.member_resource_names.len())
    }
}
