# Migration guide

## From v0.3 to v0.4

v0.4 brings Cardamum to the standards Himalaya and Ortie share: no configuration wizard, one configuration path per `-c`, plain text with no table nor colour, one exit contract, a coded failure for what a caller acts on, an `errors` list on every command that reads several cards, a unified API whose backends say what they serve, and a REPL and an MCP server. A v0.3 configuration loads as it is, unless it sets a table or colour key, resolves its CardDAV server through `carddav.discover`, holds a key Cardamum does not know, or marks several accounts default.

The text output changes everywhere, the JSON of the writes and of the raw `request` commands changes shape, exit 3 is new, and a few commands now fail earlier or report differently. The CHANGELOG lists every change; these are the ones to act on.

### Configuration and first run

- **The wizard is gone.** Bare `cardamum` prints the help, `cardamum configure` (alias `wizard`) and its `cardamum-configure` schema no longer exist, and no command offers to create a configuration. Write it from config.sample.toml, which releases and packages ship as share/doc/cardamum/config.sample.toml: run at a terminal without a configuration, Cardamum prints the commands copying it in place. The sample holds one commented section per backend and provider; uncomment the one you need.
- **`-c` takes one path per flag.** A value is no longer split on `:`, so a Windows path stays whole. Repeat the flag to merge several files, the first one the base:

  ```sh
  # v0.3
  cardamum -c public.toml:private.toml card list
  # v0.4
  cardamum -c public.toml -c private.toml card list
  ```
- **`CARDAMUM_CONFIG` is no longer read.** Pass the path through `-c`, one flag per path for what was a `:`-separated list:

  ```sh
  # v0.3
  CARDAMUM_CONFIG=~/.config/cardamum/work.toml cardamum card list
  # v0.4
  cardamum -c ~/.config/cardamum/work.toml card list
  ```
- **Every default path is read, as layers.** $XDG_CONFIG_HOME/cardamum/config.toml, $HOME/.config/cardamum/config.toml and $HOME/.cardamumrc are each deep-merged on top of the ones before, where v0.3 read the first one found. An rc file left beside a config.toml now overrides it: delete the one you do not use.
- **`-a default` names the account called `default`**, where it took the account marked `default = true`. Omit `-a` to take that one:

  ```sh
  # v0.3
  cardamum -a default card list
  # v0.4
  cardamum card list
  ```

  Without an account named `default`, `-a default` fails coded `account-missing`, saying to omit `-a`.
- **Several `default = true` accounts are refused**, naming them. Keep it on one account across the merged files, or name the account with `-a <NAME>`.
- **Table and colour keys are gone.** Each fails the configuration by name (``The `table` table is removed, …: delete it, see MIGRATION.md``): the `table` table (`table.preset`, `table.arrangement`), at the top level and in each account, the top-level `account` table (`account.list.table`), and the `addressbook.list` and `card.list.table` tables (their `*-color` keys), at the top level and in each account. Delete them; `card.list.page-size` stays.

  ```toml
  # v0.3
  table.preset = "││──╞═╪╡┆    ┬┴┌┐└┘"
  table.arrangement = "dynamic"
  card.list.page-size = 50
  card.list.table.id-color = "red"
  addressbook.list.table.name-color = "green"
  account.list.table.name-color = "green"

  # v0.4
  card.list.page-size = 50
  ```
- **`carddav.discover` is gone.** Cardamum no longer resolves a domain to its CardDAV server (PACC, SRV and TXT records, `.well-known`) on every run, and a configuration holding `carddav.discover` fails by name (``The `carddav.discover` key is removed, …, see MIGRATION.md``). Name the server in `carddav.server`: a bare host still goes through its `/.well-known/carddav` over https, a URL with a path is the context root the principal walk starts from. Google answers its `.well-known` to an authenticated request alone, which Cardamum does not make: name the address book home in `carddav.home` instead, the account's address in it.

  ```toml
  # v0.3
  carddav.discover = "example.org"
  # v0.4, the host its SRV record named
  carddav.server = "dav.example.org"

  # v0.3, Fastmail
  carddav.discover = "fastmail.com"
  # v0.4
  carddav.server = "https://carddav.fastmail.com/dav/"

  # v0.3, iCloud
  carddav.discover = "icloud.com"
  # v0.4
  carddav.server = "https://contacts.icloud.com/"

  # v0.3, Posteo
  carddav.discover = "posteo.de"
  # v0.4
  carddav.server = "https://posteo.de:8843/"

  # v0.3, Google
  carddav.discover = "gmail.com"
  # v0.4
  carddav.home = "https://www.googleapis.com/carddav/v1/principals/alice@gmail.com/lists"
  ```
