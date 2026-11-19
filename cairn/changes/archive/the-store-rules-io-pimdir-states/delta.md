---
cairn: delta
change: the-store-rules-io-pimdir-states
---

# Delta

Each requirement names its capability file in parentheses.

## MODIFIED Requirements

### Requirement: A pimdir addressbook is its collection id
(backends) The kind SHALL be compared on its media type, read by io-pimdir's `collection::media_type` (the part before any `;` parameter, trimmed), the one reading io-pimdir applies when it picks a collection's summaries. An undeclared collection, of the empty kind, SHALL belong to no application's kind: it is no address book.

### Requirement: A pimdir view is one account
(backends) With `pimdir.account` set, a name the store groups no collection under is refused coded `store-account-unknown` when the store groups any, and accepted on a store holding no collection yet, a first sync not having run; unset, the account is derived from the accounts holding collections of kind `text/vcard`, an undeclared collection of the empty kind not one.
