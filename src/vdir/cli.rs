//! # Vdir command
//!
//! Declares the vdir command tree and dispatches each subcommand against
//! the account's vdir client.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::vdir::{
    client::VdirClient, create::VdirCollectionCreateCommand, delete::VdirCollectionDeleteCommand,
    item::cli::VdirItemCommand, list::VdirCollectionListCommand,
    rename::VdirCollectionRenameCommand,
};

/// Manage vdir collections and items on the filesystem.
///
/// Operates on the raw vdir API rooted at the active account's
/// `vdir.root`. The flat verbs act on collections, which are
/// directories, and `item` on the raw item files inside them.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum VdirCommand {
    Create(VdirCollectionCreateCommand),
    Rename(VdirCollectionRenameCommand),
    Delete(VdirCollectionDeleteCommand),
    List(VdirCollectionListCommand),
    #[command(subcommand, visible_alias = "items")]
    Item(VdirItemCommand),
}

impl VdirCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        match self {
            Self::Create(cmd) => cmd.execute(printer, client),
            Self::Rename(cmd) => cmd.execute(printer, client),
            Self::Delete(cmd) => cmd.execute(printer, client),
            Self::List(cmd) => cmd.execute(printer, client),
            Self::Item(cmd) => cmd.execute(printer, client),
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use crate::cli::Cli;

    /// The collection commands name their arguments as calendula's do:
    /// the directory is an id, the display name a flag of its own.
    #[test]
    fn the_collection_arguments_read_as_the_twin() {
        let mut cli = Cli::command();
        cli.build();
        let vdir = cli.find_subcommand_mut("vdir").unwrap();

        for (verb, synopsis) in [
            ("create", "vdir create [OPTIONS] <ID>"),
            ("rename", "vdir rename [OPTIONS] <OLD-ID> <NEW-ID>"),
            ("delete", "vdir delete [OPTIONS] <ID>"),
        ] {
            let cmd = vdir.find_subcommand_mut(verb).unwrap();
            let usage = cmd.render_usage().to_string();
            assert!(usage.ends_with(synopsis), "{usage}");
        }

        let create = vdir.find_subcommand("create").unwrap();
        let name = create
            .get_arguments()
            .find(|arg| arg.get_id() == "display_name")
            .unwrap();
        assert_eq!(name.get_short(), Some('n'));
        assert_eq!(name.get_long(), Some("display-name"));
        assert_eq!(name.get_value_names().unwrap(), ["NAME"]);
    }
}
