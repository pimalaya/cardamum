---
cairn: log
change: the-shared-kit
date: 2026-11-12
---

# The shared kit

pimalaya-cli 0.4 (pinned to git rev 0fbe5dd through `[patch.crates-io]` until 0.4.1 publishes `Records`) and pimalaya-config 0.4 are in. `-c` takes one path per flag and `CARDAMUM_CONFIG` is gone; without `-c` every canonical path layers. Account resolution runs through `TomlConfig::load` and `take_account` (several defaults refused), and a person at a terminal missing a configuration meets the welcome (`cardamum -c <missing> addressbook list` under `script` printed it, exit 1).

Text output is plain: crossterm, the table module and every `table.*` and colour key are gone. `card list` prints Himalaya's header and footer around one line per card, its JSON gains `nextPage` and `--page-token`, and the backends list a skip/take window so the unified client reads one card past the page: `the last` and `Next page` are exact everywhere (checked on a scratch vdir). `addressbook list` and `account list` print `Columns`, `account check` `Fields`; the protocol renderers moved to `Columns`, `Records` and `Fields` (a subagent converted them), ariadne draws without colour, and the protocol clients lost the `Account` they carried for the table options. G2 (one `SecretResolver` per unified client) and B3 (`carddav.alpn`) landed.

86 unit tests and 8 first-run tests pass; clippy `-D warnings` passes on the default build, all features and each backend alone. `json-schema` is structurally identical but `cardamum-card-list` (`nextPage`).

Capabilities moved: config ("Canonical paths and overrides", "Account resolution is explicit or default", "A credential command is spawned once per account", added "A missing configuration meets a person with the welcome", "A bare invocation shows the help", "One default account", "No colour is configured", "A CardDAV account names its ALPN identifiers", removed "Table rendering is configurable"), commands ("A displayed count is the server's", added "Text output is plain text", "Help is plain text", "A query error is drawn without colour", "Plain output is printable", "A card page names its context and its next page", "Listings resume from their cursor", removed "comfy-table is reached through the toolkit").
