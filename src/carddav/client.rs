//! Cardamum wrapper around [`io_webdav::client::WebdavClientStd`].
//!
//! Builds the CardDAV client from the account's `[carddav]` block via
//! one of three routes: `home` short-circuits every discovery step,
//! `server` runs only the principal + addressbook-home-set walk, and
//! `discover` resolves a bare domain to a server URL (PACC first, then
//! RFC 6764: SRV, its TXT `path`, then `.well-known`) before that walk.
//! Google domains take a dedicated authenticated `.well-known` path,
//! since Google publishes nothing discoverable on `gmail.com`.

use std::{
    io::{Read, Write},
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use anyhow::{Result, anyhow, bail};
use io_http::{
    coroutine::{HttpCoroutine, HttpCoroutineState, HttpYield},
    rfc6750::bearer::HttpAuthBearer,
    rfc7617::basic::HttpAuthBasic,
    rfc8615::well_known::{Http11WellKnown, Http11WellKnownOutput},
    rfc9110::request::HttpRequest,
};
use io_webdav::{client::WebdavClientStd as Inner, rfc4918::WebdavAuth};
use pimalaya_config::toml::TomlConfig;
use pimalaya_stream::{std::stream::StreamStd, tls::Tls};
use secrecy::ExposeSecret;
use url::Url;

use pimconf::{
    pacc::client::DiscoveryPaccClientStd,
    rfc6764::{client::DiscoveryWebdavClientStd, types::DavService},
};

use crate::{
    account::context::Account,
    cli::load_or_wizard,
    config::{CarddavAuthConfig, CarddavConfig, TlsConfig},
};

const DEFAULT_RESOLVER: &str = "tcp://1.1.1.1:53";

/// Host of [`GOOGLE_API_ORIGIN`], used to open the TLS stream.
const GOOGLE_API_HOST: &str = "www.googleapis.com";
/// Origin hosting Google's CardDAV `.well-known` entry point.
const GOOGLE_API_ORIGIN: &str = "https://www.googleapis.com/";

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

/// Opens a [`WebdavClientStd`](io_webdav::client::WebdavClientStd) from
/// a [`CarddavConfig`].
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
            if is_google(&domain) {
                google_carddav_server(&auth, &tls)?
            } else {
                discover_server(&domain, &tls)
                    .ok_or_else(|| anyhow!("CardDAV discovery failed for `{domain}`"))?
            }
        }
    };

    // A bare origin (path `/`) is not necessarily the DAV context root:
    // PACC and RFC 6764 both hand back e.g. `https://carddav.fastmail.com/`,
    // yet fastmail 404s every request outside `/dav/*`. Probe
    // `.well-known/carddav` and follow its redirect first, mirroring the
    // cardamum-android connect-time probe. No redirect keeps the origin,
    // which plenty of servers serve the walk from directly.
    let server = match server.path() {
        "" | "/" => probe_carddav_context_root(&server, &tls).unwrap_or(server),
        _ => server,
    };

    let mut client = Inner::connect(&server, &tls, auth)?;
    client.current_user_principal()?;
    client.addressbook_home_set()?;

    Ok(client)
}

/// Probes `.well-known/carddav` on a bare-origin `server` with an
/// unauthenticated GET, returning the context-root redirect target when
/// the server publishes one. Silent: a failed probe or a plain response
/// (no redirect) yields `None`, leaving the origin as-is.
fn probe_carddav_context_root(server: &Url, tls: &Tls) -> Option<Url> {
    let host = server.host_str()?;
    let port = server.port_or_known_default()?;
    let request = Http11WellKnown::prepare_request(server.as_str(), "carddav").ok()?;
    let output = run_well_known(host, port, request, tls).ok()?;
    output.redirect_url
}

/// Runs a prepared `.well-known` request to completion over a fresh TLS
/// stream to `host:port`, returning the coroutine output.
fn run_well_known(
    host: &str,
    port: u16,
    request: HttpRequest,
    tls: &Tls,
) -> Result<Http11WellKnownOutput> {
    let mut stream = StreamStd::connect_tls(host, port, tls)?;
    let mut coroutine = Http11WellKnown::new(request);
    let mut buf = [0u8; 8 * 1024];
    let mut arg: Option<&[u8]> = None;

    loop {
        match coroutine.resume(arg.take()) {
            HttpCoroutineState::Complete(Ok(output)) => return Ok(output),
            HttpCoroutineState::Complete(Err(err)) => return Err(err.into()),
            HttpCoroutineState::Yielded(HttpYield::WantsWrite(bytes)) => {
                stream.write_all(&bytes)?;
            }
            HttpCoroutineState::Yielded(HttpYield::WantsRead) => {
                let n = stream.read(&mut buf)?;
                arg = Some(&buf[..n]);
            }
        }
    }
}