- **An unknown key fails the configuration**, at the top level and in an account too, naming the key, where v0.3 ignored it there (the backend and option tables refused one already): a misspelt `default` or `addressbook` no longer loads silently. Fix or delete it. A block for a backend the build leaves out still loads, unread, so one file serves a reduced build.

  ```toml
  # v0.3 loaded this, the account never default
  [accounts.example]
  defualt = true

  # v0.4: fails naming `defualt`; write
  [accounts.example]
  default = true
  ```
- **`http` and `https` hosts are ASCII.** A CardDAV or JMAP `server` or an HTTP proxy `url` with an internationalised host is refused: write the host in Punycode.

  ```toml
  # v0.3
  carddav.server = "https://bücher.example"
  # v0.4
  carddav.server = "https://xn--bcher-kva.example"
  ```
- **`pimdir.account` unset reads the store's one account.** Where v0.3 left it unset and read the collections grouped under no account, the backend now reads the one account the store groups its collections under, and refuses a store several accounts share, naming them. Set it on a shared store:

  ```toml
  [accounts.example]
  pimdir.root = "~/.local/state/neverest"
  pimdir.account = "example"
  ```
- **`carddav.alpn` is new**, `["http/1.1"]` by default, as the other network backends have. Nothing to change.
- **The credentials keep their shape.** `password.command`, `password.raw`, `token.command` and `token.raw` read as in v0.3.

### Exit codes and scripts

Cardamum follows the family exit contract (pimalaya-cli): 0 on success, 1 with the failure object printed, 2 on a command line the grammar refuses, 3 on a partial outcome, a command that printed its result without fully succeeding. v0.3 knew 0, 1 and clap's 2.

- **Exit 3 is a result, not a failure.** `addressbook list`, `card list`, `carddav report query` and `carddav report multiget` print their result and exit 3 when their `errors` list is not empty; `account check` prints its report and exits 3 when a backend failed. A script taking any non-zero exit as a failure drops a result it got:

  ```sh
  # v0.3
  cardamum --json card list > cards.json || exit 1

  # v0.4
  cardamum --json card list > cards.json
  case $? in
    0) ;;
    3) jq -r '.errors[] | "\(.id // "-")  \(.code // "-")  \(.error)"' cards.json >&2 ;;
    *) exit 1 ;;
  esac
  ```
- **A command line the grammar refuses exits 2 and, under `--json`, prints the failure object on stdout**, `{"code":"usage-invalid","error":…}`, where clap printed its text on stderr. Without `--json` clap's text still goes to stderr. A refused REPL line and an MCP tool call whose arguments the grammar refuses carry `usage-invalid` too.
- **Branch on `code`, never on `error`.** The failure object and each `errors` entry carry a stable kebab-case code (next section); `error` is free wording and moves.

### Text output

- **Every list is plain text**, with no border, no colour and no truncation: an uppercase header then one item per line, columns two spaces apart, an empty cell as `-`, the free-text column last; or, for a list of wide items (the CardDAV and JMAP addressbooks, Graph folders and contacts, vdir collections), each item the way `git log` prints a commit, a field the item does not hold printing no line. A single item prints as `Label:  value` lines. The list frame, the `Error:` lines and the `account capability` output are pimalaya-cli's, the same in Himalaya.
- **`account list` prints one line per account with no header row**, the default account marked `default`, the backends this build leaves out after the others; a script reads `cardamum --json account list`.
- **`card list` prints one line per card**, `id  email  phone  name`, with no header row, under `Account:`, `Addressbook:`, `Page:` and `Next page:`. `addressbook list` leaves the description to `--json`.

  ```text
  v0.3
  ┌────────┬──────┬──────────────────┬─────┐
  │ ID     ┆ FN   ┆ EMAIL            ┆ TEL │
  ╞════════╪══════╪══════════════════╪═════╡
  │ c1.vcf ┆ Jane ┆ jane@example.org ┆     │
  └────────┴──────┴──────────────────┴─────┘

  v0.4
  Account:      example (carddav)
  Addressbook:  contacts
  Page:         1, card 1, the last

  c1.vcf  jane@example.org  -  Jane
  ```
