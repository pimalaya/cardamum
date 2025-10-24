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

use anyhow::Result;
use clap::Parser;
use pimalaya_toolbox::terminal::printer::Printer;

use crate::{account::Account, card::table::CardsTable, client::Client};

/// List all cards.
///
/// This command allows you to list vCards from a given addressbook.
#[derive(Debug, Parser)]
pub struct ListCardsCommand {
    /// The identifier of the CardDAV addressbook to list vCards from.
    #[arg(name = "ADDRESSBOOK-ID")]
    pub addressbook_id: String,
}

impl ListCardsCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let mut client = Client::new(&account)?;

        let cards = client.list_cards(self.addressbook_id)?;
        let table = CardsTable::from(cards);
        printer.out(table)
    }
}
