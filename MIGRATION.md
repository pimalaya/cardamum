# Migration guide

## From v0.3 to the next release

The next release brings Cardamum to the standards Himalaya and Ortie share: no configuration wizard, one configuration path per `-c`, plain text with no table nor colour, a unified API whose backends say what they serve, and a REPL and an MCP server. A v0.3 configuration loads as it is, unless it sets a table or colour key under `card.list` or `addressbook`, or marks several accounts default.

What a script reads off `--json` barely moves; the text output changes everywhere, and a few commands now fail earlier or report differently. The CHANGELOG lists every change; these are the ones to act on.

### Configuration and first run

- **The wizard is gone.** Bare `cardamum` prints the help, `cardamum configure` (alias `wizard`) and its `cardamum-configure` schema no longer exist, and no command offers to create a configuration. Write it from config.sample.toml, which releases and packages ship as share/doc/cardamum/config.sample.toml: run at a terminal without a configuration, Cardamum prints the commands copying it in place. Drop the `wizard` cargo feature from a build's `--features`.
- **`-c` takes one path per flag.** A value is no longer split on `:`, so a Windows path stays whole. Repeat the flag to merge several files: `-c public.toml -c private.toml`.
- **`CARDAMUM_CONFIG` is no longer read.** Pass `-c "$CARDAMUM_CONFIG"` for one path, and one `-c` per path for what was a `:`-separated list.
- **Every default path is read, as layers.** $XDG_CONFIG_HOME/cardamum/config.toml, $HOME/.config/cardamum/config.toml and $HOME/.cardamumrc are each deep-merged on top of the ones before, where v0.3 read the first one found. An rc file left beside a config.toml now overrides it: delete the one you do not use.
- **Several `default = true` accounts are refused**, naming them. Keep it on one account across the merged files.
- **Table and colour keys are gone.** Remove `table.preset`, `table.arrangement` and the `*-color` keys of `addressbook.list.table`, `card.list.table` and `account.list.table`. Under `card.list` and `addressbook` they fail the configuration as unknown keys; at the top level and in an account they are ignored.
- **`http` and `https` hosts are ASCII.** A CardDAV or JMAP `server` or an HTTP proxy `url` with an internationalised host is refused: write the host in Punycode (`https://xn--bcher-kva.example` for `bücher.example`).
- **`carddav.alpn` is new**, `["http/1.1"]` by default, as the other network backends have.

### Text output

- **Every list is plain text**, with no border, no colour and no truncation: an uppercase header then one item per line, columns two spaces apart, an empty cell as `-`, the free-text column last; or, for a list of wide items (the CardDAV and JMAP addressbooks, Graph folders and contacts, vdir collections, `pimdir search` hits), each item the way `git log` prints a commit. A single item prints as `Label:  value` lines. A script cutting the v0.3 tables reads `--json` instead.
- **`card list` prints one line per card**, `id  email  phone  name`, with no header row, between `Account:` and `Addressbook:` above and `Page:` and `Next page:` under it. `addressbook list` leaves the description to `--json`.
- **`card list -s 0` lists every card**, where it listed none.

### JSON output

- **`card list` prints `nextPage`** beside `cards`: the next page number while cards follow, `null` on the last page. Give it back through `--page-token`.
- **A failed `account check` prints its report and exits 2**, `ok: false` and the error on each failed backend, where it printed one failure object and exited 1. Read which backends passed from `backends`.

### Commands that now fail earlier

- **A verb or a flag the account's backend does not serve is refused before connecting**, with the `capability-missing` code: `--if-match` on JMAP, Graph, vdir and People deletes, a colour on JMAP, Graph and People addressbooks, a description on Graph and People, any addressbook write on pimdir. The adapters used to refuse once connected, without a code. `cardamum account capability` lists what an account's backend serves, and each `addressbook` and `card` command's `--help` ends with a `Backends:` section.

### New

- `cardamum repl` runs commands over one account in a session, the unified client connecting once; it refuses the composer (`-i`) and stdin input (`stdin-taken`).
- `cardamum mcp` serves the unified API to an AI agent over the Model Context Protocol, the writes only with `--allow-write`.
- `cardamum account capability`.