- **A command that reads several cards ends with the ones it could not read**, one `Error:` line each, `<id>  <code>  <error>` (`-` for a missing id or code), after the list and one blank line; nothing when it read them all. `carddav report query` and `report multiget` print theirs after the table and the truncation line.

  ```text
  c1.vcf  jane@example.org  -  Jane

  Error:  c2.vcf  permission-denied  The server refused the card with status 403
  ```
- **A script cutting the v0.3 tables reads `--json` instead:**

  ```sh
  # v0.3
  cardamum card list | awk -F'│' 'NR > 3 { print $2 }'
  # v0.4
  cardamum --json card list | jq -r '.cards[].id'
  ```
- **`card list -s 0` lists every card**, where it listed none.

### JSON output

- **The failure object always carries `code`**, `null` when none applies, beside `error`, `sources` and `backtrace` (pimalaya-cli 0.4):

  ```text
  v0.3
  {"error":"…","sources":["…"],"backtrace":null}

  v0.4
  {"code":"card-not-found","error":"…","sources":["…"],"backtrace":null}
  ```

  The codes a caller acts on: `account-missing`, `account-default-missing`, `account-default-several`, `backend-not-built`, `card-not-found`, `addressbook-not-found`, `addressbook-default-missing`, `card-unreadable`, `card-invalid`, `permission-denied`, `quota-exceeded`, `rate-limited`, `capability-missing`, `usage-invalid`, `stdin-taken`, `body-pending`, `precondition-failed`, and on pimdir the store, write and search codes io-pimdir names (`store-version`, `store-busy`, `item-duplicate`, `index-version`, `query-refused`…). `cardamum json-schema` describes the failure object as `cardamum-error`.
- **Every write prints a receipt** instead of `{"message": …}`; the text form keeps its sentence.

  ```sh
  # v0.3: {"message":"Card `c1.vcf` successfully deleted"}
  cardamum --json card delete c1.vcf | jq -r '.message'
  # v0.4: {"id":"c1.vcf","queued":false}
  cardamum --json card delete c1.vcf | jq -r '.id'
  ```

  `addressbook update|delete` and `card delete` print `{id, queued}`, `card add` `{id, queued}`, `card update` `{id, keptProperties, queued}` and `addressbook create` `{id, name, queued}`, `queued` true when the write waits for a sync (pimdir), false otherwise; `carddav mkcol|proppatch|delete`, `vdir create|rename|delete`, `vdir item update|copy|move|delete`, `jmap address-book destroy`, `jmap contact-card destroy`, `msgraph contacts delete`, `msgraph contact-folders delete` and the `gpeople` deletes print `{id}`, a renamed collection's new id, as do `jmap address-book update` and `jmap contact-card update` when the server returns no object; `jmap contact-card copy` prints `{ids}`, `carddav put` `{id, etag}`, `gpeople contact-groups members` `{id, added, removed}` and `card build -o` `{path, size}`.
- **A queued write says so in its JSON.** v0.3's `card create` printed `{"id": …}` and `card update` `{"id": …, "keptProperties": […]}`; both now add `queued`, and `addressbook create` adds `name` and `queued`. Read `queued` rather than the wording to tell a pimdir write waiting for a sync:

  ```sh
  # v0.4: {"id":"urn:uuid:…","queued":true}
  cardamum --json card add -k book card.vcf | jq -r 'if .queued then "queued" else .id end'
  ```
- **`card read` prints its `card list` row plus the vCard.** The keys `card list` prints join (`addressbookId`, `fnValue`, `email`, `tel`), and `encoding` comes beside `contents`: `utf-8` when `contents` is the vCard text, `base64` when its bytes are not UTF-8 and `contents` holds their base64; v0.3 failed on such a card. `card build` carries its card the same way:

  ```sh
  # v0.3
  cardamum --json card read -k book c1.vcf | jq -r '.contents'
  # v0.4
  cardamum --json card read -k book c1.vcf | jq -r 'if .encoding == "base64" then .contents | @base64d else .contents end'
  ```
- **The raw `request` commands wrap the response**: `gpeople request`, `jmap request` and `msgraph request` print `{"response": …}`, `null` when the server sent no body, where they printed the response itself.

  ```sh
  # v0.3
  cardamum --json msgraph request get me/contacts | jq '.value[]'
  # v0.4
  cardamum --json msgraph request get me/contacts | jq '.response.value[]'
  ```
