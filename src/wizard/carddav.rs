//! CardDAV wizard.
//!
//! Two entry points, one per way the endpoint is known. A discovery
//! entry pins the context root and the authentication method, so
//! [`configure_discovered`] prompts only the credentials.
//! [`configure_manual`] handles a typed server URL: it prompts the
//! authentication strategy too. Neither connects; the wizard validates
//! the whole account once at the end (see [`crate::account::check`]),
//! and the runtime walks the principal + addressbook-home-set from the
//! stored `server`.

use anyhow::Result;
use pimalaya_cli::prompt;
use url::Url;

use crate::{
    config::{CarddavAuthConfig, CarddavConfig},
    wizard::{
        search::{Discovered, DiscoveredAuth},
        secret,
    },
};

const BASIC: &str = "Basic (username + password)";
const BEARER: &str = "Bearer (API token)";
const AUTHS: [&str; 2] = [BASIC, BEARER];

/// Configures CardDAV from a discovered entry: the context root and the
/// authentication method are pinned, only the credentials are prompted.
pub fn configure_discovered(
    email: &str,
    url: &Url,
    discovered: &Discovered,
) -> Result<CarddavConfig> {
    let auth = match discovered.auth {
        DiscoveredAuth::Password => {
            let default_login = discovered.login_default(email);
            let username = prompt::text("CardDAV username:", default_login.as_deref())?;
            let password = secret::configure("CardDAV password", None)?;
            CarddavAuthConfig::Basic { username, password }
        }
        DiscoveredAuth::Token => {
            let token = secret::configure("CardDAV API token", Some("ortie token show"))?;
            CarddavAuthConfig::Bearer { token }
        }
    };

    Ok(carddav_config(url, auth))
}

/// Configures CardDAV against a typed `server` URL, prompting the
/// authentication strategy and credentials.
pub fn configure_manual(server: &Url) -> Result<CarddavConfig> {
    let strategy = prompt::item("CardDAV authentication:", AUTHS, None)?;

    let auth = match strategy {
        BASIC => {
            let username = prompt::text::<&str>("CardDAV username:", None)?;
            let password = secret::configure("CardDAV password", None)?;
            CarddavAuthConfig::Basic { username, password }
        }
        BEARER => {
            let token = secret::configure("CardDAV API token", Some("ortie token show"))?;
            CarddavAuthConfig::Bearer { token }
        }
        _ => unreachable!(),
    };

    Ok(carddav_config(server, auth))
}

fn carddav_config(server: &Url, auth: CarddavAuthConfig) -> CarddavConfig {
    CarddavConfig {
        discover: None,
        server: Some(server.to_string()),
        home: None,
        tls: Default::default(),
        auth,
    }
}
