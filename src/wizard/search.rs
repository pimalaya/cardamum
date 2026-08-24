//! Email-driven service discovery for the wizard.
//!
//! Mirrors the cardamum-android configuration screen: the email address
//! feeds io-pim-discovery's parallel discovery (fixed provider rules,
//! PACC, RFC 6764 CardDAV resolve, RFC 8620 JMAP resolve, with a final
//! WWW-Authenticate probe refining the advertised schemes), and every
//! reachable service becomes one selectable entry carrying the
//! authentication capabilities it advertised (the concrete method is
//! picked once the service is chosen). A detected Google or Microsoft
//! account collapses to its dedicated contacts API (People, Graph),
//! which no discoverable record advertises.

use std::{collections::BTreeSet, env, fmt, time::Duration};

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

/// Upper bound on the parallel discovery fan-out. An unreachable
/// endpoint (a firewalled port, a black-hole host) must not stall the
/// interactive wizard, so mechanisms that have not reported by then are
/// abandoned and only what completed in time is offered.
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(8);

/// One selectable service to reach the account's contacts, carrying the
/// authentication capabilities it advertised. The concrete method (HTTP
/// scheme) is picked in a second prompt once the service is chosen, so a
/// service appears exactly once in the list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Discovered {
    pub kind: DiscoveredKind,
    /// Login hint advertised by the mechanism (usually the email).
    pub username: Option<String>,
    /// What the service accepts, folded across its discovered methods.
    pub auth: AuthCaps,
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
    People,
}

/// The authentication capabilities a service advertised, folded across
/// all its discovered methods. It drives the per-service auth prompt:
/// which HTTP schemes to offer, and whether the OAuth token brokers
/// appear. Cardamum reads a token an external manager (such as Ortie)
/// issues but never runs a grant itself, so OAuth is not a method of its
/// own here: it only unlocks the brokers behind the API token flow (see
/// [`super::secret`]).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AuthCaps {
    /// Basic/password auth. Often an app password (e.g. Fastmail,
    /// iCloud).
    pub basic: bool,
    /// A static bearer/API token.
    pub bearer: bool,
    /// An OAuth 2.0 grant is advertised, so a broker can issue the token.
    pub oauth: bool,
}

/// The capability queries the scheme prompt asks. Only the CardDAV and JMAP
/// flows prompt a scheme; the proprietary APIs are bearer-only.
#[cfg(any(feature = "carddav", feature = "jmap"))]
impl AuthCaps {
    /// Whether any capability was advertised. When none was (a mechanism
    /// that names no auth), the auth prompt offers every method so the
    /// user is never left without a choice.
    pub fn any(self) -> bool {
        self.basic || self.bearer || self.oauth
    }

    /// Whether a token (static or broker-issued) is on offer.
    pub fn token(self) -> bool {
        self.bearer || self.oauth
    }
}

impl fmt::Display for Discovered {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            DiscoveredKind::Carddav(url) => write!(f, "CardDAV {url}"),
            DiscoveredKind::Jmap(url) => write!(f, "JMAP {url}"),
            DiscoveredKind::Msgraph => write!(f, "Microsoft Graph API"),
            DiscoveredKind::People => write!(f, "Google People API"),
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
    /// the proprietary APIs.
    fn rank(&self) -> u8 {
        match self.kind {
            DiscoveredKind::Jmap(_) => 0,
            DiscoveredKind::Carddav(_) => 1,
            DiscoveredKind::Msgraph | DiscoveredKind::People => 2,
        }
    }
}

