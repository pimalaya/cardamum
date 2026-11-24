---
cairn: tasks
change: the-live-night-findings
---

# Tasks

- [ ] D1: bump io-gpeople once its `totalItems` fix is released; live re-run `contact-groups list --sync-token`.
- [x] D2: JMAP capability reads `mayCreateAddressBook`. Test: a session holding false reports create and update not `full`.
- [ ] D3 (ruling: keep the contacts, recommended): the help and capability detail of a People group delete say the contacts stay.
- [x] D4: `jmap address-book create` prints an id receipt. Test: the JSON holds the id and no defaulted field.
- [x] D5: `vdir rename` onto an existing collection refused coded. Test.
- [ ] Next live run: iCloud contact in `card` (owner to allow), microsoft-pimdir contacts (add `msgraph.contacts` to the live neverest rc), the google and microsoft suites.
