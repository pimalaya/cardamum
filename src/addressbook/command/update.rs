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
use io_addressbook::addressbook::Addressbook;
use pimalaya_toolbox::terminal::printer::{Message, Printer};

use crate::{account::Account, client::Client};

/// Update an addressbook.
///
/// This command allows you to update properties of an existing
/// addressbook (mostly the name, the description and the color).
#[derive(Debug, Parser)]
pub struct UpdateAddressbookCommand {
    pub id: String,
    #[arg(long, short)]
    pub name: Option<String>,
    #[arg(long, short, alias = "desc")]
    pub description: Option<String>,
    #[arg(long, short = 'C')]
    pub color: Option<String>,
}

impl UpdateAddressbookCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let mut client = Client::new(&account)?;

        let addressbook = Addressbook {
            id: self.id,
            display_name: self.name,
            description: self.description,
            color: self.color,
        };

        client.update_addressbook(addressbook)?;
        printer.out(Message::new("Addressbook successfully updated"))
    }
}
