//! # Configuration
//!
//! The TOML configuration file: the options shared by every account, the
//! accounts themselves and their per-backend blocks.
//!
//! It also renders an account back to TOML, which is how the wizard
//! writes what it discovered. The reference document, key by key, is
//! config.sample.toml.

use std::collections::HashMap;
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "people",
    feature = "pimdir"
))]
use std::path::PathBuf;

use anyhow::Result;
#[cfg(feature = "jmap")]
use anyhow::bail;
use comfy_table::ContentArrangement;
use crossterm::style::Color;
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "people"
))]
use pimalaya_config::secret::Secret;
use pimalaya_config::toml::TomlConfig;
#[cfg(any(feature = "vdir", feature = "carddav", feature = "jmap"))]
use pimalaya_config::toml::shell_expanded_string;
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "people"
))]
use pimalaya_stream::tls::{Rustls, RustlsCrypto, Tls, TlsProvider};
use serde::{Deserialize, Serialize};
#[cfg(feature = "carddav")]
use url::Url;

/// Whether a value is still what [`Default`] made it, so the serializer
/// can leave it out of a generated document.
fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}

/// The whole configuration file: the options shared by every account,
/// then the accounts themselves.
///
/// `deny_unknown_fields` is omitted so a neighbouring tool can share the
/// same file without bouncing off an unknown top-level key.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// Table rendering options, shared by every list command.
    #[serde(default)]
    pub table: TableConfig,
    /// Addressbook options, overridable per account.
    #[serde(default)]
    pub addressbook: AddressbookConfig,
    /// Card options, overridable per account.
    #[serde(default)]
    pub card: CardConfig,
    /// `account list` rendering options, global only since the listing
    /// stands outside any one account.
    #[serde(default)]
    pub account: AccountListingConfig,
    /// The accounts, keyed by their `[accounts.<name>]` table name.
    pub accounts: HashMap<String, AccountConfig>,
}

impl TomlConfig for Config {
    type Account = AccountConfig;

    fn project_name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn take_named_account(&mut self, name: &str) -> Option<(String, Self::Account)> {
        self.accounts.remove_entry(name)
    }

    fn take_default_account(&mut self) -> Option<(String, Self::Account)> {
        let name = self
            .accounts
            .iter()
            .find_map(|(name, account)| account.default.then(|| name.clone()))?;

        self.take_named_account(&name)
    }
}

/// The order a rendered account groups its keys in, most defining
/// first: what the account is, its backend, then the rendering options.
///
/// A key outside this list still renders, after the listed ones, so a
/// field added to [`AccountConfig`] can never go missing from a
/// generated document because nobody updated this table.
const RENDER_ORDER: [&str; 10] = [
    "default",
    "vdir",
    "pimdir",
    "carddav",
    "jmap",
    "msgraph",
    "people",
    "addressbook",
    "card",
    "table",
];

/// The keys naming what a backend group points at, lifted to the top of
/// their group.
///
/// Serialized alphabetically, `carddav.server` would read under the
/// `carddav.auth` credential authenticating against it.
const ENDPOINT_KEYS: [&str; 5] = ["discover", "server", "home", "home-dir", "root"];

impl AccountConfig {
    /// Renders this account as an `[accounts.<name>]` block, ready to be
    /// written to a configuration file or appended to one.
    ///
    /// What it adds to the serializer is reading order, dotted keys
    /// coming out alphabetically: groups are reordered
    /// ([`RENDER_ORDER`]), each endpoint is lifted to the top of its own
    /// ([`ENDPOINT_KEYS`]), and a blank line separates them.
    pub fn render(&self, name: &str) -> Result<String> {
        // NOTE: borrowed rather than built into a `Config`, which would
        // mean cloning the account (and so deriving `Clone` down every
        // backend config) to render it. The emitter only looks for an
        // `accounts` table, so any shape carrying one will do.
        #[derive(Serialize)]
        struct AccountDocument<'a> {
            accounts: HashMap<&'a str, &'a AccountConfig>,
        }

        let document = AccountDocument {
            accounts: HashMap::from([(name, self)]),
        };
        let rendered = pimalaya_config::toml::to_string(&document)?;

        let (header, body) = match rendered.split_once('\n') {
            Some((header, body)) => (header, body),
            None => return Ok(rendered),
        };

        let mut groups: Vec<(String, Vec<&str>)> = Vec::new();