- **The listings print `errors`.** `addressbook list` and `card list` print it beside their items, `carddav report query` and `report multiget` beside `cards` and `truncated`, `[]` when there is none: one `{id, code, error, sources}` per card the command could not read, where such a card was dropped (CardDAV, and a body that does not parse) or failed the whole listing (vdir, JMAP). Read the cards and their `errors` together; a non-empty list exits 3 (see above).

  ```text
  v0.3
  {"cards":[{"id":"c1.vcf",…}]}

  v0.4
  {"cards":[{"id":"c1.vcf",…}],"nextPage":null,"errors":[{"id":"c2.vcf","code":"permission-denied","error":"The server refused the card with status 403","sources":[]}]}
  ```

  A card whose body is no vCard or not UTF-8 is a `card-unreadable` entry alone, left out of `cards` and of the text's rows; a CardDAV row the server answered without the card's data is `permission-denied` on a 403, else `card-unreadable`, its refused properties in `sources`; a `report multiget` id the server does not hold is `card-not-found`; a `card list` the CardDAV server truncated adds an entry with a `null` id and no code, where `report query` keeps it as `truncated: true`. `cardamum json-schema` describes an entry as `cardamum-item-error`.

  ```sh
  # the cards read, then the ids that failed with their code
  cardamum --json card list | jq -r '.cards[].id'
  cardamum --json card list | jq -r '.errors[] | [.id, .code] | @tsv'
  ```
- **A failed `account check` prints its report and exits 3**, `ok: false`, the error and its `code` on each failed backend, where it printed one failure object and exited 1. Read which backends passed from `backends`:

  ```sh
  cardamum --json account check | jq -r '.backends[] | select(.ok | not) | [.backend, .code] | @tsv'
  ```
- **`card list` prints `nextPage`** beside `cards`: the next page number while cards follow, `null` on the last page. Give it back through `--page-token`. The cards also carry `uid`, `fullName`, `givenName`, `familyName`, `emails`, `phones`, `organization`, `organizationUnits`, `title` and `note`, and the address books `default`; a strict consumer accepts the new keys.
- **`carddav put` prints the id the write answered**, percent-encoded as a listing shows it, where it echoed its argument.
- **A JMAP set error of a type io-jmap does not model is named as the server sent it**, where it read `unknown`.
- **Piped `--json` prints one value per line**, and each schema of `cardamum json-schema` carries an `$id` naming the contract version.

### Command names

The commands take the names Himalaya and Calendula give them. Every v0.3 spelling is still accepted, hidden from the help, so a script keeps running; move to the new one when you touch it. What changes is the help, and the schema names of `cardamum json-schema`.

- **`addressbook delete` and `addressbook update` take the address book as a positional**, as `himalaya mailbox delete <MAILBOX>` does. `-k/--addressbook` still works and leaves the help; giving both is refused (`usage-invalid`). `addressbook update` without one still takes `addressbook.default`, `addressbook delete` never does.

  ```sh
  # v0.3
  cardamum addressbook delete -k work
  cardamum addressbook update -k work --name Work
  # v0.4
  cardamum addressbook delete work
  cardamum addressbook update work --name Work
  ```
- **`card create` is `card add`**, as `himalaya message add`; `card new` stays. `cardamum json-schema` names its schema `cardamum-card-add`, where v0.3 wrote `cardamum-card-create`, and the MCP tool is `card_add`.

  ```sh
  # v0.3
  cardamum card create -k personal ada.vcf
  # v0.4
  cardamum card add -k personal ada.vcf
  ```
- **The `gpeople` resources are plural**, as the People API and Himalaya's `gmail` name them: `gpeople connections`, `gpeople contact-groups`, `gpeople other-contacts`. The singular names, and the `people`, `contacts`, `group`, `groups` and `other` aliases, still work. `cardamum json-schema` names their schemas by the plural path (`cardamum-gpeople-connections-list`).
- **A JMAP card's `etag` is a BLAKE3 digest of its JSON**, 64 hex digits, the same from one build to the next, where v0.3 printed a 16-digit hash that changed with the Rust release a build used. An `etag` stored from v0.3 no longer matches: read the card again before passing it to `--if-match`.

### Commands that now fail earlier

