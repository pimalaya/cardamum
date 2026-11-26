---
cairn: tasks
change: the-live-night-findings
---

# Tasks

- [x] D1: bump io-gpeople once its `totalItems` fix is released; live re-run `contact-groups list --sync-token`. Bumped to 0.9.1. Live read-only re-run 2026-11-24 on google: People still answers a fresh sync token with `"totalItems": -1` (raw `gpeople request get contactGroups?syncToken=<token>`), and `cardamum -a google --json gpeople contact-groups list --sync-token <token>` now exits 0 with no groups, a `nextSyncToken` and `nextPage: null` (the night failed `invalid value: integer -1, expected u32`).
- [x] D2: JMAP capability reads `mayCreateAddressBook`. Test: a session holding false reports create and update not `full`.
- [x] D3 (ruling: keep the contacts, recommended): the help and capability detail of a People group delete say the contacts stay. Owner ruling 2026-11-24: keep the contacts, the recommended option.
- [x] D4: `jmap address-book create` prints an id receipt. Test: the JSON holds the id and no defaulted field.
- [x] D5: `vdir rename` onto an existing collection refused coded. Test.
- [x] D6: `jmap address-book update` prints an id receipt, the `/set` `updated` value holding only what the server changed (RFC 8620 §5.3). Test: an `{}` or partial answer prints the id alone.
- [x] D7: JMAP `addressbook.delete` declared `server`: Fastmail's Personal states `myRights.mayDelete: false` (read-only check, 2026-11-24). Test.
- [x] Next live run: iCloud contact in `card` (owner to allow), microsoft-pimdir contacts (add `msgraph.contacts` to the live neverest rc), the google and microsoft suites. Run on the live night of 2026-11-24 (.lanes/live/report-2.md): iCloud card writes in the default book, Google `profile get` and other contacts, microsoft-pimdir add and update drained, suites google 10 and microsoft 5. Not run: the JMAP cross-account copy (Stalwart's bug, outside cardamum).
- [x] D8: `gpeople other-contacts copy` sends the copy mask People accepts (names, emails, phones), not the read mask with `metadata` (live night 2, 2026-11-24: HTTP 400 `INVALID_ARGUMENT` on every copy). Test: `gpeople::other_contact::fields::tests::the_copy_mask_holds_only_names_emails_and_phones`, red before the fix. Live re-run: copy created a contact, deleted after.
