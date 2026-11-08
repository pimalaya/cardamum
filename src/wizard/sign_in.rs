//! # Sign-in
//!
//! Signs the reviewed pick in and tests it at once. A password or an API
//! token is typed once, shown where it goes, tested, and only then stored
//! where the user picks. OAuth hands off to Ortie, or without it to the
//! token broker the user names. A failed test offers a way on: try again,
//! change the login, or go back to the review.

use std::fmt;

use anyhow::{Error, Result, bail};
#[cfg(feature = "jmap")]
use io_jmap::client::JmapClientStd;
use pimalaya_cli::{
    prompt,
    spinner::Spinner,
    wizard::{
        keyring::{self, SecretChoice},
        ortie::{self, OrtieSignInError},
    },
};
use pimalaya_config::secret::Secret;
use secrecy::SecretString;

#[cfg(feature = "carddav")]
use crate::config::{CarddavAuthConfig, CarddavConfig};
#[cfg(feature = "gpeople")]
use crate::config::{GpeopleAuthConfig, GpeopleConfig};
#[cfg(feature = "jmap")]
use crate::config::{JmapAuthConfig, JmapConfig};
#[cfg(feature = "msgraph")]
use crate::config::{MsgraphAuthConfig, MsgraphConfig};
use crate::{
    account::check,
    config::AccountConfig,
    wizard::{
        search::{Discovered, DiscoveredKind, SignIn},
        secret,
    },
};

/// The account a sign-in is for, as the input and the derived name tell
/// it.
pub struct SignInAccount {
    /// The typed address, when one was typed.
    pub address: Option<String>,
    /// The domain of the input, which a server off it is flagged against.
    pub domain: String,
    /// The account name, keying the keyring entries and the Ortie account.
    pub name: String,
}

impl SignInAccount {
    /// Signs `pick` in and tests it, answering the account to save, or
    /// `None` when the user goes back to the review.
    pub fn sign_in(&self, pick: &Discovered) -> Result<Option<AccountConfig>> {
        match pick.sign_in {
            SignIn::Password => self.password(pick),
            SignIn::Token => self.token(pick),
            SignIn::Oauth => self.oauth(pick),
        }
    }

    /// Asks the password once, the login unless one is known, tests both,
    /// then stores the password.
    fn password(&self, pick: &Discovered) -> Result<Option<AccountConfig>> {
        let mut username = match pick.login(self.address.as_deref()) {
            Some(login) => login,
            None => prompt::text("Login:", None)?,
        };

        loop {
            let password = self.prompt_secret("Password", pick)?;
            let credential = Credential::Basic {
                username: username.clone(),
                password: Secret::Raw(password.clone()),
            };

            if !test(pick, &account(&pick.kind, credential)?) {
                match Refusal::prompt(true)? {
                    Refusal::TryAgain => continue,
                    Refusal::ChangeLogin => {
                        username = prompt::text("Login:", Some(username.as_str()))?;
                        continue;
                    }
                    Refusal::Back => return Ok(None),
                }
            }

            let password = secret::store("Password", &self.key(pick), &password)?;
            let credential = Credential::Basic { username, password };

            return account(&pick.kind, credential).map(Some);
        }
    }

    /// Asks the API token once, tests it, then stores it.
    fn token(&self, pick: &Discovered) -> Result<Option<AccountConfig>> {
        loop {
            let token = self.prompt_secret("API token", pick)?;
            let credential = Credential::Bearer(Secret::Raw(token.clone()));

            if !test(pick, &account(&pick.kind, credential)?) {
                match Refusal::prompt(false)? {
                    Refusal::Back => return Ok(None),
                    _ => continue,
                }
            }

            let token = secret::store("API token", &self.key(pick), &token)?;

            return account(&pick.kind, Credential::Bearer(token)).map(Some);
        }
    }

    /// Signs in through Ortie, or through the broker the user names when
    /// Ortie is not installed, then tests the token.
    fn oauth(&self, pick: &Discovered) -> Result<Option<AccountConfig>> {
        let name = format!("cardamum-{}", self.name);
        let input = self.address.as_deref().unwrap_or(&self.domain);

        loop {
            let token = match ortie::sign_in(input, &name, &[pick.kind.service()]) {
                Ok(argv) => secret::to_secret(SecretChoice::Command(argv))?,
                Err(OrtieSignInError::NotInstalled) => {
                    eprintln!(
                        "An OAuth 2.0 token broker issues this token, such as Ortie: \
                         https://github.com/pimalaya/ortie"
                    );

                    secret::broker_token("OAuth token", &name)?
                }
                Err(err) => {
                    eprintln!("{:#}", Error::from(err));

                    match Refusal::prompt(false)? {
                        Refusal::Back => return Ok(None),
                        _ => continue,
                    }
                }
            };

            let account = account(&pick.kind, Credential::Bearer(token))?;

            if !test(pick, &account) {
                match Refusal::prompt(false)? {
                    Refusal::Back => return Ok(None),
                    _ => continue,
                }
            }

            return Ok(Some(account));
        }
    }

