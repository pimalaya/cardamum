//! # Service discovery
//!
//! Feeds the input to io-pim-discovery's parallel discovery and turns what
//! it found into options, each a protocol at its server signed in one way
//! (a password, an API token or OAuth 2.0), ranked so the first is the
//! Auto pick the wizard takes without a question.
//!
//! Discovery runs the fixed provider rules (by domain, and by MX through
//! autoconfig), PACC, the RFC 6764 CardDAV and RFC 8620 JMAP resolves,
//! then a WWW-Authenticate probe refining the advertised schemes. The
//! People API and Microsoft Graph contacts only come out of a provider
//! rule.

use std::{collections::BTreeSet, env, fmt, time::Duration};

use anyhow::Result;
use io_pim_discovery::{
    compose::{
        client::DiscoveryComposeClientStd,
        config::{
            DiscoveryAuthMethod, DiscoveryEndpoint, DiscoveryService, DiscoveryServiceConfig,
        },
    },
    shared::dns::system_resolver,
};
use pimalaya_stream::tls::{Rustls, Tls};
use url::Url;

/// DNS-over-TCP resolver backing discovery when `CARDAMUM_DNS_RESOLVER`
/// is unset and no system resolver is found: Cloudflare's `1.1.1.1`.
const DEFAULT_RESOLVER: &str = "tcp://1.1.1.1:53";

/// Upper bound on the parallel discovery fan-out.
///
/// An unreachable endpoint (a firewalled port, a black-hole host) must not
/// stall the interactive wizard, so mechanisms that have not reported by
/// then are abandoned and only what completed in time is offered.
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(8);

/// One way to reach the account's contacts: a protocol at its server,
/// signed in one way.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Discovered {
    /// The protocol, and its server for the open standards.
    pub kind: DiscoveredKind,
    /// How the sign-in goes.
    pub sign_in: SignIn,
    /// The login the server advertised, when it did.
    pub username: Option<String>,
}

/// The protocol of an option, carrying its endpoint for the open
/// standards (the provider APIs have fixed endpoints).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiscoveredKind {
    /// A CardDAV context root.
    Carddav(Url),
    /// A JMAP session endpoint.
    Jmap(Url),
    /// The Microsoft Graph contacts API.
    Msgraph,
    /// The People API.
    Gpeople,
}

/// How an option signs in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignIn {
    /// A login and its password, sent as HTTP Basic.
    Password,
    /// A token the user generated, sent as HTTP Bearer.
    Token,
    /// An OAuth 2.0 token a broker issues, sent as HTTP Bearer.
    Oauth,
}

impl Discovered {
    /// The review's row for this option, as `CardDAV at dav.example.org,
    /// with your password`.
    pub fn sentence(&self, ortie: bool) -> String {
        format!("{}, {}", self.kind.place(), self.sign_in.phrase(ortie))
    }

    /// The login a password signs in with: the advertised username when it
    /// is an address, else the typed `address`.
    pub fn login(&self, address: Option<&str>) -> Option<String> {
        self.username
            .as_deref()
            .filter(|username| looks_like_address(username))
            .or(address.filter(|address| looks_like_address(address)))
            .map(ToString::to_string)
    }
}

impl DiscoveredKind {
    /// The protocol name, as the review and the Change list show it.
    pub fn protocol(&self) -> &'static str {
        match self {
            Self::Carddav(_) => "CardDAV",
            Self::Jmap(_) => "JMAP",
            Self::Msgraph => "Microsoft Graph",
            Self::Gpeople => "People API",
        }
    }

    /// The server of an open standard, `None` for a provider API.
    pub fn server(&self) -> Option<&Url> {
        match self {
            Self::Carddav(url) | Self::Jmap(url) => Some(url),
            Self::Msgraph | Self::Gpeople => None,
        }
    }

    /// The server's host, with its port when it is not the scheme's
    /// default, `None` for a provider API.
    pub fn host(&self) -> Option<String> {
        let url = self.server()?;
        let host = url.host_str().unwrap_or_default();

        Some(match url.port() {
            Some(port) => format!("{host}:{port}"),
            None => host.to_string(),
        })
    }

    /// The protocol at its server's host, as `CardDAV at dav.example.org`,
    /// or the provider API alone.
    pub fn place(&self) -> String {
        match self.host() {
            Some(host) => format!("{} at {host}", self.protocol()),
            None => self.protocol().to_string(),
        }
    }

    /// The io-pim-discovery wire name of the service, which Ortie takes and
    /// the keyring keys end with.
    pub fn service(&self) -> &'static str {
        match self {
            Self::Carddav(_) => "carddav",
            Self::Jmap(_) => "jmap",
            Self::Msgraph => "msgraphContacts",
            Self::Gpeople => "gpeople",
        }
    }

    /// Whether this build compiles the backend reaching it.
    pub fn supported(&self) -> bool {
        match self {
            Self::Carddav(_) => cfg!(feature = "carddav"),
            Self::Jmap(_) => cfg!(feature = "jmap"),
            Self::Msgraph => cfg!(feature = "msgraph"),
            Self::Gpeople => cfg!(feature = "gpeople"),
        }
    }

    /// The protocol's rank: a provider's own API first, then JMAP before
    /// CardDAV.
    fn rank(&self) -> u8 {
        match self {
            Self::Msgraph | Self::Gpeople => 0,
            Self::Jmap(_) => 1,
            Self::Carddav(_) => 2,
        }
    }
}

