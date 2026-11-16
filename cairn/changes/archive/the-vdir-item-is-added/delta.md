---
cairn: delta
change: the-vdir-item-is-added
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: Specific commands mirror their protocol (commands)

vdir, a local store rather than a protocol, speaks the family verbs: `vdir create` makes a collection and `vdir item add` an item, `new` its visible alias, its schema `cardamum-vdir-item-add`, as calendula's `vdir item add`.

### Requirement: A former command name stays hidden (commands)

`vdir item create` stays accepted, hidden, for `vdir item add` (`cardamum-vdir-item-add`). New scenario "The former name of vdir item add".

## REMOVED Requirements

None.
