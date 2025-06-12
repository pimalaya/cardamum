use anyhow::Result;
use clap::Parser;
use pimalaya_toolbox::terminal::printer::Printer;
use url::Url;

use crate::{account::Account, carddav::client::Client};

/// Discover addressbooks.
#[derive(Debug, Parser)]
pub struct DiscoverAddressbooksCommand {
    pub uri: Option<Url>,
}

impl DiscoverAddressbooksCommand {
    pub fn execute(self, printer: &mut impl Printer, mut account: Account) -> Result<()> {
        if let Some(uri) = self.uri {
            account.uri = uri;
        }

        let mut client = Client::new(&account)?;

        let path = client.current_user_principal()?;
        println!("current user principal: {path:?}");
        account.uri.set_path(&path);

        Ok(())
    }
}
