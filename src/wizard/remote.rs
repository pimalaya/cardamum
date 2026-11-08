//! # Remote account
//!
//! The setup of an email address, a bare domain or a server URL: discovery
//! under a spinner (see [`crate::wizard::search`]), the Auto pick and its
//! review (see [`crate::wizard::review`]), then the sign-in, tested as it
//! is typed (see [`crate::wizard::sign_in`]). Going back from a sign-in
//! returns to the review.
//!
//! Nothing discovered leads to the `Server URL:` prompt (see
//! [`crate::wizard::server`]), an empty answer stopping the wizard.

use anyhow::{Context, Result, bail};
use pimalaya_cli::{spinner::Spinner, wizard::keyring::TokenBroker};
use url::Url;

use crate::{
    config::{AccountConfig, CONFIG_SAMPLE_URL},
    wizard::{
        review,
        search::{self, Discovered, DiscoveredKind},
        server,
        sign_in::SignInAccount,
    },
};

/// Sets up the account `name` from a remote `input`, answering it signed
/// in and tested.
pub fn configure(input: &str, name: &str) -> Result<AccountConfig> {
    let remote = Remote::parse(input)?;
    let ortie = TokenBroker::Ortie.installed();
    let found = remote.discover(ortie);

    let mut pick = match found.first() {
        Some(auto) => auto.clone(),
        None => {
            eprintln!("Nothing could be discovered for {}", remote.domain);

            if !cfg!(any(feature = "carddav", feature = "jmap")) {
                return stop_undiscovered(input);
            }

            match server::prompt()? {
                Some(typed) => typed,
                None => return stop_undiscovered(input),
            }
        }
    };

    let account = SignInAccount {
        address: remote.address,
        domain: remote.domain,
        name: name.to_string(),
    };

    loop {
        pick = review::prompt(&found, pick, ortie)?;

        if let Some(config) = account.sign_in(&pick)? {
            return Ok(config);
        }
    }
}

/// A remote input, read for discovery and the sign-ins.
#[derive(Debug, Eq, PartialEq)]
struct Remote {
    /// The typed address, when one was typed.
    address: Option<String>,
    /// The address's domain, the bare domain, or the server URL's host.
    domain: String,
    /// The protocol a typed server URL's scheme keeps.
    only: Option<Only>,
}

/// The protocol a server URL typed as input narrows discovery to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Only {
    Carddav,
    Jmap,
}

impl Remote {
    /// Reads an address, a bare domain or a `scheme://` server URL.
    ///
    /// The HTTP family and `carddav(s)` keep CardDAV, `jmap(s)` keeps JMAP,
    /// and any other scheme is rejected outright.
    fn parse(input: &str) -> Result<Self> {
        if input.contains("://") {
            let url = Url::parse(input).with_context(|| format!("Invalid server URL `{input}`"))?;

            let only = match url.scheme() {
                "carddav" | "carddavs" | "http" | "https" => Only::Carddav,
                "jmap" | "jmaps" => Only::Jmap,
                other => bail!("Unsupported server scheme `{other}`"),
            };

            let Some(host) = url.host_str().filter(|host| !host.is_empty()) else {
                bail!("Server URL `{input}` names no host");
            };

            return Ok(Self {
                address: None,
                domain: host.to_string(),
                only: Some(only),
            });
        }

        match input.rsplit_once('@') {
            Some((local, domain)) => Ok(Self {
                address: (!local.is_empty()).then(|| input.to_string()),
                domain: domain.to_string(),
                only: None,
            }),
            None => Ok(Self {
                address: None,
                domain: input.to_string(),
                only: None,
            }),
        }
    }

    /// Runs discovery under a spinner, answering the options a typed
    /// scheme keeps, ranked, Auto first.
    ///
    /// A failed lookup, such as a resolver that does not answer, is
    /// reported and leaves nothing discovered.
    fn discover(&self, ortie: bool) -> Vec<Discovered> {
        let query = match &self.address {
            Some(address) => address.clone(),
            None => format!("@{}", self.domain),
        };

        let spinner = Spinner::start(format!("Looking up {}", self.domain));

        let mut found = match search::search(&query, ortie) {
            Ok(found) => found,
            Err(err) => {
                spinner.failure(format!("Cannot look up {}", self.domain));
                eprintln!("{err:#}");
                return Vec::new();
            }
        };

        if let Some(only) = self.only {
            found.retain(|option| only.keeps(&option.kind));
        }

        if found.is_empty() {
            spinner.clear();
        } else {
            spinner.success(format!("Looked up {}", self.domain));
        }

        found
    }
}

impl Only {
    /// Whether an option of `kind` is the protocol kept.
    fn keeps(self, kind: &DiscoveredKind) -> bool {
        matches!(
            (self, kind),
            (Self::Carddav, DiscoveredKind::Carddav(_)) | (Self::Jmap, DiscoveredKind::Jmap(_))
        )
    }
}

/// Stops the wizard when nothing was discovered for `input` and no server
/// was typed instead, pointing at the documented sample.
fn stop_undiscovered(input: &str) -> Result<AccountConfig> {
    bail!(
        "Could not automatically discover a configuration for `{input}`.\n\n\
         Write your account configuration by hand instead, starting from the \
         documented sample:\n  {CONFIG_SAMPLE_URL}"
    )
}

#[cfg(test)]
mod tests {
    use crate::wizard::remote::*;

    #[test]
    fn an_input_reads_as_an_address_a_domain_or_a_server() {
        assert_eq!(
            Remote::parse("alice@example.org").unwrap(),
            Remote {
                address: Some(String::from("alice@example.org")),
                domain: String::from("example.org"),
                only: None,
            }
        );
        assert_eq!(
            Remote::parse("@example.org").unwrap(),
            Remote {
                address: None,
                domain: String::from("example.org"),
                only: None,
            }
        );
        assert_eq!(
            Remote::parse("example.org").unwrap(),
            Remote {
                address: None,
                domain: String::from("example.org"),
                only: None,
            }
        );
        assert_eq!(
            Remote::parse("jmaps://api.example.org/jmap").unwrap(),
            Remote {
                address: None,
                domain: String::from("api.example.org"),
                only: Some(Only::Jmap),
            }
        );
        assert_eq!(
            Remote::parse("https://dav.example.org").unwrap().only,
            Some(Only::Carddav)
        );
    }

    #[test]
    fn an_unknown_scheme_is_rejected() {
        assert!(Remote::parse("imaps://imap.example.org").is_err());
    }
}
