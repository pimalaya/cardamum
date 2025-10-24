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

use std::process::exit;

use anyhow::Result;
use clap::Parser;
use pimalaya_toolbox::terminal::{
    printer::{Message, Printer},
    prompt,
};

use crate::{account::Account, client::Client};

/// Delete an addressbook.
///
/// This command allows you to delete an existing addressbook, by its
/// ID.
#[derive(Debug, Parser)]
pub struct DeleteAddressbookCommand {
    pub id: String,
    #[arg(long, short)]
    pub yes: bool,
}

impl DeleteAddressbookCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        if !self.yes {
            let confirm = "Do you really want to delete this addressbook";
            let confirm = format!("{confirm}? All contacts will be definitely deleted.");

            if !prompt::bool(confirm, false)? {
                exit(0);
            };
        };

        let mut client = Client::new(&account)?;

        client.delete_addressbook(self.id)?;
        printer.out(Message::new("Addressbook successfully deleted"))
    }
}