/// Searches every contacts service reachable from `email` and returns
/// one selectable entry per service, ordered by [`Discovered::rank`]. A
/// detected Google or Microsoft account yields only its dedicated
/// contacts API.
pub fn search(email: &str) -> Result<Vec<Discovered>> {
    let client = DiscoveryComposeClientStd::new(discovery_resolver(), discovery_tls());
    let services = BTreeSet::from([DiscoveryService::Carddav, DiscoveryService::Jmap]);
    let configs = client.compose_all_within(email, services, DISCOVERY_TIMEOUT)?;

    let provider = provider_of(email, &configs);
    let mut found = Vec::new();

    // NOTE: a detected provider collapses to its dedicated API, so the
    // CardDAV and JMAP entries are offered for other providers only.
    // This also drops the bogus origin-fallback CardDAV row a consumer
    // domain can surface.
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

            push_entry(&mut found, kind, config.username.clone(), &config.auth);
        }
    }

    match provider {
        Some(DiscoveryKnownProvider::Google) => found.push(Discovered {
            kind: DiscoveredKind::People,
            username: Some(email.to_string()),
            auth: AuthCaps {
                oauth: true,
                ..Default::default()
            },
        }),
        Some(DiscoveryKnownProvider::Microsoft) => found.push(Discovered {
            kind: DiscoveredKind::Msgraph,
            username: Some(email.to_string()),
            auth: AuthCaps {
                oauth: true,
                ..Default::default()
            },
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

/// Adds one entry per service, folding its advertised methods into the
/// entry's [`AuthCaps`]. A service already present keeps its first
/// endpoint and absorbs the extra capabilities, so several records for
/// the same service stay one selectable row.
fn push_entry(
    found: &mut Vec<Discovered>,
    kind: DiscoveredKind,
    username: Option<String>,
    auth: &[DiscoveryAuthMethod],
) {
    let caps = caps_of(auth);

    if let Some(entry) = found.iter_mut().find(|entry| entry.kind == kind) {
        entry.auth.basic |= caps.basic;
        entry.auth.bearer |= caps.bearer;
        entry.auth.oauth |= caps.oauth;
        return;
    }

    found.push(Discovered {
        kind,
        username,
        auth: caps,
    });
}

/// Folds a service's advertised methods into its [`AuthCaps`]: password
/// into `basic`, bearer into `bearer`, and every OAuth grant into `oauth`
/// (which only unlocks the token brokers, never a self-run grant).
fn caps_of(auth: &[DiscoveryAuthMethod]) -> AuthCaps {
    let mut caps = AuthCaps::default();

    for method in auth {
        match method {
            DiscoveryAuthMethod::Password => caps.basic = true,
            DiscoveryAuthMethod::Bearer => caps.bearer = true,
            _ => caps.oauth = true,
        }
    }

    caps
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps_fold_each_method_onto_its_axis() {
        let oauth = DiscoveryAuthMethod::OauthIssuer("https://issuer".into());

        assert_eq!(
            caps_of(&[DiscoveryAuthMethod::Password]),
            AuthCaps {
                basic: true,
                ..Default::default()
            }
        );
        assert_eq!(
            caps_of(&[DiscoveryAuthMethod::Bearer]),
            AuthCaps {
                bearer: true,
                ..Default::default()
            }
        );
        assert_eq!(
            caps_of(std::slice::from_ref(&oauth)),
            AuthCaps {
                oauth: true,
                ..Default::default()
            }
        );

        // NOTE: the Fastmail JMAP shape, bearer plus an OAuth grant and no
        // Basic, is one "API token" method whose brokers are unlocked.
        let fastmail = caps_of(&[DiscoveryAuthMethod::Bearer, oauth]);
        assert_eq!(
            fastmail,
            AuthCaps {
                bearer: true,
                oauth: true,
                ..Default::default()
            }
        );
        assert!(fastmail.token());
        assert!(!fastmail.basic);
    }

    #[test]
    fn caps_report_emptiness_and_token_offer() {
        assert!(!AuthCaps::default().any());
        assert!(!AuthCaps::default().token());

        let basic = AuthCaps {
            basic: true,
            ..Default::default()
        };
        assert!(basic.any());
        assert!(!basic.token());

        let oauth = AuthCaps {
            oauth: true,
            ..Default::default()
        };
        assert!(oauth.token());
    }

    #[test]
    fn one_service_stays_one_entry_absorbing_every_method() {
        let url = Url::parse("https://carddav.example.org/dav/").unwrap();
        let mut found = Vec::new();

        push_entry(
            &mut found,
            DiscoveredKind::Carddav(url.clone()),
            Some("alice@example.org".into()),
            &[DiscoveryAuthMethod::Password],
        );
        push_entry(
            &mut found,
            DiscoveredKind::Carddav(url),
            None,
            &[DiscoveryAuthMethod::Bearer],
        );

        assert_eq!(found.len(), 1);
        assert_eq!(
            found[0].auth,
            AuthCaps {
                basic: true,
                bearer: true,
                ..Default::default()
            }
        );
        assert_eq!(found[0].username.as_deref(), Some("alice@example.org"));
    }
}
