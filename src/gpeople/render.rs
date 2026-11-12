//! # People rendering
//!
//! The reports the People commands print: people and contact groups as
//! columns, single objects as `Label: value` lines, the raw People JSON
//! under `--json`.

use core::fmt;

use io_gpeople::v1::rest::{contact_groups::GpeopleContactGroup, people::GpeoplePerson};
use pimalaya_cli::{columns::Columns, fields::Fields, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

/// Display name of a person (display name, else unstructured name).
pub fn person_name(person: &GpeoplePerson) -> &str {
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
pub fn person_email(person: &GpeoplePerson) -> &str {
    person
        .email_addresses
        .first()
        .and_then(|email| email.value.as_deref())
        .unwrap_or("")
}

/// First phone number of a person, or the empty string.
pub fn person_phone(person: &GpeoplePerson) -> &str {
    person
        .phone_numbers
        .first()
        .and_then(|phone| phone.value.as_deref())
        .unwrap_or("")
}

/// A list of people: connections, other contacts or search results.
///
/// The text lists ID / EMAIL / PHONE / NAME; `--json` emits the raw
/// People person objects, plus any page and sync tokens.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GpeoplePersonsOutput {
    /// The raw People person objects.
    #[serde(rename = "people")]
    #[schemars(with = "Vec<serde_json::Value>")]
    pub people: Vec<GpeoplePerson>,
    /// The token to the next page, when the page was truncated.
    #[serde(rename = "nextPageToken", skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
    /// The token opening the next sync round, when the API sent one.
    #[serde(rename = "nextSyncToken", skip_serializing_if = "Option::is_none")]
    pub next_sync_token: Option<String>,
}

impl fmt::Display for GpeoplePersonsOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.people.is_empty() {
            let mut columns = Columns::new(["ID", "EMAIL", "PHONE", "NAME"]);

            for person in &self.people {
                columns.push([
                    sanitize::text(person.id()),
                    sanitize::text(person_email(person)),
                    sanitize::text(person_phone(person)),
                    sanitize::text(person_name(person)),
                ]);
            }

            write!(f, "{columns}")?;

            if self.next_sync_token.is_some() || self.next_page_token.is_some() {
                writeln!(f)?;
            }
        }

        if let Some(token) = &self.next_sync_token {
            writeln!(f, "sync-token: {}", sanitize::text(token))?;
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
pub struct GpeoplePersonOutput(
    #[schemars(with = "serde_json::Map<String, serde_json::Value>")] pub GpeoplePerson,
);

impl fmt::Display for GpeoplePersonOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let person = &self.0;
        let mut fields = Fields::default();
        fields.push("Id", sanitize::text(person.id()));
        fields.push("Name", sanitize::text(person_name(person)));
        fields.push("Email", sanitize::text(person_email(person)));
        fields.push("Phone", sanitize::text(person_phone(person)));
        write!(f, "{fields}")
    }
}

/// Bare id behind a `contactGroups/<id>` resource name.
pub fn group_id(group: &GpeopleContactGroup) -> &str {
    group
        .resource_name
        .strip_prefix("contactGroups/")
        .unwrap_or(&group.resource_name)
}

/// User-defined (or locale-formatted) name of a group.
pub fn group_name(group: &GpeopleContactGroup) -> &str {
    group
        .name
        .as_deref()
        .or(group.formatted_name.as_deref())
        .unwrap_or(&group.resource_name)
}

/// A list of contact groups.
///
/// The text lists ID / TYPE / MEMBERS / NAME; `--json` emits the raw
/// People group objects, plus any tokens.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GpeopleContactGroupsOutput {
    /// The raw People contact group objects.
    #[serde(rename = "contactGroups")]
    #[schemars(with = "Vec<serde_json::Value>")]
    pub groups: Vec<GpeopleContactGroup>,
    /// The token to the next page, when the page was truncated.
    #[serde(rename = "nextPageToken", skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
    /// The token opening the next sync round, when the API sent one.
    #[serde(rename = "nextSyncToken", skip_serializing_if = "Option::is_none")]
    pub next_sync_token: Option<String>,
}

impl fmt::Display for GpeopleContactGroupsOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.groups.is_empty() {
            let mut columns = Columns::new(["ID", "TYPE", "MEMBERS", "NAME"]);

            for group in &self.groups {
                let group_type = group
                    .group_type
                    .as_ref()
                    .map(|kind| format!("{kind:?}"))
                    .unwrap_or_default();

                columns.push([
                    sanitize::text(group_id(group)).into_owned(),
                    group_type,
                    member_count(group),
                    sanitize::text(group_name(group)).into_owned(),
                ]);
            }

            write!(f, "{columns}")?;

            if self.next_page_token.is_some() {
                writeln!(f)?;
            }
        }

        if self.next_page_token.is_some() {
            writeln!(f, "(more groups available: follow nextPageToken)")?;
        }
        Ok(())
    }
}

/// A single contact group; `--json` emits the raw People object.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct GpeopleContactGroupOutput(
    #[schemars(with = "serde_json::Map<String, serde_json::Value>")] pub GpeopleContactGroup,
);

impl fmt::Display for GpeopleContactGroupOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let group = &self.0;
        let mut fields = Fields::default();
        fields.push("Id", sanitize::text(group_id(group)));
        fields.push("Name", sanitize::text(group_name(group)));
        fields.push("Resource name", sanitize::text(&group.resource_name));
        fields.push("Members", member_count(group));
        write!(f, "{fields}")
    }
}

/// The group's member count as the server reports it, else blank.
///
/// `memberResourceNames` is not it: People fills that only on a `get`,
/// and even then only up to the requested maximum, so counting it reads
/// as an empty group in every listing.
fn member_count(group: &GpeopleContactGroup) -> String {
    group
        .member_count
        .map(|count| count.to_string())
        .unwrap_or_default()
}
