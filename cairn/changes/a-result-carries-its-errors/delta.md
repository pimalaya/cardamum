---
cairn: delta
change: a-result-carries-its-errors
---

# Delta

Each requirement names its capability file in parentheses, as Himalaya's delta of the same name does.

## ADDED Requirements

### Requirement: A listing names the items it could not read
(commands) `addressbook list` and `card list` SHALL print an `errors` list beside their items, under `--json` and in text: one pimalaya-cli item error `{id, code, error, sources}` per address book or card the command could not read, `[]` when there is none, in the order met. `id` SHALL be the item's id as the command prints ids, `null` when the backend named none; `code` a code of the vocabulary or `null`; `error` the message; `sources` the backend's own reasons, outermost first. An item listed in part (a card whose body does not parse) SHALL print in the list and in `errors`. A non-empty `errors` SHALL make the command exit 3 once it printed its result. A listing that reads the whole address book before cutting its page SHALL print its entries on the first page only.

#### Scenario: A vdir card that cannot be read
- GIVEN a vdir address book holding three cards, one of whose files cannot be read
- WHEN `cardamum --json card list` runs
- THEN it prints the two other cards and `"errors":[{"id":"<its id>","code":"card-unreadable",…}]`, and the exit status is 3

#### Scenario: A card that does not parse
- GIVEN a card whose body is no vCard
- WHEN `cardamum --json card list` runs
- THEN the card lists with its id and empty fields, an entry coded `card-unreadable` names it, and the exit status is 3

### Requirement: A command's errors print at its end
(commands) In the text output, each `errors` entry SHALL print as one `Error:` line, `<id>  <code>  <error>` with `-` for a null id or code, last in the command's output: after the items and one blank line, the `Error:` lines aligned among themselves. Nothing SHALL print, the blank line included, when `errors` is empty.

### Requirement: A server's refusal carries its code
(backends) A failure a server refused SHALL carry a code of the vocabulary Himalaya shares, read from the typed error of the protocol that answered it, the failure object, an `errors` entry and an `account check` row alike:

| Code | When |
|---|---|
| `permission-denied` | an HTTP 403 that is no rate limit (Graph, People); a CardDAV 403 whose body names no precondition, or `need-privileges` (RFC 3744 §7.1.1); a JMAP `forbidden` |
| `quota-exceeded` | an HTTP 507; a CardDAV body naming `quota-not-exceeded` (RFC 4331 §6); Graph's `ErrorQuotaExceeded`; a JMAP `overQuota` |
| `rate-limited` | an HTTP 429, or a People 403 naming a rate limit or a quota, or a JMAP `rateLimit`; cardamum retries nothing, so the first one met |
| `card-not-found` | a card request answered 404 or 410 (CardDAV, People), 404 or 400 `ErrorInvalidIdMalformed` (Graph), `notFound` (JMAP); a vdir card file or a pimdir card that does not exist |
| `addressbook-not-found` | an address book named that does not exist (vdir, pimdir) |

A refusal none of these name SHALL carry no code, its reason kept in `sources`. A CardDAV 403 naming another precondition (`valid-address-data`, `no-uid-conflict`) is a card the server would not take, no matter of rights, and SHALL carry none.

#### Scenario: A read-only address book
- GIVEN a Graph contact folder the account may read but not write
- WHEN `cardamum --json card delete <id>` runs
- THEN it exits 1 printing a failure object coded `permission-denied`

### Requirement: Each backend names the cards it could not read
(backends) A backend SHALL report a card it could not read as an `errors` entry and SHALL never drop it silently nor fail the listing for it:

- every backend: a card whose body does not parse as a vCard, or is not UTF-8, SHALL list with its id and SHALL add a `card-unreadable` entry;
- vdir: a card file that cannot be read SHALL be a `card-unreadable` entry naming the card, its error in `sources`; a card removed while it lists SHALL be left out with no entry;
- JMAP: a card that does not project SHALL be a `card-unreadable` entry, its id when it has one;
- pimdir: a body that cannot be read SHALL be a `card-unreadable` entry, the card listed with its preview; a body the store does not hold yet SHALL be no entry;
- CardDAV, Graph, People: a page or an answer that fails SHALL fail the listing whole.

