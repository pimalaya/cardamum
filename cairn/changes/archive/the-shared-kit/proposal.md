---
cairn: change
id: the-shared-kit
status: landed
created: 2026-11-12
---

# The shared kit

## Why

Third change of the-family-standards. pimalaya-cli and pimalaya-config 0.4 carry what Ortie and Himalaya share: the first-run welcome, `-c` one path per flag, typed account resolution, plain text through `Columns`, `Fields` and `Records`. 0.4 no longer has the table module, so the version move and plain text output land together.

## What

- pimalaya-cli 0.4 (`prompt`, `terminal`; `build` for the build script) and pimalaya-config 0.4 (`toml`, `secret`). pimalaya-cli is pinned through `[patch.crates-io]` to git rev 0fbe5dd, as Himalaya pins it, until pimalaya-cli 0.4.1 publishes `records::Records` and the page-cursor fix.
- `ConfigFlags`: `-c` once per path, never split on `:`, and `CARDAMUM_CONFIG` no longer read (**Breaking**). With no `-c`, every canonical path that exists loads as a layer.
- `TomlConfig::load` and the shared `take_account`: a missing configuration is a `ConfigMissingError`, several accounts marked `default` are refused naming them, an unknown account lists the ones held.
- The welcome: a person at a terminal (stderr a terminal, `--json` off) missing a configuration meets pimalaya-cli's `Welcome` on stderr, naming the path, the commands creating it from the shipped sample (or `curl` from the repository), then `addressbook list`; a script or a JSON caller gets the error. A bare `cardamum` prints the help when a configuration exists.
- Plain text: no border, no colour, nothing sized to the terminal. `table.preset`, `table.arrangement` and every `*-color` key are gone: refused as unknown under `card.list` and `addressbook` (which deny unknown keys), ignored at the top and account levels. crossterm leaves; clap loses `env` and `wrap_help`. `account list` and `addressbook list` print `Columns`, `account check` `Fields`, ariadne reports draw without colour, and the protocol-specific renderers move to `Columns`, `Records` or `Fields`.
- `card list` prints as Himalaya's `envelope list` does since envelopes-print-one-line-each: `Account: <name> (<backend>)` and `Addressbook:` above, one headerless line per card (`id  email  phone  name`), then `Page: <n>, cards <a> to <b>[, the last]` and `Next page: --page <n>`. Its JSON gains `nextPage`, taken back through `--page-token`. The backends list a window of the address book (`skip`, `take`) and the unified client reads one card past the page, so `the last` and `Next page` are exact on every backend.
- G2 of cli-alignment-leftovers: the unified client keeps one `SecretResolver` per account, a reopened backend spawning no secret command twice. B3: a `carddav.alpn` key, `["http/1.1"]` by default, as `jmap`, `msgraph` and `gpeople` have.
- The header and footer of `card list` stay in cardamum (a copy of Himalaya's `write_fields`, labels aligned two spaces after `Addressbook:`): lifting them into pimalaya-cli is the owner's call, not taken here.

## Scope / non-goals

The sample rewrite and its test are the-sample-is-tested-and-shipped's; the dependency trimming is dependencies-trimmed's. No REPL or MCP yet.
