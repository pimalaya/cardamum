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

/// Global configuration.
///
/// Represents the whole TOML user's configuration file.
/// `deny_unknown_fields` is intentionally omitted so future tooling
/// (TUI, neverest) can share the same file without bouncing off
/// unknown top-level keys.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(default)]
    pub table: TableConfig,
    #[serde(default)]
    pub addressbook: AddressbookConfig,
    #[serde(default)]
    pub card: CardConfig,
    /// `account list` rendering options (global only; there is no
    /// per-account override for the listing of accounts).
    #[serde(default)]
    pub account: AccountListingConfig,
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

/// The order the rendered account groups its keys in, most defining
/// first: what the account is, then the backend it reads contacts from,
/// and last the rendering options.
///
/// A key outside this list still renders, after the ones listed, so a
/// field added to [`AccountConfig`] can never go missing from a
/// generated document just because nobody updated this table.
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
/// their group. Serialized alphabetically, `carddav.server` would read
/// under the `carddav.auth` credential authenticating against it.
const ENDPOINT_KEYS: [&str; 5] = ["discover", "server", "home", "home-dir", "root"];

impl AccountConfig {
    /// Renders this account as an `[accounts.<name>]` block, ready to be
    /// written to a configuration file or appended to one.
    ///
    /// The serializer decides what is written, so a field left at its
    /// default is omitted and nothing has to be listed here twice. What
    /// this adds is reading order: the flattened dotted keys come out
    /// alphabetically, which buries the endpoint under the credentials
    /// that authenticate against it, and runs every group together. The
    /// groups are reordered, the endpoint is lifted to the top of its
    /// own, and a blank line separates them.
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

        // The emitter writes the header itself, and everything below it
        // is one dotted key per line.
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

            // The endpoint is what the group is about, so it reads first;
            // the credentials and the quirks qualify it.
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
    /// Whether the commands with no `-a <NAME>` run against this
    /// account.
    #[serde(default, skip_serializing_if = "is_default")]
    pub default: bool,

    #[serde(default)]
    pub table: TableConfig,
    #[serde(default)]
    pub addressbook: AddressbookConfig,
    #[serde(default)]
    pub card: CardConfig,

    #[cfg(feature = "vdir")]
    pub vdir: Option<VdirConfig>,
    #[cfg(feature = "pimdir")]
    pub pimdir: Option<PimdirConfig>,
    #[cfg(feature = "carddav")]
    pub carddav: Option<CarddavConfig>,
    #[cfg(feature = "jmap")]
    pub jmap: Option<JmapConfig>,
    #[cfg(feature = "msgraph")]
    pub msgraph: Option<MsgraphConfig>,
    #[cfg(feature = "people")]
    pub people: Option<PeopleConfig>,
}

/// Vdir configuration.
#[cfg(feature = "vdir")]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct VdirConfig {
    /// Directory holding every collection of the account; each
    /// immediate subdirectory is one addressbook.
    #[serde(deserialize_with = "shell_expanded_string")]
    pub home_dir: String,
}

/// pimdir configuration.
#[cfg(feature = "pimdir")]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct PimdirConfig {
    /// The store directory (holds `pimdir.db` and `objects/`).
    pub root: PathBuf,
    /// The replica source name this client opens the store as. Reads are
    /// source-independent, but a staged write (create, edit, delete) is
    /// attributed to this source, so for the change to propagate it must
    /// match the source name the sync engine drives for this device.
    /// Usually left unset: it is auto-detected from the store, a store
    /// synced as a single source being opened as that source. Set it only
    /// to disambiguate a store synced from two sources.
    #[serde(default)]
    pub source: Option<String>,
    /// The account the collections are grouped under, for a store shared
    /// by several accounts (or by several domains, as the mobile apps do).
    /// Unset reads every collection in the store.
    #[serde(default)]
    pub account: Option<String>,
}

/// CardDAV configuration.
#[cfg(feature = "carddav")]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CarddavConfig {
    /// Bare domain resolved to a server URL at runtime: PACC first,
    /// then RFC 6764 (SRV record, its TXT `path`, then `.well-known`);
    /// Google domains use an authenticated `.well-known` probe.
    /// Convenient but adds DNS + HTTP round-trips on every run; `server`
    /// or `home` win when set and skip this.
    pub discover: Option<String>,
    /// DAV context root. Principal + addressbook-home-set discovery
    /// start from this URL; the `.well-known` step is skipped. Accepts
    /// a full URL, a bare domain, or `domain:port`; bare authorities
    /// default to `https://`.
    pub server: Option<String>,
    /// Pre-resolved addressbook home-set URL. Skips every discovery
    /// step; the client lists addressbooks at this URL.
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
    /// HTTP Basic authentication (RFC 7617).
    Basic {
        #[serde(deserialize_with = "shell_expanded_string")]
        username: String,
        password: Secret,
    },
    /// HTTP Bearer authentication (RFC 6750).
    Bearer { token: Secret },
}

