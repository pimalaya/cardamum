//! # JSON Schema registry
//!
//! Maps a CLI-invocation key, the command path joined with hyphens and
//! prefixed `cardamum-`, to the JSON Schema of that command's `--json`
//! payload. [`JsonSchemaCommand`] writes one file per entry.
//!
//! Every runnable command appears, a write by its receipt, and a command
//! printing one of two types is described by both, through `OneOf`, and a
//! protocol command, whose server's refusal is its result, by its own
//! type or the refusal, through `Refusable`.
//!
//! Protocol-specific entries are gated behind the same cargo features as
//! their command modules, so the registry stays coherent under any
//! feature combination, none included.
//!
//! [`JsonSchemaCommand`]: pimalaya_cli::clap::commands::json_schema::JsonSchemaCommand

#[cfg(any(feature = "gpeople", feature = "jmap", feature = "msgraph"))]
use pimalaya_cli::output::Paginated;
use pimalaya_cli::{clap::commands::json_schema::JsonSchemaRegistry, error::ErrorReport};
use schemars::JsonSchema;

use crate::error::ErrorCode;
#[cfg(any(
    feature = "carddav",
    feature = "gpeople",
    feature = "jmap",
    feature = "msgraph",
    feature = "vdir"
))]
use crate::shared::receipt::IdReceipt;

