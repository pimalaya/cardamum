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
#[cfg(feature = "pimdir")]
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
    tls::{Rustls, RustlsCrypto, Tls, TlsProvider, TlsServer},
};
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
use secrecy::SecretString;
#[cfg(not(all(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople",
    feature = "vdir",
    feature = "pimdir"
)))]
use serde::de::IgnoredAny;
use serde::{Deserialize, Deserializer};
#[cfg(any(feature = "carddav", feature = "jmap"))]
use url::Url;

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
    /// Refused: the output is plain text, drawn in no table.
    #[serde(default)]
    #[allow(dead_code)]
    pub table: Option<TableRemoved>,
    /// Refused: `account list` prints plain text, in no colour.
    #[serde(default)]
    #[allow(dead_code)]
    pub account: Option<AccountTableRemoved>,
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
            (name, account)
        });

        entry
    }
}

impl AccountConfig {
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
}

/// Account configuration, refusing a key it does not know.
///
/// The block of a backend the build leaves out loads unread, so one file
/// serves every build.
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
    /// Refused: the output is plain text, drawn in no table.
    #[serde(default)]
    #[allow(dead_code)]
    pub table: Option<TableRemoved>,
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
    /// Directory holding the account's collections, one addressbook per
    /// immediate subdirectory.
    #[serde(deserialize_with = "shell_expanded_string")]
    pub home_dir: String,
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
    /// Bare domain resolved to a server URL on every run.
    ///
    /// PACC first, then RFC 6764: SRV record, its TXT `path`, then
    /// `.well-known`, which a Google domain probes authenticated. It
    /// costs DNS and HTTP round-trips, and `server` or `home` skip it.
    pub discover: Option<String>,
    /// DAV context root, where the principal and home set walk starts,
    /// skipping the DNS and PACC round-trips `discover` pays.
    ///
    /// Accepts a full URL, a bare domain or `domain:port`, a bare
    /// authority defaulting to `https`. A value naming no path is probed
    /// once through `.well-known/carddav`, a bare origin not being
    /// necessarily the context root.
    pub server: Option<String>,
    /// Pre-resolved addressbook home set URL, where the client lists
    /// addressbooks, skipping every discovery step.
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
    /// Authentication configuration.
    pub auth: CarddavAuthConfig,
}

/// CardDAV authentication configuration.
#[cfg(feature = "carddav")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum CarddavAuthConfig {
    /// No credentials, for a server that asks for none.
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
    /// Refused: `addressbook list` prints plain text, in no colour.
    #[serde(default)]
    #[allow(dead_code)]
    pub list: Option<AddressbookListRemoved>,
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
    /// Refused: `card list` prints plain text, in no colour.
    #[serde(default)]
    #[allow(dead_code)]
    pub table: Option<CardListTableRemoved>,
}

/// The error of a table this release removed, naming it and the migration
/// notes.
fn removed<E: serde::de::Error>(table: &str, why: &str) -> E {
    E::custom(format!(
        "The `{table}` table is removed, {why}: delete it, see MIGRATION.md"
    ))
}

/// The `table` table, at the root or in an account, which no
/// configuration holds any more.
#[derive(Clone, Debug)]
pub enum TableRemoved {}

impl<'de> Deserialize<'de> for TableRemoved {
    fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Err(removed(
            "table",
            "the output being plain text with no border",
        ))
    }
}

/// The top-level `account` table, which no configuration holds any more.
#[derive(Clone, Debug)]
pub enum AccountTableRemoved {}

impl<'de> Deserialize<'de> for AccountTableRemoved {
    fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Err(removed("account", "`account list` printing no colour"))
    }
}

/// The `addressbook.list` table, which no configuration holds any more.
#[derive(Clone, Debug)]
pub enum AddressbookListRemoved {}

impl<'de> Deserialize<'de> for AddressbookListRemoved {
    fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Err(removed(
            "addressbook.list",
            "`addressbook list` printing no colour",
        ))
    }
}

/// The `card.list.table` table, which no configuration holds any more.
#[derive(Clone, Debug)]
pub enum CardListTableRemoved {}

impl<'de> Deserialize<'de> for CardListTableRemoved {
    fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Err(removed("card.list.table", "`card list` printing no colour"))
    }
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
            client: None,
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
        Url::parse(server)?
    } else {
        Url::parse(&format!("{default_scheme}://{server}"))?
    };

    let scheme = url.scheme();

    if !allowed.contains(&scheme) {
        bail!("Invalid server scheme `{scheme}`: expected one of {allowed:?}");
    }

    Ok(url)
}

