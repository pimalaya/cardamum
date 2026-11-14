# Migration guide

## From v0.3 to the next release

The next release brings Cardamum to the standards Himalaya and Ortie share: no configuration wizard, one configuration path per `-c`, plain text with no table nor colour, a unified API whose backends say what they serve, and a REPL and an MCP server. A v0.3 configuration loads as it is, unless it sets a table or colour key, holds a key Cardamum does not know, or marks several accounts default.

What a script reads off `--json` barely moves; the text output changes everywhere, and a few commands now fail earlier or report differently. The CHANGELOG lists every change; these are the ones to act on.

### Configuration and first run

- **The wizard is gone.** Bare `cardamum` prints the help, `cardamum configure` (alias `wizard`) and its `cardamum-configure` schema no longer exist, and no command offers to create a configuration. Write it from config.sample.toml, which releases and packages ship as share/doc/cardamum/config.sample.toml: run at a terminal without a configuration, Cardamum prints the commands copying it in place. Drop the `wizard` cargo feature from a build's `--features`.
- **`-c` takes one path per flag.** A value is no longer split on `:`, so a Windows path stays whole. Repeat the flag to merge several files: `-c public.toml -c private.toml`.
- **`CARDAMUM_CONFIG` is no longer read.** Pass `-c "$CARDAMUM_CONFIG"` for one path, and one `-c` per path for what was a `:`-separated list.
- **Every default path is read, as layers.** $XDG_CONFIG_HOME/cardamum/config.toml, $HOME/.config/cardamum/config.toml and $HOME/.cardamumrc are each deep-merged on top of the ones before, where v0.3 read the first one found. An rc file left beside a config.toml now overrides it: delete the one you do not use.
- **Several `default = true` accounts are refused**, naming them. Keep it on one account across the merged files.
- **Table and colour keys are gone.** Remove the `table` table (`table.preset`, `table.arrangement`), at the top level and in each account, the top-level `account` table (`account.list.table`), and the `addressbook.list` and `card.list.table` tables (their `*-color` keys). Each fails the configuration by name, pointing here.
- **An unknown key fails the configuration**, at the top level and in an account too, naming the key, where v0.3 ignored it there: a misspelt `default` or `addressbook` no longer loads silently. A block for a backend the build leaves out still loads, unread.
- **`http` and `https` hosts are ASCII.** A CardDAV or JMAP `server` or an HTTP proxy `url` with an internationalised host is refused: write the host in Punycode (`https://xn--bcher-kva.example` for `bücher.example`).
- **`carddav.alpn` is new**, `["http/1.1"]` by default, as the other network backends have.

### Text output

- **Every list is plain text**, with no border, no colour and no truncation: an uppercase header then one item per line, columns two spaces apart, an empty cell as `-`, the free-text column last; or, for a list of wide items (the CardDAV and JMAP addressbooks, Graph folders and contacts, vdir collections, `pimdir search` hits), each item the way `git log` prints a commit. A single item prints as `Label:  value` lines. A script cutting the v0.3 tables reads `--json` instead.
- **`card list` prints one line per card**, `id  email  phone  name`, with no header row, under `Account:`, `Addressbook:`, `Page:` and `Next page:`. `addressbook list` leaves the description to `--json`.
- **A listing ends with the cards it could not read**, one `Error:` line each, `<id>  <code>  <error>`, after the list and one blank line; nothing when it read them all.
- **`card list -s 0` lists every card**, where it listed none.

### JSON output

- **`card list` prints `nextPage`** beside `cards`: the next page number while cards follow, `null` on the last page. Give it back through `--page-token`.
- **A failed `account check` prints its report and exits 3**, `ok: false`, the error and its `code` (`store-busy`, `store-uncreated`...) on each failed backend, where it printed one failure object and exited 1. Read which backends passed from `backends`.
- **`addressbook list` and `card list` print `errors`** beside their items, `[]` when there is none: one `{id, code, error, sources}` per card the listing could not read (`card-unreadable`, a CardDAV 403 row `permission-denied`, a truncated CardDAV listing an entry with a `null` id and no code). A non-empty `errors` exits 3 once the result is printed, where such a card was dropped or failed the whole listing with exit 1: read the cards and their `errors` together rather than taking exit 3 as a failure.
- **A card or an address book not found carries a code**, `card-not-found` or `addressbook-not-found`, and a server's refusal `permission-denied`, `quota-exceeded` or `rate-limited`, where their `code` was `null`.
- **A command line the grammar refuses under `--json` prints the failure object**, `{"code":"usage-invalid","error":…}` on stdout, and still exits 2, where clap printed its text on stderr.

### Commands that now fail earlier

- **A verb or a flag the account's backend does not serve is refused before connecting**, with the `capability-missing` code: `--if-match` on JMAP, Graph, vdir and People deletes, a colour on JMAP, Graph and People addressbooks, a description on Graph and People, any addressbook write on pimdir. The adapters used to refuse once connected, without a code. `cardamum account capability` lists what an account's backend serves, and each `addressbook` and `card` command's `--help` ends with a `Backends:` section.

### The pimdir store

- **A store an earlier neverest wrote is refused**, with the `store-version` code: this release reads a store of pimdir storage-draft-11 alone, and nothing migrates a draft store. Run the neverest that wrote it until nothing is queued, upgrade neverest to a release on the same draft, move the store aside, then run `neverest init` and sync again. Do not delete the store first: its queued writes live there alone. A search index of an earlier draft is refused with `index-version` until the sync engine rebuilds it.
- **A write queued over a card that changed is parked, not applied.** `card update` and `card delete` given `--if-match` name the version they matched on the write they queue, so a sync changing the card before applying it parks the write; and a `card update` or a `card delete` of a card in conflict is parked too, the card and its diverging body kept, where a delete settled the conflict. The command still answers that the write is queued: read `cardamum pimdir status` after a sync, which lists every parked write with the reason.

### New

- `cardamum pimdir search` and `cardamum pimdir count` answer from the store's search index, with the coverage of the answer: under `--json`, `coverage` carries `complete`, `bodiless`, `missing`, `unparseable`, `unindexed`, `behind` and `filling`, the three counts of bodies out of sight standing at 0 under a query none of whose terms reads a card's body.
- `cardamum repl` runs commands over one account in a session, the unified client connecting once; it refuses the composer (`-i`) and stdin input (`stdin-taken`).
- `cardamum mcp` serves the unified API to an AI agent over the Model Context Protocol, the writes only with `--allow-write`.
- `cardamum account capability` lists, for each capability of the unified API, the backend serving it, its level and what the level leaves out.
