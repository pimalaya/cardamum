---
cairn: log
change: the-codes-follow-the-family
date: 2026-11-15
---

# The codes follow the family

Tasks C8, C11, C14 and C15 of cli-alignment-2026-11-15. Capabilities **commands** and **backends** moved.

- **CardDAV**: a 412 is `precondition-failed`, a `no-uid-conflict` `item-duplicate` (src/carddav/client.rs), as calendula codes CalDAV; the test that asserted no code for the conflict now asserts it.
- **No address book**: a card command without `-k` over an account setting no `addressbook.default` is `addressbook-default-missing`, naming the flag and the key (uncoded before), as calendula codes `calendar-default-missing`.
- **Not found**: vdir and JMAP word a missing card ``Card `<id>` not found``, the backend's text in `sources`, as CardDAV, Graph and People did and calendula does; vdir printed io-vdir's text alone.
- **pimdir**: Himalaya's table of 29 relayed codes replaces cardamum's 17 (`store-not-current`, `sync-version`, `store-owned`, `store-staging`, `store-in-flight`, `store-reset`, `flag-invalid`, `conflict-unread`, `no-performer`, `ambiguous-performer`, `index-zone-unknown` and `tag-refused` join the vocabulary).
