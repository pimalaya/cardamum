//! # Account discovery
//!
//! The half of the wizard that decides what the account is: one prompt
//! takes an email address, a server URL or a local folder path, and its
//! shape orients the setup.
//!
//! A folder is a local vdir or pimdir store, tested once configured. An
//! email, a bare domain or a server URL is a remote account (see
//! [`crate::wizard::remote`]): discovery picks the best option, the review
//! shows it, and the sign-in is tested as it is typed.
//!
//! What becomes of the account, a file to create, a block to append or a
//! document on stdout, belongs to [`crate::wizard::configure`], which is
//! also where the welcome and the prompts around this one live.

use std::path::Path;

use anyhow::{Result, bail};
use pimalaya_cli::{prompt, spinner::Spinner};
use url::Url;

#[cfg(any(feature = "vdir", feature = "pimdir"))]
use crate::wizard::local;
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
use crate::wizard::remote;
use crate::{account::check, config::AccountConfig};

/// The label of the single wizard prompt.
const ENDPOINT_PROMPT: &str = "Email:";

/// Asks the one question the wizard starts with: an email address, a
/// server URL or a folder path.
pub fn prompt_input() -> Result<String> {
    let input = prompt::text::<&str>(ENDPOINT_PROMPT, None)?;
    let input = input.trim();

    if input.is_empty() {
        bail!("Empty input: enter an email address, a server URL, or a folder path");
    }

    Ok(input.to_string())
}

/// Configures the account `name` from `input`, answering it tested and
/// left non-default.
///
/// Claiming the default depends on the configuration the account lands
/// in, [`crate::wizard::configure`]'s business.
#[cfg_attr(
    not(any(
        feature = "carddav",
        feature = "jmap",
        feature = "msgraph",
        feature = "gpeople"
    )),
    allow(unused_variables)
)]
pub fn run(input: &str, name: &str) -> Result<AccountConfig> {
    match is_path(input) {
        true => {
            let account = configure_local(input)?;

            let spinner = Spinner::start("Testing account configuration");

            if let Err(err) = check::test_account(&account) {
                spinner.failure("Account configuration test failed");
                return Err(err);
            }

            spinner.success("Account configuration is valid");

            Ok(account)
        }
        #[cfg(any(
            feature = "carddav",
            feature = "jmap",
            feature = "msgraph",
            feature = "gpeople"
        ))]
        false => remote::configure(input, name),
        #[cfg(not(any(
            feature = "carddav",
            feature = "jmap",
            feature = "msgraph",
            feature = "gpeople"
        )))]
        false => bail!("`{input}` needs a network backend, but none is compiled in"),
    }
}

/// Proposes the account name from the input shape: the first label of the
/// domain, or the folder name of a local path.
pub fn default_account_name(input: &str) -> String {
    if is_path(input) {
        let raw = input.strip_prefix("file://").unwrap_or(input);
        return Path::new(raw)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("personal")
            .to_string();
    }

    if let Ok(url) = Url::parse(input)
        && let Some(host) = url.host_str()
    {
        return first_label(host);
    }

    match input.rsplit_once('@') {
        Some((_, domain)) => first_label(domain),
        None => first_label(input),
    }
}

/// Configures a local backend from a typed folder path.
#[cfg(any(feature = "vdir", feature = "pimdir"))]
fn configure_local(input: &str) -> Result<AccountConfig> {
    let raw = input.strip_prefix("file://").unwrap_or(input);
    let root = shellexpand::tilde(raw).into_owned();
    if !Path::new(&root).is_dir() {
        bail!("No such folder `{raw}`");
    }

    let mut account = AccountConfig::default();

    match local::configure(root.into())? {
        #[cfg(feature = "vdir")]
        local::Local::Vdir(config) => account.vdir = Some(config),
        #[cfg(feature = "pimdir")]
        local::Local::Pimdir(config) => account.pimdir = Some(config),
    }

    Ok(account)
}

/// Rejects a typed folder path, no local backend being compiled in.
#[cfg(not(any(feature = "vdir", feature = "pimdir")))]
fn configure_local(input: &str) -> Result<AccountConfig> {
    bail!("`{input}` looks like a folder path, but no local backend is compiled in")
}

/// The first dot-separated label of a host or domain.
fn first_label(host: &str) -> String {
    host.split('.').next().unwrap_or(host).to_string()
}

/// Whether the input names a filesystem path (absolute, home-relative,
/// explicitly relative, or a `file://` URL) rather than an endpoint.
fn is_path(input: &str) -> bool {
    input.starts_with("file://")
        || input.starts_with('/')
        || input.starts_with('~')
        || input.starts_with("./")
        || input.starts_with("../")
}

#[cfg(test)]
mod tests {
    use crate::wizard::discover::*;

    #[test]
    fn account_name_defaults_to_the_first_domain_label() {
        assert_eq!(default_account_name("clement.douin@posteo.net"), "posteo");
        assert_eq!(default_account_name("alice@mail.example.co.uk"), "mail");
        assert_eq!(default_account_name("@posteo.net"), "posteo");
        assert_eq!(default_account_name("posteo.net"), "posteo");
    }

    #[test]
    fn account_name_defaults_to_the_last_path_component() {
        assert_eq!(
            default_account_name("/home/alice/contacts/personal"),
            "personal"
        );
        assert_eq!(default_account_name("~/contacts/work"), "work");
        assert_eq!(
            default_account_name("file:///var/contacts/archive"),
            "archive"
        );
    }
}
