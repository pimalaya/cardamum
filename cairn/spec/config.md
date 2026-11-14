---
cairn: spec
capability: config
status: current
---

# Configuration

Configuration is loaded by pimalaya-config from a TOML file. The schema ([config.rs](../../src/config.rs)) is multi-account: a top-level block of shared options plus named `[accounts.<name>]` blocks, each carrying one or more backend sub-blocks. The full field reference lives in [config.sample.toml](../../config.sample.toml).

### Requirement: Canonical paths and overrides
Without `-c/--config`, the configuration SHALL load every existing canonical path as a layer, in this order: `$XDG_CONFIG_HOME/cardamum/config.toml`, `$HOME/.config/cardamum/config.toml`, `$HOME/.cardamumrc`, each deep-merged on top of the ones before, paths naming the same file read once. `-c/--config` SHALL replace the canonical paths entirely. It MAY be repeated: the first path is the base and the next ones are deep-merged on top, in order. Each occurrence SHALL be one path, never split on a delimiter, and no environment variable SHALL be read for it.

### Requirement: Account resolution is explicit or default
`cli::resolve_account` SHALL load the configuration through pimalaya-config's `TomlConfig::load` and take the account through its `take_account`: the one named by `-a`, else the one marked `default = true`. A config that exists but lacks the requested account, or has no default, is a hard error naming the fix. When no configuration is found at all, a command needing one SHALL fail with a `ConfigMissingError` naming the path it looked at (the first `-c` path, else the default location) and pointing at the documented sample, unless a person meets the welcome instead. Cardamum SHALL NOT prompt to create a configuration, nor write one.

### Requirement: Account resolution failures name what is missing
Each of the three ways account resolution fails SHALL name what is missing and what to do about it: a missing configuration names the path it looked for, a missing named account lists the accounts the configuration does hold, and a missing default names both ways of picking one.

### Requirement: One backend block per backend
Each account block SHALL carry at most one sub-block per backend (`vdir`, `pimdir`, `carddav`, `jmap`, `msgraph`, `gpeople`), each deserialized with `deny_unknown_fields` when its backend is compiled in. An account may declare several, and `--backend` picks between them (see [backends.md](backends.md)).

### Requirement: An unknown key is refused
Every table of the configuration, the root and each account included, SHALL refuse a key it does not know, failing the load with the key's name, so a typo is an error rather than a silently ignored option. The block of a backend the build leaves out (and `proxy` in a build with no network backend) SHALL load unread, so one file serves every build.

#### Scenario: A typo in an account
- GIVEN `[accounts.example]` holding `defualt = true`
- WHEN the configuration is loaded
- THEN it fails naming the unknown field `defualt`

### Requirement: Secrets are read, never written
A credential field SHALL be a pimalaya-config `Secret`: a raw value in the file, or a command whose stdout is the secret (an argv array for a known keyring provider or token broker, a string for a shell command). Cardamum only ever reads a secret; it never stores, refreshes or issues one. An OAuth 2.0 token therefore comes from an external broker such as Ortie, and a missing secret surfaces when the account is tested.

### Requirement: A credential command is spawned once per account
The backends of one account SHALL resolve their credentials through a single pimalaya-config `SecretResolver`, which spawns each distinct command once and hands its value to every field naming it. `account check` reaches every configured backend, so an account naming one `pass` entry from its `carddav` and `jmap` tables would otherwise pay two key unlocks for one entry, and the unified client keeps its resolver across the reconnection that follows a composer.

Distinctness SHALL be the command as the configuration wrote it: a shell line and the argv spelling that runs it are two commands, since reading one as the other means guessing what the configuration meant. A raw secret resolves to itself, having nothing to spawn.

A resolver SHALL live no longer than the account it is assembled for. It holds plaintext, so it belongs where an account is reached as a whole and is dropped with it, never held on a client nor shared between accounts.