/// Discovers a CardDAV server URL for `domain`, trying each mechanism
/// in turn and returning the first hit: PACC, then the RFC 6764 chain
/// (SRV, its TXT `path`, then `.well-known`). Silent; the wizard wraps
/// the individual mechanisms with its own spinners.
pub fn discover_server(domain: &str, tls: &Tls) -> Option<Url> {
    discover_via_pacc(domain, tls).or_else(|| discover_via_rfc6764(domain, tls))
}

/// PACC discovery (draft-ietf-mailmaint-pacc): returns the advertised
/// CardDAV URL when the provider publishes one.
pub fn discover_via_pacc(domain: &str, tls: &Tls) -> Option<Url> {
    let resolver = Url::parse(DEFAULT_RESOLVER).expect("DEFAULT_RESOLVER must be a valid URL");
    let mut client = DiscoveryPaccClientStd::new(resolver).with_tls(tls.clone());
    let config = client.discover(domain).ok()?;
    let carddav = config.protocols.carddav?;
    Url::parse(&carddav.url).ok()
}

/// RFC 6764 §6 discovery: resolves the SRV record (secure first), its
/// TXT `path` context, then `.well-known` on the resolved host, falling
/// back to `https://<domain>` when the domain publishes nothing. Wraps
/// pimconf's `resolve`, which performs the steps in the RFC's order.
pub fn discover_via_rfc6764(domain: &str, tls: &Tls) -> Option<Url> {
    let resolver = Url::parse(DEFAULT_RESOLVER).expect("DEFAULT_RESOLVER must be a valid URL");
    let mut client = DiscoveryWebdavClientStd::new(resolver).with_tls(tls.clone());
    client.resolve(domain, DavService::Carddav).ok()
}

/// Whether `domain` is a Google consumer mail domain, which serves
/// CardDAV behind a non-standard authenticated entry point rather than
/// the discoverable records RFC 6764 relies on.
pub fn is_google(domain: &str) -> bool {
    matches!(
        domain.to_ascii_lowercase().as_str(),
        "gmail.com" | "googlemail.com"
    )
}

/// Resolves the Google CardDAV context root via an authenticated
/// PROPFIND to `https://www.googleapis.com/.well-known/carddav`.
///
/// Google publishes no SRV/`.well-known` records on `gmail.com`, and
/// its `.well-known` endpoint only 301-redirects for an authenticated
/// PROPFIND (a plain GET 404s). So this reuses the HTTP well-known
/// request builder, swaps the method to PROPFIND, and adds the OAuth
/// 2.0 bearer; the surfaced redirect target is the context root the
/// principal walk then runs against.
fn google_carddav_server(auth: &WebdavAuth, tls: &Tls) -> Result<Url> {
    let WebdavAuth::Bearer(bearer) = auth else {
        bail!("Google CardDAV requires OAuth 2.0 bearer authentication");
    };

    let mut request = Http11WellKnown::prepare_request(GOOGLE_API_ORIGIN, "carddav")?;
    request.method = "PROPFIND".into();
    let request = request
        .header("Authorization", bearer.to_authorization())
        .header("Depth", "0");

    let output = run_well_known(GOOGLE_API_HOST, 443, request, tls)?;

    if let Some(url) = output.redirect_url {
        return Ok(url);
    }

    // No redirect means Google rejected the probe (most often a 401
    // from an invalid or expired token); surface its status and body so
    // the real cause is visible instead of a generic "no redirect".
    let status = *output.response.status;
    let body = String::from_utf8_lossy(&output.response.body);
    let body = body.trim();

    if body.is_empty() {
        bail!("Google `.well-known/carddav` probe failed with HTTP {status}");
    }

    bail!("Google `.well-known/carddav` probe failed with HTTP {status}: {body}")
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

pub fn tls_with_http_alpn(config: TlsConfig) -> Tls {
    config.into_tls(vec!["http/1.1".into()])
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
