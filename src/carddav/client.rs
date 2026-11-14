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
use io_webdav::{
    client::{WebdavClientStd as Inner, WebdavClientStdConnectOptions, WebdavClientStdError},
    rfc4918::{WebdavAuth, follow_redirects::WebdavFollowRedirectsError, send::WebdavSendError},
    rfc6578::sync_collection::WebdavSyncCollectionError,
};
use pimalaya_config::secret::SecretResolver;
use pimalaya_stream::{
    proxy::Proxy,
    stream::{Stream, TlsConnectOptions},
    tls::Tls,
};
use secrecy::ExposeSecret;
use url::Url;

use crate::{
    config::{AccountConfig, CarddavAuthConfig, CarddavConfig, ProxyConfig},
    error::ErrorCode,
};

/// DNS resolver the discovery mechanisms query.
const DEFAULT_RESOLVER: &str = "tcp://1.1.1.1:53";

/// Host of [`GOOGLE_API_ORIGIN`], used to open the TLS stream.
const GOOGLE_API_HOST: &str = "www.googleapis.com";
/// Origin hosting Google's CardDAV `.well-known` entry point.
const GOOGLE_API_ORIGIN: &str = "https://www.googleapis.com/";

/// CardDAV client the protocol-specific commands run against.
///
/// Derefs to the io-webdav client.
pub struct CarddavClient {
    inner: Inner,
}