### Requirement: A partial outcome keeps its result
(mcp) A tool whose command ends with a partial outcome (a listing whose `errors` is not empty, an `account check` with a failed backend) SHALL return its result as `structuredContent` beside `isError: true`, so an agent reads the items and their `errors` together.

## MODIFIED Requirements

### Requirement: Exit statuses
(commands) The binary SHALL exit 0 on success, 1 with the failure object printed, 2 on a command line the grammar refuses (an unknown flag or command, a missing argument, a bad value), and 3 on a partial outcome, a command that printed its result without fully succeeding (`pimalaya_cli::error::PartialOutcome`): `account check` with a failed backend, and a listing whose `errors` is not empty. The binary SHALL parse its command line through `pimalaya_cli::clap::parse`: help and version print as clap prints them; a refused command line with a `--json` word before any `--` SHALL print on stdout the failure object coded `usage-invalid`, its `error` clap's message without the `error: ` prefix nor styles, and without `--json` clap's rendering on stderr.

#### Scenario: A refused command line under JSON
- GIVEN any configuration, or none
- WHEN `cardamum --json card list --nope` runs
- THEN it exits 2, prints nothing on stderr and prints `{"code":"usage-invalid","error":"unexpected argument '--nope' found…","sources":[],"backtrace":null}`

### Requirement: A JSON error carries a stable code
(commands) A failure a caller is expected to act on SHALL carry a stable code, printed under `--json` as a `code` string beside `error`, `sources` and `backtrace`, and in an `errors` entry beside `id`, `error` and `sources`. The code SHALL be found anywhere in the error chain, so added context does not hide it, or read from the typed error of the protocol that refused ("A server's refusal carries its code"). A failure with no code SHALL print `"code": null`. The plain output SHALL stay unchanged. Codes are kebab-case and never renamed; `error` stays free wording. A pimdir store's code is the one io-pimdir names (`PimdirError::code()`), never read off its message. Every code SHALL be declared in every build, whatever the backends compiled in.

| Code | Raised when |
|---|---|
| `usage-invalid` | the command line, a REPL line or an MCP tool call's arguments do not match the grammar (pimalaya-cli's `CommonCode::UsageInvalid`); the command line exits 2 |
| `stdin-taken` | a command inside a session (`repl`, `mcp`) would read its input off stdin, which carries the session's commands |
| `capability-missing` | the backend the account resolves to does not serve a capability the command reaches; nothing was read or written |
| `card-not-found` | the card a command names is not in the address book: the failure of `card read`, `card update` or `card delete` |
| `addressbook-not-found` | the address book a command names does not exist; nothing was read or written |
| `card-unreadable` | the backend answered for a card cardamum could not read: its file could not be read (vdir), its body did not parse as a vCard or is not UTF-8, it did not project (JMAP), its body could not be read (pimdir); an `errors` entry |
| `permission-denied` | the server refused the request to the account (see "A server's refusal carries its code") |
| `quota-exceeded` | the write would exceed the account's quota |
| `rate-limited` | the server throttled the request; cardamum retries nothing |
| `body-pending` | a listed card's body is not local yet (pimdir); a sync brings it |
| `precondition-failed` | a write's `--if-match` names a version the card no longer has (pimdir) |
| `store-uncreated` | the pimdir root holds no store; its sync engine creates it |
| `store-version` | the pimdir store is of another format version than this cardamum reads; an earlier neverest's store is reinstalled through neverest, its queue drained first, a later one's asks for a cardamum upgrade |
| `store-foreign` | the pimdir root holds a database another program wrote |
| `store-corrupt` | the pimdir store's two version stamps disagree, or its schema differs from the one its version declares |
| `store-hash-algo` | the pimdir store names its bodies by a hash this cardamum does not know |
| `store-exposed` | another user can read or write the pimdir store, or a link stands where a lock file belongs; restrict it |
| `store-busy` | another writer held the pimdir store, past the wait on a write and at once on `account check`, which does not wait; retry |
| `lock-replaced` | the pimdir store's staging lock file was replaced while a write held it; nothing was queued, retry once no other process sets the store up |
| `unsupported` | a source syncing the address book does not support the write (pimdir STORAGE §15.6); nothing was queued |
| `item-duplicate` | a create names a `UID` the pimdir address book holds, synced or queued; nothing was queued |
| `action-malformed` | a queued pimdir action's payload is malformed (`pimdir status`); the sync engine's drain parks it |
| `index-missing` | the pimdir store has no search index yet (`pimdir search`, `pimdir count`); its sync engine builds it |
| `index-outdated` | the pimdir search index was built otherwise or from another store; its sync engine rebuilds it |
| `index-version` | the pimdir search index is of another draft or shape; its sync engine rebuilds one of an earlier draft and leaves any other |
| `index-fts5-missing` | the SQLite cardamum links lacks FTS5 or predates 3.43, which the pimdir search index needs |
| `query-refused` | the pimdir search query is malformed or names an unknown field or value (SEARCH §8.4); nothing was searched |
| `query-unanswerable` | the pimdir search index could not answer an accepted query, an implementation fault |

