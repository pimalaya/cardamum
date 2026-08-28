---
cairn: change
change: duplicate-link-id-mints-an-item
---

# Delta

## ADDED Requirements

### Requirement: A UID is not an address
The pimdir backend SHALL NOT assume an item's link id is the `UID` its body carries, nor that a `UID` identifies at most one card in a collection. A store may hold two cards of one address book sharing a `UID`, keyed apart by the store (pimdir SPEC §9), and both SHALL list, read and act as ordinary cards, addressed by their own public `seq`.

What stays unique is the key and the public id: `(collection, link_id)` still names one item and `seq` still names one card. What ends is the link id being derivable from the body, so a read that re-derives a `UID` in order to address a row is addressing an unknown number of them.

RFC 6352 §5.1 requires the `UID` to be unique in the collection and servers do not always enforce it, most often after a repeated import. The store now keeps both copies rather than one, and a backend resolving an identity to whichever row came first would hide a card the server holds.

#### Scenario: A duplicated card lists twice
- GIVEN an address book whose store holds two items whose bodies carry one `UID`
- WHEN the backend lists it
- THEN both appear, each with its own public id, and neither is marked

## MODIFIED Requirements

### Requirement: pimdir derivations match the sync engine
The link id and the `v: 1` summary a pimdir write records SHALL be derived by `io_pimdir::conventions::card`, the format's own derivations, so a card Cardamum stages links and summarizes exactly as the same card arriving through a sync. The link id is the bare `UID`, with nothing prepended. A queued action carries no sort key: the format leaves the key to the sync that pushes the create, and a producer deriving one would order an item the connector is about to reorder.

A derivation is what a write carries, not a lookup: the store keys an item it received from a source by the derived id only when that id is free in the collection, and mints a distinct one otherwise (pimdir SPEC §9). Reading a card by re-deriving its `UID` is therefore not addressing, and the public `seq` is.

## REMOVED Requirements
