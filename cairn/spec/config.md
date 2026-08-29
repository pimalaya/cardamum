---
cairn: spec
capability: config
status: current
---

# Configuration

Configuration is loaded by pimalaya-config from a TOML file. The schema ([config.rs](../../src/config.rs)) is multi-account: a top-level block of shared options plus named `[accounts.<name>]` blocks, each carrying one or more backend sub-blocks. The full field reference lives in [config.sample.toml](../../config.sample.toml).

### Requirement: Canonical paths and overrides
The configuration SHALL be loaded from the first valid path among `$XDG_CONFIG_HOME/cardamum/config.toml`, `$HOME/.config/cardamum/config.toml` and `$HOME/.cardamumrc`. The path is overridable with `-c <PATH>` or `CARDAMUM_CONFIG=<PATH>`, and several paths may be passed at once separated by `:`, the first being the base and the rest deep-merged on top of it.

### Requirement: Account resolution is explicit or default
`cli::resolve_account` SHALL select the account named by `-a`, else the one marked `default = true`. A config that exists but lacks the requested account, or has no default, is a hard error naming the fix. Only when no config file exists at all is the wizard proposed.

### Requirement: Account resolution failures name what is missing
Each of the three ways account resolution fails SHALL name what is missing and what to do about it: a missing configuration names the path it looked for, a missing named account lists the accounts the configuration does hold, and a missing default names both ways of picking one.

### Requirement: One backend block per backend
Each account block SHALL carry at most one sub-block per compiled-in backend (`vdir`, `pimdir`, `carddav`, `jmap`, `msgraph`, `people`), each deserialized with `deny_unknown_fields` so a typo is an error rather than a silently ignored option. An account may declare several, and `--backend` picks between them (see [backends.md](backends.md)).

### Requirement: Secrets are read, never written
A credential field SHALL be a pimalaya-config `Secret`: a raw value in the file, or a command whose stdout is the secret (an argv array for a known keyring provider or token broker, a string for a shell command). Cardamum only ever reads a secret; it never stores, refreshes or issues one. An OAuth 2.0 token therefore comes from an external broker such as Ortie, and a missing secret surfaces when the account is tested.

### Requirement: A credential command is spawned once per account
The backends of one account SHALL resolve their credentials through a single pimalaya-config `SecretResolver`, which spawns each distinct command once and hands its value to every field naming it. `account check` and the wizard's connection test reach every configured backend, so an account naming one `pass` entry from its `carddav` and `jmap` tables would otherwise pay two key unlocks for one entry.

Distinctness SHALL be the command as the configuration wrote it: a shell line and the argv spelling that runs it are two commands, since reading one as the other means guessing what the configuration meant. A raw secret resolves to itself, having nothing to spawn.

A resolver SHALL live no longer than the account it is assembled for. It holds plaintext, so it belongs where an account is reached as a whole and is dropped with it, never held on a client nor shared between accounts.

### Requirement: Path fields are shell-expanded
A configuration field naming a filesystem path SHALL expand `~` and environment variables before use, so a path written home-relative resolves against the real home rather than the current directory.

### Requirement: Table rendering is configurable
The `table.preset` option SHALL accept a `comfy-table` v7 positional preset string, one character per component, mapped onto the v8 typed style builder by [shared/table.rs](../../src/shared/table.rs). Keeping the v7 spelling means an existing configuration stays valid across the v8 upgrade. A character position left out, or set to a space, draws nothing.