#### Scenario: A stale version
- GIVEN a pimdir card whose body changed since it was read
- WHEN `cardamum --json card delete --if-match <old etag>` names it
- THEN the command exits 1, queues nothing and prints `{"code":"precondition-failed","error":"Precondition failed: …","sources":[],"backtrace":null}`

#### Scenario: An unknown card
- GIVEN a vdir address book holding no card `nope`
- WHEN `cardamum --json card read nope` runs
- THEN the command exits 1 and prints a failure object coded `card-not-found`

### Requirement: The JSON contract is described and versioned
(commands) `json-schema` SHALL describe the failure object under the key `cardamum-error`, its `code` typed by the code vocabulary and pimalaya-cli's `usage-invalid`, and the item error of an `errors` list under `cardamum-item-error`, its `code` typed by the code vocabulary. Every generated schema SHALL carry an `$id` naming the contract version: the major version of cardamum, or `0.<minor>` before 1.0. Every registered payload SHALL be a JSON object, checked by a test: a command whose shapes are one untagged type answers an empty object for a shape carrying nothing (an abandoned `card create -i`), never `null`. Piped `--json` output SHALL print one value per line.

### Requirement: A card page names its context and its next page
(commands) `card list` SHALL print above the cards `Account: <name> (<backend>)`, `Addressbook: <id>`, `Page: <n>, cards <a> to <b>` (`card <a>` for one, `empty` for none, `all` when unpaged) followed by `, the last` when no card follows, and `Next page: --page <n>` while one does, the values aligned in one column; then a blank line and one line per card with no column header (`id  email  phone  name`, the free-text name last, `-` for an empty cell). Below the cards SHALL print only the command's errors ("A command's errors print at its end").

### Requirement: A listing is whole or fails
(backends) A listing SHALL never come back shorter than the backend holds without saying so: a card it could not read SHALL be an `errors` entry ("Each backend names the cards it could not read"), and a listing that cannot be read at all SHALL fail. A CardDAV listing whose answer is not one whole `multistatus` document (an HTML page, a cut body, an empty one) SHALL fail with io-webdav's error; until io-webdav returns the rows it could not read, a CardDAV row refused per href is left out and a listing the server truncated reads complete. A vdir listing SHALL leave out a card removed while it lists. A Graph listing SHALL follow its next page through io-msgraph's `page_get`, which refuses a link off the Graph origin, so the token never follows a link elsewhere.

## REMOVED Requirements

None.