impl SignIn {
    /// How the review words this sign-in, OAuth reading as a browser
    /// sign-in when Ortie runs it.
    pub fn phrase(self, ortie: bool) -> &'static str {
        match self {
            Self::Password => "with your password",
            Self::Token => "with an API token",
            Self::Oauth if ortie => "through your browser",
            Self::Oauth => "with a token from a broker",
        }
    }

    /// The sign-in's rank within one server: OAuth first when Ortie can run
    /// it, a password first otherwise, an API token in between.
    fn rank(self, ortie: bool) -> u8 {
        match (self, ortie) {
            (Self::Oauth, true) | (Self::Password, false) => 0,
            (Self::Token, _) => 1,
            (Self::Password, true) | (Self::Oauth, false) => 2,
        }
    }
}

impl fmt::Display for SignIn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Password => f.write_str("Password"),
            Self::Token => f.write_str("API token"),
            Self::Oauth => f.write_str("OAuth 2.0"),
        }
    }
}

/// Searches every contacts service reachable from `query` (an address, or
/// `@domain`) and returns its options ranked, Auto first.
///
/// Only the services this build compiles are asked for, so a build with
/// no network backend runs no discovery at all.
pub fn search(query: &str, ortie: bool) -> Result<Vec<Discovered>> {
    let services = services();

    if services.is_empty() {
        return Ok(Vec::new());
    }

    let client = DiscoveryComposeClientStd::new(discovery_resolver(), discovery_tls());
    let configs = client.compose_all_within(query, services, DISCOVERY_TIMEOUT)?;

    let mut found = options_of(&configs);
    rank(&mut found, ortie);

    Ok(found)
}

/// Resolver used by discovery: the `CARDAMUM_DNS_RESOLVER` override first,
/// then the system resolver, then the Cloudflare default.
///
/// Preferring the system resolver avoids leaking the email domain to a
/// third party, and the override works around networks blocking the
/// default.
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

/// The discovery services of the backends this build compiles.
///
/// An empty set would ask io-pim-discovery for every service, so the
/// caller skips discovery on it.
fn services() -> BTreeSet<DiscoveryService> {
    [
        (cfg!(feature = "carddav"), DiscoveryService::Carddav),
        (cfg!(feature = "jmap"), DiscoveryService::Jmap),
        (cfg!(feature = "gpeople"), DiscoveryService::Gpeople),
        (cfg!(feature = "msgraph"), DiscoveryService::MsgraphContacts),
    ]
    .into_iter()
    .filter_map(|(compiled, service)| compiled.then_some(service))
    .collect()
}

/// Turns the discovered configs into options, one per protocol, server and
/// sign-in, in discovery order.
///
/// An option already there absorbs a later one reading the same, taking
/// its advertised username when it had none.
fn options_of(configs: &[DiscoveryServiceConfig]) -> Vec<Discovered> {
    let mut found: Vec<Discovered> = Vec::new();

    for config in configs {
        let Some(kind) = kind_of(config) else {
            continue;
        };

        for sign_in in sign_ins_of(&kind, &config.auth) {
            let same = found
                .iter_mut()
                .find(|option| option.kind == kind && option.sign_in == sign_in);

            match same {
                Some(option) => {
                    if option.username.is_none() {
                        option.username = config.username.clone();
                    }
                }
                None => found.push(Discovered {
                    kind: kind.clone(),
                    sign_in,
                    username: config.username.clone(),
                }),
            }
        }
    }

    found
}

/// The option kind of a discovered config, `None` for a service the
/// wizard does not configure or an endpoint that is not a URL.
fn kind_of(config: &DiscoveryServiceConfig) -> Option<DiscoveredKind> {
    let DiscoveryEndpoint::Http(raw) = &config.endpoint else {
        return None;
    };

    let url = Url::parse(raw).ok()?;

    match config.service {
        DiscoveryService::Carddav => Some(DiscoveredKind::Carddav(url)),
        DiscoveryService::Jmap => Some(DiscoveredKind::Jmap(url)),
        DiscoveryService::Gpeople => Some(DiscoveredKind::Gpeople),
        DiscoveryService::MsgraphContacts => Some(DiscoveredKind::Msgraph),
        _ => None,
    }
}

