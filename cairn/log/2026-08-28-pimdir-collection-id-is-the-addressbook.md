---
cairn: log
date: 2026-08-28
change: pimdir-collection-id-is-the-addressbook
---

# A pimdir addressbook is its collection id

Neverest binds a source's collections under a namespace, so the address book it caches is keyed `carddav/default`. Cardamum already addressed it by that whole id, which is the right shape and was never written down: a collection id is opaque to the store, which neither parses nor validates it (pimdir SPEC 9.2) and models hierarchy through `parent` rather than through a separator, so shortening one is a guess at the producer's convention rather than a lookup. Himalaya had shipped such a guess, with a `pimdir.namespace` key to rescue the stores it could not decide, and removed both the same day. Stating the rule here is what stops it being added.

Writing it down found the two things that made the rule hard to live with.

## What landed

**The kind narrows at one seam, in src/pimdir/backend.rs.** `collections()` returned the account's collections of every kind, and only `list_addressbooks` filtered them down to `text/vcard`. `known_collection` read the same unfiltered list, so an id naming a collection of another kind passed the check. Against a store Neverest fills with mail, calendars and contacts alike, `card list -k imap/INBOX` listed a mailbox's messages as blank contacts, and a create would have staged a vCard into it. The filter moved into `collections()`, which both callers read, so an id names an address book or it names nothing.

**The refusal names what the account holds.** `Addressbook \`default\` not found` became `Addressbook \`default\` not found; this account holds: carddav/default`. An id carrying a namespace is not guessable, so an error asking for one has to show the choices, and the same message now answers a wrong-kind id and a mistyped one.

## Capabilities moved

- backends: added *A pimdir addressbook is its collection id*

## Verification

Built clean and run against the live Neverest store at `~/.local/state/neverest/posteo`, which holds eighteen collections across three kinds under one account. Before the fix, `card list -k imap/INBOX` printed sixteen blank contacts; after it, that id and `default` are both refused naming `carddav/default`, `card list -k carddav/default` lists the contacts, and `addressbook list` is unchanged.
