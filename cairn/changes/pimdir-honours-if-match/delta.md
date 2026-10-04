---
cairn: change
change: pimdir-honours-if-match
---

# Delta

## ADDED Requirements

### Requirement: A pimdir card's version is its body hash
The pimdir backend SHALL report as a card's `etag`, in `card list` and `card read`, the store's hash of its body, the pending queue folded in, so a staged update moves it at once. A card whose body is not local SHALL report none.

`update_card` and `delete_card` SHALL refuse, before anything is queued, a write whose `if_match` is not that version, surrounding double quotes ignored, with an error starting `Precondition failed:` and naming the version found and the one expected. A card with no local body matches no version.

### Requirement: A delete can be gated on a version
`card delete` SHALL take `--if-match <ETAG>`. CardDAV SHALL send it as `If-Match` and pimdir SHALL check it as an update does; the backends that cannot gate a delete (vdir, jmap, msgraph, gpeople) SHALL refuse it rather than drop it.

## MODIFIED Requirements

### Requirement: pimdir writes are staged queue actions
A pimdir write SHALL append one action to the store's queue (pimdir SPEC §15.1) through a producer opened for that write and dropped after it: `create_card` to `add`, `update_card` to `update`, `delete_card` to `remove`. The body reaches the blob tree through the blob writer before the row that pins it is appended, and the action addresses the item by the public `seq` that is already the card's shared id. `update_card` and `delete_card` SHALL honour `--if-match` against the card's version; the engine still reconciles an applied edit against the base body it recorded at sync time: the precondition is the caller's, the merge the store's. Because a queued create carries no public id until the owner applies it, `create_card` SHALL report the card's link id instead.
