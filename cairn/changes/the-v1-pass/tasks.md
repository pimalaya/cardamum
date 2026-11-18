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
- [ ] COH-5: a protocol refusal and a partial write are a result, exit 3 (owner ruling 2026-11-18), the `request` commands included; himalaya's shape first
- [ ] Single-resource protocol reads print raw (owner ruling 2026-11-18)
- [ ] io-pimdir next release: `PimdirFormatRefusal` text (C1), `enqueue_user_body` (C2), `PimdirActionHandle` queue rows (C4)
- [ ] Live check on the Microsoft test account: Graph default-folder resolution of CAR-5
- [x] CAL-14 twin: a configured `pimdir.account` the store lacks is refused, naming its accounts (src/pimdir/client.rs `resolve_account`)
- [x] The `.well-known` probe runs on an `https` origin alone, as calendula 7ac87ce (src/carddav/client.rs; src/account/check.rs test)
