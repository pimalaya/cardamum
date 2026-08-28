---
cairn: delta
change: pimdir-collection-id-is-the-addressbook
---

## ADDED Requirements

### Requirement: A pimdir addressbook is its collection id
The pimdir backend SHALL show and accept an address book as the store's collection id, verbatim: the collection `carddav/default` is the address book `carddav/default`. It SHALL NOT derive, strip or accept a shortened spelling, and no configuration SHALL offer one.

A sync engine binds a source's collections under a namespace, so an id carries one; the store is opaque to it, neither parsing nor validating an id (pimdir SPEC 9.2) and modelling hierarchy through `parent` rather than through a separator. Shortening is therefore a guess at the producer's convention, and one that makes a single address book answer to two spellings.

An id SHALL name an address book or name nothing: one store holds the collections of every kind a sync caches, so the kind SHALL narrow them at the one seam both the listing and the id check read, never at the listing alone. A kind-less collection counts, a sync having created one before kinds were declared.

An id naming no address book of the account SHALL be refused naming the ones it holds. Ids carry the sync engine's namespace and are not guessable, so an error asking for one that shows none leaves the user nothing to act on.

#### Scenario: A mailbox is not an address book
- GIVEN a store a sync fills with mail, calendar and contact collections alike
- WHEN a card command addresses a `message/rfc822` collection
- THEN it is refused naming the address books the account holds, rather than listing the mailbox's messages as blank contacts
