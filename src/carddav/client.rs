//! # CardDAV client
//!
//! Wraps [`io_webdav::client::WebdavClientStd`], built from the
//! account's `[carddav]` block.
//!
//! The block picks one of three routes: `home` short-circuits every
//! discovery step, `server` runs only the principal and
//! addressbook-home-set walk, and `discover` resolves a bare domain to
//! a server URL (PACC, then RFC 6764) before that walk.
//!
//! Google domains take a dedicated authenticated `.well-known` path,
//! since Google publishes nothing discoverable on `gmail.com`.

use std::{
    io::{Read, Write},
    ops::{Deref, DerefMut},
};

use anyhow::{Result, anyhow, bail};
use io_http::{
    coroutine::{HttpCoroutine, HttpCoroutineState, HttpYield},
    rfc6750::bearer::HttpAuthBearer,
    rfc7617::basic::HttpAuthBasic,
    rfc8615::well_known::{Http11WellKnown, Http11WellKnownOutput},
    rfc9110::request::HttpRequest,
};
use io_pim_discovery::{
    pacc::client::DiscoveryPaccClientStd,
    rfc6764::{client::DiscoveryWebdavClientStd, service::DiscoveryDavService},
};
use io_webdav::{client::WebdavClientStd as Inner, rfc4918::WebdavAuth};
use pimalaya_stream::{
    stream::{Stream, TlsConnectOptions},
    tls::Tls,
};
use secrecy::ExposeSecret;
use url::Url;

use crate::{
    account::context::Account,
    config::{AccountConfig, CarddavAuthConfig, CarddavConfig, Config, TlsConfig},
};

/// DNS resolver the discovery mechanisms query.
const DEFAULT_RESOLVER: &str = "tcp://1.1.1.1:53";

/// Host of [`GOOGLE_API_ORIGIN`], used to open the TLS stream.
const GOOGLE_API_HOST: &str = "www.googleapis.com";
/// Origin hosting Google's CardDAV `.well-known` entry point.
const GOOGLE_API_ORIGIN: &str = "https://www.googleapis.com/";

/// CardDAV client the protocol-specific commands run against.
///
/// Derefs to the io-webdav client, and carries the merged account so
/// commands can reach its display settings.
pub struct CarddavClient {
    inner: Inner,
    pub account: Account,
}

impl CarddavClient {
    /// Pairs a connected io-webdav client with its account.
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

/// Merges the resolved config and account, then opens the client.
///
/// Bails when the account carries no `[carddav]` block.
pub fn build_carddav_client(
    config: Config,
    name: String,
    mut account_config: AccountConfig,
) -> Result<CarddavClient> {
    let carddav_config = account_config
        .carddav
        .take()
        .ok_or_else(|| anyhow!("CardDAV config is missing for account `{name}`"))?;
    let account = Account::from(config).merge(Account::from(account_config));
    let inner = open_carddav_client(carddav_config)?;
    Ok(CarddavClient::new(inner, account))
}

/// Opens a [`WebdavClientStd`](io_webdav::client::WebdavClientStd) from
/// a [`CarddavConfig`].
///
/// `home` skips every discovery step; `server` resolves the principal
/// and addressbook-home-set from the given context root; `discover`
/// resolves a bare domain to that root through io-pim-discovery first.
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

    // NOTE: a bare origin is not necessarily the DAV context root.
    // Discovery hands back `https://carddav.fastmail.com/`, yet Fastmail
    // 404s everything outside `/dav/*`, so probe `.well-known/carddav`
    // and follow its redirect.
    let server = match server.path() {
        "" | "/" => probe_carddav_context_root(&server, &tls).unwrap_or(server),
        _ => server,
    };

    let mut client = Inner::connect(&server, &tls, auth)?;
    client.current_user_principal()?;
    client.addressbook_home_set()?;

    Ok(client)
}

