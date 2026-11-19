---
cairn: delta
change: the-v1-pass
---

# Delta

## ADDED Requirements

### Requirement: A vdir move or copy never replaces an item
`vdir item move` and `vdir item copy` SHALL refuse a target address book holding an item of the same id, with the code `item-duplicate` and the message "Cannot <verb> item `<id>` into `<target>`: it holds an item of that id already", exit 1, both items left as they were.

#### Scenario: One contact in two books
- GIVEN the address books `work` and `perso` both holding `x.vcf`
- WHEN `cardamum --json vdir item move work perso x` runs
- THEN the command exits 1 with the code `item-duplicate`, and both files are unchanged

## MODIFIED Requirements

## REMOVED Requirements