### Requirement: Path fields are shell-expanded
A configuration field naming a filesystem path SHALL expand `~` and environment variables as the value is deserialized, never at a call site, so a path written home-relative resolves against the real home rather than the current directory and no reader of the field can forget. An optional path SHALL carry `serde(default)` alongside its deserializer, an absent key never reaching it.

### Requirement: A DAV server may ask for no credentials
`CarddavAuthConfig` SHALL carry a credential-less variant alongside Basic and Bearer, spelled `auth = "none"`, which emits no `Authorization` header.

### Requirement: The composer is a command, not a library
`card.composer` SHALL name a command, at the top level and per account, rather than cardamum linking one editor in. What crosses the boundary is a path to a file holding vCard bytes, so the value can be tCard today and a graphical editor tomorrow, and cardamum learns no projection, no TOML and no editor.

It SHALL be a pimalaya-config `CommandConfig`, taking the shell line and the argv list every other command field takes. The command must block until the edit is done, which the documentation SHALL say, since a graphical editor returning immediately is the way this fails in practice.

### Requirement: A proxy is set per account and per backend
An account's `proxy` SHALL apply to every network backend of that account (CardDAV, JMAP, Microsoft Graph, Google People) whose own block names no `proxy`, and a backend's own `proxy` SHALL win over it. With neither, the connection SHALL read the `all_proxy` and `https_proxy` environment variables, honouring `no_proxy`. CardDAV `discover` lookups SHALL read the environment only, io-pim-discovery taking no proxy.

`proxy.url` SHALL take `socks5://`, `socks5h://` or `http://`. `proxy.username` and `proxy.password` SHALL override the URL's user info, the password being a secret like any credential; a password without a username SHALL be rejected.

### Requirement: A missing configuration meets a person with the welcome
When no configuration exists at the first `-c` path or the default location, and stderr is a terminal with `--json` off, a bare `cardamum`, every command resolving an account and `account list` SHALL print pimalaya-cli's welcome on stderr and exit 1, printing nothing on stdout. The welcome SHALL frame Cardamum in one sentence, name the missing path, give the commands creating it from config.sample.toml (copied from share/doc/cardamum next to the binary when it is installed, downloaded with `curl` otherwise), invite the reader to edit it, and give the `addressbook list` command carrying the `-c` paths. Otherwise the missing-configuration error SHALL stand, as the failure object under `--json`.

### Requirement: A bare invocation shows the help
A bare `cardamum` SHALL print the help when a configuration exists, a broken one included, its error left to the first command reading it. Without one it SHALL fail as a command resolving an account does, the welcome or the error. Nothing prompts.

### Requirement: One default account
Resolving the default account SHALL fail, naming them sorted, when several accounts are marked `default = true` once the configuration is loaded, layers and `-c` files merged. An account named with `-a` SHALL still resolve.

### Requirement: No colour is configured
The configuration SHALL hold no colour and no table shape. The tables v0.3 read for them SHALL be refused by name, the error pointing at MIGRATION.md: `table` at the root and in an account, the top-level `account`, and `addressbook.list` and `card.list.table` at the root and in an account.

#### Scenario: A v0.3 table preset
- GIVEN a configuration holding `table.preset`
- WHEN it is loaded
- THEN it fails naming the `table` table as removed and MIGRATION.md

### Requirement: A CardDAV account names its ALPN identifiers
`carddav.alpn` SHALL list the ALPN protocol identifiers offered during the TLS handshake, `["http/1.1"]` by default, as `jmap`, `msgraph` and `gpeople` do.

### Requirement: One sample configuration, with a section per backend
config.sample.toml SHALL hold one `[accounts.example]` table, the options every account shares active at their defaults, then one commented section per backend or provider (CardDAV, CardDAV without credentials, Google CardDAV, JMAP, Fastmail, Microsoft Graph, Google People, vdir, pimdir). A test SHALL parse every variant pimalaya-config's `sample::variants` derives from it through the loader `-c` uses, `example` resolving as the default account and declaring a backend.
