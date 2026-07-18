use core::fmt;

use anyhow::Result;
use clap::Parser;
use io_msgraph::v1::rest::users::MsgraphUser;
use pimalaya_cli::printer::Printer;
use serde::Serialize;

use crate::msgraph::client::MsgraphClient;

/// GET the signed-in user (Graph `/me`).
///
/// JSON output: the raw Graph user object.
#[derive(Debug, Parser)]
pub struct MsgraphProfileGetCommand;

impl MsgraphProfileGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let user = client.me()?.response;
        printer.out(ProfileReport(user))
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct ProfileReport(pub MsgraphUser);

impl fmt::Display for ProfileReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let user = &self.0;
        writeln!(f, "id: {}", user.id)?;
        writeln!(
            f,
            "display-name: {}",
            user.display_name.as_deref().unwrap_or("")
        )?;
        writeln!(f, "mail: {}", user.mail.as_deref().unwrap_or(""))?;
        writeln!(
            f,
            "user-principal-name: {}",
            user.user_principal_name.as_deref().unwrap_or("")
        )
    }
}