/// The sign-ins a config's methods offer, each once: every OAuth grant is
/// one OAuth sign-in, and a server advertising nothing offers a password
/// and an API token. A provider API signs in through OAuth alone.
fn sign_ins_of(kind: &DiscoveredKind, auth: &[DiscoveryAuthMethod]) -> Vec<SignIn> {
    let mut sign_ins = Vec::new();

    for method in auth {
        let sign_in = match method {
            DiscoveryAuthMethod::Password => SignIn::Password,
            DiscoveryAuthMethod::Bearer => SignIn::Token,
            _ => SignIn::Oauth,
        };

        if !sign_ins.contains(&sign_in) {
            sign_ins.push(sign_in);
        }
    }

    if kind.server().is_none() {
        sign_ins.retain(|sign_in| *sign_in == SignIn::Oauth);
    } else if sign_ins.is_empty() {
        sign_ins = vec![SignIn::Password, SignIn::Token];
    }

    sign_ins
}

/// Ranks `found` in place: by protocol, then by server in discovery
/// order, then by sign-in.
fn rank(found: &mut [Discovered], ortie: bool) {
    let mut servers: Vec<DiscoveredKind> = Vec::new();

    for option in found.iter() {
        if !servers.contains(&option.kind) {
            servers.push(option.kind.clone());
        }
    }

    found.sort_by_key(|option| {
        let server = servers.iter().position(|kind| *kind == option.kind);
        (option.kind.rank(), server, option.sign_in.rank(ortie))
    });
}

/// Whether a string is a full `local@domain` address (both parts
/// non-empty), rejecting the bare-domain `@domain` form.
fn looks_like_address(value: &str) -> bool {
    value
        .split_once('@')
        .is_some_and(|(local, domain)| !local.is_empty() && !domain.is_empty())
}

