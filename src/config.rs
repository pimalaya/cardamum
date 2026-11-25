//! # Configuration
//!
//! The TOML configuration file: the options shared by every account, the
//! accounts themselves and their per-backend blocks.
//!
//! The reference document, key by key, is config.sample.toml.

use std::collections::HashMap;
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople",
    feature = "pimdir"
))]
use std::path::PathBuf;

#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
use anyhow::Result;
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
use anyhow::bail;
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
use pimalaya_config::secret::{Secret, SecretResolver};
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
use pimalaya_config::toml::opt_shell_expanded_path;
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople",
    feature = "pimdir"
))]
use pimalaya_config::toml::shell_expanded_path;
#[cfg(any(feature = "vdir", feature = "carddav", feature = "jmap"))]
use pimalaya_config::toml::shell_expanded_string;
use pimalaya_config::{command::CommandConfig, toml::TomlConfig};
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
use pimalaya_stream::{
    proxy::{Proxy, ProxyAuth},
    tls::{Rustls, RustlsCrypto, Tls, TlsClient, TlsProvider, TlsServer},
};
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
use secrecy::SecretString;
use serde::Deserialize;
#[cfg(not(all(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople",
    feature = "vdir",
    feature = "pimdir"
)))]
use serde::de::IgnoredAny;
#[cfg(feature = "carddav")]
use serde::{Deserializer, de::Error as _};
#[cfg(any(feature = "carddav", feature = "jmap"))]
use url::{ParseError, Url};

/// The whole configuration file: the options shared by every account,
/// then the accounts themselves, refusing a key it does not know.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Config {
    /// Addressbook options, overridable per account.
    #[serde(default)]
    pub addressbook: AddressbookConfig,
    /// Card options, overridable per account.
    #[serde(default)]
    pub card: CardConfig,
    /// The accounts, keyed by their `[accounts.<name>]` table name.
    pub accounts: HashMap<String, AccountConfig>,
}

impl TomlConfig for Config {
    type Account = AccountConfig;

    fn project_name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn sample_url() -> &'static str {
        "https://github.com/pimalaya/cardamum/blob/master/config.sample.toml"
    }

    fn accounts(&self) -> impl Iterator<Item = (&str, bool)> {
        let accounts = self.accounts.iter();
        accounts.map(|(name, account)| (name.as_str(), account.default))
    }

    fn take_named_account(&mut self, name: &str) -> Option<(String, Self::Account)> {
        let entry = self.accounts.remove_entry(name);

        #[cfg(any(
            feature = "carddav",
            feature = "jmap",
            feature = "msgraph",
            feature = "gpeople"
        ))]
        let entry = entry.map(|(name, mut account)| {
            account.inherit_proxy();
            account.inherit_tls();
            (name, account)
        });

        entry
    }
}

impl AccountConfig {
    /// The backends the account configures a block for, built or not, in
    /// the order `auto` resolves them.
    pub fn backends(&self) -> Vec<&'static str> {
        let declared = [
            self.vdir.is_some(),
            self.pimdir.is_some(),
            self.carddav.is_some(),
            self.jmap.is_some(),
            self.msgraph.is_some(),
            self.gpeople.is_some(),
        ];

        let names = crate::backend::NAMES.into_iter().zip(declared);
        names
            .filter(|(_, declared)| *declared)
            .map(|(name, _)| name)
            .collect()
    }

    /// Hands the account proxy to every network backend naming none of
    /// its own.
    #[cfg(any(
        feature = "carddav",
        feature = "jmap",
        feature = "msgraph",
        feature = "gpeople"
    ))]
    fn inherit_proxy(&mut self) {
        let Some(proxy) = &self.proxy else {
            return;
        };

        let slots = [
            #[cfg(feature = "carddav")]
            self.carddav.as_mut().map(|c| &mut c.proxy),
            #[cfg(feature = "jmap")]
            self.jmap.as_mut().map(|c| &mut c.proxy),
            #[cfg(feature = "msgraph")]
            self.msgraph.as_mut().map(|c| &mut c.proxy),
            #[cfg(feature = "gpeople")]
            self.gpeople.as_mut().map(|c| &mut c.proxy),
        ];

        for slot in slots.into_iter().flatten() {
            slot.get_or_insert_with(|| proxy.clone());
        }
    }

    /// Hands the account TLS to every network backend, each field its
    /// own `tls` leaves unset taking the account's.
    #[cfg(any(
        feature = "carddav",
        feature = "jmap",
        feature = "msgraph",
        feature = "gpeople"
    ))]
    fn inherit_tls(&mut self) {
        let Some(tls) = &self.tls else {
            return;
        };

        let slots = [
            #[cfg(feature = "carddav")]
            self.carddav.as_mut().map(|c| &mut c.tls),
            #[cfg(feature = "jmap")]
            self.jmap.as_mut().map(|c| &mut c.tls),
            #[cfg(feature = "msgraph")]
            self.msgraph.as_mut().map(|c| &mut c.tls),
            #[cfg(feature = "gpeople")]
            self.gpeople.as_mut().map(|c| &mut c.tls),
        ];

        for slot in slots.into_iter().flatten() {
            slot.inherit(tls);
        }
    }
}

