use std::{collections::HashMap, fs, path::Path, path::PathBuf};

use anyhow::{Context, Result};
use comfy_table::ContentArrangement;
use crossterm::style::Color;
#[cfg(feature = "carddav")]
use pimalaya_config::secret::Secret;
use pimalaya_config::toml::TomlConfig;
#[cfg(any(feature = "vdir", feature = "carddav"))]
use pimalaya_config::toml::shell_expanded_string;
use pimalaya_stream::tls::{Rustls, RustlsCrypto, Tls, TlsProvider};
use serde::{Deserialize, Serialize};
use url::Url;

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

impl Config {
    /// Serializes `self` to TOML and writes it to `path`, creating any
    /// missing parent directories. Used by the wizard to persist a
    /// freshly-built configuration.
    pub fn write(&self, path: &Path) -> Result<()> {
        let toml = toml::to_string_pretty(self).context("Serialize TOML config error")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Create TOML config parent `{}` error", parent.display())
            })?;
        }

        fs::write(path, toml)
            .with_context(|| format!("Write TOML config `{}` error", path.display()))?;

        Ok(())
    }
}

/// Account configuration.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct AccountConfig {
    #[serde(default)]
    pub default: bool,

    #[serde(default)]
    pub table: TableConfig,
    #[serde(default)]
    pub addressbook: AddressbookConfig,
    #[serde(default)]
    pub card: CardConfig,

    #[cfg(feature = "vdir")]
    pub vdir: Option<VdirConfig>,
    #[cfg(feature = "carddav")]
    pub carddav: Option<CarddavConfig>,
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

/// CardDAV configuration.
#[cfg(feature = "carddav")]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CarddavConfig {
    /// Bare domain resolved to a server URL via RFC 6764 (SRV +
    /// `.well-known`) by pimconf. Convenient but adds DNS + HTTP
    /// round-trips on every run; prefer `server` once it is known.
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

/// Global / per-account table rendering knobs shared across every
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
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TlsConfig {
    pub provider: Option<TlsProviderConfig>,
    #[serde(default)]
    pub rustls: RustlsConfig,
    pub cert: Option<PathBuf>,
}

/// SSL/TLS provider configuration.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum TlsProviderConfig {
    Rustls,
    NativeTls,
}

/// Rustls configuration.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RustlsConfig {
    pub crypto: Option<RustlsCryptoConfig>,
}

/// Rustls crypto provider configuration.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum RustlsCryptoConfig {
    Aws,
    Ring,
}

impl From<TlsConfig> for Tls {
    fn from(config: TlsConfig) -> Self {
        Tls {
            provider: config.provider.map(|config| match config {
                TlsProviderConfig::Rustls => TlsProvider::Rustls,
                TlsProviderConfig::NativeTls => TlsProvider::NativeTls,
            }),
            rustls: Rustls {
                crypto: config.rustls.crypto.map(|config| match config {
                    RustlsCryptoConfig::Aws => RustlsCrypto::Aws,
                    RustlsCryptoConfig::Ring => RustlsCrypto::Ring,
                }),
                alpn: Vec::new(),
            },
            cert: config.cert,
        }
    }
}
