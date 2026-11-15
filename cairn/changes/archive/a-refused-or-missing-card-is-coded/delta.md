---
cairn: delta
change: a-refused-or-missing-card-is-coded
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: A server's refusal carries its code
(backends) A failure a server refused SHALL carry a code of the vocabulary Himalaya shares, read from the typed error of the protocol that answered it, in the failure object, an `errors` entry and an `account check` row alike. One reader per backend codes its typed errors, walking the error chain after any coded error, so the typed errors stay in the chain the backends return; the failure object is coded once the command failed, its messages and causes printing as before.

| Code | When |
|---|---|
| `permission-denied` | an HTTP 403 that is no rate limit (Graph, People); a CardDAV 403 whose body names no precondition nor the vCard, or `need-privileges` (RFC 3744 §7.1.1); a CardDAV listing, `report query` or `report multiget` row, or its refused `address-data`, answered 403; a JMAP `forbidden` |
| `quota-exceeded` | an HTTP 507; a CardDAV body naming `quota-not-exceeded` (RFC 4331 §6); Graph's `ErrorQuotaExceeded`; a JMAP `overQuota` |
| `rate-limited` | an HTTP 429, or a People 403 naming a rate limit or a quota, or a JMAP `rateLimit`; cardamum retries nothing, so the first one met |
| `card-not-found` | a card request answered 404 or 410 (CardDAV, People), a `carddav report multiget` row included, 404 or 400 `ErrorInvalidIdMalformed` (Graph), `notFound` (JMAP ContactCard); a vdir card file or a pimdir card that does not exist, or a pimdir card id that is no number; on the protocol commands naming one card as on the unified ones, the server's message kept there |
| `addressbook-not-found` | an address book named that does not exist (vdir, pimdir); a JMAP AddressBook `notFound`; a CardDAV collection answered 404 or 410, a Graph folder 404 or a malformed folder id, a People group 404, an id JMAP's `AddressBook/get` does not hold ("A network backend names an unknown address book") |
| `card-invalid` | a CardDAV write answered 400 or 403 whose body names the `valid-address-data` or `supported-address-data` precondition (RFC 6352 §6.3.2.1) or the vCard itself (`VCARD parse error`), the predicate cardamum's hint reads, so the two never disagree |
| `precondition-failed` | a CardDAV write answered 412, the `If-Match` it sent naming a version the card no longer has |
| `item-duplicate` | a CardDAV write refused for the address book's `no-uid-conflict` precondition (RFC 6352 §6.3.2.1), as calendula codes the CalDAV one |

A refusal none of these name SHALL carry no code, its reason kept in `sources`. A CardDAV 403 naming another precondition is a card the server would not take, no matter of rights, and SHALL NOT read `permission-denied`: `no-uid-conflict` is `item-duplicate`, `valid-address-data` `card-invalid`, keeping the server's answer as the cause of cardamum's hint, and any other carries none.

#### Scenario: A vCard the server refuses
- GIVEN a CardDAV server answering a `PUT` 403 with the plain body `VCARD parse error`
- WHEN `cardamum --json card add -k card c.vcf` runs
- THEN it exits 1 coded `card-invalid`, its error the hint naming `UID` and `N`, the server's answer in `sources`

#### Scenario: A read-only address book
- GIVEN a Graph contact folder the account may read but not write
- WHEN `cardamum --json card delete <id>` runs
- THEN it exits 1 printing a failure object coded `permission-denied`

### Requirement: A JSON error carries a stable code
(commands) A failure a caller is expected to act on SHALL carry a stable code, printed under `--json` as a `code` string beside `error`, `sources` and `backtrace`, and in an `errors` entry beside `id`, `error` and `sources`. The code SHALL be found anywhere in the error chain, so added context does not hide it, or read from the typed error of the protocol that refused ("A server's refusal carries its code"). A failure with no code SHALL print `"code": null`. The plain output SHALL stay unchanged. Codes are kebab-case and never renamed; `error` stays free wording. A pimdir store's code is the one io-pimdir names (`PimdirError::code()`), never read off its message. Every code SHALL be declared in every build, whatever the backends compiled in.