/// Account configuration, refusing a key it does not know.
///
/// The block of a backend the build leaves out loads unread, so one file
/// serves every build, and a command reaching it fails coded
/// `backend-not-built`.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AccountConfig {
    /// Whether a command passing no `-a <NAME>` runs against it.
    #[serde(default)]
    pub default: bool,
    /// Addressbook options, overriding the global ones.
    #[serde(default)]
    pub addressbook: AddressbookConfig,
    /// Card options, overriding the global ones.
    #[serde(default)]
    pub card: CardConfig,
    /// Proxy every network backend of this account goes through, unless
    /// its own block names one.
    #[cfg(any(
        feature = "carddav",
        feature = "jmap",
        feature = "msgraph",
        feature = "gpeople"
    ))]
    #[serde(default)]
    pub proxy: Option<ProxyConfig>,
    /// TLS every network backend of this account uses, each field its own
    /// `tls` leaves unset taking the account's.
    #[cfg(any(
        feature = "carddav",
        feature = "jmap",
        feature = "msgraph",
        feature = "gpeople"
    ))]
    pub tls: Option<TlsConfig>,
    /// The local vdir home this account reads.
    #[cfg(feature = "vdir")]
    pub vdir: Option<VdirConfig>,
    /// The local pimdir store this account reads.
    #[cfg(feature = "pimdir")]
    pub pimdir: Option<PimdirConfig>,
    /// The CardDAV server this account talks to.
    #[cfg(feature = "carddav")]
    pub carddav: Option<CarddavConfig>,
    /// The JMAP server this account talks to.
    #[cfg(feature = "jmap")]
    pub jmap: Option<JmapConfig>,
    /// The Microsoft Graph account this account talks to.
    #[cfg(feature = "msgraph")]
    pub msgraph: Option<MsgraphConfig>,
    /// The Google People account this account talks to.
    #[cfg(feature = "gpeople")]
    pub gpeople: Option<GpeopleConfig>,
    /// The account proxy, unread by a build with no network backend.
    #[cfg(not(any(
        feature = "carddav",
        feature = "jmap",
        feature = "msgraph",
        feature = "gpeople"
    )))]
    #[serde(default)]
    #[allow(dead_code)]
    pub proxy: Option<IgnoredAny>,
    /// The account TLS, unread by a build with no network backend.
    #[cfg(not(any(
        feature = "carddav",
        feature = "jmap",
        feature = "msgraph",
        feature = "gpeople"
    )))]
    #[serde(default)]
    #[allow(dead_code)]
    pub tls: Option<IgnoredAny>,
    /// The vdir block, unread by a build without vdir.
    #[cfg(not(feature = "vdir"))]
    #[serde(default)]
    #[allow(dead_code)]
    pub vdir: Option<IgnoredAny>,
    /// The pimdir block, unread by a build without pimdir.
    #[cfg(not(feature = "pimdir"))]
    #[serde(default)]
    #[allow(dead_code)]
    pub pimdir: Option<IgnoredAny>,
    /// The CardDAV block, unread by a build without CardDAV.
    #[cfg(not(feature = "carddav"))]
    #[serde(default)]
    #[allow(dead_code)]
    pub carddav: Option<IgnoredAny>,
    /// The JMAP block, unread by a build without JMAP.
    #[cfg(not(feature = "jmap"))]
    #[serde(default)]
    #[allow(dead_code)]
    pub jmap: Option<IgnoredAny>,
    /// The Microsoft Graph block, unread by a build without Graph.
    #[cfg(not(feature = "msgraph"))]
    #[serde(default)]
    #[allow(dead_code)]
    pub msgraph: Option<IgnoredAny>,
    /// The Google People block, unread by a build without People.
    #[cfg(not(feature = "gpeople"))]
    #[serde(default)]
    #[allow(dead_code)]
    pub gpeople: Option<IgnoredAny>,
}

/// Vdir configuration.
#[cfg(feature = "vdir")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct VdirConfig {
    /// The vdir home: the directory holding the account's collections, one
    /// addressbook per immediate subdirectory.
    ///
    /// Environment variables and a leading tilde are expanded as the value
    /// is read, as `pimdir.root`.
    #[serde(deserialize_with = "shell_expanded_string")]
    pub root: String,
}

/// pimdir configuration.
#[cfg(feature = "pimdir")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct PimdirConfig {
    /// The store directory, holding `pimdir.db` and `objects/`.
    ///
    /// Environment variables and a leading tilde are expanded as the
    /// value is read. It must already exist: creating one is the sync
    /// engine's job.
    #[serde(deserialize_with = "shell_expanded_path")]
    pub root: PathBuf,
    /// The account the collections are grouped under, for a store shared
    /// by several accounts or domains, as the mobile apps do.
    ///
    /// Unset reads the one account the store groups its collections under,
    /// or none; a store grouping several is refused until one is named.
    #[serde(default)]
    pub account: Option<String>,
}

