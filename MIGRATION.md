# Migrating Cardamum v0.1 → v2

Cardamum v2 lands on the Himalaya v2 stack (pimalaya-cli, pimalaya-config, pimalaya-stream, io-vdir 0.0.3, io-webdav 0.0.1, and the JMAP / Microsoft Graph / Google People protocol crates). The CLI structure changes accordingly.

## Command tree

| v0.1 | v2 |
| --- | --- |
| `cardamum addressbooks list` | `cardamum addressbook list` |
| `cardamum addressbooks create <name>` | `cardamum addressbook create <name>` |
| `cardamum cards list <book>` | `cardamum card list <book>` |
| `cardamum cards read <book> <id>` | `cardamum card read <book> <id>` |
| `cardamum cards update <book> <id>` | `cardamum card update <book> <id> --file <path>` |
| (none) | `cardamum vdir <list|create|rename|delete>` |
| (none) | `cardamum carddav <discover|propfind|report|list|create|delete>` |
| (none) | `cardamum account <list|check|configure>` |

The new `--backend` flag pins the shared subcommands to a specific backend (default: `auto`).

## TOML configuration

The `carddav.*` block keeps its name and gains the `discover`/`server-url`/`home-url` routes for resolving the addressbook home set:

```toml
carddav.discover.host = "example.org"
carddav.auth.basic.username = "user"
carddav.auth.basic.password.raw = "pwd"
```

The vdir block is unchanged:

```toml
vdir.home-dir = "~/.local/share/vdirsyncer/contacts"
```

## License

Cardamum is now dual-licensed under `MIT OR Apache-2.0` (was `AGPL-3.0`), matching Himalaya. The per-file license headers were dropped.
