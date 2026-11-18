---
cairn: tasks
change: the-v1-pass
---

# Tasks

- [x] CAR-1: `vdir item move|copy` refuses a same-id target, `item-duplicate` (tests/duplicate.rs)
- [x] CAR-2: the name flags edit the `N` line in place, parameters kept, a 2.1 quoted-printable value decoded first (src/shared/card/fields.rs)
- [x] CAR-3: 2.1 quoted-printable and CHARSET decoded in the projection (src/shared/card/project.rs)
- [x] CAR-8: unified card verbs on vdir act on `.vcf` items only (src/vdir/backend.rs `locate_card`); a body opening no `BEGIN:VCARD` is `card-unreadable` (src/shared/client.rs `unreadable`)
- [x] CAR-4 with TWN-4: the `.well-known` redirect followed over https only, a `server` naming a scheme read as written (src/carddav/client.rs)
- [x] CAR-11: `carddav delete <ADDRESSBOOK> <CARD-ID>` (id required), `carddav rmcol <ADDRESSBOOK>`; `carddav put --if-none-match` takes `*` alone, the one value io-webdav sends (src/carddav/delete.rs, rmcol.rs, put.rs)
- [x] TWN-3: the refusal coded `no-performer`, as calendula (src/pimdir/backend.rs `writable`, cairn/spec/backends.md)
- [x] CAR-5: card read, update, delete act only on a card the named address book holds on JMAP (`addressBookIds`), Graph (`parentFolderId`) and People (memberships); a delete reads first (src/{jmap,msgraph,gpeople}/backend.rs). Graph default-folder resolution to check by hand on the Microsoft test account
- [x] io-pimdir 0.16.0: `pimdir search` pages by its cursor token, `--page` walks it (src/pimdir/search.rs, index.rs); PIM-1 (hostile VTIMEZONE abort) gone with it
- [x] v1: `removed_key!` gone (src/config.rs), a removed key refused by serde as unknown; MIGRATION.md removed; CHANGELOG and cairn/spec/config.md follow
- [x] carddav-query-filter folded and archived (3f78dba)
- [x] The landed lines folded into cairn/spec, CHANGELOG, log 2026-11-18-the-v1-pass
- [ ] CAR-14: private composer draft through pimalaya-cli `file::private_options`, once released
- [x] COH-5: a protocol refusal and a partial write are a result, exit 3 (owner ruling 2026-11-18), the `request` commands included; himalaya's shape first
- [x] Single-resource protocol reads print raw (owner ruling 2026-11-18)
- [ ] io-pimdir next release: `PimdirFormatRefusal` text (C1), `enqueue_user_body` (C2), `PimdirActionHandle` queue rows (C4)
- [ ] Live check on the Microsoft test account: Graph default-folder resolution of CAR-5
- [x] CAL-14 twin: a configured `pimdir.account` the store lacks is refused, naming its accounts (src/pimdir/client.rs `resolve_account`)
- [x] The `.well-known` probe runs on an `https` origin alone, as calendula 7ac87ce (src/carddav/client.rs; src/account/check.rs test)
- [x] COH-5 landed: a server's refusal in the `carddav`, `jmap`, `msgraph` and `gpeople` namespaces prints `{refusal: {status, reason, message, code, retryAfter}}`, exit 3, `jmap contact-card copy` prints `{ids, errors}` (src/shared/refusal.rs, src/error.rs `CodedCause`, readers in src/carddav/client.rs, src/msgraph/client.rs, src/gpeople/client.rs, src/jmap/error.rs `JmapSetRefusal`, dispatch in src/{carddav,jmap,msgraph,gpeople}/cli.rs, src/jmap/set.rs, src/jmap/contact_card/copy.rs, src/json_schema.rs `refusable!`; tests in those cli.rs and src/shared/refusal.rs)
- [x] Single reads raw: `msgraph contacts|contact-folders|profile get`, `gpeople connections|contact-groups|profile get`, `jmap address-book|contact-card get` print the provider's payload (src/msgraph/client.rs `raw_get`, src/gpeople/client.rs `raw_get`, src/jmap/raw.rs, the get.rs files, src/msgraph/*/render.rs, src/gpeople/render.rs, src/jmap/render.rs; tests `a_single_read_prints_the_payload_as_sent`, `a_get_prints_the_objects_as_sent`)
- [x] pimdir refusals coded as Himalaya's: `store-account-ambiguous`, `store-account-unknown` (src/pimdir/client.rs `resolve_account`), `page-token-invalid` (src/pimdir/search.rs, src/error.rs `page_token_invalid`), `row-not-found` and `confirmation-required` on `pimdir queue cancel`, refused before asking (src/pimdir/backend.rs `uncancellable`, src/pimdir/queue/cancel.rs); test tests/pimdir_refusals.rs
- [x] With `pimdir.account` unset, only the accounts holding address books count (src/pimdir/client.rs `resolve_account`), as himalaya and calendula
- [x] CAR-12 (1) a source with anything after its first card refused, appended lines end as the card does; (2) a `tel:`/`mailto:` scheme is no type (src/shared/card/fields.rs); (3) an invalid vdir id `card-not-found` (src/vdir/client.rs); (5) `jmap contact-card query` prints `queryState` (src/jmap/render.rs); (4) and (8) done earlier (2d6ae68, d7cb9d6)
- [x] CAR-12 (6): `msgraph contacts create|update` send the JSON as written through `MsgraphClient::raw_send` and print the answer raw (src/msgraph/client.rs, contacts/{input,create,update}.rs)
- [x] CAR-6: JMAP, Graph, People `card list` read the page they print (src/{jmap,msgraph,gpeople}/backend.rs, src/shared/client.rs)
- [x] CAR-7: People stale etag `precondition-failed` (src/gpeople/client.rs), `card add -k <unknown>` `addressbook-not-found` on JMAP and CardDAV (src/jmap/backend.rs, src/carddav/backend.rs), JMAP method errors in typed calls coded (src/jmap/error.rs)
- [ ] CAR-12 (7): Graph `addressbook list` at every folder depth (neverest precedent of 2026-11-17)
