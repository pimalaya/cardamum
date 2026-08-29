//! # Shared arguments
//!
//! Clap arguments reused across the shared API commands.

use clap::Parser;

/// Names the addressbook a shared API command operates on.
#[derive(Debug, Parser)]
pub struct AddressbookIdArg {
    /// Addressbook the command operates on.
    ///
    /// Falls back to the `addressbook.default` config when omitted,
    /// otherwise the command bails.
    #[arg(short = 'k', long = "addressbook", value_name = "ADDRESSBOOK-ID")]
    pub id: Option<String>,
}