| Code | Raised when |
|---|---|
| `usage-invalid` | the command line, a REPL line or an MCP tool call's arguments do not match the grammar (pimalaya-cli's `CommonCode::UsageInvalid`); the command line exits 2 |
| `account-missing` | no account has the name `-a` gives, `default` included (pimalaya-config's `AccountError::Missing`) |
| `account-default-missing` | no `-a` names an account and none is marked `default = true` (`AccountError::NoDefault`) |
| `account-default-several` | no `-a` names an account and several are marked `default = true` (`AccountError::SeveralDefaults`) |
| `backend-not-built` | the backend the account resolves to is configured but left out of the build; the error names the cargo feature (see backends.md, "A backend left out of the build is named") |
| `stdin-taken` | a command inside a session (`repl`, `mcp`) would read its input off stdin, which carries the session's commands |
| `capability-missing` | the backend the account resolves to does not serve a capability the command reaches; nothing was read or written |
| `card-not-found` | the card a command names is not in the address book: the failure of `card read`, `card update` or `card delete`, worded by cardamum ("A not-found failure is cardamum's sentence"); an `errors` entry of `carddav report multiget`; a protocol command naming one card (`carddav get` and `delete`, `msgraph contacts get`, `update` and `delete`, `gpeople connections get`, `update` and `delete`), the server's message kept |
| `addressbook-not-found` | the address book a command names does not exist; nothing was read or written |
| `addressbook-default-missing` | a command needing an address book got no `-k` and the account sets no `addressbook.default`; nothing was read or written |
| `permission-denied` | the server refused the request to the account (see "A server's refusal carries its code") |
| `quota-exceeded` | the write would exceed the account's quota |
| `rate-limited` | the server throttled the request; cardamum retries nothing |
| `card-unreadable` | the backend answered for a card cardamum could not read: its listing, `report query` or `report multiget` row came without its data (CardDAV), its file could not be read (vdir), its body did not parse as a vCard or is not UTF-8, it did not project (JMAP), its body could not be read or is gone from the blob tree (pimdir); an `errors` entry |
| `card-invalid` | the server refused the card written as no vCard it takes (CardDAV, "A server's refusal carries its code"); nothing was written, and the card is what to fix, not the account's rights |
| `body-pending` | a listed card's body is not local yet (pimdir); a sync brings it |
| `precondition-failed` | a write's `--if-match` names a version the card no longer has: pimdir's check, a CardDAV server answering 412, or the Graph `changeKey` or JMAP digest read before the write |
| `store-uncreated` | the pimdir root holds no store; its sync engine creates it |
| `store-version` | the pimdir store is of another format version than this cardamum reads; an earlier neverest's store is reinstalled through neverest, its queue drained first, a later one's asks for a cardamum upgrade |
| `store-foreign` | the pimdir root holds a database another program wrote |
| `store-corrupt` | the pimdir store's two version stamps disagree, or its schema differs from the one its version declares |
| `store-not-current` | the pimdir store holds no schema yet and the verb creates none; its sync engine creates it |
| `store-hash-algo` | the pimdir store names its bodies by a hash this cardamum does not know |
| `sync-version` | the pimdir store's sync state was written by another version of its sync engine |
| `store-exposed` | another user can read or write the pimdir store, or a link stands where a lock file belongs; restrict it |
| `store-busy` | another writer held the pimdir store, past the wait on a write and at once on `account check`, which does not wait; retry |
| `store-owned` | a sync owns the pimdir store; retry once it ends |
| `store-staging` | a producer is staging a body in the pimdir store; retry |
| `store-in-flight` | a pimdir store verb is between two of its chunks; retry |
| `store-reset` | the pimdir store is not the one that answered a queue row's handle, a copy restored since; nothing was done with the row |
| `lock-replaced` | the pimdir store's staging lock file was replaced while a write held it; nothing was queued, retry once no other process sets the store up |
| `unsupported` | a source syncing the address book does not support the write (pimdir STORAGE §15.6); nothing was queued |
| `item-duplicate` | a create names a `UID` the address book holds: on pimdir synced or queued, nothing queued; on CardDAV the server's `no-uid-conflict` (RFC 6352 §6.3.2.1) |
| `flag-invalid` | the pimdir store cannot hold a flag; nothing was staged |
| `action-malformed` | a queued pimdir action's payload is malformed (`pimdir status`); the sync engine's drain parks it |
| `conflict-unread` | a pimdir conflict holds a body nobody has read; fetch it first |
| `no-performer` | no source of the pimdir store can perform the intent |
| `ambiguous-performer` | several sources of the pimdir store can perform the intent and none is chosen |
| `index-missing` | the pimdir store has no search index yet (`pimdir search`, `pimdir count`); its sync engine builds it |
| `index-outdated` | the pimdir search index was built otherwise or from another store; its sync engine rebuilds it |
| `index-version` | the pimdir search index is of another draft or shape; its sync engine rebuilds one of an earlier draft and leaves any other |
| `index-zone-unknown` | the zone the pimdir search index reads floating times in is not an IANA zone |
| `index-fts5-missing` | the SQLite cardamum links lacks FTS5 or predates 3.43, which the pimdir search index needs |
| `query-refused` | the pimdir search query is malformed or names an unknown field or value (SEARCH §8.4); nothing was searched |
| `query-unanswerable` | the pimdir search index could not answer an accepted query, an implementation fault |
| `tag-refused` | a pimdir tag names no flag, or adds and removes one flag; nothing was staged |

#### Scenario: A stale version
- GIVEN a pimdir card whose body changed since it was read
- WHEN `cardamum --json card delete --if-match <old etag>` names it
- THEN the command exits 1, queues nothing and prints `{"code":"precondition-failed","error":"Precondition failed: …","sources":[],"backtrace":null}`

#### Scenario: An unknown card
- GIVEN a vdir address book holding no card `nope`
- WHEN `cardamum --json card read nope` runs
- THEN the command exits 1 and prints a failure object coded `card-not-found`, its `error` ``Card `nope` not found`` and io-vdir's text in `sources`

## REMOVED Requirements

None.