/// CardDAV configuration.
#[cfg(feature = "carddav")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CarddavConfig {
    /// DAV context root, where the principal and home set walk starts.
    ///
    /// Accepts a full URL, a bare domain or `domain:port`, a bare
    /// authority defaulting to `https`. A value naming no path is probed
    /// once through `.well-known/carddav`, a bare origin not being
    /// necessarily the context root.
    pub server: Option<String>,
    /// Pre-resolved addressbook home set URL, where the client lists
    /// addressbooks, skipping every discovery step. Its host is ASCII,
    /// an internationalised one written in Punycode.
    #[serde(default, deserialize_with = "ascii_url")]
    pub home: Option<Url>,
    /// TLS configuration.
    #[serde(default)]
    pub tls: TlsConfig,
    /// ALPN protocol identifiers offered during the TLS handshake.
    #[serde(default = "default_http_alpn")]
    pub alpn: Vec<String>,
    /// Proxy the connection goes through, falling back to the account one
    /// and then to the `all_proxy`/`https_proxy` environment variables.
    #[serde(default)]
    pub proxy: Option<ProxyConfig>,
    /// Authentication configuration, absent sending no credential.
    #[serde(default)]
    pub auth: CarddavAuthConfig,
}

/// CardDAV authentication configuration.
#[cfg(feature = "carddav")]
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum CarddavAuthConfig {
    /// No credentials, for a server that asks for none: no `auth` at all
    /// reads as this.
    #[default]
    None,
    /// HTTP Basic authentication, per RFC 7617.
    Basic {
        /// The username the server knows the principal by.
        #[serde(deserialize_with = "shell_expanded_string")]
        username: String,
        /// The matching password, read from the configured secret.
        password: Secret,
    },
    /// HTTP Bearer authentication, per RFC 6750.
    Bearer { token: Secret },
}

/// JMAP configuration (RFC 8620 + RFC 9610).
#[cfg(feature = "jmap")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct JmapConfig {
    /// The JMAP server address.
    ///
    /// A bare authority (`fastmail.com`, `mail.example.com:8080`) is
    /// discovered through `GET /.well-known/jmap`, a full URL reaches the
    /// session endpoint directly. Schemes: `http`, `https`, `jmap` and
    /// `jmaps`, the last two mapping to the first two.
    pub server: String,
    /// TLS configuration.
    #[serde(default)]
    pub tls: TlsConfig,
    /// ALPN identifiers offered during the TLS handshake, `["http/1.1"]`
    /// by default since JMAP rides on HTTP/1.1.
    ///
    /// An empty list skips ALPN negotiation. Only rustls reads it,
    /// `native-tls` ignores ALPN.
    #[serde(default = "io_jmap::client::JmapClientStd::default_alpn")]
    pub alpn: Vec<String>,
    /// Proxy the connection goes through, falling back to the account one
    /// and then to the `all_proxy`/`https_proxy` environment variables.
    #[serde(default)]
    pub proxy: Option<ProxyConfig>,
    /// Authentication configuration.
    pub auth: JmapAuthConfig,
}

/// JMAP authentication configuration.
#[cfg(feature = "jmap")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum JmapAuthConfig {
    /// A whole Authorization header value, sent verbatim.
    Header(Secret),
    /// A bearer token, OAuth 2.0 access token or provider API token.
    Bearer { token: Secret },
    /// HTTP Basic authentication.
    Basic {
        /// The username the server knows the account by.
        #[serde(deserialize_with = "shell_expanded_string")]
        username: String,
        /// The matching password, read from the configured secret.
        password: Secret,
    },
}

/// Microsoft Graph configuration.
#[cfg(feature = "msgraph")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct MsgraphConfig {
    /// Graph user id owning the contacts, `me` by default.
    ///
    /// A user id or a principal name targets another mailbox.
    #[serde(default = "default_msgraph_user_id")]
    pub user_id: String,
    /// TLS configuration.
    #[serde(default)]
    pub tls: TlsConfig,
    /// ALPN identifiers offered during the TLS handshake, `["http/1.1"]`
    /// by default since the Graph API rides on HTTP/1.1.
    ///
    /// An empty list skips ALPN negotiation. Only rustls reads it,
    /// `native-tls` ignores ALPN.
    #[serde(default = "default_http_alpn")]
    pub alpn: Vec<String>,
    /// Proxy the connection goes through, falling back to the account one
    /// and then to the `all_proxy`/`https_proxy` environment variables.
    #[serde(default)]
    pub proxy: Option<ProxyConfig>,
    /// Authentication configuration.
    pub auth: MsgraphAuthConfig,
}

/// Microsoft Graph authentication configuration.
#[cfg(feature = "msgraph")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct MsgraphAuthConfig {
    /// OAuth 2.0 access token sent as `Bearer <token>`, the only
    /// authorization the Graph API accepts.
    pub token: Secret,
}

#[cfg(feature = "msgraph")]
fn default_msgraph_user_id() -> String {
    String::from("me")
}

/// Google People configuration.
#[cfg(feature = "gpeople")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct GpeopleConfig {
    /// TLS configuration.
    #[serde(default)]
    pub tls: TlsConfig,
    /// ALPN identifiers offered during the TLS handshake, `["http/1.1"]`
    /// by default since the People API rides on HTTP/1.1.
    ///
    /// An empty list skips ALPN negotiation. Only rustls reads it,
    /// `native-tls` ignores ALPN.
    #[serde(default = "default_http_alpn")]
    pub alpn: Vec<String>,
    /// Proxy the connection goes through, falling back to the account one
    /// and then to the `all_proxy`/`https_proxy` environment variables.
    #[serde(default)]
    pub proxy: Option<ProxyConfig>,
    /// Authentication configuration.
    pub auth: GpeopleAuthConfig,
}

