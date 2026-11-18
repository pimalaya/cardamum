---
cairn: tasks
change: the-v1-pass
---

# Tasks

- [x] CAR-1: `vdir item move|copy` refuses a same-id target, `item-duplicate` (tests/duplicate.rs)
- [x] CAR-2: the name flags edit the `N` line in place, parameters kept, a 2.1 quoted-printable value decoded first (src/shared/card/fields.rs)
- [x] CAR-3: 2.1 quoted-printable and CHARSET decoded in the projection (src/shared/card/project.rs)
- [x] CAR-8: unified card verbs on vdir act on `.vcf` items only (src/vdir/backend.rs `locate_card`); a body opening no `BEGIN:VCARD` is `card-unreadable` (src/shared/client.rs `unreadable`)
- [ ] CAR-14: private composer draft
- [x] CAR-4 with TWN-4: the `.well-known` redirect followed over https only, a `server` naming a scheme read as written (src/carddav/client.rs)
- [ ] CAR-11: `carddav delete` card id required, `carddav rmcol`, `--if-none-match` value
- [ ] TWN-3: `no-performer` (after TRI-3 lands in src/pimdir)
- [ ] CAR-5: address book honoured on JMAP, Graph, People
- [ ] io-pimdir 0.16 bump
- [ ] v1: drop `removed_key!` and MIGRATION.md
- [ ] Fold the delta and carddav-query-filter's into cairn/spec, log, CHANGELOG, archive both changes