/// JMAP configuration (RFC 8620 + RFC 9610).
#[cfg(feature = "jmap")]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct JmapConfig {
    /// The JMAP server address.
    ///
    /// Accepts either a bare authority (`fastmail.com`,
    /// `mail.example.com:8080`) for automatic discovery via
    /// `GET /.well-known/jmap`, or a full URL
    /// (`https://api.fastmail.com/jmap/session`) to connect directly
    /// to the session endpoint. Supported schemes: `http`, `https`,
    /// `jmap` (mapped to http), `jmaps` (mapped to https).
    pub server: String,

    /// TLS configuration.
    #[serde(default)]
    pub tls: TlsConfig,

    /// ALPN protocol identifiers offered during the TLS handshake.
    /// Defaults to `["http/1.1"]` (JMAP rides on HTTP/1.1). Set to
    /// `[]` to skip ALPN negotiation entirely. Only relevant for the
    /// rustls provider; `native-tls` ignores ALPN.
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
    /// Full raw Authorization header value, sent verbatim.
    Header(Secret),
    /// Bearer token (OAuth 2.0 access token or provider API token).
    Bearer { token: Secret },
    /// HTTP Basic authentication (username + password).
    Basic {
        #[serde(deserialize_with = "shell_expanded_string")]
        username: String,
        password: Secret,
    },
}

/// Microsoft Graph configuration.
#[cfg(feature = "msgraph")]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct MsgraphConfig {
    /// Graph user id (the contacts owner). Defaults to `me`, the
    /// authenticated user; set it to a user id or principal name to
    /// target another mailbox.
    #[serde(default = "default_msgraph_user_id")]
    pub user_id: String,

    /// TLS configuration.
    #[serde(default)]
    pub tls: TlsConfig,

    /// ALPN protocol identifiers offered during the TLS handshake.
    /// Defaults to `["http/1.1"]` (the Graph API rides on HTTP/1.1).
    /// Set to `[]` to skip ALPN negotiation entirely. Only relevant
    /// for the rustls provider; `native-tls` ignores ALPN.
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
    /// OAuth 2.0 bearer access token; sent as `Bearer <token>`. It is
    /// the only authorization the Graph API accepts.
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

    /// ALPN protocol identifiers offered during the TLS handshake.
    /// Defaults to `["http/1.1"]` (the People API rides on HTTP/1.1).
    /// Set to `[]` to skip ALPN negotiation entirely. Only relevant
    /// for the rustls provider; `native-tls` ignores ALPN.
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
    /// OAuth 2.0 bearer access token; sent as `Bearer <token>`. It is
    /// the only authorization the People API accepts.
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
    /// Addressbook id used by `card` commands when their
    /// `-k/--addressbook` flag is omitted.
    pub default: Option<String>,
    #[serde(default)]
    pub list: AddressbookListConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AddressbookListConfig {
    #[serde(default)]
    pub table: AddressbookListTableConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AddressbookListTableConfig {
    pub id_color: Option<Color>,
    pub name_color: Option<Color>,
    pub description_color: Option<Color>,
    pub color_color: Option<Color>,
}

/// Card-level rendering options.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CardConfig {
    #[serde(default)]
    pub list: CardListConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CardListConfig {
    /// Default `-s/--page-size` value for `cards list`. The CLI flag
    /// wins when passed; otherwise the merged account/global value
    /// wins; otherwise the hard fallback (25) is used.
    pub page_size: Option<u32>,
    #[serde(default)]
    pub table: CardListTableConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CardListTableConfig {
    pub id_color: Option<Color>,
    pub fn_color: Option<Color>,
    pub email_color: Option<Color>,
    pub tel_color: Option<Color>,
}

/// `account list` rendering options. Top-level only; there is no
/// per-account override.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AccountListingConfig {
    #[serde(default)]
    pub list: AccountListingListConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AccountListingListConfig {
    #[serde(default)]
    pub table: AccountListingTableConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AccountListingTableConfig {
    pub name_color: Option<Color>,
    pub backends_color: Option<Color>,
    pub default_color: Option<Color>,
}

/// Global / per-account table rendering options shared across every
/// list command.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TableConfig {
    /// `comfy_table` preset string. Defaults to `UTF8_FULL_CONDENSED`.
    pub preset: Option<String>,
    /// Column-arrangement strategy. Defaults to `Dynamic`.
    pub arrangement: Option<TableArrangementConfig>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum TableArrangementConfig {
    #[default]
    Dynamic,
    DynamicFullWidth,
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
    pub provider: Option<TlsProviderConfig>,
    #[serde(default)]
    pub rustls: RustlsConfig,
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
    Rustls,
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
    Aws,
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

/// Parses a `server` config string into a [`Url`]: a full URL is used
/// verbatim, a bare `host[:port]` defaults to `default_scheme`;
/// schemes outside `allowed` are rejected.
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
