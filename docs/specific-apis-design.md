# Protocol-specific API design

How cardamum's per-backend (protocol-specific) subcommands are structured.
Companion to the shared `addressbook` / `card` API: where the shared commands
are cross-protocol and normalized, the specific commands **match the remote
protocol's own structure and vocabulary**, following Himalaya's direction.

## Principle

Himalaya exposes each backend's raw API *"matching the name of its protocol
counterpart, exposed like the protocol itself."* The structure is chosen by the
protocol's own shape:

| Protocol shape | CLI structure | Himalaya reference |
| --- | --- | --- |
| Flat command set (verbs) | flat list, command = protocol verb | `imap`: `select create delete status expunge search sort fetch append copy move raw` |
| Object / REST (resources) | nested by object type; sub-verbs = protocol methods | `jmap`: `mailbox/email/…` × `get query create update destroy`; `msgraph`: `mail-folder/message/…` × `list get create update delete move copy` |
| Filesystem | collection verbs flat + item subcommand | `maildir`: `create rename delete list` + `messages` + `flags` |

Mapping cardamum's five contacts backends onto that rule:

- **CardDAV** (WebDAV) → flat method list (like IMAP).
- **JMAP / msgraph / people** (object/REST) → nested by resource (like Himalaya jmap/msgraph).
- **vdir** (filesystem) → collection verbs flat + item subcommand (like maildir).

The specific API's reason to exist is to expose what the shared API hides:
ETags/CTags, sync-tokens, `changes`/`delta`, multiget, discovery, precondition
headers, protocol-native ids, and raw native payloads.

## Settled decisions

- **A — CardDAV = flat WebDAV-method list** (not resource-nested).
- **B — jmap is in scope** (the only remaining backend without a specific API).
- **C — raw-faithful passthrough**: friendly tables for default human output,
  `--json` emits the raw native payload, create/update accept the raw native
  representation, and each backend gets a raw escape hatch (CardDAV: raw
  PROPFIND/REPORT/PROPPATCH XML body; object backends: raw JSON in/out, plus an
  optional generic request passthrough).

## Conventions (all backends)

- Verb names track the **protocol**, not the shared API: JMAP uses `destroy`
  (alias `rm`); Graph/People use `delete`; CardDAV uses WebDAV method names.
- Surface the **native ids and metadata** the shared layer normalizes away:
  ETags, CTags, sync-tokens, hrefs, Graph `changeKey`, People
  `etag`/`resourceName`.
- `--json` prints the **raw native payload** (vCard bytes / JSContact / Graph /
  People JSON); default human output is a friendly table.
- **Incremental-sync primitives are the headline** of each specific API:
  `report sync` / `changes` / `delta` / `--sync-token`.
- **Precondition control** where the protocol has it (CardDAV
  `--if-match`/`--if-none-match`; JMAP/Graph/People/vdir do not).
- Each command's `--help` names its protocol counterpart (RFC / API operation),
  like Himalaya's imap commands.

---

## vdir (filesystem)

