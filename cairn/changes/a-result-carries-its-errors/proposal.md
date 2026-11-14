---
cairn: change
id: a-result-carries-its-errors
status: active
created: 2026-11-14
---

# A result carries its errors

## Why

Himalaya landed "a result carries its errors" on 2026-11-14 (himalaya/cairn/changes/a-result-carries-its-errors): a command listing or acting on several items prints an `errors` list beside its result, one pimalaya-cli item error `{id, code, error, sources}` per item it could not serve, and exits 3 when the list is not empty. The family vocabulary gained `permission-denied`, `quota-exceeded`, `rate-limited`, and Himalaya's `message-not-found`, `message-unreadable` and `mailbox-unreadable`. Neverest followed the same day (neverest/cairn/changes/a-refusal-carries-its-code): a server's refusal is coded by a reader per protocol walking the error chain. pimalaya-cli's log of the same name says calendula and cardamum adopt `errors` "when they print several items that can fail one by one".

Cardamum has two such commands, `addressbook list` and `card list`, and today a card it cannot serve is lost or takes the listing with it:

- **Dropped silently.** A card whose vCard does not parse lists with empty fields, and a non-UTF-8 body with no preview (src/shared/card/project.rs:47-50, src/shared/card/list.rs:237-240). io-webdav drops a CardDAV row the server refused (a per-href 403 or 404) and cardamum ignores the `truncated` flag of a 507 row (src/carddav/backend.rs:105-120), so a cut listing reads complete.
- **The whole listing fails.** A vdir file that cannot be read (src/vdir/backend.rs:101-125, through io-vdir's `list_items`), a JMAP card that does not project (src/jmap/backend.rs:174-197, src/jmap/project.rs:30-43), a pimdir blob that cannot be read (src/pimdir/backend.rs:458-494).
- **No code.** No failure says whether the server refused the account (403), its quota (507) or the rate (429), and a card or address book not found is an uncoded `bail!` on every backend (pimdir src/pimdir/backend.rs:514-517 and :562, vdir src/vdir/backend.rs:204, JMAP src/jmap/backend.rs:211). JMAP SetErrors are flattened to strings (`format_set_error`, src/jmap/backend.rs:113, :146, :162, :236-238, :278-280, :301-306).

The libraries move first: the family moved to pimalaya-stream 0.4 on 2026-11-13, and pimalaya-cli 0be1b1f carries the `ItemError` this change prints.

## What

Four phases, each one commit, in this order. Phases 1 to 3 need nothing outside this repository but Phase 0; Phase 4 waits on io-webdav.

### Phase 0, outside this repository (blocking Phase 1)

io-pim-discovery's change stream-0-4 (io-pim-discovery/cairn/changes/stream-0-4) lands and is pushed: cardamum's CardDAV discovery hands it a `Tls` (src/carddav/client.rs:214, :227), so cardamum cannot take pimalaya-stream 0.4 before it. Its commit hash is the rev Phase 1 pins. If the owner has released io-webdav 0.9, io-gpeople 0.8, io-pim-discovery 0.13 or pimalaya-cli 0.4.1 by then, Phase 1 takes the release instead of the pin.

### Phase 1, the libraries

- Cargo.toml: `pimalaya-stream` 0.4, `io-http` 0.8, `io-jmap` 0.9, `io-msgraph` 0.9; dev `io-oauth` 0.7; `[patch.crates-io]` (no comment, sorted): `io-gpeople` e4a9cac80d023688118dbae88f8535145e8d2c9b (the stream 0.4 bump, rebased over e507d82, so the year-less `BDAY` fix stays), `io-webdav` 185ab63be5859c5ef6115bef97e4df5cb1187973, `io-pim-discovery` at the Phase 0 commit, `pimalaya-cli` 0be1b1f08d19ccb9d16ee8e1d4266c27c96e0cdd (eb37962 plus `ItemError` and the registry spare, additive only). Then `cargo update`, which takes pimalaya-config 0.4.1 (a prompting secret command reads the terminal again).
- src/config.rs:607: `TlsConfig::into_tls` fills `server: self.cert.map(|cert| TlsServer { cert })` and `client: None`. The `tls.cert` key keeps its name and meaning; config.sample.toml needs no edit.
- Expected compile breaks: that one field, and nothing else (io-http 0.8, io-oauth 0.7, io-jmap 0.9, io-msgraph 0.9, io-webdav 185ab63 and io-gpeople e4a9cac change only their dependencies; cardamum already implements pimalaya-config 0.4's `TomlConfig`).
- `cargo tree -d --all-features -e normal` lists no family crate twice (pimalaya-stream, io-http, io-oauth).
- CHANGELOG: the dependency line under Changed follows; a Fixed line for the prompting secret command (pimalaya-config 0.4.1), as neverest worded it.

### Phase 2, a refusal and a missing item carry their code

- src/error.rs gains, declared in every build (a `cfg_attr(..., allow(dead_code))` on the enum as neverest does, so the `cardamum-error` schema does not change with the features; the existing pimdir codes keep their gates unless the agent finds ungating them simpler and schema-neutral):
  - `permission-denied`, `quota-exceeded`, `rate-limited`, spelled as Himalaya's;
  - `card-not-found`: the card id a command names is not in the address book; the failure of `card read|update|delete`, or an `errors` entry;
  - `addressbook-not-found`: the address book named does not exist;
  - `card-unreadable`: the backend answered for a card cardamum could not read; an `errors` entry, or the failure of `card read`;
  - `addressbook-unreadable`: an address book listed whose properties could not be read. Kept only if Phase 3 finds a backend meeting it (a vdir address book whose metadata file cannot be read is the candidate, unverified); otherwise left out of this change.
- One reader per protocol beside its client, `pub(crate) fn refusal_code(cause: &(dyn std::error::Error + 'static)) -> Option<ErrorCode>`, and in src/error.rs the chain walk `error::code(err)` = `code_of(err)` else the first cause a reader codes (neverest/src/error.rs, `refusal_code`). Readers, each mirroring neverest's (file:line in the tasks):
  - CardDAV (io-webdav `WebdavClientStdError`): a 507 or a body naming `quota-not-exceeded` (RFC 4331 §6) `quota-exceeded`; 429 `rate-limited`; 403 `permission-denied` when its body names `need-privileges` (RFC 3744 §7.1.1) or no precondition; a 404 or 410 on a card request `card-not-found`. src/carddav/backend.rs:215-232 rewrites the `valid-address-data` refusal with `anyhow!`, losing the typed error: it keeps it as a cause (`context`) instead.
  - Graph (`MsgraphClientStdError::Send(send)`, `send.api()`): `ErrorQuotaExceeded` or 507 `quota-exceeded`, 429 `rate-limited`, 403 `permission-denied`; 404, or 400 with `ErrorInvalidIdMalformed`, `card-not-found` on a contact request.
  - People (`GpeopleClientStdError::Send(GpeopleSendError::Api(..))`): neverest's `google_code` (neverest/src/throttle.rs) copied into cardamum with the parts of `google_refusal` it reads: 429, or a 403 naming a rate limit or a quota, `rate-limited`; any other 403 `permission-denied`; 404 `card-not-found` on a person request.
  - JMAP: the SetErrors are coded where they are built, `format_set_error` turning into a `CodedError` (they are no `std::error::Error` in a chain): `forbidden` `permission-denied`, `notFound` `card-not-found`, `overQuota` (an `Other` type today) `quota-exceeded`, `rateLimit` `rate-limited`, any other uncoded with its type in the message. `get_card` not found (src/jmap/backend.rs:211) is `card-not-found`.
  - vdir: a card file that does not exist is `card-not-found` (io-vdir's variant to be read from io-vdir 0.3's `VdirClientError`), an address book directory that does not exist `addressbook-not-found` (src/vdir/backend.rs:204).
  - pimdir: src/pimdir/backend.rs:562 `card-not-found`, :514-517 `addressbook-not-found`.
- `account check` rows read their `code` through `error::code` (src/account/check.rs:289), so a refused credential's 403 is coded as other failures are.
- No retry is added: cardamum retries nothing, so `rate-limited` codes the first 429 a request met. The code table says so.

### Phase 3, a listing names what it could not read

- `addressbook list` and `card list` print an `errors` list beside their items, under `--json` and in text: one `pimalaya_cli::error::ItemError<ErrorCode>` (the alias `ItemError` in src/error.rs, as himalaya/src/error.rs declares it) per card or address book the command could not read, `[]` when there is none, in the order met. `id` is the card's id as `card list` prints it, `null` when the backend named none. A non-empty `errors` makes the command return `PartialOutcome::default()` once printed: exit 3.
- What becomes an entry, per backend:
  - every backend: a card whose body does not parse as a vCard, or is not UTF-8, is listed as today (its id, the rest empty) and gains a `card-unreadable` entry naming its id and the parse error;
  - vdir: `list_items` is replaced by io-vdir 0.3's `list_readable_items`, each file it could not read a `card-unreadable` entry with the card's id (the file stem, as cardamum names vdir cards) and the `io::Error` as its source; a card removed while it lists stays left out, no entry;
  - JMAP: a card that does not project (src/jmap/project.rs:30-43, today a `collect::<Result<_>>`) is a `card-unreadable` entry with its id when it has one, the other cards listed; a card io-jmap could not deserialize still fails the call (io-jmap's ContactCard has no `unparsed` yet, see Scope);
  - pimdir: a blob read error is a `card-unreadable` entry, the card listed with its preview; a blob missing from the store (not hydrated yet) stays what it is today, a normal state, not an entry;
  - Graph and People: a page that fails fails the listing whole, as Himalaya's Graph listing does; nothing per card can fail after deserialization (`to_vcard` is infallible).
- Paging: every backend but pimdir reads the whole address book and cuts the page in the client (src/shared/client.rs, `window`), so its entries print on the first page only, as Himalaya's Maildir listing does (else every page would repeat them). pimdir pages in the store: an entry prints on the page that held its card.
- Text: a list prints its paging in its header, and a command's errors print at the end of its output (owner decisions 2026-11-14: no list footer; the errors are the command's own footer, not the list's). `card list` moves its paging fields (`Page:`, `Next page:`, printed below the cards today, src/shared/card/list.rs:144-230) into its header after `Addressbook:`. Each entry prints as an `Error:` field at the very end, after the items and one blank line, `<id>  <code>  <error>` with `-` for a null id or code (the item error's `Display`), the `Error:` lines aligned among themselves; `addressbook list` (a bare table, src/shared/addressbook/list.rs:133-153) the same after its table. Nothing prints, the blank line included, when `errors` is empty. A later change moves both lists onto pimalaya-cli's shared listing module.
- Schema: `json-schema` registers the item error under `cardamum-item-error`; the registry test `every_registered_key_names_a_command` (src/json_schema.rs:354) knows the key beside `cardamum-error`, and `every_command_is_described` keeps passing through pimalaya-cli 0be1b1f's spare. `errors` is a required key of both outputs.
- MCP: `card_list` and `addressbook_list` return the result with its `errors` as `structuredContent` beside `isError: true` on exit 3, which `ToolResult::printed` already does (src/mcp.rs:260). The server instructions (src/mcp.rs:44-56) say a listing may carry `errors`.
- REPL: a session keeps its client after a partial outcome (check `Session`'s handling of an `Err` that is a `PartialOutcome`; calendula's drops the client on any `Err`, src/session.rs:44-66 there).

### Phase 4, after io-webdav (not part of the parallel run)

io-webdav's change a-listing-returns-what-it-could-not-read (io-webdav/cairn/changes/a-listing-returns-what-it-could-not-read) makes `CarddavCardList` return the rows it could not read and keeps `truncated`. Once it is pushed and pinned: a CardDAV row refused per href is a `card-unreadable` entry (403 `permission-denied`, coded from its row status), and a listing the server truncated (a 507 row, RFC 6578 §3.6 wording) is an entry with a `null` id and no code saying the listing is incomplete. Until then the CardDAV listing keeps today's behaviour, which the spec states.

### Docs

CHANGELOG Unreleased (Keep a Changelog, `**Breaking:**` as cardamum writes it): Added the codes and `errors`; **Breaking:** changed `addressbook list` and `card list` to exit 3 with an `errors` list when a card could not be read, and vdir and JMAP listings to list the other cards where they failed whole; Changed the not-found failures to carry `card-not-found` and `addressbook-not-found`, and `card list` to print its paging above the cards. MIGRATION.md, section "From v0.3 to the next release", "JSON output": `errors` and exit 3. README: an exit status FAQ line if it has none (Himalaya added one). Doc comments of the two outputs (src/shared/card/list.rs:26-33, src/shared/addressbook/list.rs:77-78) list `errors`. src/error.rs's header (:7-8, "no `code` field") is stale: the spec prints `"code": null`.

## Decisions

- **Same id as Himalaya's and pimalaya-cli's change**, so the family's history reads across repositories.
- **Listings only.** No unified cardamum command takes several ids (`card delete|read|update` take one), so a write keeps failing whole, coded. The protocol namespaces (`carddav`, `jmap`, `msgraph`, `gpeople`, `pimdir`, `vdir`) keep printing what their protocol answered, as in Himalaya: `jmap contact-card copy` and `gpeople contact-group members` stay whole failures after acting.
- **Item codes named by cardamum's nouns**: `card-*` and `addressbook-*`, as Himalaya names its `message-*` and `mailbox-*`. The shared refusal codes keep Himalaya's spelling.
- **A body that does not parse is listed and named.** As Himalaya lists a message decoded in part beside its `message-unreadable` entry, a card listed with its id but no fields gets a `card-unreadable` entry instead of passing silently.
- **A missing pimdir blob is no error.** A store lacks bodies until its sync engine hydrates them; an entry would make every listing of a young store exit 3.
- **Paging in the header, errors at the end**: the owner decided on 2026-11-14 that a list prints no footer, so the paging moves above the cards; a command's errors are no part of the list but the end of the command's output, so they print last.
- **No retries**, so `rate-limited` is the first 429 met; adding retries is another change.
- These decisions are proposals: the owner reviews this change before any phase starts, per AGENTS.md.

## Scope / non-goals

- io-jmap's ContactCard deserialization failure keeps failing the call: making it an entry needs io-jmap to keep what a get misses for cards (io-jmap's a-get-keeps-what-it-misses did it for emails). Left for an io-jmap change.
- `gpeople`'s unified `create_card` creating the card then bailing when the group add misses (src/gpeople/backend.rs:268-283) is a partial write of one item, out of scope.
- The active change card-search (cairn/changes/card-search) adds `card search` over several address books, a listing too: whichever lands second gives `card search` the `errors` list and adopts the other's codes (its `query-unsupported` included).
- The active change carddav-query-filter is untouched.
- No version bump; no release.

## Order and parallel work

This change runs in parallel with calendula's (calendula/cairn/changes/a-result-carries-its-errors), ortie's and carillon's: they touch other repositories. The two shared inputs are written once elsewhere: io-pim-discovery's stream-0-4 (Phase 0, before Phase 1) and io-webdav's a-listing-returns-what-it-could-not-read (before Phase 4). Calendula reads the same Phase 2 readers: the two agents copy neverest's, never each other's half-done ones.