#[cfg(test)]
mod tests {
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
    /// The commented sections of config.sample.toml, each a provider or a
    /// backend a user uncomments.
    #[cfg(all(
        feature = "carddav",
        feature = "jmap",
        feature = "msgraph",
        feature = "gpeople",
        feature = "vdir",
        feature = "pimdir"
    ))]
    const SECTIONS: [&str; 9] = [
        "# CardDAV",
        "# CardDAV without credentials",
        "# Google CardDAV",
        "# JMAP",
        "# Fastmail",
        "# Microsoft Graph",
        "# Google People",
        "# vdir",
        "# pimdir",
    ];

    /// Checks config.sample.toml: one `[accounts.example]` table and no
    /// other, its sections each parsed through the loader `-c` uses with
    /// every variant on top, `example` resolving as the default account
    /// and declaring a backend.
    #[cfg(all(
        feature = "carddav",
        feature = "jmap",
        feature = "msgraph",
        feature = "gpeople",
        feature = "vdir",
        feature = "pimdir"
    ))]
    #[test]
    fn the_sample_configuration_parses_with_each_variant() {
        use std::{fs, io::Write};

        use pimalaya_config::{
            sample::{Variant, variants},
            toml::TomlConfig,
        };
        use tempfile::NamedTempFile;

        use super::Config;

        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config.sample.toml");
        let sample = fs::read_to_string(&path).unwrap();
        let headers: Vec<_> = sample.lines().filter(|l| l.starts_with('[')).collect();
        assert_eq!(headers, ["[accounts.example]"]);

        for Variant { name, text } in variants(&sample, &SECTIONS) {
            let context = format!("config.sample.toml with {name} uncommented");

            let mut file = NamedTempFile::new().unwrap();
            file.write_all(text.as_bytes()).unwrap();

            let mut config = Config::load(&[file.path().to_path_buf()])
                .unwrap_or_else(|err| panic!("{context}: {err:?}"));
            let (account, account_config) = config
                .take_account(None)
                .unwrap_or_else(|err| panic!("{context}: {err:?}"));

            let blocks = [
                account_config.carddav.is_some(),
                account_config.jmap.is_some(),
                account_config.msgraph.is_some(),
                account_config.gpeople.is_some(),
                account_config.vdir.is_some(),
                account_config.pimdir.is_some(),
            ];

            assert_eq!(account, "example", "{context}");
            assert!(blocks.contains(&true), "{context}: no backend declared");
        }
    }
    /// A host is ASCII: an `https` server naming a non-ASCII one is refused
    /// when parsed, its Punycode form working.
    #[cfg(feature = "jmap")]
    #[test]
    fn a_non_ascii_host_is_refused() {
        use super::parse_server;

        let schemes = ["http", "https", "jmap", "jmaps"];
        assert!(parse_server("https://b\u{fc}cher.example/jmap", "https", &schemes).is_err());
        assert!(parse_server("b\u{fc}cher.example", "https", &schemes).is_err());

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
        let err = refusal("[accounts.example]\nvdir.home-dri = \"~/Contacts\"\n");
        assert!(err.contains("unknown field `home-dri`"), "{err}");

        let toml = "[accounts.example]\ncarddav.server = \"dav.example.org\"\n\
                    carddav.auth = \"none\"\ncarddav.tls.certificate = \"ca.pem\"\n";
        let err = refusal(toml);
        assert!(err.contains("unknown field `certificate`"), "{err}");

        let err = refusal("[accounts.example]\ncard.list.page-sise = 10\n");
        assert!(err.contains("unknown field `page-sise`"), "{err}");
    }

    /// The tables v0.3 read and this release removed are refused by name,
    /// pointing at the migration notes, at the root and in an account.
    #[test]
    fn a_removed_table_is_refused_by_name() {
        let tables = [
            ("table", "table.preset = \"x\"\n"),
            (
                "addressbook.list",
                "addressbook.list.table.id-color = \"red\"\n",
            ),
            ("card.list.table", "card.list.table.fn-color = \"red\"\n"),
        ];

        for (table, line) in tables {
            let root = format!("{line}[accounts.example]\n");
            let account = format!("[accounts.example]\n{line}");

            for toml in [root, account] {
                let err = refusal(&toml);
                assert!(err.contains(&format!("`{table}` table")), "{err}");
                assert!(err.contains("MIGRATION.md"), "{err}");
            }
        }

        let err = refusal("account.list.table.name-color = \"red\"\n[accounts.example]\n");
        assert!(err.contains("`account` table"), "{err}");
        assert!(err.contains("MIGRATION.md"), "{err}");
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
