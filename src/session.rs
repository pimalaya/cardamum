//! # Session
//!
//! The account the commands of one run share, and the unified client kept
//! across them.
//!
//! A one-shot invocation runs a single command over a fresh [`Session`],
//! so it builds exactly the client that command needs. The [`crate::repl`]
//! keeps its session across lines and the [`crate::mcp`] server across
//! tool calls, so the unified client connects once. A command that fails
//! drops the client it ran on, its connection being possibly dead or left
//! mid-exchange, and the next command builds it again; a partial outcome
//! (a listing with `errors`) keeps it, the command having run to its end.
//! The protocol
//! commands build their own client each time.

use std::path::PathBuf;

use anyhow::Result;
use pimalaya_cli::{error::partial_outcome_of, printer::Printer};

use crate::{backend::Backend, cli::resolve_account, shared::client::AddressbookClient};

/// The clients of one run.
#[derive(Default)]
pub struct Session<'a> {
    /// The account every client of the session is built for.
    pub account: SessionAccount<'a>,
    /// The client of the unified commands, built on first use.
    pub unified: Option<AddressbookClient>,
}

/// The account a session runs against, as the global flags name it.
#[derive(Clone, Copy, Debug, Default)]
pub struct SessionAccount<'a> {
    /// The configuration paths, `-c` or the canonical ones.
    pub config_paths: &'a [PathBuf],
    /// The account `-a` names, the default one when `None`.
    pub name: Option<&'a str>,
    /// The backend `--backend` allows for the unified commands.
    pub backend: Backend,
}

impl Session<'_> {
    /// Runs a unified command over the kept client, building it when the
    /// session holds none, and dropping it when the command fails short of
    /// a partial outcome. A failure is explained by the client
    /// ([`AddressbookClient::explain`]).
    pub fn run_unified<P: Printer>(
        &mut self,
        printer: &mut P,
        command: impl FnOnce(&mut P, &mut AddressbookClient) -> Result<()>,
    ) -> Result<()> {
        let client = match self.unified.take() {
            Some(kept) => self.unified.insert(kept),
            None => {
                let account = self.account;
                let (config, name, account_config) =
                    resolve_account(account.config_paths, account.name)?;
                let client = AddressbookClient::new(config, name, account_config, account.backend)?;
                self.unified.insert(client)
            }
        };

        let result = command(printer, client).map_err(|err| client.explain(err));

        // NOTE: a partial outcome ran to its end, its connection whole.
        if let Err(err) = &result
            && partial_outcome_of(err).is_none()
        {
            self.unified = None;
        }

        result
    }
}

#[cfg(all(test, feature = "vdir"))]
mod tests {
    use anyhow::anyhow;
    use pimalaya_cli::{error::PartialOutcome, printer::CapturePrinter};

    use super::*;
    use crate::config::{AccountConfig, Config, VdirConfig};

    /// A session holding a vdir client over an empty home.
    fn session() -> Session<'static> {
        let account = AccountConfig {
            vdir: Some(VdirConfig {
                root: String::from("/nonexistent"),
            }),
            ..Default::default()
        };
        let client =
            AddressbookClient::new(Config::default(), "a".into(), account, Backend::Auto).unwrap();

        Session {
            unified: Some(client),
            ..Default::default()
        }
    }

    /// A partial outcome keeps the client, a failure drops it.
    #[test]
    fn a_partial_outcome_keeps_the_client_a_failure_drops_it() {
        let mut session = session();
        let mut printer = CapturePrinter::default();

        let partial =
            session.run_unified(&mut printer, |_, _| Err(PartialOutcome::default().into()));
        assert!(partial.is_err());
        assert!(session.unified.is_some());

        let failed = session.run_unified(&mut printer, |_, _| Err(anyhow!("Gone")));
        assert!(failed.is_err());
        assert!(session.unified.is_none());
    }
}
