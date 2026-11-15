---
cairn: delta
change: pimdir-follows-storage-draft-13
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: pimdir status reports the store and its parked actions (commands)

Adds the refusals standing in the account's address books (`PimdirReader::refusals`): `refused: [{"addressbookId", "itemId", "source", "reason", "refusedAt"}]`, always present, printed in text as `CARD  ADDRESSBOOK  SOURCE  REASON` then `Refused by the server, held until the card is edited or removed`.

### Requirement: The pimdir queue is read and retracted (commands)

`show` adds `refusals: [{"collection", "source", "reason", "refusedAt"}]`, the refusals standing on the card an applied row acted on, a text line `Refused by <source> in <collection> at <refusedAt>: <reason>` each, the state staying `applied`; `seq` is the card the row acted on for every kind, the text saying `on card <seq>`.

### Requirement: A pimdir store of another version is reinstalled (backends)

The version read is `storage-draft-13`; the scenario's store is a draft-11 one, the message naming both and the rebuild.

## REMOVED Requirements

None.