    /// Asks a password or an API token once, after naming where it goes.
    fn prompt_secret(&self, label: &str, pick: &Discovered) -> Result<SecretString> {
        let endpoints: Vec<&str> = pick
            .kind
            .server()
            .map(|url| url.as_str())
            .into_iter()
            .collect();
        let secret = keyring::prompt_password(label, &endpoints, Some(&self.domain))?;

        Ok(secret)
    }

    /// The keyring key a typed secret is stored at, as
    /// `cardamum-example-carddav`.
    fn key(&self, pick: &Discovered) -> String {
        format!("cardamum-{}-{}", self.name, pick.kind.service())
    }
}

/// What a sign-in produced, as a backend's authentication takes it.
#[cfg_attr(not(any(feature = "carddav", feature = "jmap")), allow(dead_code))]
enum Credential {
    /// A login and its password, sent as HTTP Basic.
    Basic { username: String, password: Secret },
    /// A token, sent as HTTP Bearer.
    Bearer(Secret),
}

/// The way on after a failed sign-in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Refusal {
    TryAgain,
    ChangeLogin,
    Back,
}

impl Refusal {
    /// Asks the way on, offering to change the login when there is one.
    fn prompt(login: bool) -> Result<Self> {
        let mut items = vec![Self::TryAgain];

        if login {
            items.push(Self::ChangeLogin);
        }

        items.push(Self::Back);

        Ok(prompt::item("What next?", items, None)?)
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TryAgain => f.write_str("Try again"),
            Self::ChangeLogin => f.write_str("Change the login"),
            Self::Back => f.write_str("Back to the review"),
        }
    }
}

/// Tests `account` under a spinner, printing why it failed when it did.
fn test(pick: &Discovered, account: &AccountConfig) -> bool {
    let place = pick.kind.place();
    let spinner = Spinner::start(format!("Signing in to {place}"));

    match check::test_account(account) {
        Ok(()) => {
            spinner.success(format!("Signed in to {place}"));
            true
        }
        Err(err) => {
            spinner.failure(format!("Cannot sign in to {place}"));
            eprintln!("{err:#}");
            false
        }
    }
}

/// The account reaching `kind` with `credential`, left non-default.
#[cfg_attr(
    all(
        feature = "carddav",
        feature = "jmap",
        feature = "msgraph",
        feature = "gpeople"
    ),
    allow(unreachable_patterns)
)]
fn account(kind: &DiscoveredKind, credential: Credential) -> Result<AccountConfig> {
    let mut account = AccountConfig::default();

    match kind {
        #[cfg(feature = "carddav")]
        DiscoveredKind::Carddav(url) => {
            let auth = match credential {
                Credential::Basic { username, password } => {
                    CarddavAuthConfig::Basic { username, password }
                }
                Credential::Bearer(token) => CarddavAuthConfig::Bearer { token },
            };

            account.carddav = Some(CarddavConfig {
                discover: None,
                server: Some(url.to_string()),
                home: None,
                tls: Default::default(),
                proxy: None,
                auth,
            });
        }
        #[cfg(feature = "jmap")]
        DiscoveredKind::Jmap(url) => {
            let auth = match credential {
                Credential::Basic { username, password } => {
                    JmapAuthConfig::Basic { username, password }
                }
                Credential::Bearer(token) => JmapAuthConfig::Bearer { token },
            };

            account.jmap = Some(JmapConfig {
                server: url.to_string(),
                tls: Default::default(),
                proxy: None,
                alpn: JmapClientStd::default_alpn(),
                auth,
            });
        }
        #[cfg(feature = "msgraph")]
        DiscoveredKind::Msgraph => {
            let Credential::Bearer(token) = credential else {
                bail!("Microsoft Graph signs in with a token only");
            };

            account.msgraph = Some(MsgraphConfig {
                user_id: String::from("me"),
                tls: Default::default(),
                proxy: None,
                alpn: vec![String::from("http/1.1")],
                auth: MsgraphAuthConfig { token },
            });
        }
        #[cfg(feature = "gpeople")]
        DiscoveredKind::Gpeople => {
            let Credential::Bearer(token) = credential else {
                bail!("The People API signs in with a token only");
            };

            account.gpeople = Some(GpeopleConfig {
                tls: Default::default(),
                proxy: None,
                alpn: vec![String::from("http/1.1")],
                auth: GpeopleAuthConfig { token },
            });
        }
        kind => bail!("{} is not compiled in this build", kind.protocol()),
    }

    Ok(account)
}
