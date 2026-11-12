---
cairn: delta
change: the-shared-kit
---

# Delta

## ADDED Requirements

### Requirement: A missing configuration meets a person with the welcome
(config) When no configuration exists at the first `-c` path or the default location, and stderr is a terminal with `--json` off, a bare `cardamum`, every command resolving an account and `account list` SHALL print pimalaya-cli's welcome on stderr and exit 1, printing nothing on stdout. The welcome SHALL frame Cardamum in one sentence, name the missing path, give the commands creating it from config.sample.toml (copied from share/doc/cardamum next to the binary when it is installed, downloaded with `curl` otherwise), invite the reader to edit it, and give the `addressbook list` command carrying the `-c` paths. Otherwise the missing-configuration error SHALL stand, as the failure object under `--json`.

### Requirement: A bare invocation shows the help
(config) A bare `cardamum` SHALL print the help when a configuration exists, a broken one included, its error left to the first command reading it. Without one it SHALL fail as a command resolving an account does, the welcome or the error. Nothing prompts.

### Requirement: One default account
(config) Resolving the default account SHALL fail, naming them sorted, when several accounts are marked `default = true` once the configuration is loaded, layers and `-c` files merged. An account named with `-a` SHALL still resolve.

### Requirement: No colour is configured
(config) The configuration SHALL hold no colour and no table shape: `table.preset`, `table.arrangement` and the `*.list.table.*-color` keys are unknown keys, refused where their table denies unknown keys (`card.list`, `addressbook`) and ignored at the top level and the account level, which tolerate unknown keys for a neighbouring tool.

### Requirement: A CardDAV account names its ALPN identifiers
(config) `carddav.alpn` SHALL list the ALPN protocol identifiers offered during the TLS handshake, `["http/1.1"]` by default, as `jmap`, `msgraph` and `gpeople` do.

### Requirement: Text output is plain text
(commands) A text output SHALL be plain text: no border, no colour, nothing sized to or truncated for the terminal, and it SHALL end with a newline. A list of short rows prints as aligned columns, a list of wide items as records, a single item as `Label:  value` fields (pimalaya-cli `Columns`, `Records`, `Fields`).

### Requirement: Help is plain text
(commands) `--help` and `-h` SHALL print without colour, each paragraph on one line, never wrapped to the terminal width.

### Requirement: A query error is drawn without colour
(commands) The report `pimdir search` and `pimdir count` draw over a query they refuse SHALL carry no colour escape, on a terminal as in the `--json` error string.

### Requirement: Plain output is printable
(commands) A field taken from a card or a server (a name, an address, an id, an error a server wrote) SHALL have its control characters and bidi controls replaced before it is printed as text (pimalaya-cli `sanitize::text`). JSON strings and the raw bytes a command is asked for SHALL remain unchanged.

### Requirement: A card page names its context and its next page
(commands) `card list` SHALL print `Account: <name> (<backend>)` and `Addressbook: <id>` above the cards, a blank line, one line per card with no column header (`id  email  phone  name`, the free-text name last, `-` for an empty cell), a blank line, then `Page: <n>, cards <a> to <b>` (`card <a>` for one, `empty` for none, `all` when unpaged) followed by `, the last` when no card follows, and `Next page: --page <n>` while one does, the values aligned in one column.

### Requirement: Listings resume from their cursor
(commands) `card list` SHALL print `nextPage` beside `cards`: the next page number, as a string, while cards follow the page, `null` otherwise, and SHALL take it back through `--page-token`, which wins over `--page`. Whether cards follow is exact on every backend: the unified client reads one card past the page.

## MODIFIED Requirements

### Requirement: Canonical paths and overrides
(config) Without `-c/--config`, the configuration SHALL load every existing canonical path as a layer, in this order: `$XDG_CONFIG_HOME/cardamum/config.toml`, `$HOME/.config/cardamum/config.toml`, `$HOME/.cardamumrc`, each deep-merged on top of the ones before, paths naming the same file read once. `-c/--config` SHALL replace the canonical paths entirely. It MAY be repeated: the first path is the base and the next ones are deep-merged on top, in order. Each occurrence SHALL be one path, never split on a delimiter, and no environment variable SHALL be read for it.

### Requirement: Account resolution is explicit or default
(config) `cli::resolve_account` SHALL load the configuration through pimalaya-config's `TomlConfig::load` and take the account through its `take_account`: the one named by `-a`, else the one marked `default = true`. A config that exists but lacks the requested account, or has no default, is a hard error naming the fix. When no configuration is found at all, a command needing one SHALL fail with a `ConfigMissingError` naming the path it looked at (the first `-c` path, else the default location) and pointing at the documented sample, unless a person meets the welcome instead. Cardamum SHALL NOT prompt to create a configuration, nor write one.

### Requirement: A credential command is spawned once per account
(config) The backends of one account SHALL resolve their credentials through a single pimalaya-config `SecretResolver`, which spawns each distinct command once and hands its value to every field naming it. `account check` reaches every configured backend, so an account naming one `pass` entry from its `carddav` and `jmap` tables would otherwise pay two key unlocks for one entry, and the unified client keeps its resolver across the reconnection that follows a composer.

### Requirement: A displayed count is the server's
(commands) A text output SHALL NOT render an absent value as a default that reads as fact. A count the API returns only on request SHALL be requested, or left blank (the rest of the requirement unchanged).

## REMOVED Requirements

### Requirement: Table rendering is configurable
### Requirement: comfy-table is reached through the toolkit
