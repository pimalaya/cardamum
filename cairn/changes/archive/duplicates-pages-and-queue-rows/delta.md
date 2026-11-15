---
cairn: delta
change: duplicates-pages-and-queue-rows
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: A collection created twice is refused
(backends) `addressbook create` naming an address book the backend holds already SHALL fail coded `addressbook-duplicate`, ``Addressbook `<name>` already exists``, exit 1, creating nothing and leaving the existing one as it was, on every backend that can tell, as pimalaya-cli's error.md asks of every collection noun. vdir: a directory of that name under the home, checked before anything is written, `vdir create` refused the same way, where io-vdir would create over it and rewrite its metadata. CardDAV: a `MKCOL` refused 405, the status RFC 4918 §9.3.1 gives a URL already mapped and RFC 5689 §3.2 keeps for the extended `MKCOL`, or 403, which Fastmail answers (testing/carddav-specific.md), on an id the home set then lists, one `PROPFIND` of the home set on that path alone; any other refusal, or a listing that fails or does not hold the id (iCloud refuses every `MKCOL`), keeps the server's. Microsoft Graph: a folder refused `ErrorFolderExists`. Google People: a contact group refused 409, which contactGroups.create answers a name the user holds. The server's answer SHALL be in `sources`. JMAP lets two AddressBooks share a name (RFC 9610 §2 asks no uniqueness), so it holds no duplicate to refuse, and pimdir creates no address book (`capability-missing`, "A cache renames nothing it cannot push"). The protocol commands creating a collection on a server (`carddav mkcol`, `msgraph contact-folders create`, `gpeople contact-groups create`, `jmap address-book create`) keep the server's refusal as it is.

#### Scenario: An address book created twice
- GIVEN a vdir home holding the address book `work`, named `Work` and described
- WHEN `cardamum --json addressbook create work` or `cardamum --json vdir create work -n Other` runs
- THEN each exits 1 coded `addressbook-duplicate`, ``Addressbook `work` already exists``, and `work` keeps its name and description

### Requirement: Protocol listings page by nextPage
(commands) A protocol listing that can stop before the end SHALL print `nextPage`, the provider's cursor as a string, `null` once the provider gives none, and SHALL take it back through `--page-token` (pimalaya-cli's `PageTokenArg`), as pimalaya-cli's output.md "Paging resumes" asks. `gpeople connections list`, `gpeople other-contacts list` and `gpeople contact-groups list` print People's `nextPageToken`, sent back as the `pageToken`, the other flags given again. `msgraph contacts list`, `msgraph contact-folders list`, `msgraph contact-folders child-folders` and `msgraph contacts delta` print Graph's `@odata.nextLink`, which `--page-token` gets as it is: the link carries the folder and the query, so the flags shaping them conflict with it and `child-folders` then takes no folder id, and a link off the Graph origin is refused before the token leaves (io-msgraph). `jmap contact-card query` prints the position the next page starts at, which `--page-token` sends as the query's `position`, the other flags given again: `null` once that position reaches the query's `total`, or, the server giving none, after a page shorter than `--limit`, and a token that is no integer from 0 fails ``Invalid page token <token>``.

The provider's cursor key SHALL NOT print beside `nextPage`; the rest of the payload stays raw, People's `nextSyncToken` and the `@odata.deltaLink` ending a Graph delta round included, the latter still given back through `--delta-link`. The output is pimalaya-cli's `output::Paginated` around the listing: in text the listing, then a `Next page: <cursor>` line, one blank line apart when the listing printed anything. A search the provider answers with no cursor (`gpeople connections search`, `gpeople other-contacts search`) prints none.

#### Scenario: A Graph listing resumed
- GIVEN a Graph contact folder holding more contacts than `--top 2`
- WHEN `cardamum --json msgraph contacts list --top 2` runs, then `cardamum --json msgraph contacts list --page-token <nextPage>`
- THEN the first prints `{"contacts":[…],"nextPage":"https://graph.microsoft.com/v1.0/…"}` with no `@odata.nextLink`, and the second gets that link and prints the next page, `nextPage` `null` on the last one

## MODIFIED Requirements

### Requirement: A server's refusal carries its code
(backends) One row joins its table, after `addressbook-not-found`, the rest as it stands:

| `addressbook-duplicate` | `addressbook create` refused on an address book the backend holds ("A collection created twice is refused"): a CardDAV `MKCOL` answered 403 or 405 on an id the home set lists, Graph's `ErrorFolderExists`, a People contactGroups.create answered 409 |

