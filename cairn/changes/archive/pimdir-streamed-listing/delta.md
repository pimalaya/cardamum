---
cairn: change
change: pimdir-streamed-listing
---

# Delta

## ADDED Requirements

### Requirement: A pimdir card listing streams the store
`list_cards` on pimdir SHALL walk the address book in keyset batches of the store's contacts order and hold one batch of rows and the page it returns, never the address book. The cards before the page SHALL be counted off their rows without reading a body, a body SHALL be read only for a card the page returns, and the walk SHALL stop once the page is full.

## MODIFIED Requirements

## REMOVED Requirements
