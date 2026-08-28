---
cairn: change
id: pimdir-collection-id-is-the-addressbook
status: landed
created: 2026-08-28
---

# A pimdir addressbook is its collection id

## Why

The sync engine binds a source's collections under a namespace, so an address book it caches is keyed `carddav/default` rather than `default`. Cardamum already addresses it by that id and shows it whole, which is the right shape: a collection id is opaque to the store, which neither parses nor validates it (pimdir SPEC 9.2) and models hierarchy through `parent` rather than through a separator, so shortening one would be a guess at the producer's convention rather than a lookup.

Himalaya learnt this the long way, having shipped a derived short name and a `pimdir.namespace` key to rescue the cases the derivation could not decide, and has just removed both. Cardamum never grew either, so the alignment is to say so before someone adds one, and to make the rule usable.

Usable is where two gaps showed. An id that names no collection is refused, but the refusal does not say what the store does hold, and an id carrying a namespace nobody can guess is one the user has to be shown. Worse, `known_collection` read the store's collections unfiltered while the listing filtered them by kind, so an id naming a collection of another kind passed the check: against a store Neverest fills with mail, calendars and contacts alike, `card list -k imap/INBOX` listed a mailbox's messages as blank contacts, and a create would have staged a vCard into it.

## What

The requirement is stated: a pimdir addressbook is its collection id, verbatim, with no derived spelling and no configuration offering one.

The kind filter moves into `collections()`, the one seam both the listing and the check read, so an id names an address book or it names nothing. The refusal then names the address books the account holds, so the id to type is in the error that asks for it.
