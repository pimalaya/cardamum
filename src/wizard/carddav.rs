//! # CardDAV wizard
//!
//! A discovery entry pins the context root, the HTTP authentication
//! scheme is picked among the advertised ones (skipped when only one
//! qualifies), then its credentials are prompted.
//!
//! It does not connect: the wizard validates the whole account once at
//! the end (see [`crate::account::check`]), and the runtime walks the
//! principal and the addressbook-home-set from the stored `server`.

use anyhow::Result;
use pimalaya_cli::prompt;
use url::Url;

use crate::{
    config::{CarddavAuthConfig, CarddavConfig},
    wizard::{
        search::{AuthCaps, Discovered},
        secret,
    },
};

const BASIC: &str = "Basic (username + password)";
const BEARER: &str = "Bearer (API token)";

/// Configures CardDAV from a discovered entry, prompting credentials.
pub fn configure_discovered(
    account_name: &str,
    email: &str,
    url: &Url,
    discovered: &Discovered,
) -> Result<CarddavConfig> {
    let auth = prompt_auth(
        account_name,
        discovered.login_default(email).as_deref(),
        discovered.auth,
    )?;

    Ok(CarddavConfig {
        discover: None,
        server: Some(url.to_string()),
        home: None,
        tls: Default::default(),
        auth,
    })
}

/// Prompts the authentication scheme from `caps`, then its credentials.
///
/// Both schemes are offered when none was advertised, and the token flow
/// shows the OAuth brokers only when a grant was.
fn prompt_auth(
    account_name: &str,
    login_hint: Option<&str>,
    caps: AuthCaps,
) -> Result<CarddavAuthConfig> {
    let mut schemes = Vec::new();
    if caps.basic || !caps.any() {
        schemes.push(BASIC);
    }
    if caps.token() || !caps.any() {
        schemes.push(BEARER);
    }

    let scheme = if schemes.len() == 1 {
        schemes[0]
    } else {
        prompt::item("CardDAV authentication:", schemes, None)?
    };

    let key = format!("{account_name}-carddav");
    Ok(match scheme {
        BASIC => {
            let username = prompt::text("Login:", login_hint)?;
            let password = secret::configure_password("CardDAV password", &key)?;
            CarddavAuthConfig::Basic { username, password }
        }
        BEARER => {
            let token =
                secret::configure_token("CardDAV API token", &key, caps.oauth || !caps.any())?;
            CarddavAuthConfig::Bearer { token }
        }
        _ => unreachable!(),
    })
}
