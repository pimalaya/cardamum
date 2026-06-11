//! Cardamum wrapper around [`io_webdav::client::WebdavClientStd`].
//!
//! Builds the CardDAV client from the account's `[carddav]` block via
//! one of three routes: `home` short-circuits every discovery step,
//! `server` runs only the principal + addressbook-home-set walk, and
//! `discover` resolves a bare domain to a server URL through pimconf
//! (RFC 6764 SRV + `.well-known`) before that walk.

use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use anyhow::{Result, anyhow};
use io_http::{rfc6750::bearer::HttpAuthBearer, rfc7617::basic::HttpAuthBasic};
use io_webdav::{client::WebdavClientStd as Inner, rfc4918::WebdavAuth};
use pimalaya_config::toml::TomlConfig;
use pimalaya_stream::tls::Tls;
use secrecy::ExposeSecret;
use url::Url;

use pimconf::rfc6764::{client::DiscoveryRfc6764ClientStd, types::DavService};

use crate::{
    account::context::Account,
    cli::load_or_wizard,
    config::{CarddavAuthConfig, CarddavConfig, TlsConfig},
};

const DEFAULT_RESOLVER: &str = "tcp://1.1.1.1:53";

pub struct CarddavClient {
    inner: Inner,
    pub account: Account,
}

impl CarddavClient {
    pub fn new(inner: Inner, account: Account) -> Self {
        Self { inner, account }
    }
}

impl Deref for CarddavClient {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for CarddavClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Loads the configuration, picks the active account, builds the
/// merged [`Account`] then opens the CardDAV client.
pub fn build_carddav_client(
    config_paths: &[PathBuf],
    account_name: Option<&str>,
) -> Result<CarddavClient> {
    let mut config = load_or_wizard(config_paths)?;
    let (name, mut ac) = config
        .take_account(account_name)?
        .ok_or_else(|| anyhow!("Cannot find account"))?;
    let carddav_config = ac
        .carddav
        .take()
        .ok_or_else(|| anyhow!("CardDAV config is missing for account `{name}`"))?;
    let account = Account::from(config).merge(Account::from(ac));
    let inner = open_carddav_client(carddav_config)?;
    Ok(CarddavClient::new(inner, account))
}

/// Opens a [`WebdavClientStd`] from a [`CarddavConfig`].
///
/// `home` skips every discovery step; `server` resolves principal +
/// addressbook-home-set from the given context root; `discover`
/// resolves a bare domain to that context root through pimconf first.
pub fn open_carddav_client(config: CarddavConfig) -> Result<Inner> {
    let CarddavConfig {
        discover,
        server,
        home,
        tls,
        auth,
    } = config;

    let tls = tls_with_http_alpn(tls);
    let auth = build_auth(auth)?;

    if let Some(home) = home {
        let mut client = Inner::connect(&home, &tls, auth)?;
        client.addressbook_home_set = Some(home);
        return Ok(client);
    }

    let server = match server {
        Some(server) => parse_carddav_server(&server)?,
        None => {
            let domain = discover
                .ok_or_else(|| anyhow!("CardDAV config needs `server`, `home`, or `discover`"))?;
            resolve_server(&domain, &tls)?
        }
    };

    let mut client = Inner::connect(&server, &tls, auth)?;
    client.current_user_principal()?;
    client.addressbook_home_set()?;

    Ok(client)
}

/// Resolves a bare domain to a CardDAV context root via pimconf
/// (RFC 6764 SRV + `.well-known`), using the same TLS profile for the
/// `.well-known` probe.
fn resolve_server(domain: &str, tls: &Tls) -> Result<Url> {
    let resolver = Url::parse(DEFAULT_RESOLVER).expect("DEFAULT_RESOLVER must be a valid URL");
    let mut client = DiscoveryRfc6764ClientStd::new(resolver).with_tls(tls.clone());
    let server = client.resolve(domain, DavService::Carddav)?;
    Ok(server)
}

/// Parses a `server` config string into a [`Url`].
///
/// Accepts a full URL, a bare domain, or `domain:port`; anything
/// without an explicit `http`/`https` scheme defaults to `https://`,
/// since `url` would otherwise read the leading label of `domain:port`
/// as the scheme.
pub fn parse_carddav_server(server: &str) -> Result<Url> {
    let url = match Url::parse(server) {
        Ok(url) if matches!(url.scheme(), "http" | "https") => url,
        _ => Url::parse(&format!("https://{server}"))?,
    };

    Ok(url)
}

fn tls_with_http_alpn(config: TlsConfig) -> Tls {
    let mut tls: Tls = config.into();
    tls.rustls.alpn = vec!["http/1.1".into()];
    tls
}

fn build_auth(auth: CarddavAuthConfig) -> Result<WebdavAuth> {
    Ok(match auth {
        CarddavAuthConfig::Basic { username, password } => {
            let password = password.get()?;
            WebdavAuth::Basic(HttpAuthBasic::new(username, password.expose_secret()))
        }
        CarddavAuthConfig::Bearer { token } => {
            let token = token.get()?;
            WebdavAuth::Bearer(HttpAuthBearer::new(token.expose_secret()))
        }
    })
}
