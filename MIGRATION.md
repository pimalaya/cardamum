# Migrating Cardamum v0.1 to v2

Cardamum v2 lands on the Himalaya v2 stack (pimalaya-cli, pimalaya-config, pimalaya-stream, io-vdir, io-webdav, io-pimdir, and the JMAP / Microsoft Graph / Google People protocol crates). The CLI structure changes accordingly.

## Command tree

| v0.1 | v2 |
| --- | --- |
| `cardamum addressbooks list` | `cardamum addressbook list` |
| `cardamum addressbooks create <name>` | `cardamum addressbook create <name>` |
| `cardamum cards list <book>` | `cardamum card list -k <book>` |
| `cardamum cards read <book> <id>` | `cardamum card read -k <book> <id>` |
| `cardamum cards update <book> <id>` | `cardamum card update -k <book> <id> <VCARD>` |
| (none) | `cardamum vdir <list\|create\|rename\|delete>` and `cardamum vdir item <...>` |
| (none) | `cardamum carddav <discover\|propfind\|proppatch\|mkcol\|report\|get\|put\|delete>` |
| (none) | `cardamum jmap <address-book\|contact-card\|session\|request>` |
| (none) | `cardamum msgraph <contact-folder\|contact\|profile\|request>` |
| (none) | `cardamum people <contact-group\|connection\|other-contact\|profile\|request>` |
| (none) | `cardamum account <list\|check>` |

The plural `addressbooks` / `cards` forms stay as hidden aliases. The new `--backend` flag pins the shared subcommands to a specific backend (default: `auto`). The addressbook is now the `-k/--addressbook` flag rather than a positional argument, falling back to `addressbook.default`; `card create` and `card update` take their vCard as a trailing positional (a path, raw contents, or `-` for stdin) rather than `--file`.

There is no `account configure`: `cardamum configure` (alias `wizard`) runs the wizard, which is now the single way to create an account. A bare `cardamum` offers it when no configuration is found, and shows the help otherwise.

## TOML configuration

The `carddav.*` block keeps its name and gains three routes for resolving the addressbook home set, in decreasing order of magic:

```toml
carddav.discover = "example.org"
carddav.server = "https://carddav.example.org/dav/addressbooks"
#carddav.home = "https://carddav.example.org/dav/addressbooks/user/me/default"

carddav.auth.basic.username = "user"
carddav.auth.basic.password.command = "pass show example"
```

The vdir block is unchanged:

```toml
vdir.home-dir = "~/.local/share/vdirsyncer/contacts"
```

Four backends are new: `jmap`, `msgraph`, `people` and `pimdir`, each with its own account block. The full field reference lives in [config.sample.toml](./config.sample.toml).

## The wizard

`cardamum configure` runs a discovery-driven wizard aligned with Himalaya's, and it is offered on a bare `cardamum` or on any command finding no configuration. It takes one input (an email address, a server URL, or a local folder path), lists the services it discovers, asks how to authenticate, tests the connection, then writes the account into the configuration file, creating it or appending an `[accounts.<name>]` block to the one already there. It configures only what it discovers: there is no hand-entry of server fields, so a self-hosted server publishing no discovery record is configured by writing the account yourself from config.sample.toml.

## License

Cardamum is now dual-licensed under `MIT OR Apache-2.0` (was `AGPL-3.0`), matching Himalaya. The per-file license headers were dropped.
