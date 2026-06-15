use clap::Parser;

/// Shared `-k/--addressbook` argument naming the addressbook a
/// shared-API command operates on. Resolve the id through the account
/// (the flag wins, otherwise `addressbook.default`, otherwise bail).
#[derive(Debug, Parser)]
pub struct AddressbookIdArg {
    /// Addressbook the command operates on. Falls back to the
    /// `addressbook.default` config when omitted, otherwise the command
    /// bails.
    #[arg(short = 'k', long = "addressbook", value_name = "ADDRESSBOOK-ID")]
    pub id: Option<String>,
}