- **A verb or a flag the account's backend does not serve is refused before connecting**, with the `capability-missing` code: `--if-match` on JMAP, Graph, vdir and People deletes, a colour on JMAP, Graph and People addressbooks, a description on Graph and People, any addressbook write on pimdir. The adapters used to refuse once connected, without a code. `cardamum account capability` lists what an account's backend serves, and each `addressbook` and `card` command's `--help` ends with a `Backends:` section.
- **A backend the build leaves out fails coded `backend-not-built`**, naming the cargo feature: its block still loads, so one configuration serves every build, but a reduced build no longer falls through to the next backend the account declares. An account declaring `carddav` then `jmap` ran on JMAP in a build without CardDAV; it now fails there, as `--backend jmap` still picks JMAP:

  ```sh
  # v0.3, a build without carddav: ran on JMAP
  cardamum card list
  # v0.4
  cardamum --backend jmap card list
  ```

  `account check` reports such a backend as a failed row (exit 3), and `account list --json` names it in the new `notBuilt`, `backends` still holding the backends the build carries alone; both follow the order `auto` resolves them (vdir, pimdir, carddav, jmap, msgraph, gpeople), where `backends` started with CardDAV.
- **A CardDAV answer that is not one whole multistatus fails** (an HTML page, a cut or empty body) instead of listing fewer cards (io-webdav 0.8).
- **A vdir id that is empty or holds a path separator is refused** (io-vdir 0.3).

### The libraries

The release moves onto the family line every Pimalaya CLI shares: pimalaya-cli, pimalaya-config and pimalaya-stream 0.4, io-pimdir 0.12, io-webdav 0.8, io-vdir 0.3, io-jmap 0.9, io-msgraph 0.9, io-gpeople 0.7 and io-http 0.8; io-pim-discovery leaves with `carddav.discover`. Most of it shows in the sections above (the failure object, the list frame, the exit contract, the pimdir store); the rest to act on:

- **A CardDAV address book outside the home set is listed and addressed by its path** rather than its last segment. A script that stored such an id reads it again from `addressbook list`.
- **A response whose head passes 64 KiB or whose body passes 256 MiB fails the exchange** with a typed error, and a connection closing mid-body fails, where they grew the buffer or spun.
- **A build from source** names the TLS and backend features as before; the `pimdir` feature now builds io-pimdir's search too.

### The pimdir store

- **A store an earlier neverest wrote is refused**, with the `store-version` code: this release reads a store of pimdir storage-draft-11 alone, and nothing migrates a draft store. Run the neverest that wrote it until nothing is queued, upgrade neverest to a release on the same draft, move the store aside, then run `neverest init` and sync again. Do not delete the store first: its queued writes live there alone. A search index of an earlier draft is refused with `index-version` until the sync engine rebuilds it, and a store another user can read with `store-exposed`.
- **A write queued over a card that changed is parked, not applied.** `card update` and `card delete` given `--if-match` name the version they matched on the write they queue, so a sync changing the card before applying it parks the write; and a `card update` or a `card delete` of a card in conflict is parked too, the card and its diverging body kept, where a delete settled the conflict. The command still answers that the write is queued: read `cardamum pimdir queue list` after a sync, which lists every parked write with the reason, and `cardamum pimdir queue cancel <ROW>` retracts one.

### New

- `cardamum pimdir status` reports the store behind the account: its address books, how many of their cards are downloaded, how many creations are queued, and the parked writes.
- `cardamum pimdir queue list`, `show` and `cancel`, as in Himalaya, list the cards queued for creation, which no listing shows before a sync applies them, and the parked rows (`row` and `kind` under `--json`), say where one row stands, and retract one, taking the store owner's role for that one write.
- `cardamum pimdir search` and `cardamum pimdir count` answer from the store's search index, with the coverage of the answer: under `--json`, `coverage` carries `complete`, `bodiless`, `missing`, `unparseable`, `unindexed`, `behind` and `filling`, the three counts of bodies out of sight standing at 0 under a query none of whose terms reads a card's body. `pimdir search` pages as `card list` does, `nextPage` taken back by `--page-token`, and lists a card the store cannot read in `errors`, exiting 3. Their options are read wherever they stand, after the query too, so a query word starting with `-`, a negated term, goes after `--`, or reads as `not`:

  ```sh
  cardamum --json pimdir search alice --page-size 5
  cardamum pimdir search -k work -- alice -bob
  cardamum pimdir search alice not bob
  ```
- `cardamum repl` runs commands over one account in a session, the unified client connecting once; it refuses the composer (`-i`) and stdin input (`stdin-taken`).
- `cardamum mcp` serves the unified API to an AI agent over the Model Context Protocol, the writes only with `--allow-write`.
- `cardamum account capability` lists, for each capability of the unified API, the backend serving it, its level and what the level leaves out: `{account, capabilities: [{name, backend, level, detail}]}` under `--json`, pimalaya-cli's shape, shared with Himalaya.