/// Probes `.well-known/carddav` on a bare-origin `server` with a GET.
///
/// Returns the context-root redirect target when the server publishes
/// one. Silent: a failed probe or a response without a redirect leaves
/// the origin as-is.
fn probe_carddav_context_root(server: &Url, tls: &Tls) -> Option<Url> {
    let host = server.host_str()?;
    let port = server.port_or_known_default()?;
    let request = Http11WellKnown::prepare_request(server.as_str(), "carddav").ok()?;
    let output = run_well_known(host, port, request, tls).ok()?;
    output.redirect_url
}

/// Runs a prepared `.well-known` request over a fresh TLS stream.
fn run_well_known(
    host: &str,
    port: u16,
    request: HttpRequest,
    tls: &Tls,
) -> Result<Http11WellKnownOutput> {
    let opts = TlsConnectOptions {
        tls: tls.clone(),
        ..Default::default()
    };
    let mut stream = Stream::connect_tls(host, port, opts)?;
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

/// Discovers a CardDAV server URL for `domain`, first hit winning.
///
/// Tries PACC, then the RFC 6764 chain. Silent: the wizard wraps the
/// individual mechanisms with its own spinners.
pub fn discover_server(domain: &str, tls: &Tls) -> Option<Url> {
    discover_via_pacc(domain, tls).or_else(|| discover_via_rfc6764(domain, tls))
}

/// PACC discovery of the CardDAV URL (draft-ietf-mailmaint-pacc).
pub fn discover_via_pacc(domain: &str, tls: &Tls) -> Option<Url> {
    let resolver = Url::parse(DEFAULT_RESOLVER).expect("DEFAULT_RESOLVER must be a valid URL");
    let mut client = DiscoveryPaccClientStd::new(resolver).with_tls(tls.clone());
    let config = client.discover(domain).ok()?;
    let carddav = config.protocols.carddav?;
    Url::parse(&carddav.url).ok()
}

/// RFC 6764 §6 discovery of the CardDAV URL.
///
/// Resolves the SRV record (secure first), its TXT `path` context, then
/// `.well-known` on the resolved host, falling back to
/// `https://<domain>` when the domain publishes nothing.
pub fn discover_via_rfc6764(domain: &str, tls: &Tls) -> Option<Url> {
    let resolver = Url::parse(DEFAULT_RESOLVER).expect("DEFAULT_RESOLVER must be a valid URL");
    let mut client = DiscoveryWebdavClientStd::new(resolver).with_tls(tls.clone());
    client.resolve(domain, DiscoveryDavService::Carddav).ok()
}

/// Whether `domain` is a Google consumer mail domain.
///
/// Google serves CardDAV behind a non-standard authenticated entry
/// point rather than the records RFC 6764 relies on.
pub fn is_google(domain: &str) -> bool {
    matches!(
        domain.to_ascii_lowercase().as_str(),
        "gmail.com" | "googlemail.com"
    )
}

/// Resolves the Google CardDAV context root with a PROPFIND probe.
///
/// Google publishes neither SRV nor `.well-known` records on
/// `gmail.com`, and its `.well-known` endpoint only 301-redirects for
/// an authenticated PROPFIND (a plain GET 404s), so the well-known
/// builder is reused with its method swapped and a bearer added.
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

    // NOTE: no redirect means Google rejected the probe, most often a
    // 401 from an expired token, so surface its status and body rather
    // than a generic "no redirect".
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
/// without an explicit `http` or `https` scheme defaults to `https://`,
/// since `url` would otherwise read the leading label of `domain:port`
/// as the scheme.
pub fn parse_carddav_server(server: &str) -> Result<Url> {
    let url = match Url::parse(server) {
        Ok(url) if matches!(url.scheme(), "http" | "https") => url,
        _ => Url::parse(&format!("https://{server}"))?,
    };

    Ok(url)
}

/// Builds the TLS configuration, negotiating HTTP/1.1 through ALPN.
pub fn tls_with_http_alpn(config: TlsConfig) -> Tls {
    config.into_tls(vec!["http/1.1".into()])
}

/// Resolves the configured credential into a WebDAV auth scheme.
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
