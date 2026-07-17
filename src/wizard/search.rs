//! Email-driven service discovery for the wizard.
//!
//! Mirrors the cardamum-android configuration screen: the email address
//! feeds io-pim-discovery's parallel discovery (fixed provider rules,
//! PACC, RFC 6764 CardDAV resolve, RFC 8620 JMAP resolve, with a final
//! WWW-Authenticate probe refining the advertised schemes), and every
//! reachable service and authentication method becomes one selectable
//! entry. A detected Google or Microsoft account collapses to its
//! dedicated contacts API (People, Graph), which no discoverable record
//! advertises.

use std::{collections::BTreeSet, env, fmt};

use anyhow::Result;
use io_pim_discovery::{
    compose::{
        client::DiscoveryComposeClientStd,
        config::{
            DiscoveryAuthMethod, DiscoveryConfigSource, DiscoveryEndpoint, DiscoveryService,
            DiscoveryServiceConfig,
        },
        providers::DiscoveryKnownProvider,
    },
    shared::dns::system_resolver,
};
use pimalaya_stream::tls::{Rustls, Tls};
use url::Url;

/// DNS-over-TCP resolver backing discovery when `CARDAMUM_DNS_RESOLVER`
/// is unset and no system resolver is found: Cloudflare's `1.1.1.1`.
const DEFAULT_RESOLVER: &str = "tcp://1.1.1.1:53";

/// One selectable way to reach the account's contacts: a discovered
/// service paired with one of its authentication methods.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Discovered {
    pub kind: DiscoveredKind,
    /// Login hint advertised by the mechanism (usually the email).
    pub username: Option<String>,
    /// How to authenticate against the service.
    pub auth: DiscoveredAuth,
}

/// The discovered service kind, carrying its endpoint for the open
/// standards (the proprietary APIs have fixed endpoints).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiscoveredKind {
    /// A CardDAV context root.
    Carddav(Url),
    /// A JMAP session endpoint.
    Jmap(Url),
    /// The Microsoft Graph API (Microsoft accounts only).
    Msgraph,
    /// The Google People API (Google accounts only).
    Google,
}

/// The discovered authentication method, reduced to what the CLI can
/// configure: a password, or a bearer token. Every OAuth 2.0 grant
/// collapses to a token; the CLI never runs a grant itself, it reads a
/// token an external manager (such as Ortie) issues and refreshes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoveredAuth {
    Password,
    Token,
}

impl fmt::Display for Discovered {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let auth = match self.auth {
            DiscoveredAuth::Password => "Password",
            DiscoveredAuth::Token => "API token",
        };

        match &self.kind {
            DiscoveredKind::Carddav(url) => write!(f, "CardDAV {url} ({auth})"),
            DiscoveredKind::Jmap(url) => write!(f, "JMAP {url} ({auth})"),
            DiscoveredKind::Msgraph => write!(f, "Microsoft Graph API ({auth})"),
            DiscoveredKind::Google => write!(f, "Google People API ({auth})"),
        }
    }
}

impl Discovered {
    /// Best default login for the credential prompt: the advertised
    /// username when it looks like an address, else the searched email
    /// when the user typed a full one, else nothing (a bare domain,
    /// whose synthesized `@domain` form is rejected here). Only the
    /// CardDAV and JMAP flows prompt a login.
    #[cfg(any(feature = "carddav", feature = "jmap"))]
    pub fn login_default(&self, email: &str) -> Option<String> {
        self.username
            .clone()
            .filter(|username| looks_like_address(username))
            .or_else(|| looks_like_address(email).then(|| email.to_string()))
    }

    /// Ranks an entry for the selection list: JMAP before CardDAV before
    /// the proprietary APIs, then API token before password.
    fn rank(&self) -> (u8, u8) {
        let service = match self.kind {
            DiscoveredKind::Jmap(_) => 0,
            DiscoveredKind::Carddav(_) => 1,
            DiscoveredKind::Msgraph | DiscoveredKind::Google => 2,
        };
        let auth = match self.auth {
            DiscoveredAuth::Token => 0,
            DiscoveredAuth::Password => 1,
        };

        (service, auth)
    }
}

