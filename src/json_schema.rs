//! # JSON Schema registry
//!
//! Maps a CLI-invocation key, the command path joined with hyphens and
//! prefixed `cardamum-`, to the JSON Schema of that command's `--json`
//! payload. [`JsonSchemaCommand`] writes one file per entry.
//!
//! Only the commands returning data are listed: a command confirming a
//! write prints a `Message`, whose shape is the same everywhere and
//! carries nothing to describe.
//!
//! Protocol-specific entries are gated behind the same cargo features as
//! their command modules, so the registry stays coherent under any
//! feature combination, none included.
//!
//! [`JsonSchemaCommand`]: pimalaya_cli::clap::commands::JsonSchemaCommand

use std::collections::BTreeMap;

use schemars::schema_for;
use serde_json::Value;

/// Builds the command-to-schema map consumed by `json-schema <DIR>`.
///
/// Each value describes the type the command hands to the printer. A
/// command whose backend answers with nothing to show falls back to a
/// `Message`, which no entry describes.
pub fn schemas() -> BTreeMap<String, Value> {
    let mut schemas = BTreeMap::new();

    macro_rules! insert {
        ($key:expr, $ty:ty) => {
            schemas.insert(
                $key.to_string(),
                serde_json::to_value(schema_for!($ty)).unwrap(),
            );
        };
    }

    insert!(
        "cardamum-configure",
        crate::wizard::configure::ConfigureOutput
    );
    insert!(
        "cardamum-account-list",
        crate::account::list::AccountListOutput
    );
    insert!(
        "cardamum-account-check",
        crate::account::check::AccountCheckOutput
    );
    insert!(
        "cardamum-addressbook-list",
        crate::shared::addressbook::list::AddressbookListOutput
    );
    insert!(
        "cardamum-addressbook-create",
        crate::shared::addressbook::create::AddressbookCreateOutput
    );
    insert!(
        "cardamum-card-list",
        crate::shared::card::list::CardListOutput
    );
    insert!(
        "cardamum-card-read",
        crate::shared::card::read::CardReadOutput
    );
    insert!(
        "cardamum-card-create",
        crate::shared::card::create::CardCreateOutput
    );
    insert!(
        "cardamum-card-update",
        crate::shared::card::update::CardUpdateOutput
    );

    #[cfg(feature = "carddav")]
    {
        insert!(
            "cardamum-carddav-discover",
            crate::carddav::discover::CarddavDiscoverOutput
        );
        insert!(
            "cardamum-carddav-propfind",
            crate::carddav::propfind::CarddavPropfindOutput
        );
        insert!(
            "cardamum-carddav-get",
            crate::carddav::get::CarddavGetOutput
        );
        // NOTE: `report query` and `report multiget` both answer with the
        // card table of `report::entries`.
        insert!(
            "cardamum-carddav-report-query",
            crate::carddav::report::entries::CarddavCardEntriesOutput
        );
        insert!(
            "cardamum-carddav-report-multiget",
            crate::carddav::report::entries::CarddavCardEntriesOutput
        );
        insert!(
            "cardamum-carddav-report-sync",
            crate::carddav::report::sync::CarddavReportSyncOutput
        );
        insert!(
            "cardamum-carddav-report-raw",
            crate::carddav::report::raw::CarddavReportRawOutput
        );
    }

    #[cfg(feature = "jmap")]
    {
        use crate::jmap::render::{
            JmapAddressBookOutput, JmapAddressBooksOutput, JmapChangesOutput,
            JmapContactCardOutput, JmapContactCardsOutput,
        };

        insert!("cardamum-jmap-address-book-get", JmapAddressBooksOutput);
        insert!("cardamum-jmap-address-book-create", JmapAddressBookOutput);
        insert!("cardamum-jmap-address-book-update", JmapAddressBookOutput);
        insert!("cardamum-jmap-address-book-changes", JmapChangesOutput);
        insert!("cardamum-jmap-contact-card-get", JmapContactCardsOutput);
        insert!("cardamum-jmap-contact-card-query", JmapContactCardsOutput);
        insert!("cardamum-jmap-contact-card-create", JmapContactCardOutput);
        insert!("cardamum-jmap-contact-card-update", JmapContactCardOutput);
        insert!("cardamum-jmap-contact-card-changes", JmapChangesOutput);
        insert!(
            "cardamum-jmap-session-get",
            crate::jmap::session::get::JmapSessionGetOutput
        );
        insert!(
            "cardamum-jmap-request",
            crate::shared::raw_json::RawJsonOutput
        );
    }

    #[cfg(feature = "msgraph")]
    {
        use crate::msgraph::{
            contact_folders::render::{MsgraphContactFolderOutput, MsgraphContactFoldersOutput},
            contacts::render::{MsgraphContactOutput, MsgraphContactsOutput},
        };

        insert!(
            "cardamum-msgraph-contact-folders-list",
            MsgraphContactFoldersOutput
        );
        insert!(
            "cardamum-msgraph-contact-folders-child-folders",
            MsgraphContactFoldersOutput
        );
        insert!(
            "cardamum-msgraph-contact-folders-get",
            MsgraphContactFolderOutput
        );
        insert!(
            "cardamum-msgraph-contact-folders-create",
            MsgraphContactFolderOutput
        );
        insert!(
            "cardamum-msgraph-contact-folders-rename",
            MsgraphContactFolderOutput
        );
        insert!("cardamum-msgraph-contacts-list", MsgraphContactsOutput);
        insert!("cardamum-msgraph-contacts-get", MsgraphContactOutput);
        insert!("cardamum-msgraph-contacts-create", MsgraphContactOutput);
        insert!("cardamum-msgraph-contacts-update", MsgraphContactOutput);
        insert!(
            "cardamum-msgraph-contacts-delta",
            crate::msgraph::contacts::delta::MsgraphContactDeltaOutput
        );
        insert!(
            "cardamum-msgraph-profile-get",
            crate::msgraph::profile::get::MsgraphProfileGetOutput
        );
        insert!(
            "cardamum-msgraph-request",
            crate::shared::raw_json::RawJsonOutput
        );
    }

    #[cfg(feature = "people")]
    {
        use crate::people::render::{
            PeopleContactGroupOutput, PeopleContactGroupsOutput, PeoplePersonOutput,
            PeoplePersonsOutput,
        };

        insert!(
            "cardamum-people-contact-group-list",
            PeopleContactGroupsOutput
        );
        insert!(
            "cardamum-people-contact-group-get",
            PeopleContactGroupOutput
        );
        insert!(
            "cardamum-people-contact-group-create",
            PeopleContactGroupOutput
        );
        insert!(
            "cardamum-people-contact-group-update",
            PeopleContactGroupOutput
        );
        insert!("cardamum-people-connection-list", PeoplePersonsOutput);
        insert!("cardamum-people-connection-get", PeoplePersonOutput);
        insert!("cardamum-people-connection-create", PeoplePersonOutput);
        insert!("cardamum-people-connection-update", PeoplePersonOutput);
        insert!("cardamum-people-connection-search", PeoplePersonsOutput);
        insert!("cardamum-people-other-contact-list", PeoplePersonsOutput);
        insert!("cardamum-people-other-contact-search", PeoplePersonsOutput);
        insert!("cardamum-people-other-contact-copy", PeoplePersonOutput);
        insert!("cardamum-people-profile-get", PeoplePersonOutput);
        insert!(
            "cardamum-people-request",
            crate::shared::raw_json::RawJsonOutput
        );
    }

    #[cfg(feature = "vdir")]
    {
        insert!(
            "cardamum-vdir-list",
            crate::vdir::list::VdirCollectionListOutput
        );
        insert!(
            "cardamum-vdir-item-list",
            crate::vdir::item::list::VdirItemListOutput
        );
        insert!(
            "cardamum-vdir-item-get",
            crate::vdir::item::get::VdirItemGetOutput
        );
        insert!(
            "cardamum-vdir-item-create",
            crate::vdir::item::create::VdirItemCreateOutput
        );
    }

    schemas
}

#[cfg(test)]
mod tests {
    use clap::{Command, CommandFactory};

    use super::schemas;
    use crate::cli::Cli;

    /// Collects every command path of the tree, hyphen-joined.
    fn paths(command: &Command, prefix: &str, into: &mut Vec<String>) {
        for sub in command.get_subcommands() {
            let path = format!("{prefix}-{}", sub.get_name());
            paths(sub, &path, into);
            into.push(path);
        }
    }

    #[test]
    fn every_registered_key_names_a_command() {
        let mut known = Vec::new();
        paths(&Cli::command(), "cardamum", &mut known);

        for key in schemas().keys() {
            assert!(known.contains(key), "{key} names no command");
        }
    }
}
