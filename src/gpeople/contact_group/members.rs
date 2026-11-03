//! # Contact group members command
//!
//! Adds and removes members of a contact group
//! (`contactGroups.members.modify`).

use core::fmt;

use anyhow::{Result, bail};
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::gpeople::client::GpeopleClient;

/// Add and/or remove members of a contact group.
///
/// Members are person resource names (`people/<id>`).
///
/// JSON output: `{"id", "added", "removed"}`.
#[derive(Debug, Parser)]
pub struct GpeopleContactGroupMembersCommand {
    /// Group id (the segment after `contactGroups/`).
    #[arg(value_name = "GROUP-ID")]
    pub group_id: String,
    /// Person resource name to add (`people/<id>`); repeatable.
    #[arg(long, value_name = "RESOURCE-NAME")]
    pub add: Vec<String>,
    /// Person resource name to remove (`people/<id>`); repeatable.
    #[arg(long, value_name = "RESOURCE-NAME")]
    pub remove: Vec<String>,
}

impl GpeopleContactGroupMembersCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GpeopleClient) -> Result<()> {
        if self.add.is_empty() && self.remove.is_empty() {
            bail!("Pass at least one --add or --remove person resource name");
        }

        let resource_name = format!("contactGroups/{}", self.group_id);
        let modified = client
            .contact_group_members_modify(&resource_name, &self.add, &self.remove)?
            .response;

        if !modified.not_found_resource_names.is_empty() {
            bail!(
                "Some members were not found: {:?}",
                modified.not_found_resource_names
            );
        }

        printer.out(GpeopleMembersOutput {
            id: self.group_id,
            added: self.add.len(),
            removed: self.remove.len(),
        })
    }
}

/// The contact group whose members `gpeople contact-group members` changed.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GpeopleMembersOutput {
    /// The contact group id.
    pub id: String,
    /// How many members were asked to be added.
    pub added: usize,
    /// How many members were asked to be removed.
    pub removed: usize,
}

impl fmt::Display for GpeopleMembersOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { id, added, removed } = self;
        writeln!(
            f,
            "Contact group `{id}` membership successfully updated (+{added} / -{removed})"
        )
    }
}
