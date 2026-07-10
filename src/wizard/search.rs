//! Email-driven service discovery for the wizard.
//!
//! Mirrors the cardamum-android configuration screen: the email
//! address feeds pimconf's parallel search (fixed provider rules,
//! PACC, RFC 6764 DAV resolve, RFC 8620 JMAP resolve, with a final
//! WWW-Authenticate probe refining the advertised schemes), and every
//! discovered service and authentication method combination becomes
//! one selectable entry. Google and Microsoft addresses additionally
//! surface their proprietary contacts APIs (People, Graph), which no
//! discoverable record advertises.

use std::collections::BTreeSet;

use anyhow::Result;
use pimalaya_stream::tls::Tls;
use pimconf::search::{
    client::SearchClientStd,
    providers::Provider,
    types::{AuthMethod, ConfigSource, Endpoint, Service},
};
use url::Url;

/// DNS-over-TCP resolver backing the discovery lookups.
const DEFAULT_RESOLVER: &str = "tcp://1.1.1.1:53";

/// One selectable way to reach the account's contacts: a discovered
/// service endpoint paired with one of its authentication methods.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Discovered {
    pub kind: DiscoveredKind,
    /// Username hint advertised by the mechanism (usually the email).
    pub username: Option<String>,
    /// How to authenticate against the endpoint.
    pub auth: DiscoveredAuth,
}

/// The discovered service kind, with its endpoint when the protocol is
/// an open standard (the proprietary APIs have fixed endpoints).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiscoveredKind {
    Carddav(Url),
    Jmap(Url),
    Msgraph,
    Google,
}

/// The discovered authentication method, reduced to what the CLI can
/// configure: a password, or a bearer token. OAuth 2.0 grants collapse
/// to a token; the CLI never runs a grant itself, it reads a token an
/// external manager (ortie) issues and refreshes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoveredAuth {
    Password,
    Token,
}

impl core::fmt::Display for Discovered {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let auth = match self.auth {
            DiscoveredAuth::Password => "password",
            DiscoveredAuth::Token => "token",
        };

        match &self.kind {
            DiscoveredKind::Carddav(url) => write!(f, "CardDAV {url} ({auth})"),
            DiscoveredKind::Jmap(url) => write!(f, "JMAP {url} ({auth})"),
            DiscoveredKind::Msgraph => write!(f, "Microsoft Graph API ({auth})"),
            DiscoveredKind::Google => write!(f, "Google People API ({auth})"),
        }
    }
}

/// Searches every contacts service reachable from `email`: one entry
/// per discovered CardDAV/JMAP endpoint and authentication method,
/// plus the proprietary contacts API of a detected Google or Microsoft
/// account.
pub fn search(email: &str) -> Result<Vec<Discovered>> {
    let resolver = Url::parse(DEFAULT_RESOLVER).expect("DEFAULT_RESOLVER must be a valid URL");
    let tls = Tls {
        rustls: pimalaya_stream::tls::Rustls {
            alpn: vec!["http/1.1".into()],
            ..Default::default()
        },
        ..Default::default()
    };

    let client = SearchClientStd::new(resolver, tls);
    let services = BTreeSet::from([Service::Carddav, Service::Jmap]);
    let configs = client.search_all(email, services)?;

    let mut found = Vec::new();
    let mut provider = email
        .rsplit_once('@')
        .and_then(|(_, domain)| Provider::from_domain(domain));

    for config in configs {
        // NOTE: MX-detected providers (custom domains on Google
        // Workspace or Microsoft 365) surface through the source of
        // their fixed configs.
        if let ConfigSource::Provider(detected) = config.source {
            provider.get_or_insert(detected);
        }

        let Endpoint::Http(raw) = &config.endpoint else {
            continue;
        };
        let Ok(url) = Url::parse(raw) else {
            continue;
        };

        let kind = match config.service {
            Service::Carddav => DiscoveredKind::Carddav(url),
            Service::Jmap => DiscoveredKind::Jmap(url),
            _ => continue,
        };

        for auth in &config.auth {
            let auth = match auth {
                AuthMethod::Password => DiscoveredAuth::Password,
                // Bearer and every OAuth 2.0 grant reduce to pasting a
                // token; the CLI does not run grants (see [`DiscoveredAuth`]).
                _ => DiscoveredAuth::Token,
            };

            let entry = Discovered {
                kind: kind.clone(),
                username: config.username.clone(),
                auth,
            };
            if !found.contains(&entry) {
                found.push(entry);
            }
        }
    }

    match provider {
        Some(Provider::Google) => found.push(Discovered {
            kind: DiscoveredKind::Google,
            username: Some(email.to_string()),
            auth: DiscoveredAuth::Token,
        }),
        Some(Provider::Microsoft) => found.push(Discovered {
            kind: DiscoveredKind::Msgraph,
            username: Some(email.to_string()),
            auth: DiscoveredAuth::Token,
        }),
        None => (),
    }

    Ok(found)
}
