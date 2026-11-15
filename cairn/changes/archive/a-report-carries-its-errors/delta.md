---
cairn: delta
change: a-report-carries-its-errors
---

# Delta

Each requirement names its capability file in parentheses, as the delta of a-result-carries-its-errors does.

## ADDED Requirements

### Requirement: A CardDAV report names the cards it could not read
(commands) `carddav report query` and `carddav report multiget` SHALL print an `errors` list beside `cards` and `truncated`, under `--json` and in text, as "A listing names the items it could not read" says: one pimalaya-cli item error per row the server answered without the card's data, coded as a CardDAV listing row is ("Each backend names the cards it could not read"), and on `report multiget` a row answered 404 or 410, an id the address book does not hold, coded `card-not-found`; `[]` when there is none. A non-empty `errors` SHALL make the command exit 3 once it printed its result. A result the server truncated SHALL stay `truncated`, its text line kept, and SHALL add no entry: a query's `--limit` asks for the cut (RFC 6352 §8.6.1), so a truncation is no partial outcome there. In the text the `Error:` lines print last ("A command's errors print at its end"), after the table and the truncation line, with no blank line leading an output that holds nothing else.

#### Scenario: An id the server does not hold
- GIVEN an address book holding `c1.vcf` and no `c9.vcf`
- WHEN `cardamum --json carddav report multiget <book> c1.vcf c9.vcf` runs
- THEN it prints `c1.vcf` in `cards` and `"errors":[{"id":"c9.vcf","code":"card-not-found",…}]`, and the exit status is 3

## MODIFIED Requirements

### Requirement: Exit statuses
(commands) The binary SHALL exit 0 on success, 1 with the failure object printed, 2 on a command line the grammar refuses (an unknown flag or command, a missing argument, a bad value), and 3 on a partial outcome, a command that printed its result without fully succeeding (`pimalaya_cli::error::PartialOutcome`): `account check` with a failed backend, and a listing or a CardDAV `report query` or `report multiget` whose `errors` is not empty. The binary SHALL parse its command line through `pimalaya_cli::clap::parse`: help and version print as clap prints them; a refused command line with a `--json` word before any `--` SHALL print on stdout the failure object coded `usage-invalid`, its `error` clap's message without the `error: ` prefix nor styles, and without `--json` clap's rendering on stderr.

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
| `card-not-found` | the card a command names is not in the address book: the failure of `card read`, `card update` or `card delete`; an `errors` entry of `carddav report multiget` |
| `addressbook-not-found` | the address book a command names does not exist; nothing was read or written |
| `permission-denied` | the server refused the request to the account (see "A server's refusal carries its code") |
| `quota-exceeded` | the write would exceed the account's quota |
| `rate-limited` | the server throttled the request; cardamum retries nothing |
| `card-unreadable` | the backend answered for a card cardamum could not read: its listing, `report query` or `report multiget` row came without its data (CardDAV), its file could not be read (vdir), its body did not parse as a vCard or is not UTF-8, it did not project (JMAP), its body could not be read (pimdir); an `errors` entry |
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

### Requirement: A server's refusal carries its code
(backends) A failure a server refused SHALL carry a code of the vocabulary Himalaya shares, read from the typed error of the protocol that answered it, in the failure object, an `errors` entry and an `account check` row alike. One reader per backend codes its typed errors, walking the error chain after any coded error, so the typed errors stay in the chain the backends return; the failure object is coded once the command failed, its messages and causes printing as before.

| Code | When |
|---|---|
| `permission-denied` | an HTTP 403 that is no rate limit (Graph, People); a CardDAV 403 whose body names no precondition, or `need-privileges` (RFC 3744 §7.1.1); a CardDAV listing, `report query` or `report multiget` row, or its refused `address-data`, answered 403; a JMAP `forbidden` |
| `quota-exceeded` | an HTTP 507; a CardDAV body naming `quota-not-exceeded` (RFC 4331 §6); Graph's `ErrorQuotaExceeded`; a JMAP `overQuota` |
| `rate-limited` | an HTTP 429, or a People 403 naming a rate limit or a quota, or a JMAP `rateLimit`; cardamum retries nothing, so the first one met |
| `card-not-found` | a card request answered 404 or 410 (CardDAV, People), a `carddav report multiget` row included, 404 or 400 `ErrorInvalidIdMalformed` (Graph), `notFound` (JMAP ContactCard); a vdir card file or a pimdir card that does not exist, or a pimdir card id that is no number |
| `addressbook-not-found` | an address book named that does not exist (vdir, pimdir); a JMAP AddressBook `notFound` |

A refusal none of these name SHALL carry no code, its reason kept in `sources`. A CardDAV 403 naming another precondition (`valid-address-data`, `no-uid-conflict`) is a card the server would not take, no matter of rights, and SHALL carry none; a refused `valid-address-data` keeps the server's answer as the cause of cardamum's hint.

#### Scenario: A read-only address book
- GIVEN a Graph contact folder the account may read but not write
- WHEN `cardamum --json card delete <id>` runs
- THEN it exits 1 printing a failure object coded `permission-denied`

## REMOVED Requirements