/// Google People authentication configuration.
#[cfg(feature = "gpeople")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct GpeopleAuthConfig {
    /// OAuth 2.0 access token sent as `Bearer <token>`, the only
    /// authorization the People API accepts.
    pub token: Secret,
}

#[cfg(any(feature = "carddav", feature = "msgraph", feature = "gpeople"))]
fn default_http_alpn() -> Vec<String> {
    vec![String::from("http/1.1")]
}

/// Addressbook-level options.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AddressbookConfig {
    /// Addressbook id the `card` commands take when their
    /// `-k/--addressbook` flag is omitted.
    pub default: Option<String>,
}

/// Card-level options.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CardConfig {
    /// Command a card is edited through, spawned on the path of a
    /// temporary vCard file it edits in place.
    ///
    /// A shell line or an argv list, the path appended as its last
    /// argument: `composer = "tcard edit"` runs `tcard edit <PATH>`.
    pub composer: Option<CommandConfig>,
    /// `card list` options.
    #[serde(default)]
    pub list: CardListConfig,
}

/// `card list` options.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CardListConfig {
    /// Default `-s/--page-size` value.
    ///
    /// The flag wins when passed, then this value, then the hard
    /// fallback of 25.
    pub page_size: Option<u32>,
}

/// Proxy configuration.
///
/// `url` is a `socks5://`, `socks5h://` or `http://` proxy URL. Its user
/// info authenticates too, but `username` and `password` keep the secret
/// out of the URL.
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ProxyConfig {
    /// The proxy URL.
    pub url: String,
    /// The proxy username, required by `password`.
    pub username: Option<String>,
    /// The proxy password.
    pub password: Option<Secret>,
}

#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
impl ProxyConfig {
    /// Resolves an optional configuration, an absent one reading the
    /// environment at connect time.
    pub fn resolve(config: Option<Self>, resolver: &mut SecretResolver) -> Result<Proxy> {
        match config {
            Some(config) => config.try_into_proxy(resolver),
            None => Ok(Proxy::System),
        }
    }

    /// Resolves the configuration into a runtime [`Proxy`], the password
    /// going through `resolver`.
    pub fn try_into_proxy(self, resolver: &mut SecretResolver) -> Result<Proxy> {
        let mut proxy = Proxy::from_url(&self.url)?;

        let auth = match (self.username, self.password) {
            (None, None) => return Ok(proxy),
            (None, Some(_)) => bail!("Proxy password requires a username"),
            (Some(user), pass) => ProxyAuth {
                user,
                pass: match pass {
                    Some(pass) => resolver.resolve(pass)?,
                    None => SecretString::default(),
                },
            },
        };

        match &mut proxy {
            Proxy::Socks5 { auth: slot, .. } | Proxy::Http { auth: slot, .. } => {
                *slot = Some(auth);
            }
            Proxy::None | Proxy::System => {}
        }

        Ok(proxy)
    }
}

/// SSL/TLS configuration.
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TlsConfig {
    /// The TLS implementation the connection uses.
    pub provider: Option<TlsProviderConfig>,
    /// Rustls-only options.
    #[serde(default)]
    pub rustls: RustlsConfig,
    /// Path to an additional PEM certificate to trust.
    ///
    /// Environment variables and a leading tilde are expanded as the
    /// value is read.
    #[serde(default, deserialize_with = "opt_shell_expanded_path")]
    pub cert: Option<PathBuf>,
    /// A client certificate, presented when the server asks for one.
    pub client: Option<TlsClientConfig>,
}

#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
impl TlsConfig {
    /// Takes from `account` each field this block leaves unset: the
    /// provider, the rustls crypto, the trusted certificate and the
    /// client certificate.
    pub fn inherit(&mut self, account: &TlsConfig) {
        if self.provider.is_none() {
            self.provider.clone_from(&account.provider);
        }
        if self.rustls.crypto.is_none() {
            self.rustls.crypto.clone_from(&account.rustls.crypto);
        }
        if self.cert.is_none() {
            self.cert.clone_from(&account.cert);
        }
        if self.client.is_none() {
            self.client.clone_from(&account.client);
        }
    }
}

/// Client certificate configuration, both in PEM format.
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TlsClientConfig {
    /// The certificate, optionally followed by its intermediates.
    #[serde(deserialize_with = "shell_expanded_path")]
    pub cert: PathBuf,
    /// The private key, PKCS#8 under native-tls.
    #[serde(deserialize_with = "shell_expanded_path")]
    pub key: PathBuf,
}

/// SSL/TLS provider configuration.
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum TlsProviderConfig {
    /// The pure-Rust rustls stack.
    Rustls,
    /// The platform TLS stack, through native-tls.
    NativeTls,
}

/// Rustls configuration.
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RustlsConfig {
    /// The crypto provider rustls builds on.
    pub crypto: Option<RustlsCryptoConfig>,
}

/// Rustls crypto provider configuration.
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum RustlsCryptoConfig {
    /// aws-lc-rs.
    Aws,
    /// ring.
    Ring,
}