        for line in body.lines().filter(|line| !line.trim().is_empty()) {
            let key = line.split(['.', ' ']).next().unwrap_or(line).to_string();

            match groups.iter_mut().find(|(name, _)| *name == key) {
                Some((_, lines)) => lines.push(line),
                None => groups.push((key, vec![line])),
            }
        }

        groups.sort_by_key(|(key, _)| {
            RENDER_ORDER
                .iter()
                .position(|known| known == key)
                .unwrap_or(RENDER_ORDER.len())
        });

        let mut document = format!("{header}\n");

        for (index, (_, mut lines)) in groups.into_iter().enumerate() {
            if index > 0 {
                document.push('\n');
            }

            // NOTE: the endpoint is what the group is about, so it reads
            // first; the credentials and the quirks qualify it.
            lines.sort_by_key(|line| {
                let field = line.split(['.', ' ']).nth(1).unwrap_or_default();

                ENDPOINT_KEYS
                    .iter()
                    .position(|known| *known == field)
                    .unwrap_or(ENDPOINT_KEYS.len())
            });

            for line in lines {
                document.push_str(line);
                document.push('\n');
            }
        }

        Ok(document)
    }
}

/// Account configuration.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct AccountConfig {
    /// Whether a command passing no `-a <NAME>` runs against it.
    #[serde(default, skip_serializing_if = "is_default")]
    pub default: bool,
    /// Table rendering options, overriding the global ones.
    #[serde(default)]
    pub table: TableConfig,
    /// Addressbook options, overriding the global ones.
    #[serde(default)]
    pub addressbook: AddressbookConfig,
    /// Card options, overriding the global ones.
    #[serde(default)]
    pub card: CardConfig,
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
    #[cfg(feature = "people")]
    pub people: Option<PeopleConfig>,
}

/// Vdir configuration.
#[cfg(feature = "vdir")]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct VdirConfig {
    /// Directory holding the account's collections, one addressbook per
    /// immediate subdirectory.
    #[serde(deserialize_with = "shell_expanded_string")]
    pub home_dir: String,
}

/// pimdir configuration.
#[cfg(feature = "pimdir")]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct PimdirConfig {
    /// The store directory, holding `pimdir.db` and `objects/`.
    ///
    /// It must already exist: creating one is the sync engine's job.
    pub root: PathBuf,
    /// The account the collections are grouped under, for a store shared
    /// by several accounts or domains, as the mobile apps do.
    ///
    /// Unset reads every collection in the store.
    #[serde(default)]
    pub account: Option<String>,
}

/// CardDAV configuration.
#[cfg(feature = "carddav")]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CarddavConfig {
    /// Bare domain resolved to a server URL on every run.
    ///
    /// PACC first, then RFC 6764: SRV record, its TXT `path`, then
    /// `.well-known`, which a Google domain probes authenticated. It
    /// costs DNS and HTTP round-trips, and `server` or `home` skip it.
    pub discover: Option<String>,
    /// DAV context root, where principal and home set discovery start.
    ///
    /// The `.well-known` step is skipped. Accepts a full URL, a bare
    /// domain or `domain:port`, a bare authority defaulting to `https`.
    pub server: Option<String>,
    /// Pre-resolved addressbook home set URL, where the client lists
    /// addressbooks, skipping every discovery step.
    pub home: Option<Url>,
    /// TLS configuration.
    #[serde(default)]
    pub tls: TlsConfig,
    /// Authentication configuration.
    pub auth: CarddavAuthConfig,
}

/// CardDAV authentication configuration.
#[cfg(feature = "carddav")]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum CarddavAuthConfig {
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
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    /// Authentication configuration.
    pub auth: JmapAuthConfig,
}

/// JMAP authentication configuration.
#[cfg(feature = "jmap")]
#[derive(Clone, Debug, Deserialize, Serialize)]
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
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    /// Authentication configuration.
    pub auth: MsgraphAuthConfig,
}

/// Microsoft Graph authentication configuration.
#[cfg(feature = "msgraph")]
#[derive(Clone, Debug, Deserialize, Serialize)]
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
#[cfg(feature = "people")]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct PeopleConfig {
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
    /// Authentication configuration.
    pub auth: PeopleAuthConfig,
}

/// Google People authentication configuration.
#[cfg(feature = "people")]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct PeopleAuthConfig {
    /// OAuth 2.0 access token sent as `Bearer <token>`, the only
    /// authorization the People API accepts.
    pub token: Secret,
}

#[cfg(any(feature = "msgraph", feature = "people"))]
fn default_http_alpn() -> Vec<String> {
    vec![String::from("http/1.1")]
}

