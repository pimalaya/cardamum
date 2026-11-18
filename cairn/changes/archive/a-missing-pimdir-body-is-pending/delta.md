---
cairn: delta
change: a-missing-pimdir-body-is-pending
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: Each backend names the cards it could not read (backends)

pimdir: a body the blob tree no longer holds is no entry, the card listed with its preview as an unfetched one (pimdir GUIDE §14); a body that cannot be read stays a `card-unreadable` entry.

### Requirement: pimdir is a cache, not a server (backends)

`get_card` on a card whose body is not local, never fetched or its blob file gone or failing its name, fails coded `body-pending` with the unfetched sentence, exit 1, as Himalaya. One scenario added.

### Requirement: A JSON error carries a stable code (commands)

`card-unreadable` no longer covers a body gone from the blob tree; `body-pending` is the failure of `card read` on a card whose body is not local, either way.

### Requirement: pimdir searches cards through the store's index (commands)

A hit whose body is gone from the blob tree lists as usual; the scenario holds an unreadable hit and a hit whose body is gone.

## REMOVED Requirements

None.
