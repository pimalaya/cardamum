---
cairn: delta
change: the-kit-carries-the-shared-outputs
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: The account outputs are the family's
(commands) `account list` SHALL print pimalaya-cli's `account::AccountListOutput` and `account check` its `account::AccountCheckOutput`, each row coded by cardamum's vocabulary, so both read as in Himalaya and Calendula. Their JSON keeps its shape: `{"accounts": [{"name", "default", "backends", "notBuilt"}]}` and `{"account", "backends": [{"backend", "ok", "code", "error"}]}`. The text of `account list` SHALL be one line per account with no header row: the name, its backends in one cell (the built ones, then those the build leaves out, each marked `(not built)`), and `default` on the default account. The text of `account check` SHALL be `Account: <name>`, then one `<backend>: OK` or `<backend>: FAIL (<error>)` line per backend, the name and the error defused.

#### Scenario: Two accounts
- GIVEN the accounts `home` over vdir and `work` over CardDAV, the default
- WHEN `cardamum account list` runs
- THEN it prints `home  vdir     -` then `work  carddav  default`, with no header line

### Requirement: A version line prints the binary's version
(repl) The command enum a session parses SHALL carry the binary's version (`#[command(version, long_version = long_version!())]`), so a `-V` or `--version` line, alone or after a command, prints what the command line prints: on stdout in a text session, on stderr under `--json`.

#### Scenario: The version inside a session
- GIVEN a text session
- WHEN the line `-V` is read
- THEN stdout holds `cardamum <version>` and the session goes on

## MODIFIED Requirements

### Requirement: A session refuses what cannot finish inside it
(repl) Inside a session, a `repl` or `mcp` line SHALL be refused by pimalaya-cli's REPL before cardamum runs anything, with `Cannot start a session inside a session`, coded `usage-invalid`; cardamum keeps no refusal of its own for it. The composer (`-i`) SHALL be refused by name, its editor needing the terminal the session reads, and a command that would read its input off stdin SHALL be refused with `stdin-taken`.

### Requirement: A call the grammar refuses is a usage failure
(mcp) A tool call whose arguments the tool's input schema admits but its command's grammar refuses SHALL answer a result with `isError`, its failure object coded `usage-invalid` and worded by the tool's grammar (`pimalaya_cli::mcp::grammar::Grammar::usage_error`), which names each argument as the tool's input does, never clap's usage text: a missing argument as ``Missing argument `card-id` ``, a refused value as ``Invalid value `-1` for `page-size`: <reason>``. The server SHALL go on.

#### Scenario: A card read missing its id
- GIVEN a server over a vdir account
- WHEN `card_read` is called with `addressbook` alone
- THEN the result is an error whose failure object is coded `usage-invalid` and reads ``Missing argument `card-id` ``

### Requirement: A backend left out of the build is named
(backends) When the backend `auto` resolves to is one the build leaves out, the unified commands and `account capability` SHALL fail coded `backend-not-built`, the message naming the cargo feature ("This build leaves out the `gpeople` backend, rebuild with `--features gpeople`"), rather than take the next declared backend or report none configured. `--backend` naming a backend the build leaves out SHALL be refused as a command line (`usage-invalid`, exit 2), with the same message. Under `auto`, `account check` SHALL report each declared backend the build leaves out as a failed row coded `backend-not-built`, at its place in the resolution order. `account list` SHALL name under `--json` the declared backends the build carries in `backends`, each a value `--backend` takes, and those it leaves out in `notBuilt`, both in the resolution order; the text SHALL print both in one cell, the built ones then those left out, each left-out one marked `(not built)` ("The account outputs are the family's", commands).

#### Scenario: account list keeps the two apart
- GIVEN a build with `vdir` and without `gpeople`, and an account declaring both
- WHEN `cardamum --json account list` runs
- THEN the account reads `"backends":["vdir"]` and `"notBuilt":["gpeople"]`, and the text cell `vdir,gpeople (not built)`

#### Scenario: An account over a backend left out
- GIVEN a build without `gpeople` and an account declaring `gpeople` alone
- WHEN `cardamum --json card list -k book` runs
- THEN it exits 1, its failure object coded `backend-not-built`, its error naming `--features gpeople`

#### Scenario: auto keeps its choice across builds
- GIVEN a build without `vdir` and an account declaring `vdir` and `carddav`
- WHEN `cardamum --json account capability` runs
- THEN it fails coded `backend-not-built`, where `--backend carddav` resolves to CardDAV

### Requirement: A build can be composed, and captured
(commands) `card build` SHALL take `-i/--interactive` and `--composer <COMMAND>`, the same pair the two write verbs carry, and SHALL print what the composer wrote rather than sending it anywhere. This is where a card is judged before it is written: the menu that used to offer a look is gone, and a build is the look.

`-o/--output <PATH>` SHALL write the card to that file instead of printing it, answering pimalaya-cli's `output::SavedOutput` rather than the card: `{path, size}` under `--json`, `Saved <size> bytes to <path>` in text. The flag earns its place on this command alone: the composer inherits stdout, so `card build -i > card.vcf` hands the editor the file as its terminal, and with `-i` there is no redirection to fall back on.

An abandoned interactive build SHALL print nothing at all, and exit as a success. The output of this command is a vCard someone pipes onwards, and a line saying no card was built is not one.

### Requirement: No colour is configured
(config) The configuration SHALL hold no colour and no table shape. The tables v0.3 read for them SHALL be refused by name in the family's sentence (pimalaya-config's `removed_key!`), which points at MIGRATION.md: `table` at the root and in an account, the top-level `account`, and `addressbook.list` and `card.list.table` at the root and in an account, as in ``The `table` table is removed, the output being plain text with no border: delete it, see MIGRATION.md``.

#### Scenario: A v0.3 table preset
- GIVEN a configuration holding `table.preset`
- WHEN it is loaded
- THEN it fails naming the `table` table as removed and MIGRATION.md

### Requirement: CardDAV resolves no domain
(config) `carddav.discover` SHALL be refused by name in the family's sentence (pimalaya-config's `removed_key!`), ``The `carddav.discover` key is removed, cardamum resolving no domain: <instead>, see MIGRATION.md``, the instead naming `carddav.server` as its replacement (Fastmail's `https://carddav.fastmail.com/dav/`) and `carddav.home` on Google. A build without `carddav` loads the block unread.

#### Scenario: A v0.3 discover route
- GIVEN an account holding `carddav.discover = "fastmail.com"`
- WHEN a command loads the configuration
- THEN it fails naming the `carddav.discover` key as removed, `carddav.server`, `carddav.home` and MIGRATION.md

## REMOVED Requirements