/// Addressbook-level options.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AddressbookConfig {
    /// Addressbook id the `card` commands take when their
    /// `-k/--addressbook` flag is omitted.
    pub default: Option<String>,
    /// `addressbook list` options.
    #[serde(default)]
    pub list: AddressbookListConfig,
}

/// `addressbook list` options.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AddressbookListConfig {
    /// Colors of the `addressbook list` table.
    #[serde(default)]
    pub table: AddressbookListTableConfig,
}

/// Colors of the `addressbook list` table.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AddressbookListTableConfig {
    /// Color of the ID column.
    pub id_color: Option<Color>,
    /// Color of the NAME column.
    pub name_color: Option<Color>,
    /// Color of the DESC column.
    pub description_color: Option<Color>,
    /// Color of the COLOR column.
    pub color_color: Option<Color>,
}

/// Card-level rendering options.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CardConfig {
    /// `card list` options.
    #[serde(default)]
    pub list: CardListConfig,
}

/// `card list` options.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CardListConfig {
    /// Default `-s/--page-size` value.
    ///
    /// The flag wins when passed, then this value, then the hard
    /// fallback of 25.
    pub page_size: Option<u32>,
    /// Colors of the `card list` table.
    #[serde(default)]
    pub table: CardListTableConfig,
}

/// Colors of the `card list` table.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CardListTableConfig {
    /// Color of the ID column.
    pub id_color: Option<Color>,
    /// Color of the FN column.
    pub fn_color: Option<Color>,
    /// Color of the EMAIL column.
    pub email_color: Option<Color>,
    /// Color of the TEL column.
    pub tel_color: Option<Color>,
}

/// `account list` rendering options, top-level only.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AccountListingConfig {
    /// `account list` options.
    #[serde(default)]
    pub list: AccountListingListConfig,
}

/// `account list` options.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AccountListingListConfig {
    /// Colors of the `account list` table.
    #[serde(default)]
    pub table: AccountListingTableConfig,
}

/// Colors of the `account list` table.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AccountListingTableConfig {
    /// Color of the NAME column.
    pub name_color: Option<Color>,
    /// Color of the BACKENDS column.
    pub backends_color: Option<Color>,
    /// Color of the DEFAULT column.
    pub default_color: Option<Color>,
}

/// Table rendering options shared by every list command.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TableConfig {
    /// `comfy_table` preset string, `UTF8_FULL_CONDENSED` by default.
    pub preset: Option<String>,
    /// Column arrangement strategy, `dynamic` by default.
    pub arrangement: Option<TableArrangementConfig>,
}

/// How a table arranges its columns.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum TableArrangementConfig {
    /// Fit the columns to the terminal width.
    #[default]
    Dynamic,
    /// Fit the columns and stretch the table to the full width.
    DynamicFullWidth,
    /// Let each column take the width of its content.
    Disabled,
}

impl From<TableArrangementConfig> for ContentArrangement {
    fn from(arrangement: TableArrangementConfig) -> Self {
        match arrangement {
            TableArrangementConfig::Dynamic => ContentArrangement::Dynamic,
            TableArrangementConfig::DynamicFullWidth => ContentArrangement::DynamicFullWidth,
            TableArrangementConfig::Disabled => ContentArrangement::Disabled,
        }
    }
}

/// SSL/TLS configuration.
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "people"
))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TlsConfig {
    /// The TLS implementation the connection uses.
    pub provider: Option<TlsProviderConfig>,
    /// Rustls-only options.
    #[serde(default)]
    pub rustls: RustlsConfig,
    /// Path to an additional PEM certificate to trust.
    pub cert: Option<PathBuf>,
}

/// SSL/TLS provider configuration.
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "people"
))]
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    feature = "people"
))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
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
    feature = "people"
))]
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    feature = "people"
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
            cert: self.cert,
        }
    }
}

/// Parses a `server` config string into a URL.
///
/// A full URL is taken verbatim, a bare `host[:port]` takes
/// `default_scheme`, and a scheme outside `allowed` is rejected.
#[cfg(feature = "jmap")]
pub fn parse_server(server: &str, default_scheme: &str, allowed: &[&str]) -> Result<url::Url> {
    let url = if server.contains("://") {
        url::Url::parse(server)?
    } else {
        url::Url::parse(&format!("{default_scheme}://{server}"))?
    };

    let scheme = url.scheme();

    if !allowed.contains(&scheme) {
        bail!("Invalid server scheme `{scheme}`: expected one of {allowed:?}");
    }

    Ok(url)
}