/// Searches every contacts service reachable from `email` and returns
/// one selectable entry per service and authentication method, ordered
/// by [`Discovered::rank`]. A detected Google or Microsoft account
/// yields only its dedicated contacts API.
pub fn search(email: &str) -> Result<Vec<Discovered>> {
    let client = DiscoveryComposeClientStd::new(discovery_resolver(), discovery_tls());
    let services = BTreeSet::from([DiscoveryService::Carddav, DiscoveryService::Jmap]);
    let configs = client.compose_all(email, services)?;

    let provider = provider_of(email, &configs);
    let mut found = Vec::new();

    // A detected provider collapses to its dedicated API, so the
    // discovered CardDAV/JMAP entries are offered for other providers
    // only (this also drops any bogus origin-fallback CardDAV row a
    // consumer domain might surface).
    if provider.is_none() {
        for config in &configs {
            let DiscoveryEndpoint::Http(raw) = &config.endpoint else {
                continue;
            };
            let Ok(url) = Url::parse(raw) else {
                continue;
            };

            let kind = match config.service {
                DiscoveryService::Carddav => DiscoveredKind::Carddav(url),
                DiscoveryService::Jmap => DiscoveredKind::Jmap(url),
                _ => continue,
            };

            push_entries(&mut found, kind, config.username.clone(), &config.auth);
        }
    }

    match provider {
        Some(DiscoveryKnownProvider::Google) => found.push(Discovered {
            kind: DiscoveredKind::Google,
            username: Some(email.to_string()),
            auth: DiscoveredAuth::Token,
        }),
        Some(DiscoveryKnownProvider::Microsoft) => found.push(Discovered {
            kind: DiscoveredKind::Msgraph,
            username: Some(email.to_string()),
            auth: DiscoveredAuth::Token,
        }),
        None => {}
    }

    found.sort_by_key(Discovered::rank);
    Ok(found)
}

/// Resolves the provider from the email domain (fast path for consumer
/// addresses), falling back to any provider-tagged config, which
/// catches custom domains detected through their MX records.
fn provider_of(email: &str, configs: &[DiscoveryServiceConfig]) -> Option<DiscoveryKnownProvider> {
    let by_domain = email
        .rsplit_once('@')
        .and_then(|(_, domain)| DiscoveryKnownProvider::from_domain(domain));

    by_domain.or_else(|| {
        configs.iter().find_map(|config| match config.source {
            DiscoveryConfigSource::Provider(provider) => Some(provider),
            _ => None,
        })
    })
}

/// Turns each authentication method of a service into one entry,
/// skipping duplicates (several OAuth grants collapse to one token).
fn push_entries(
    found: &mut Vec<Discovered>,
    kind: DiscoveredKind,
    username: Option<String>,
    auth: &[DiscoveryAuthMethod],
) {
    for method in auth {
        let auth = match method {
            DiscoveryAuthMethod::Password => DiscoveredAuth::Password,
            // Bearer and every OAuth 2.0 grant reduce to pasting a
            // token (see [`DiscoveredAuth`]).
            _ => DiscoveredAuth::Token,
        };

        let entry = Discovered {
            kind: kind.clone(),
            username: username.clone(),
            auth,
        };
        if !found.contains(&entry) {
            found.push(entry);
        }
    }
}

/// Whether a string is a full `local@domain` address (both parts
/// non-empty), rejecting the bare-domain `@domain` form.
#[cfg(any(feature = "carddav", feature = "jmap"))]
fn looks_like_address(value: &str) -> bool {
    value
        .split_once('@')
        .is_some_and(|(local, domain)| !local.is_empty() && !domain.is_empty())
}

/// Resolver used by discovery: the `CARDAMUM_DNS_RESOLVER` override
/// first, then the system resolver (`/etc/resolv.conf` on unix, the
/// network adapters on windows), then the Cloudflare default. This
/// avoids leaking the email domain to a third-party resolver and works
/// around networks that block the default.
pub fn discovery_resolver() -> Url {
    if let Ok(resolver) = env::var("CARDAMUM_DNS_RESOLVER")
        && let Ok(url) = resolver.parse()
    {
        return url;
    }

    if let Some(url) = system_resolver() {
        return url;
    }

    DEFAULT_RESOLVER
        .parse()
        .expect("DEFAULT_RESOLVER must be a valid URL")
}

/// TLS profile for the HTTPS-bound discovery mechanisms; they only
/// speak HTTP/1.1 to `.well-known` endpoints.
fn discovery_tls() -> Tls {
    Tls {
        rustls: Rustls {
            alpn: vec!["http/1.1".into()],
            ..Default::default()
        },
        ..Default::default()
    }
}
