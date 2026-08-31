//! # People rendering
//!
//! The reports the People commands print: people and contact groups as
//! tables, single objects as key/value lines, the raw People JSON under
//! `--json`.

use core::fmt;

use io_people::v1::rest::{contact_groups::PeopleContactGroup, people::PeoplePerson};
use pimalaya_cli::table::{Cell, Color, Row, Table};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{people::project, shared::table::style_from_preset};

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

/// A list of people: connections, other contacts or search results.
///
/// The table shows ID / NAME / EMAIL / PHONE; `--json` emits the raw
/// People person objects, plus any page and sync tokens.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PeoplePersonsOutput {
    /// The `comfy_table` preset the table is drawn with.
    #[serde(skip)]
    pub preset: String,
    /// Color of the ID column.
    #[serde(skip)]
    pub id_color: Color,
    /// The raw People person objects.
    #[serde(rename = "people")]
    #[schemars(with = "Vec<serde_json::Value>")]
    pub people: Vec<PeoplePerson>,
    /// The token to the next page, when the page was truncated.
    #[serde(rename = "nextPageToken", skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
    /// The token opening the next sync round, when the API sent one.
    #[serde(rename = "nextSyncToken", skip_serializing_if = "Option::is_none")]
    pub next_sync_token: Option<String>,
}

impl fmt::Display for PeoplePersonsOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(style_from_preset(&self.preset))
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
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct PeoplePersonOutput(#[schemars(with = "serde_json::Value")] pub PeoplePerson);

impl fmt::Display for PeoplePersonOutput {
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

/// A list of contact groups.
///
/// The table shows ID / NAME / TYPE / MEMBERS; `--json` emits the raw
/// People group objects, plus any tokens.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PeopleContactGroupsOutput {
    /// The `comfy_table` preset the table is drawn with.
    #[serde(skip)]
    pub preset: String,
    /// Color of the ID column.
    #[serde(skip)]
    pub id_color: Color,
    /// The raw People contact group objects.
    #[serde(rename = "contactGroups")]
    #[schemars(with = "Vec<serde_json::Value>")]
    pub groups: Vec<PeopleContactGroup>,
    /// The token to the next page, when the page was truncated.
    #[serde(rename = "nextPageToken", skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
    /// The token opening the next sync round, when the API sent one.
    #[serde(rename = "nextSyncToken", skip_serializing_if = "Option::is_none")]
    pub next_sync_token: Option<String>,
}

impl fmt::Display for PeopleContactGroupsOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(style_from_preset(&self.preset))
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
                    .add_cell(Cell::new(member_count(group)));
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
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct PeopleContactGroupOutput(#[schemars(with = "serde_json::Value")] pub PeopleContactGroup);

impl fmt::Display for PeopleContactGroupOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let group = &self.0;
        writeln!(f, "id: {}", group_id(group))?;
        writeln!(f, "name: {}", group_name(group))?;
        writeln!(f, "resource-name: {}", group.resource_name)?;
        writeln!(f, "members: {}", member_count(group))
    }
}

/// The group's member count as the server reports it, else blank.
///
/// `memberResourceNames` is not it: People fills that only on a `get`,
/// and even then only up to the requested maximum, so counting it reads
/// as an empty group in every listing.
fn member_count(group: &PeopleContactGroup) -> String {
    group
        .member_count
        .map(|count| count.to_string())
        .unwrap_or_default()
}