/// Builds the command-to-schema registry consumed by `json-schema <DIR>`.
///
/// Each value describes the type the command hands to the printer,
/// `cardamum-error` the failure object every command prints instead, and
/// `cardamum-item-error` the entry of a listing's `errors`.
pub fn registry() -> JsonSchemaRegistry {
    let mut registry = JsonSchemaRegistry::new("cardamum", env!("CARGO_PKG_VERSION"));

    macro_rules! insert {
        ($key:expr, $ty:ty) => {
            registry.insert::<$ty>($key);
        };
    }

    #[cfg(any(
        feature = "carddav",
        feature = "gpeople",
        feature = "jmap",
        feature = "msgraph"
    ))]
    macro_rules! refusable {
        ($key:expr, $ty:ty) => {
            registry.insert::<crate::shared::refusal::Refusable<$ty>>($key);
        };
    }

    insert!(
        "cardamum-account-capability",
        pimalaya_cli::capability::AccountCapabilityOutput
    );
    insert!(
        "cardamum-account-list",
        pimalaya_cli::account::AccountListOutput
    );
    insert!(
        "cardamum-account-check",
        pimalaya_cli::account::AccountCheckOutput<ErrorCode>
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
    // NOTE: a build prints the card, or under `-o` the file written.
    insert!(
        "cardamum-card-build",
        OneOf<crate::shared::card::build::CardBuildOutput, pimalaya_cli::output::SavedOutput>
    );
    insert!("cardamum-card-add", crate::shared::card::add::CardAddOutput);
    insert!(
        "cardamum-card-update",
        crate::shared::card::update::CardUpdateOutput
    );
    for key in [
        "cardamum-addressbook-update",
        "cardamum-addressbook-delete",
        "cardamum-card-delete",
    ] {
        insert!(key, crate::shared::receipt::WriteReceipt);
    }

    #[cfg(feature = "carddav")]
    {
        refusable!(
            "cardamum-carddav-discover",
            crate::carddav::discover::CarddavDiscoverOutput
        );
        refusable!(
            "cardamum-carddav-propfind",
            crate::carddav::propfind::CarddavPropfindOutput
        );
        refusable!(
            "cardamum-carddav-get",
            crate::carddav::get::CarddavGetOutput
        );
        // NOTE: `report query` and `report multiget` both answer with the
        // card table of `report::entries`.
        refusable!(
            "cardamum-carddav-report-query",
            crate::carddav::report::entries::CarddavCardEntriesOutput
        );
        refusable!(
            "cardamum-carddav-report-multiget",
            crate::carddav::report::entries::CarddavCardEntriesOutput
        );
        refusable!(
            "cardamum-carddav-report-sync",
            crate::carddav::report::sync::CarddavReportSyncOutput
        );
        refusable!(
            "cardamum-carddav-report-raw",
            crate::carddav::report::raw::CarddavReportRawOutput
        );
        refusable!(
            "cardamum-carddav-put",
            crate::carddav::put::CarddavPutOutput
        );
        for key in [
            "cardamum-carddav-mkcol",
            "cardamum-carddav-rmcol",
            "cardamum-carddav-proppatch",
            "cardamum-carddav-delete",
        ] {
            refusable!(key, IdReceipt);
        }
    }

    #[cfg(feature = "jmap")]
    {
        use crate::jmap::render::{
            JmapAddressBookOutput, JmapAddressBooksOutput, JmapChangesOutput,
            JmapContactCardOutput, JmapContactCardsOutput,
        };

        refusable!("cardamum-jmap-address-book-get", JmapAddressBooksOutput);
        refusable!("cardamum-jmap-address-book-create", JmapAddressBookOutput);
        // NOTE: an update the server answers with no changed property
        // prints its receipt.
        refusable!(
            "cardamum-jmap-address-book-update",
            OneOf<JmapAddressBookOutput, IdReceipt>
        );
        refusable!("cardamum-jmap-address-book-destroy", IdReceipt);
        refusable!("cardamum-jmap-address-book-changes", JmapChangesOutput);
        refusable!("cardamum-jmap-contact-card-get", JmapContactCardsOutput);
        refusable!(
            "cardamum-jmap-contact-card-query",
            Paginated<JmapContactCardsOutput>
        );
        refusable!("cardamum-jmap-contact-card-create", JmapContactCardOutput);
        refusable!(
            "cardamum-jmap-contact-card-update",
            OneOf<JmapContactCardOutput, IdReceipt>
        );
        refusable!("cardamum-jmap-contact-card-destroy", IdReceipt);
        refusable!(
            "cardamum-jmap-contact-card-copy",
            crate::jmap::set::JmapSetReceipt
        );
        refusable!("cardamum-jmap-contact-card-changes", JmapChangesOutput);
        refusable!(
            "cardamum-jmap-session-get",
            crate::jmap::session::get::JmapSessionGetOutput
        );
        refusable!(
            "cardamum-jmap-request",
            crate::shared::raw_json::RawJsonOutput
        );
    }

    #[cfg(feature = "msgraph")]
    {
        use crate::msgraph::{
            contact_folders::render::{MsgraphContactFolderOutput, MsgraphContactFoldersOutput},
            contacts::render::{MsgraphContactGetOutput, MsgraphContactsOutput},
        };

        refusable!(
            "cardamum-msgraph-contact-folders-list",
            Paginated<MsgraphContactFoldersOutput>
        );
        refusable!(
            "cardamum-msgraph-contact-folders-child-folders",
            Paginated<MsgraphContactFoldersOutput>
        );
        refusable!(
            "cardamum-msgraph-contact-folders-get",
            crate::msgraph::contact_folders::render::MsgraphContactFolderGetOutput
        );
        refusable!(
            "cardamum-msgraph-contact-folders-create",
            MsgraphContactFolderOutput
        );
        refusable!(
            "cardamum-msgraph-contact-folders-rename",
            MsgraphContactFolderOutput
        );
        refusable!(
            "cardamum-msgraph-contacts-list",
            Paginated<MsgraphContactsOutput>
        );
        refusable!("cardamum-msgraph-contacts-get", MsgraphContactGetOutput);
        refusable!("cardamum-msgraph-contacts-create", MsgraphContactGetOutput);
        refusable!("cardamum-msgraph-contacts-update", MsgraphContactGetOutput);
        refusable!(
            "cardamum-msgraph-contacts-delta",
            Paginated<crate::msgraph::contacts::delta::MsgraphContactDeltaOutput>
        );
        refusable!(
            "cardamum-msgraph-profile-get",
            crate::msgraph::profile::get::MsgraphProfileGetOutput
        );
        refusable!(
            "cardamum-msgraph-request",
            crate::shared::raw_json::RawJsonOutput
        );
        refusable!("cardamum-msgraph-contacts-delete", IdReceipt);
        refusable!("cardamum-msgraph-contact-folders-delete", IdReceipt);
    }

    #[cfg(feature = "gpeople")]
    {
        use crate::gpeople::render::{
            GpeopleContactGroupOutput, GpeopleContactGroupsOutput, GpeoplePersonOutput,
            GpeoplePersonsOutput,
        };

        refusable!(
            "cardamum-gpeople-contact-groups-list",
            Paginated<GpeopleContactGroupsOutput>
        );
        refusable!(
            "cardamum-gpeople-contact-groups-get",
            crate::gpeople::render::GpeopleContactGroupGetOutput
        );
        refusable!(
            "cardamum-gpeople-contact-groups-create",
            GpeopleContactGroupOutput
        );
        refusable!(
            "cardamum-gpeople-contact-groups-update",
            GpeopleContactGroupOutput
        );
        refusable!(
            "cardamum-gpeople-connections-list",
            Paginated<GpeoplePersonsOutput>
        );
        refusable!(
            "cardamum-gpeople-connections-get",
            crate::gpeople::render::GpeoplePersonGetOutput
        );
        refusable!("cardamum-gpeople-connections-create", GpeoplePersonOutput);
        refusable!("cardamum-gpeople-connections-update", GpeoplePersonOutput);
        refusable!("cardamum-gpeople-connections-search", GpeoplePersonsOutput);
        refusable!(
            "cardamum-gpeople-other-contacts-list",
            Paginated<GpeoplePersonsOutput>
        );
        refusable!(
            "cardamum-gpeople-other-contacts-search",
            GpeoplePersonsOutput
        );
        refusable!("cardamum-gpeople-other-contacts-copy", GpeoplePersonOutput);
        refusable!(
            "cardamum-gpeople-profile-get",
            crate::gpeople::render::GpeoplePersonGetOutput
        );
        refusable!(
            "cardamum-gpeople-request",
            crate::shared::raw_json::RawJsonOutput
        );
        refusable!("cardamum-gpeople-connections-delete", IdReceipt);
        refusable!("cardamum-gpeople-contact-groups-delete", IdReceipt);
        refusable!(
            "cardamum-gpeople-contact-groups-members",
            crate::gpeople::contact_group::members::GpeopleMembersOutput
        );
    }

    #[cfg(feature = "pimdir")]
    {
        insert!(
            "cardamum-pimdir-status",
            crate::pimdir::status::PimdirStatusOutput
        );
        insert!(
            "cardamum-pimdir-queue-list",
            crate::pimdir::queue::list::PimdirQueueListOutput
        );
        insert!(
            "cardamum-pimdir-queue-show",
            crate::pimdir::queue::show::PimdirQueueShowOutput
        );
        insert!(
            "cardamum-pimdir-queue-cancel",
            crate::pimdir::queue::cancel::PimdirQueueCancelOutput
        );
        insert!(
            "cardamum-pimdir-search",
            crate::pimdir::search::PimdirSearchOutput
        );
        insert!(
            "cardamum-pimdir-count",
            crate::pimdir::count::PimdirCountOutput
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
            "cardamum-vdir-item-add",
            crate::vdir::item::add::VdirItemAddOutput
        );
        for key in [
            "cardamum-vdir-create",
            "cardamum-vdir-rename",
            "cardamum-vdir-delete",
            "cardamum-vdir-item-update",
            "cardamum-vdir-item-copy",
            "cardamum-vdir-item-move",
            "cardamum-vdir-item-delete",
        ] {
            insert!(key, IdReceipt);
        }
    }

    insert!("cardamum-error", ErrorReport<ErrorCode>);
    insert!("cardamum-item-error", crate::error::ItemError);

    registry
}

/// The schema of a command printing one of two types, told apart by their
/// keys.
#[derive(JsonSchema)]
#[serde(untagged)]
#[allow(dead_code)]
enum OneOf<A, B> {
    A(A),
    B(B),
}

#[cfg(test)]
mod tests {
    use clap::{Command, CommandFactory};

    use super::registry;
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

        // NOTE: the failure object every command prints and the item
        // error a listing carries, not commands.
        known.push("cardamum-error".into());
        known.push("cardamum-item-error".into());

        for key in registry().schemas.keys() {
            assert!(known.contains(key), "{key} names no command");
        }
    }

    #[test]
    fn every_payload_is_an_object() {
        assert_eq!(registry().non_object_roots(), Vec::<String>::new());
    }

    /// The commands printing nothing a schema could describe.
    const SILENT: &[&str] = &["cardamum", "cardamum-repl", "cardamum-mcp"];

    #[test]
    fn every_command_is_described() {
        let problems = registry().check(&Cli::command(), SILENT);
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }
}