/// TLS profile for the HTTPS-bound discovery mechanisms, which only speak
/// HTTP/1.1 to `.well-known` endpoints.
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
    use io_pim_discovery::compose::config::DiscoveryConfigSource;

    use crate::wizard::search::*;

    /// A config for `service` at the HTTP `endpoint`, signing in with
    /// `auth`.
    fn config(
        service: DiscoveryService,
        endpoint: &str,
        auth: Vec<DiscoveryAuthMethod>,
    ) -> DiscoveryServiceConfig {
        DiscoveryServiceConfig {
            service,
            endpoint: DiscoveryEndpoint::Http(endpoint.to_string()),
            username: None,
            auth,
            source: DiscoveryConfigSource::Pacc,
            resolved: false,
        }
    }

    /// An authorization code grant at `issuer`.
    fn grant(issuer: &str) -> DiscoveryAuthMethod {
        DiscoveryAuthMethod::OauthAuthorizationCodeGrant {
            authorization_endpoint: format!("{issuer}/authorize"),
            token_endpoint: format!("{issuer}/token"),
            scope: None,
            resource: None,
            scopes_supported: Vec::new(),
        }
    }

    /// The (protocol, sign-in) pairs of `found`, in order.
    fn plan(found: &[Discovered]) -> Vec<(&'static str, SignIn)> {
        found
            .iter()
            .map(|option| (option.kind.protocol(), option.sign_in))
            .collect()
    }

    fn carddav(url: &str) -> DiscoveredKind {
        DiscoveredKind::Carddav(Url::parse(url).unwrap())
    }

    #[test]
    fn two_options_reading_the_same_are_one() {
        let device = DiscoveryAuthMethod::OauthDeviceAuthorizationGrant {
            device_authorization_endpoint: String::from("https://auth.example.org/device"),
            token_endpoint: String::from("https://auth.example.org/token"),
            scope: None,
            resource: None,
            scopes_supported: Vec::new(),
        };

        let mut advertised = config(
            DiscoveryService::Carddav,
            "https://dav.example.org/dav/",
            vec![DiscoveryAuthMethod::Password],
        );
        advertised.username = Some(String::from("alice@example.org"));

        let configs = [
            config(
                DiscoveryService::Carddav,
                "https://dav.example.org/dav/",
                vec![DiscoveryAuthMethod::Password],
            ),
            advertised,
            config(
                DiscoveryService::Jmap,
                "https://api.example.org/jmap/session",
                vec![
                    grant("https://auth.example.org"),
                    device,
                    DiscoveryAuthMethod::OauthIssuer(String::from("https://auth.example.org")),
                ],
            ),
        ];

        let found = options_of(&configs);

        assert_eq!(
            plan(&found),
            [("CardDAV", SignIn::Password), ("JMAP", SignIn::Oauth)]
        );
        assert_eq!(found[0].username.as_deref(), Some("alice@example.org"));
    }

    #[test]
    fn a_server_advertising_no_sign_in_offers_a_password_and_a_token() {
        let configs = [config(
            DiscoveryService::Jmap,
            "https://api.example.org/jmap/session",
            Vec::new(),
        )];

        assert_eq!(
            plan(&options_of(&configs)),
            [("JMAP", SignIn::Password), ("JMAP", SignIn::Token)]
        );
    }

    #[test]
    fn a_provider_api_signs_in_through_oauth_alone() {
        let configs = [
            config(
                DiscoveryService::MsgraphContacts,
                "https://graph.example.org/v1.0/",
                vec![
                    grant("https://login.example.org"),
                    DiscoveryAuthMethod::Password,
                ],
            ),
            config(
                DiscoveryService::Imap,
                "https://imap.example.org",
                Vec::new(),
            ),
        ];

        assert_eq!(
            plan(&options_of(&configs)),
            [("Microsoft Graph", SignIn::Oauth)]
        );
    }

    #[test]
    fn auto_is_oauth_first_with_ortie_and_a_password_first_without() {
        let configs = [
            config(
                DiscoveryService::Carddav,
                "https://dav.example.org/",
                vec![DiscoveryAuthMethod::Password],
            ),
            config(
                DiscoveryService::Jmap,
                "https://api.example.org/jmap/session",
                vec![
                    DiscoveryAuthMethod::Password,
                    DiscoveryAuthMethod::Bearer,
                    grant("https://auth.example.org"),
                ],
            ),
        ];

        let mut found = options_of(&configs);
        rank(&mut found, true);

        assert_eq!(
            plan(&found),
            [
                ("JMAP", SignIn::Oauth),
                ("JMAP", SignIn::Token),
                ("JMAP", SignIn::Password),
                ("CardDAV", SignIn::Password),
            ]
        );

        rank(&mut found, false);

        assert_eq!(
            plan(&found),
            [
                ("JMAP", SignIn::Password),
                ("JMAP", SignIn::Token),
                ("JMAP", SignIn::Oauth),
                ("CardDAV", SignIn::Password),
            ]
        );
    }

    #[test]
    fn a_provider_api_ranks_first_and_servers_keep_their_order() {
        let configs = [
            config(
                DiscoveryService::Carddav,
                "https://first.example.org/",
                vec![DiscoveryAuthMethod::Password],
            ),
            config(
                DiscoveryService::Carddav,
                "https://second.example.org/",
                vec![grant("https://auth.example.org")],
            ),
            config(
                DiscoveryService::Gpeople,
                "https://people.example.org/",
                vec![grant("https://auth.example.org")],
            ),
        ];

        let mut found = options_of(&configs);
        rank(&mut found, true);

        assert_eq!(found[0].kind, DiscoveredKind::Gpeople);
        assert_eq!(found[1].kind, carddav("https://first.example.org/"));
        assert_eq!(found[1].sign_in, SignIn::Password);
        assert_eq!(found[2].kind, carddav("https://second.example.org/"));
    }

    #[test]
    fn the_login_is_an_advertised_address_else_the_typed_one() {
        let mut option = Discovered {
            kind: carddav("https://dav.example.org/"),
            sign_in: SignIn::Password,
            username: Some(String::from("alice")),
        };

        assert_eq!(
            option.login(Some("alice@example.org")).as_deref(),
            Some("alice@example.org")
        );
        assert_eq!(option.login(None), None);
        assert_eq!(option.login(Some("@example.org")), None);

        option.username = Some(String::from("a.smith@example.net"));

        assert_eq!(
            option.login(Some("alice@example.org")).as_deref(),
            Some("a.smith@example.net")
        );
    }

    #[test]
    fn the_review_names_the_pick_in_plain_words() {
        let option = |kind, sign_in| Discovered {
            kind,
            sign_in,
            username: None,
        };

        let jmap = DiscoveredKind::Jmap(Url::parse("https://api.example.org:8443/jmap").unwrap());

        assert_eq!(
            option(carddav("https://dav.example.org/dav/"), SignIn::Password).sentence(true),
            "CardDAV at dav.example.org, with your password"
        );
        assert_eq!(
            option(jmap.clone(), SignIn::Token).sentence(true),
            "JMAP at api.example.org:8443, with an API token"
        );
        assert_eq!(
            option(jmap, SignIn::Oauth).sentence(true),
            "JMAP at api.example.org:8443, through your browser"
        );
        assert_eq!(
            option(DiscoveredKind::Gpeople, SignIn::Oauth).sentence(true),
            "People API, through your browser"
        );
        assert_eq!(
            option(DiscoveredKind::Msgraph, SignIn::Oauth).sentence(false),
            "Microsoft Graph, with a token from a broker"
        );
    }
}