impl CarddavClient {
    /// Wraps a connected io-webdav client.
    pub fn new(inner: Inner) -> Self {
        Self { inner }
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

/// Opens the CardDAV client of the account.
///
/// Bails when the account carries no `[carddav]` block.
pub fn build_carddav_client(
    name: String,
    mut account_config: AccountConfig,
) -> Result<CarddavClient> {
    let carddav_config = account_config
        .carddav
        .take()
        .ok_or_else(|| anyhow!("CardDAV config is missing for account `{name}`"))?;
    let inner = open_carddav_client(carddav_config, &mut SecretResolver::new())?;
    Ok(CarddavClient::new(inner))
}

/// Opens a [`WebdavClientStd`](io_webdav::client::WebdavClientStd) from a
/// [`CarddavConfig`], resolving its credential through `resolver`, so an
/// account naming one credential command from several of its backends
/// spawns it once.
///
/// `home` skips every discovery step; `server` resolves the principal
/// and addressbook-home-set from the given context root; `discover`
/// resolves a bare domain to that root through io-pim-discovery first.
pub fn open_carddav_client(config: CarddavConfig, resolver: &mut SecretResolver) -> Result<Inner> {
    let CarddavConfig {
        discover,
        server,
        home,
        tls,
        alpn,
        proxy,
        auth,
    } = config;

    let tls = tls.into_tls(alpn);
    let auth = build_auth(auth, resolver)?;
    let proxy = ProxyConfig::resolve(proxy, resolver)?;

    if let Some(home) = home {
        let options = WebdavClientStdConnectOptions { tls, proxy };
        let mut client = Inner::connect(&home, auth, options)?;
        client.addressbook_home_set = Some(home);
        return Ok(client);
    }

    let server = match server {
        Some(server) => parse_carddav_server(&server)?,
        None => {
            let domain = discover
                .ok_or_else(|| anyhow!("CardDAV config needs `server`, `home`, or `discover`"))?;
            if is_google(&domain) {
                google_carddav_server(&auth, &tls, &proxy)?
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
        "" | "/" => probe_carddav_context_root(&server, &tls, &proxy).unwrap_or(server),
        _ => server,
    };

    let options = WebdavClientStdConnectOptions { tls, proxy };
    let mut client = Inner::connect(&server, auth, options)?;
    client.current_user_principal()?;
    client.addressbook_home_set()?;

    Ok(client)
}

/// Probes `.well-known/carddav` on a bare-origin `server` with a GET.
///
/// Returns the context-root redirect target when the server publishes
/// one. Silent: a failed probe or a response without a redirect leaves
/// the origin as-is.
fn probe_carddav_context_root(server: &Url, tls: &Tls, proxy: &Proxy) -> Option<Url> {
    let host = server.host_str()?;
    let port = server.port_or_known_default()?;
    let request = Http11WellKnown::prepare_request(server.as_str(), "carddav").ok()?;
    let output = run_well_known(host, port, request, tls, proxy).ok()?;
    output.redirect_url
}

/// Runs a prepared `.well-known` request over a fresh TLS stream.
fn run_well_known(
    host: &str,
    port: u16,
    request: HttpRequest,
    tls: &Tls,
    proxy: &Proxy,
) -> Result<Http11WellKnownOutput> {
    let opts = TlsConnectOptions {
        tls: tls.clone(),
        proxy: proxy.clone(),
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
/// Tries PACC, then the RFC 6764 chain.
fn discover_server(domain: &str, tls: &Tls) -> Option<Url> {
    discover_via_pacc(domain, tls).or_else(|| discover_via_rfc6764(domain, tls))
}

/// PACC discovery of the CardDAV URL (draft-ietf-mailmaint-pacc).
fn discover_via_pacc(domain: &str, tls: &Tls) -> Option<Url> {
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
fn discover_via_rfc6764(domain: &str, tls: &Tls) -> Option<Url> {
    let resolver = Url::parse(DEFAULT_RESOLVER).expect("DEFAULT_RESOLVER must be a valid URL");
    let mut client = DiscoveryWebdavClientStd::new(resolver).with_tls(tls.clone());
    client.resolve(domain, DiscoveryDavService::Carddav).ok()
}

/// Whether `domain` is a Google consumer mail domain.
///
/// Google serves CardDAV behind a non-standard authenticated entry
/// point rather than the records RFC 6764 relies on.
fn is_google(domain: &str) -> bool {
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
fn google_carddav_server(auth: &WebdavAuth, tls: &Tls, proxy: &Proxy) -> Result<Url> {
    let WebdavAuth::Bearer(bearer) = auth else {
        bail!("Google CardDAV requires OAuth 2.0 bearer authentication");
    };

    let mut request = Http11WellKnown::prepare_request(GOOGLE_API_ORIGIN, "carddav")?;
    request.method = "PROPFIND".into();
    let request = request
        .header("Authorization", bearer.to_authorization())
        .header("Depth", "0");

    let output = run_well_known(GOOGLE_API_HOST, 443, request, tls, proxy)?;

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

/// Resolves the configured credential into a WebDAV auth scheme.
///
/// The credential goes through `resolver`, so a command it already
/// spawned for this account is not run again.
fn build_auth(auth: CarddavAuthConfig, resolver: &mut SecretResolver) -> Result<WebdavAuth> {
    Ok(match auth {
        CarddavAuthConfig::None => WebdavAuth::None,
        CarddavAuthConfig::Basic { username, password } => {
            let password = resolver.resolve(password)?;
            WebdavAuth::Basic(HttpAuthBasic::new(username, password.expose_secret()))
        }
        CarddavAuthConfig::Bearer { token } => {
            let token = resolver.resolve(token)?;
            WebdavAuth::Bearer(HttpAuthBearer::new(token.expose_secret()))
        }
    })
}

/// The status and body of an answer that refused a request, wherever
/// io-webdav surfaces it.
fn refused(err: &WebdavClientStdError) -> Option<(u16, &str)> {
    match err {
        WebdavClientStdError::Send(WebdavSendError::HttpStatus { status, body })
        | WebdavClientStdError::WebdavSyncCollection(WebdavSyncCollectionError::Send(
            WebdavSendError::HttpStatus { status, body },
        ))
        | WebdavClientStdError::WebdavFollowRedirects(WebdavFollowRedirectsError::HttpStatus {
            status,
            body,
        }) => Some((*status, body)),
        _ => None,
    }
}

/// The code of a CardDAV refusal in an error chain, as neverest reads
/// it.
///
/// A 507, or a body naming `quota-not-exceeded` (RFC 4331 §6), is
/// `quota-exceeded`; a 429 `rate-limited`; a 403 `permission-denied`
/// when its body names `need-privileges` (RFC 3744 §7.1.1) or no
/// precondition at all. Any other 403 names a precondition the written
/// card failed (RFC 6352 §6.3.2), no matter of rights.
pub(crate) fn refusal_code(cause: &(dyn std::error::Error + 'static)) -> Option<ErrorCode> {
    let (status, body) = refused(cause.downcast_ref::<WebdavClientStdError>()?)?;
    status_code(status, body)
}

/// The code of a listing row the server answered with `status` and no
/// body, as [`refusal_code`] reads a refusal: a 403 `permission-denied`.
pub(crate) fn row_code(status: u16) -> Option<ErrorCode> {
    status_code(status, "")
}

/// The code of a refusal of `status` answered with `body`.
fn status_code(status: u16, body: &str) -> Option<ErrorCode> {
    match status {
        _ if body.contains("quota-not-exceeded") => Some(ErrorCode::QuotaExceeded),
        507 => Some(ErrorCode::QuotaExceeded),
        429 => Some(ErrorCode::RateLimited),
        403 if body.contains("need-privileges") || !names_precondition(body) => {
            Some(ErrorCode::PermissionDenied)
        }
        _ => None,
    }
}

/// Whether a refusal's body is a `DAV:error` element, which names the
/// precondition the request failed (RFC 4918 §16).
fn names_precondition(body: &str) -> bool {
    body.contains(":error") || body.contains("<error")
}

/// Whether a card request found no card: 404, or 410 for one gone.
pub(crate) fn is_missing(err: &WebdavClientStdError) -> bool {
    matches!(
        err,
        WebdavClientStdError::Send(WebdavSendError::HttpStatus {
            status: 404 | 410,
            ..
        })
    )
}