#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
impl TlsConfig {
    /// Converts the config into a [`Tls`] carrying the given ALPN
    /// protocol identifiers.
    pub fn into_tls(self, alpn: Vec<String>) -> Tls {
        Tls {
            provider: self.provider.map(|config| match config {
                TlsProviderConfig::Rustls => TlsProvider::Rustls,
                TlsProviderConfig::NativeTls => TlsProvider::NativeTls,
            }),
            rustls: Rustls {
                crypto: self.rustls.crypto.map(|config| match config {
                    RustlsCryptoConfig::Aws => RustlsCrypto::Aws,
                    RustlsCryptoConfig::Ring => RustlsCrypto::Ring,
                }),
                alpn,
            },
            server: self.cert.map(|cert| TlsServer { cert }),
            client: self.client.map(|client| TlsClient {
                cert: client.cert,
                key: client.key,
            }),
        }
    }
}

/// Parses a `server` config string into a URL.
///
/// A full URL is taken verbatim, a bare `host[:port]` takes
/// `default_scheme`, and a scheme outside `allowed` is rejected.
#[cfg(feature = "jmap")]
pub fn parse_server(server: &str, default_scheme: &str, allowed: &[&str]) -> Result<Url> {
    let url = if server.contains("://") {
        parse_url(server)?
    } else {
        parse_url(&format!("{default_scheme}://{server}"))?
    };

    let scheme = url.scheme();

    if !allowed.contains(&scheme) {
        bail!("Invalid server scheme `{scheme}`: expected one of {allowed:?}");
    }

    Ok(url)
}

/// Parses `input` into a URL, a host that is not ASCII refused with the
/// way out: its Punycode form, url linking no Unicode IDNA back end.
#[cfg(any(feature = "carddav", feature = "jmap"))]
pub fn parse_url(input: &str) -> Result<Url> {
    match Url::parse(input) {
        Err(ParseError::IdnaError) => bail!(
            "Host `{}` is not ASCII: write it in its Punycode form (xn--...)",
            host_of(input)
        ),
        result => Ok(result?),
    }
}

/// Deserializes an optional URL through [`parse_url`].
#[cfg(feature = "carddav")]
fn ascii_url<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Url>, D::Error> {
    let input = String::deserialize(deserializer)?;
    let url = parse_url(&input).map_err(D::Error::custom)?;
    Ok(Some(url))
}

