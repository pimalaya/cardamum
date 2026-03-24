// This file is part of Cardamum, a CLI to manage contacts.
//
// Copyright (C) 2025 soywod <clement.douin@posteo.net>
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU Affero General Public License
// as published by the Free Software Foundation, either version 3 of
// the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public
// License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use io_addressbook::card::Card;
use pimalaya_toolbox::terminal::printer::Printer;
use serde::Serialize;

use crate::{
    account::Account,
    card::utils::{build_group_names_by_cid, contact_group_names},
    client::Client,
};

/// Read the content of a card.
///
/// This command allows you to read the content of a vCard, from the
/// given addressbook.
#[derive(Debug, Parser)]
pub struct ReadCardCommand {
    /// The identifier of the addressbook where the vCard should be
    /// read from.
    #[arg(name = "ADDRESSBOOK-ID")]
    pub addressbook_id: String,

    /// The identifier of the card that should be read.
    #[arg(name = "CARD-ID")]
    pub id: String,

    /// Print the raw vCard only.
    #[arg(long)]
    pub raw: bool,
}

impl ReadCardCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let mut client = Client::new(&account)?;
        let card = client.read_card(&self.addressbook_id, &self.id)?;

        if self.raw {
            return printer.out(card.to_string().trim_end());
        }

        let cards = client.list_cards(&self.addressbook_id)?;
        let group_names_by_cid = build_group_names_by_cid(&cards);
        let output = ReadCardOutput {
            id: card.id.clone(),
            addressbook_id: card.addressbook_id.clone(),
            groups: contact_group_names(&card, &group_names_by_cid),
            vcard: card,
        };

        printer.out(output)
    }
}

#[derive(Clone, Debug, Serialize)]
struct ReadCardOutput {
    id: String,
    addressbook_id: String,
    groups: Vec<String>,
    #[serde(serialize_with = "serialize_card")]
    vcard: Card,
}

impl fmt::Display for ReadCardOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let groups = if self.groups.is_empty() {
            String::from("(none)")
        } else {
            self.groups.join(", ")
        };

        writeln!(f, "Groups: {groups}")?;
        writeln!(f)?;
        write!(f, "{}", self.vcard.to_string().trim_end())
    }
}

fn serialize_card<S>(card: &Card, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(card.to_string().trim_end())
}
