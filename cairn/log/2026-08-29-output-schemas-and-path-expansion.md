---
cairn: log
change: output-schemas-and-path-expansion
landed: 2026-08-29
---

# Every data command describes its JSON, and every path field expands where it is read

Group B of the cross-repository CLI alignment. Four items applied to Cardamum, two of them user-visible.

**Path expansion moved onto the fields** (config.rs, pimdir/client.rs): `pimdir.root` carries `#[serde(deserialize_with = "shell_expanded_path")]` and `PimdirClient::new` destructures the config instead of expanding it, so the expansion is a property of the field rather than of one constructor. `tls.cert` had none at all: a `cert = "~/ca.pem"` reached rustls as the literal `./~/ca.pem`, which is the same bug one layer down and far less visible, since it only surfaces on an account pinning its own certificate. It now carries `#[serde(default, deserialize_with = "opt_shell_expanded_path")]`, a private helper next to the config type with a TODO for the shared one pimalaya-config does not ship yet. The `default` is load-bearing: without it an absent key reaches the deserializer and fails.

**CardDAV accepts a credential-less server** (config.rs, carddav/client.rs, wizard/carddav.rs): `CarddavAuthConfig::None` joins Basic and Bearer, spelled `auth = "none"`, mapping onto the `WebdavAuth::None` io-webdav has spoken all along. Calendula's CalDAV twin already had it. The wizard offers it where discovery advertised no scheme at all, which is the case a server asking for nothing presents. Additive: every spelling that parsed before parses unchanged.

**Every data command names what it prints** (40 types across five backends plus the shared and meta commands): an output type is now `<Domain><Target><Verb>Output`, so `CardsTable` is `CardListOutput`, `SyncReport` is `CarddavReportSyncOutput`, `PersonReport` is `PeoplePersonOutput` and `RawJson` is `RawJsonOutput`. Each derives `JsonSchema` and documents every public field, which is where the schema descriptions come from. `carddav propfind` answers one untagged `CarddavPropfindOutput` over its two shapes, one command having one schema. Renaming moved no payload: every renamed type serializes exactly what it did.

**Four confirmations were data wearing a message** (`addressbook create`, `card create`, `card update`, `vdir item create`): each printed a server-assigned identifier, and the update the vCard properties the server would not let go, inside the prose of a `Message`. Under `--json` that is one string a consumer has to parse English out of. Each now answers its own `*Output`, and the terminal wording is unchanged word for word, which is what the `Display` impls are for. That is the one payload change in the release, and the CHANGELOG marks it breaking.

**Memo** (schemars 1): the contacts wire types carry no `JsonSchema`. io-jmap and io-msgraph gate one behind a `schemars` feature but derive it on their mail types only, and io-people has no such feature, so no feature flag would have helped. Each field holding one is described as `serde_json::Value` through `#[schemars(with = ...)]`, the same escape hatch Himalaya's `MessageView` uses. That is honest: the payload is the provider's to define.

**The registry** (json_schema.rs, cli.rs): 57 entries under the full feature set, keyed by the CLI invocation path, behind a `json-schema` command aliased `json-schemas`, matching Himalaya. A test walks the clap tree and fails on any key naming no command, so the registry cannot drift as the tree moves. Confirmations stay `Message` and are deliberately absent from it.

**Two warnings the feature sweep uncovered**, both older than this change: `wizard/search.rs` tested `AuthCaps::any` and `token`, which are gated on the DAV features, from ungated tests, so `cargo test` did not compile on the four backends that pull neither; and `shared/client.rs` imported `SecretResolver` ungated for three gated uses. Both are now gated, so every backend builds, tests and lints clean on its own.

**Toolkit layering** (17 files): every `comfy_table` import goes through `pimalaya_cli::table` and the direct dependency is gone, as Calendula already did. The two inline `url::Url` paths in `parse_server` became a `use`, its import gate widened to cover the JMAP feature that needs it.

Verified: 60 unit tests green with every feature, and 17 to 33 on each backend alone; `cargo check` and `clippy --all-targets` warning-free on every feature set; `json-schema --dir` writing 57 files; and the live eight-account configuration loading through `account list`, its tilde-rooted pimdir store listing and reading as before.

Spec updated: `config` (MODIFIED: "Path fields are shell-expanded"; ADDED: "A DAV server may ask for no credentials"), `commands` (ADDED: "A data command answers a named output type", "The JSON output of every data command has a schema", "comfy-table is reached through the toolkit").
