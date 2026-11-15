---
cairn: change
id: pimdir-follows-storage-draft-13
status: landed
created: 2026-11-15
---

# The pimdir backend follows storage-draft-13

## Why

pimdir draft-13 (storage-, sync- and search-draft-13, pimdir 1c6442e) records a push a server refuses for good on the binding it was pushed for, and asks a producer to show such a row applied and refused by that source, with the reason (STORAGE §15.4). A store a draft-13 sync engine writes is refused by the draft-11 io-pimdir cardamum built on, so the frontend follows. Task N31 of cli-alignment-2026-11-15, with X14.

## What

- io-pimdir 0.14.0 from crates.io (d184622 plus its release commit); a git pin at d184622 served while it was unreleased, dropped once it published the same night.
- `pimdir queue show`: an applied row carries `refusals: [{collection, source, reason, refusedAt}]` (`PimdirActionStatus::Applied::refusals`), a text line `Refused by <source> in <collection> at <refusedAt>: <reason>` each; `seq` names the card the row acted on for every kind, the receipt naming it since draft-13, so the text says `on card <seq>`, not `creating card <seq>`.
- `pimdir status`: `refused: [{addressbookId, itemId, source, reason, refusedAt}]` from `PimdirReader::refusals`, per address book of the account, a list `CARD  ADDRESSBOOK  SOURCE  REASON` and a closing sentence in text.
- A draft-11 store is refused `store-version`, the message naming the rebuild through neverest.
- Write receipts (`card add`, `card update`...) are printed at enqueue time, before any push, so no refusal can stand there: unchanged.
