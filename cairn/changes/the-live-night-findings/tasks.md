---
cairn: tasks
change: the-live-night-findings
---

# Tasks

- [ ] D1: bump io-gpeople once its `totalItems` fix is released; live re-run `contact-groups list --sync-token`.
- [x] D2: JMAP capability reads `mayCreateAddressBook`. Test: a session holding false reports create and update not `full`.
- [x] D3 (ruling: keep the contacts, recommended): the help and capability detail of a People group delete say the contacts stay. Owner ruling 2026-11-24: keep the contacts, the recommended option.
- [x] D4: `jmap address-book create` prints an id receipt. Test: the JSON holds the id and no defaulted field.
- [x] D5: `vdir rename` onto an existing collection refused coded. Test.
- [x] D6: `jmap address-book update` prints an id receipt, the `/set` `updated` value holding only what the server changed (RFC 8620 §5.3). Test: an `{}` or partial answer prints the id alone.
- [x] D7: JMAP `addressbook.delete` declared `server`: Fastmail's Personal states `myRights.mayDelete: false` (read-only check, 2026-11-24). Test.
- [ ] Next live run: iCloud contact in `card` (owner to allow), microsoft-pimdir contacts (add `msgraph.contacts` to the live neverest rc), the google and microsoft suites.