/// The host `input` names, as written: what follows the scheme, short of
/// the path, the user info and the port.
#[cfg(any(feature = "carddav", feature = "jmap"))]
fn host_of(input: &str) -> &str {
    let rest = input.split_once("://").map_or(input, |(_, rest)| rest);
    let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    let host = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);

    match host.rsplit_once(':') {
        Some((name, port)) if port.bytes().all(|b| b.is_ascii_digit()) => name,
        _ => host,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[cfg(any(
        feature = "carddav",
        feature = "jmap",
        feature = "msgraph",
        feature = "gpeople",
        feature = "pimdir"
    ))]
    use std::{env::var, path::PathBuf};

    #[cfg(feature = "pimdir")]
    #[test]
    fn a_mandatory_path_expands_the_leading_tilde() {
        use super::PimdirConfig;

        let home = PathBuf::from(var("HOME").expect("HOME must be set"));
        let config: PimdirConfig = toml::from_str(r#"root = "~/Contacts""#).unwrap();

        assert_eq!(config.root, home.join("Contacts"));
    }

    #[cfg(any(
        feature = "carddav",
        feature = "jmap",
        feature = "msgraph",
        feature = "gpeople"
    ))]
    #[test]
    fn an_optional_path_expands_the_leading_tilde_and_stays_absent() {
        use super::TlsConfig;

        let home = PathBuf::from(var("HOME").expect("HOME must be set"));

        let config: TlsConfig = toml::from_str(r#"cert = "~/ca.pem""#).unwrap();
        assert_eq!(config.cert, Some(home.join("ca.pem")));

        let config: TlsConfig = toml::from_str("").unwrap();
        assert_eq!(config.cert, None);
    }

    /// A client certificate comes with its key, both paths expanded, and
    /// a certificate without its key is refused.
    #[cfg(any(
        feature = "carddav",
        feature = "jmap",
        feature = "msgraph",
        feature = "gpeople"
    ))]
    #[test]
    fn a_client_certificate_needs_its_key() {
        use super::TlsConfig;

        let home = PathBuf::from(var("HOME").expect("HOME must be set"));

        let config: TlsConfig =
            toml::from_str(r#"client = { cert = "~/me.pem", key = "~/me.key.pem" }"#).unwrap();
        let client = config.client.unwrap();
        assert_eq!(client.cert, home.join("me.pem"));
        assert_eq!(client.key, home.join("me.key.pem"));

        assert!(toml::from_str::<TlsConfig>(r#"client.cert = "~/me.pem""#).is_err());
    }

    /// A backend's `tls` takes from the account's each field it leaves
    /// unset, the client certificate as one.
    #[cfg(all(feature = "carddav", feature = "jmap"))]
    #[test]
    fn an_account_tls_fills_each_field_a_backend_leaves_unset() {
        use pimalaya_config::toml::TomlConfig;

        use super::{Config, TlsProviderConfig};

        let mut config: Config = toml::from_str(
            "[accounts.a]\n\
             tls.provider = \"native-tls\"\n\
             tls.cert = \"/ca.pem\"\n\
             tls.client = { cert = \"/me.pem\", key = \"/me.key\" }\n\
             carddav.server = \"dav.example.org\"\n\
             carddav.auth = \"none\"\n\
             carddav.tls.cert = \"/dav.pem\"\n\
             jmap.server = \"jmap.example.org\"\n\
             jmap.auth.bearer.token.raw = \"token\"",
        )
        .unwrap();

        let (_, account) = config.take_named_account("a").unwrap();
        let carddav = account.carddav.unwrap().tls;
        let jmap = account.jmap.unwrap().tls;

        assert!(matches!(
            carddav.provider,
            Some(TlsProviderConfig::NativeTls)
        ));
        assert_eq!(carddav.cert, Some(PathBuf::from("/dav.pem")));
        assert_eq!(carddav.client.unwrap().key, PathBuf::from("/me.key"));
        assert!(matches!(jmap.provider, Some(TlsProviderConfig::NativeTls)));
        assert_eq!(jmap.cert, Some(PathBuf::from("/ca.pem")));
    }

    #[cfg(feature = "carddav")]
    #[test]
    fn carddav_auth_parses_every_scheme() {
        use super::{CarddavAuthConfig, CarddavConfig};

        let credential_less: CarddavConfig =
            toml::from_str("server = \"dav.example.org\"\nauth = \"none\"").unwrap();
        assert!(matches!(credential_less.auth, CarddavAuthConfig::None));

        let basic: CarddavConfig = toml::from_str(
            "server = \"dav.example.org\"\n\
             auth.basic.username = \"me\"\n\
             auth.basic.password.raw = \"secret\"",
        )
        .unwrap();
        assert!(matches!(basic.auth, CarddavAuthConfig::Basic { .. }));

        let bearer: CarddavConfig =
            toml::from_str("server = \"dav.example.org\"\nauth.bearer.token.raw = \"token\"")
                .unwrap();
        assert!(matches!(bearer.auth, CarddavAuthConfig::Bearer { .. }));
    }

    /// A block with no `auth` loads, sending no credential, as
    /// `auth = "none"` does.
    #[cfg(feature = "carddav")]
    #[test]
    fn a_carddav_block_without_auth_loads() {
        use super::{CarddavAuthConfig, CarddavConfig};

        let config: CarddavConfig = toml::from_str("server = \"dav.example.org\"").unwrap();
        assert!(matches!(config.auth, CarddavAuthConfig::None));

        let config: super::Config =
            toml::from_str("[accounts.a]\ncarddav.home = \"http://localhost:5232/a/\"").unwrap();
        let carddav = config.accounts["a"].carddav.as_ref().unwrap();
        assert!(matches!(carddav.auth, CarddavAuthConfig::None));
    }

    /// A CardDAV `server` or `home` whose host is not ASCII is refused,
    /// the message naming the host and its Punycode form as the way out.
    #[cfg(feature = "carddav")]
    #[test]
    fn a_non_ascii_dav_host_names_its_punycode_form() {
        use crate::carddav::client::parse_carddav_server;

        let hint = "Host `b\u{fc}cher.example` is not ASCII: write it in its Punycode form";

        for server in [
            "https://b\u{fc}cher.example/dav/",
            "b\u{fc}cher.example",
            "b\u{fc}cher.example:8443",
        ] {
            let err = parse_carddav_server(server).unwrap_err().to_string();
            assert!(err.contains(hint), "{server}: {err}");
        }

        let home = "home = \"https://alice@b\u{fc}cher.example:8443/dav/\"";
        let err = toml::from_str::<super::CarddavConfig>(home).unwrap_err();
        assert!(err.to_string().contains(hint), "{err}");

        let url = parse_carddav_server("https://xn--bcher-kva.example/dav/").unwrap();
        assert_eq!(url.host_str(), Some("xn--bcher-kva.example"));
    }

    #[cfg(all(feature = "carddav", feature = "jmap"))]
    #[test]
    fn an_account_proxy_fills_every_backend_naming_none() {
        use pimalaya_config::toml::TomlConfig;

        use super::Config;

        let mut config: Config = toml::from_str(
            "[accounts.a]\n\
             proxy.url = \"socks5h://localhost:1080\"\n\
             carddav.server = \"dav.example.org\"\n\
             carddav.auth = \"none\"\n\
             jmap.server = \"jmap.example.org\"\n\
             jmap.proxy.url = \"http://proxy.example.org:3128\"\n\
             jmap.auth.bearer.token.raw = \"token\"",
        )
        .unwrap();

        let (_, account) = config.take_named_account("a").unwrap();
        let carddav = account.carddav.unwrap().proxy.unwrap();
        let jmap = account.jmap.unwrap().proxy.unwrap();

        assert_eq!(carddav.url, "socks5h://localhost:1080");
        assert_eq!(jmap.url, "http://proxy.example.org:3128");
    }

    /// Checks config.sample.toml parses as written and once per rank fully
    /// uncommented, `example` its only account and the default each time,
    /// declaring a backend once uncommented.
    #[test]
    fn the_sample_configuration_parses_with_each_variant() {
        use std::{fs, path::PathBuf};

        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config.sample.toml");
        let sample = fs::read_to_string(&path).unwrap();
        let headers: Vec<_> = sample.lines().filter(|l| l.starts_with('[')).collect();
        assert_eq!(headers, ["[accounts.example]"]);

        load_sample(&sample, "as written");

        let paragraphs = sample_paragraphs(&sample);

        for rank in 1..=sample_ranks(&paragraphs) {
            let text = sample_variant(&paragraphs, rank);
            let backends = load_sample(&text, &format!("variant {rank}"));
            assert!(!backends.is_empty(), "variant {rank}: no backend declared");
        }
    }

    /// One line of config.sample.toml.
    struct SampleLine<'a> {
        /// The line as written.
        line: &'a str,
        /// The code before the comment, one `#` off when commented.
        code: &'a str,
        /// Whether the code is commented.
        commented: bool,
        /// The field the code sets, a secret's `raw` and `command` being
        /// one.
        field: Option<&'a str>,
        /// The other values the comment lists in brackets.
        values: Vec<&'a str>,
    }

    impl<'a> SampleLine<'a> {
        fn parse(line: &'a str) -> Self {
            let (code, comment) = line.split_once('\t').unwrap_or((line, ""));
            let commented = code.starts_with('#');
            let code = code.strip_prefix('#').unwrap_or(code);

            let field = code
                .split_once(" = ")
                .map(|(key, _)| key)
                .filter(|key| {
                    key.chars()
                        .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
                })
                .map(|key| {
                    let secret = key.strip_suffix(".raw").or(key.strip_suffix(".command"));
                    secret.unwrap_or(key)
                });

            let values = comment
                .trim_end()
                .strip_suffix(']')
                .and_then(|comment| comment.rsplit_once('['))
                .map(|(_, values)| values.split(", ").collect())
                .unwrap_or_default();

            Self {
                line,
                code,
                commented,
                field,
                values,
            }
        }

        /// The first two segments of the field, the table a replacing
        /// paragraph shares with the one above it (`carddav.auth`).
        fn table(&self) -> Option<&'a str> {
            let field = self.field?;
            let end = field
                .match_indices('.')
                .nth(1)
                .map_or(field.len(), |(i, _)| i);
            Some(&field[..end])
        }

        /// The values the line offers: its code as written, then each
        /// bracketed value in its place, typed like the written one.
        fn options(&self) -> Vec<String> {
            let (key, value) = self.code.split_once(" = ").unwrap();
            let mut options = vec![self.code.to_owned()];

            for other in &self.values {
                if value.starts_with('"') {
                    options.push(format!("{key} = \"{other}\""));
                } else {
                    options.push(format!("{key} = {other}"));
                }
            }

            options
        }
    }

    /// Loads `text` through the loader `-c` uses, into the backends its
    /// default account, `example`, declares.
    fn load_sample(text: &str, name: &str) -> Vec<&'static str> {
        use std::io::Write;

        use pimalaya_config::toml::TomlConfig;
        use tempfile::NamedTempFile;

        use super::Config;

        let context = format!("config.sample.toml, {name}");

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(text.as_bytes()).unwrap();

        let mut config = Config::load(&[file.path().to_path_buf()])
            .unwrap_or_else(|err| panic!("{context}: {err:?}\n{text}"));
        let (account, config) = config
            .take_account(None)
            .unwrap_or_else(|err| panic!("{context}: {err:?}"));

        assert_eq!(account, "example", "{context}");
        config.backends()
    }

    fn sample_paragraphs(sample: &str) -> Vec<Vec<SampleLine<'_>>> {
        let paragraphs = sample.split("\n\n");
        let paragraphs = paragraphs.map(|p| p.lines().map(SampleLine::parse).collect());
        paragraphs.collect()
    }

    /// Whether `below`, all commented, replaces `above`, setting keys under
    /// a table one of its lines sets.
    fn sample_replaces(above: &[SampleLine], below: &[SampleLine]) -> bool {
        let mut code = below.iter().filter(|line| line.field.is_some());
        let Some(first) = code.next() else {
            return false;
        };

        if !first.commented || code.any(|line| !line.commented) {
            return false;
        }

        above
            .iter()
            .filter_map(SampleLine::table)
            .any(|t| Some(t) == first.table())
    }

    /// The values each field takes in turn, the lines setting it in their
    /// order, each followed by its bracketed values.
    fn sample_options<'a>(lines: &[&'a SampleLine<'a>]) -> HashMap<&'a str, Vec<String>> {
        let mut options: HashMap<&str, Vec<String>> = HashMap::new();

        for line in lines {
            if let Some(field) = line.field {
                options.entry(field).or_default().extend(line.options());
            }
        }

        options
    }

    /// How many variants give every value its turn: the most values of
    /// one field, two when a paragraph replaces another.
    fn sample_ranks(paragraphs: &[Vec<SampleLine>]) -> usize {
        let lines: Vec<&SampleLine> = paragraphs.iter().flatten().collect();
        let options = sample_options(&lines);
        let values = options.values().map(Vec::len).max().unwrap_or(0);

        let replaces = paragraphs.windows(2).any(|w| sample_replaces(&w[0], &w[1]));

        if replaces { values.max(2) } else { values }
    }

    /// The sample at `rank`, every line uncommented, each field set to its
    /// value of that rank (the last when it has fewer).
    ///
    /// A replacing paragraph stays out at rank 1 and replaces the one
    /// above it from rank 2.
    fn sample_variant(paragraphs: &[Vec<SampleLine>], rank: usize) -> String {
        let mut dropped = vec![false; paragraphs.len()];

        for i in 1..paragraphs.len() {
            if sample_replaces(&paragraphs[i - 1], &paragraphs[i]) {
                let i = if rank == 1 { i } else { i - 1 };
                dropped[i] = true;
            }
        }

        let kept = paragraphs
            .iter()
            .zip(&dropped)
            .filter(|(_, dropped)| !**dropped);
        let lines: Vec<&SampleLine> = kept.flat_map(|(lines, _)| lines).collect();

        let mut options = sample_options(&lines);
        let mut text = Vec::with_capacity(lines.len());

        for line in lines {
            let Some(field) = line.field else {
                text.push(line.line.to_owned());
                continue;
            };

            if let Some(values) = options.remove(field) {
                text.push(values[rank.min(values.len()) - 1].clone());
            }
        }

        text.join("\n")
    }

    /// A host is ASCII: an `https` server naming a non-ASCII one is refused
    /// when parsed, its Punycode form working.
    #[cfg(feature = "jmap")]
    #[test]
    fn a_non_ascii_host_is_refused() {
        use super::parse_server;

        let schemes = ["http", "https", "jmap", "jmaps"];
        assert!(parse_server("https://b\u{fc}cher.example/jmap", "https", &schemes).is_err());
        let err = parse_server("b\u{fc}cher.example", "https", &schemes).unwrap_err();
        let hint = "Host `b\u{fc}cher.example` is not ASCII: write it in its Punycode form";
        assert!(err.to_string().contains(hint), "{err}");

        let url = parse_server("https://xn--bcher-kva.example/jmap", "https", &schemes).unwrap();
        assert_eq!(url.host_str(), Some("xn--bcher-kva.example"));
        let url = parse_server("xn--bcher-kva.example", "https", &schemes).unwrap();
        assert_eq!(url.host_str(), Some("xn--bcher-kva.example"));
    }

    /// Parses `toml` as a whole configuration, into its error message.
    fn refusal(toml: &str) -> String {
        let err = toml::from_str::<crate::config::Config>(toml).unwrap_err();
        err.to_string()
    }

    #[test]
    fn an_unknown_root_key_is_refused() {
        let err = refusal("colour = \"red\"\n[accounts.example]\n");
        assert!(err.contains("unknown field `colour`"), "{err}");
    }

    #[test]
    fn an_unknown_account_key_is_refused() {
        let err = refusal("[accounts.example]\ndefualt = true\n");
        assert!(err.contains("unknown field `defualt`"), "{err}");
    }

    /// A backend table and the tables nested in it refuse a key they do
    /// not know, as the option tables do.
    #[cfg(all(feature = "vdir", feature = "carddav"))]
    #[test]
    fn an_unknown_backend_or_nested_key_is_refused() {
        let err = refusal("[accounts.example]\nvdir.rot = \"~/Contacts\"\n");
        assert!(err.contains("unknown field `rot`"), "{err}");

        let toml = "[accounts.example]\ncarddav.server = \"dav.example.org\"\n\
                    carddav.auth = \"none\"\ncarddav.tls.certificate = \"ca.pem\"\n";
        let err = refusal(toml);
        assert!(err.contains("unknown field `certificate`"), "{err}");

        let err = refusal("[accounts.example]\ncard.list.page-sise = 10\n");
        assert!(err.contains("unknown field `page-sise`"), "{err}");
    }

    /// `vdir.root` expands a leading tilde and an environment variable as it
    /// is read, as `pimdir.root` does.
    #[cfg(feature = "vdir")]
    #[test]
    fn a_vdir_root_expands_its_tilde_and_variables() {
        use super::VdirConfig;

        let home = std::env::var("HOME").expect("HOME must be set");

        for root in ["~/Contacts", "$HOME/Contacts"] {
            let config: VdirConfig = toml::from_str(&format!("root = {root:?}")).unwrap();
            assert_eq!(config.root, format!("{home}/Contacts"), "{root}");
        }
    }

    /// A key v0.3 read and v1 removed is an unknown key, refused by serde
    /// as any other, with no sentence of its own: `vdir.home-dir`,
    /// `carddav.discover` and the output tables.
    #[test]
    fn a_removed_key_is_an_unknown_key() {
        let mut keys = vec![
            ("table.preset = \"x\"\n[accounts.example]\n", "table"),
            (
                "account.list.table.name-color = \"red\"\n[accounts.example]\n",
                "account",
            ),
            (
                "[accounts.example]\naddressbook.list.table.id-color = \"red\"\n",
                "list",
            ),
            (
                "[accounts.example]\ncard.list.table.fn-color = \"red\"\n",
                "table",
            ),
        ];

        if cfg!(feature = "vdir") {
            keys.push((
                "[accounts.example]\nvdir.home-dir = \"~/Contacts\"\n",
                "home-dir",
            ));
        }

        if cfg!(feature = "carddav") {
            keys.push((
                "[accounts.example]\ncarddav.discover = \"fastmail.com\"\n",
                "discover",
            ));
        }

        for (toml, key) in keys {
            let err = refusal(toml);
            assert!(
                err.contains(&format!("unknown field `{key}`")),
                "{toml}: {err}"
            );
            assert!(!err.contains("MIGRATION"), "{err}");
        }
    }

    /// A build leaving a backend out still loads a file configuring it, a
    /// file serving several builds.
    #[cfg(not(feature = "gpeople"))]
    #[test]
    fn a_backend_left_out_of_the_build_still_loads() {
        let toml = "[accounts.example]\ngpeople.auth.token.raw = \"token\"\n\
                    proxy.url = \"socks5h://localhost:1080\"\n";
        toml::from_str::<crate::config::Config>(toml).unwrap();
    }
}