### Requirement: The pimdir queue is read and retracted
(commands) `pimdir queue list`, `pimdir queue show <ROW>` and `pimdir queue cancel <ROW>`, gated on the `pimdir` feature, SHALL read and retract the account's queue as Himalaya's `pimdir queue` does. `list` SHALL print the card creations staged in the account's address books, or in the one `-k` names (`addressbook-not-found` for one the account does not hold), then every row of any kind the owner parked there, each as the family's queue row (pimalaya-cli's output.md, "A queue row keeps one shape") plus `fullName`: `{"cards": [<row>], "parked": [<row>]}`, a row `{"row", "kind", "collection", "linkId", "itemId", "producer", "queuedAt", "attempts", "unconfirmed", "error", "fullName"}`, every key printed, `null` or not. `kind` is the action kind io-pimdir reads from the row, never re-spelled; `linkId` the link id of the card the row creates, the one `card add` reported; `itemId` the id `card list` prints of the card the row addresses, `null` for a creation; `attempts` how many times the sync engine tried the row; `unconfirmed` whether its outcome is unknown; `error` `null` while the row is pending, the store's text once it is parked; `fullName` the full name of the body the row pins, else of the card it addresses, `null` when neither is local. A parked row's card is read from its payload: one that does not decode, and an intent or a `set-performer`, name none. The text prints them under `Account:` and `Addressbook:` with the cancelling command. `show` SHALL print where one row stands, `{"row", "state", "collection", "kind", "attempts", "error", "appliedAt", "seq"}`, `state` one of `pending`, `parked`, `unconfirmed`, `applied`, `unknown` and `reset`, `seq` the card an applied add created, the id `card read` takes; a row anchored on a collection that is not one of the account's address books SHALL read `unknown`. `cancel` SHALL ask for confirmation unless `-y`, refuse a row applied already or unknown to the account before the store is opened for writing, then take the store owner's role for that one write (io-pimdir's `PimdirStore::open_current` and `drop_action`), refused at once coded `store-owned` while a sync holds it, and print `{"row", "kind", "unconfirmed"}`. It is the one write cardamum makes as the store's owner, and the only way back for a queued creation, which has no id a `card delete` could take.

#### Scenario: A queued card is cancelled
- GIVEN a pimdir address book `book` a source syncs, and `cardamum card add -k book` queued a card as row 3
- WHEN `cardamum --json pimdir queue list` runs, then `cardamum --json pimdir queue cancel -y 3`
- THEN the listing holds `{"row":3,"kind":"add","collection":"book","linkId":"<UID>","itemId":null,…,"attempts":0,"unconfirmed":false,"error":null,"fullName":"<FN>"}` under `cards`, the cancel prints `{"row":3,"kind":"add","unconfirmed":false}`, and `pimdir queue show 3` then reads `unknown`

#### Scenario: A parked update
- GIVEN the card `7` of the pimdir address book `book`, whose `card update` a sync parked
- WHEN `cardamum --json pimdir queue list` runs
- THEN its row under `parked` reads `"kind":"update"`, `"linkId":null`, `"itemId":"7"`, the store's text as `error` and the full name of the body the update pins as `fullName`

### Requirement: Specific commands are raw-faithful
(commands) A specific command SHALL surface the native ids and metadata the shared layer normalizes away (ETags, CTags, sync tokens, hrefs, the Graph `changeKey`, the People `etag` and `resourceName`), and SHALL make the incremental-sync primitive its headline (`report sync`, `changes`, `delta`). Default human output is a friendly table; `--json` SHALL emit the raw native payload, the provider's paging cursor alone lifted into `nextPage` ("Protocol listings page by nextPage"), and create and update SHALL accept the raw native representation. Each backend keeps a raw escape hatch: a raw XML body for CardDAV, raw JSON in and out plus a generic request passthrough for the object backends, whose `--json` prints the response verbatim under `response` (`null` when the server sent no body), so the payload is an object whatever the server answered. Precondition control is offered only where the protocol has it (CardDAV `--if-match` and `--if-none-match`).

### Requirement: JSON output keys are camelCase
(commands) Every output type SHALL carry `serde(rename_all = "camelCase")`, so a `--json` key spelling more than one word reads `addressbookId` rather than `addressbook_id`. That is the convention of the wire formats the backends speak, and unlike a hyphenated key it needs no quoting in a `jq` path. The configuration vocabulary is a separate one and stays kebab-case.

A field renamed onto a provider's own wire key SHALL keep that spelling, a container attribute never reaching a field carrying its own `rename`: `@odata.deltaLink`, `nextSyncToken`, `contactGroups` and the JMAP `list` are the provider quoted back to the caller, not ours to restyle. A provider's paging cursor is the one key not quoted back: it prints as `nextPage` ("Protocol listings page by nextPage").

### Requirement: A JSON error carries a stable code
(commands) One row joins its table, after `addressbook-default-missing`, the rest as it stands:

| `addressbook-duplicate` | `addressbook create` or `vdir create` names an address book the backend holds already (backends, "A collection created twice is refused"): a vdir directory, a CardDAV collection the home set lists, a Graph contact folder, a People contact group; nothing was created |

## REMOVED Requirements