Collection verbs flat + an `item` subcommand (mirrors maildir's `messages`).
Current commands (`create rename delete list`) cover only collections.

| Op | Command | io-vdir | Status |
| --- | --- | --- | --- |
| create collection | `create <name> [-d -C]` | `create_collection` | ✅ exists |
| rename collection | `rename <src> <target>` | `rename_collection` | ✅ exists |
| delete collection | `delete <name>` | `delete_collection` | ✅ exists |
| list collections | `list` | `list_collections` | ✅ exists |
| list items (raw, **any kind**) | `item list <collection>` | `list_items` | ✅ done |
| get item (byte-faithful) | `item get <collection> <id>` | `get_item` | ✅ done |
| create item (`--kind`, else sniffed) | `item create <collection> <input>` | `store_item` | ✅ done |
| update item (kind preserved) | `item update <collection> <id> <input>` | `store_item` | ✅ done |
| delete item | `item delete <collection> <id>` | `delete_item` | ✅ done |
| copy / move item across collections | `item {copy,move}` | `copy_item` / `move_item` | deferred (available) |

**Value-add over shared `card`:** items expose the raw filename id, byte size,
and **kind** (`vcard` *or* `icalendar` — shared `card` filters to vCard only).

**Polish (done):** `rename`/`delete` of a missing collection previously surfaced
a raw OS error (V4); now `VdirClient::collection_path` validates and bails
"Collection `<name>` not found" (also used by every item command).

**Landed 2026-07-18 (iteration 1):** `vdir item {list,get,create,update,delete}`
(+ `items` / `ls` / `new` / `rm` aliases) and the V4 fix. `item copy`/`item move`
deferred to a follow-up.

---

## CardDAV (WebDAV) — flat method list

Restructure the current (`discover propfind report list create delete`) into a
faithful flat WebDAV surface. Each command = one WebDAV/CardDAV method.

| WebDAV / CardDAV op | Command | io-webdav | Status |
| --- | --- | --- | --- |
| current-user-principal + home-set (5397/6764) | `discover` | `current_user_principal`, `addressbook_home_set` | ✅ done |
| PROPFIND (books + CTag/sync-token, or card id+ETag) | `propfind [addressbook]` | `list_addressbooks`, `enum_cards` | ✅ done |
| PROPPATCH (displayname/description/color) | `proppatch` | `update_addressbook` | ✅ done |
| extended MKCOL (5689) | `mkcol` | `create_addressbook` | ✅ done (alias `create`) |
| REPORT `addressbook-query` | `report query` | `list_cards` | ✅ done |
| REPORT `addressbook-multiget` | `report multiget` | `multiget_cards` | ✅ done |
| REPORT `sync-collection` (6578 sync-token) | `report sync` | `sync_cards` | ✅ done (**headline**) |
| GET card (raw vCard + ETag) | `get` | `read_card` | ✅ done |
| PUT card (`--if-match` / `--if-none-match`) | `put` | `create_card` / `update_card` | ✅ done |
| DELETE (collection or card) | `delete` | `delete_addressbook` / `delete_card` | ✅ done |

**Landed 2026-07-18 (iteration 2):** the full flat surface above, all backed by
existing io-webdav typed methods (no lib change). Old `list` / `create` / the
card-scoped `propfind` / flat `report` were replaced.

**Deferred:** the raw-XML escape hatch (`propfind`/`proppatch`/`report --xml`,
decision C) — feasible via io-webdav's `pub stream` + `auth()`, but its own
focused sub-step; the semantic commands cover the real surface. Also **C1**:
`carddav discover` errors on a `home`-configured account (principal never
walked) — pre-existing, fix best-effort in a follow-up.

---

## msgraph (Microsoft Graph) — nested by resource

Mirrors Himalaya's `msgraph` (profile / mail-folder / message) on the contacts
side. Grounded in `io_msgraph::v1::client`.

| Graph resource | Command | io-msgraph |
| --- | --- | --- |
| **contact-folder** (aliases `folder(s)`) | `contact-folder {list, child-folders, get, create, rename, delete}` | `contact_folders_list`, `contact_child_folders_list`, `contact_folder_get/create/update/delete` | ✅ done |
| **contact** (aliases `contacts`) | `contact {list, get, create, update, delete, delta}` | `contacts_list`, `contact_get/create/update/delete`, `contacts_delta` | ✅ done |
| **profile** (`me`) | `profile get` | `me` | ✅ done |

**Landed 2026-07-18 (iteration 3):** the full surface above; `create`/`update`
take raw Graph contact JSON (file / inline / stdin), `--json` emits raw Graph
payloads, list/delta are single-page (`--top`, surfaced `@odata.nextLink` /
`@odata.deltaLink`). **Native-only surface:** `contact delta` (incremental sync).
**Deferred:** the optional generic `msgraph request <method> <path>` passthrough.
Lesson: a positional arg must not be field-named after a global flag (`json`
collides with `--json`).

---

## people (Google People API) — nested by resource

Grounded in `io_people::v1::client`.

| People resource | Command | io-people |
| --- | --- | --- |
| **contact-group** (aliases `group(s)`) | `contact-group {list, get, create, update, delete, members}` | `contact_groups_list`, `contact_group_get/create/update/delete`, `contact_group_members_modify` |
| **connection** (aliases `people/contacts`) | `connection {list, get, create, update, delete, search}` | `connections_list` (syncToken), `person_get`, `contact_create/update/delete`, `contacts_search` |
| **other-contact** (aliases `other`) | `other-contact {list, search, copy}` | `other_contacts_list/search`, `other_contact_copy` |
| **profile** (`me`) | `profile get` | `person_get` on `people/me` |

**Native-only surface (absent from shared):** `connection list --sync-token`
(incremental), the entire `other-contact` source (auto-collected contacts), and
`contact-group members` (add/remove). **Raw (C):** raw People JSON in/out;
optional generic `people request` passthrough (stretch).

---

## jmap (RFC 8620 + 9610) — nested by object

Object protocol → nested by JMAP object type, sub-verbs = JMAP methods.
Grounded in `io_jmap::client` (rfc9610).

| JMAP object / method | Command | io-jmap |
| --- | --- | --- |
| AddressBook/get | `address-book get` | `address_book_get` |
| AddressBook/set (create/update/destroy) | `address-book {create, update, destroy}` | `address_book_set` |
| AddressBook/changes | `address-book changes` | `address_book_changes` |
| ContactCard/get | `contact-card get` | `contact_card_get` |
| ContactCard/query | `contact-card query` | `contact_card_query` |
| ContactCard/set (create/update/destroy) | `contact-card {create, update, destroy}` | `contact_card_set` |
| ContactCard/changes | `contact-card changes` | `contact_card_changes` |
| ContactCard/copy | `contact-card copy` | `contact_card_copy` |
| Session | `session get` | `session_get` |

(AddressBook has no `/query` in RFC 9610 / io-jmap — omitted deliberately.)
**Native-only surface:** `changes` (both objects), `session get` (capabilities),
`contact-card copy` (cross-account). **Raw (C):** create/update accept raw
JSContact JSON; `--json` emits raw JSContact; optional generic `jmap request`
(a raw method-call array POST) passthrough (stretch).

---

## Iteration order (implement/adjust + test, one at a time)

1. **vdir** — add the `item` subcommand (smallest; proves the pattern on a local backend).
2. **carddav** — restructure to the flat method list + add sync/multiget/proppatch/get/put (highest value; io-webdav already backs it).
3. **msgraph** — new, from the Himalaya template.
4. **people** — new.
5. **jmap** — new (io-jmap already exposes every method).

Each step: implement/adjust, run the shared suite for regressions, then test the
new specific commands live (throwaway data, golden rule) and add a
`docs/testing/<backend>-*.md` update.
