//! # Secret prompts
//!
//! Turns what pimalaya-cli's secret prompts answer into the [`Secret`] the
//! configuration stores: [`store`] keeps a password or an API token once
//! it passed its test, [`broker_token`] names the OAuth 2.0 token broker
//! printing a fresh token, and [`to_secret`] converts any answer, such as
//! the argv Ortie reads its token with.

use anyhow::{Result, bail};
use pimalaya_cli::wizard::keyring::{self, SecretChoice};
use pimalaya_config::{command::CommandConfig, secret::Secret};
use secrecy::SecretString;

/// Asks where to keep a tested `secret`, storing it at `key` in the
/// keyring picked, and answers the [`Secret`] reading it back.
pub fn store(label: &str, key: &str, secret: &SecretString) -> Result<Secret> {
    to_secret(keyring::prompt_store(label, key, secret)?)
}

/// Prompts for the command reading an OAuth 2.0 token through the shared
/// picker, the token brokers among the keyrings, `account` seeding the
/// broker account and the keyring entry.
pub fn broker_token(label: &str, account: &str) -> Result<Secret> {
    to_secret(keyring::prompt_token(label, account, true)?)
}

/// Turns a picker choice into the [`Secret`] the configuration stores.
pub fn to_secret(choice: SecretChoice) -> Result<Secret> {
    Ok(match choice {
        SecretChoice::Command(argv) => command_secret(argv)?,
        SecretChoice::Shell(line) => shell_secret(&line)?,
        SecretChoice::Raw(secret) => Secret::Raw(secret),
    })
}

/// Builds a [`Secret::Command`] from an argv, the form a known keyring
/// provider or token broker yields.
///
/// It serializes back as a TOML array.
fn command_secret(argv: Vec<String>) -> Result<Secret> {
    let Some((program, args)) = argv.split_first() else {
        bail!("Empty command for secret");
    };

    Ok(Secret::Command(CommandConfig::Argv {
        program: program.clone(),
        args: args.to_vec(),
    }))
}

/// Builds a [`Secret::Command`] from a shell command line, the fallback
/// form a user typed by hand.
///
/// It serializes back as a TOML string.
fn shell_secret(line: &str) -> Result<Secret> {
    let line = line.trim();
    if line.is_empty() {
        bail!("Empty shell command for secret");
    }

    Ok(Secret::Command(CommandConfig::Shell(line.to_owned())))
}

#[cfg(test)]
mod tests {
    use crate::wizard::secret::*;

    #[test]
    fn empty_command_secret_is_rejected() {
        assert!(command_secret(Vec::new()).is_err());
    }

    #[test]
    fn blank_shell_secret_is_rejected() {
        assert!(shell_secret("   ").is_err());
    }
}
