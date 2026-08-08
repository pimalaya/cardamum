use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::people::{
    client::PeopleClient, other_contact::fields::OTHER_CONTACT_FIELDS, project,
    render::PersonReport,
};

/// Copy an other contact into the user's contacts
/// (`otherContacts.copyOtherContactToMyContactsGroup`).
///
/// JSON output: the raw People person now in `myContacts`.
#[derive(Debug, Parser)]
pub struct PeopleOtherContactCopyCommand {
    /// Other-contact resource name (`otherContacts/<id>`), as shown in
    /// the ID column of `other-contact list`.
    #[arg(value_name = "RESOURCE-NAME")]
    pub resource_name: String,
}

impl PeopleOtherContactCopyCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: PeopleClient) -> Result<()> {
        let person = client
            .other_contact_copy(
                &self.resource_name,
                OTHER_CONTACT_FIELDS,
                project::READ_FIELDS,
                &[],
            )?
            .response;

        printer.out(PersonReport(person))
    }
}
