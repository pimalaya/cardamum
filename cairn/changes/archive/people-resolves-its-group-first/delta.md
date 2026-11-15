---
cairn: delta
change: people-resolves-its-group-first
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: People resolves its contact group before acting
(backends) The People backend SHALL read the contact group a card command names (`contactGroups.get`, no members) before it lists, reads or writes anything through it, myContacts excepted, every account holding it: a group People does not hold SHALL fail coded `addressbook-not-found`, ``Addressbook `<id>` not found`` with People's answer in `sources`, after that one request. A listing narrows every connection to the group's members on the client, and a create lands in myContacts before its membership is added, so without it a typo lists as an empty group and a create leaves a contact behind before failing. A membership refused after the create SHALL say the card was created in Contacts and not added to the group. `addressbook update` and `addressbook delete` of a group People does not hold SHALL fail with the same code.

#### Scenario: A card added to a group Google does not hold
- GIVEN a Google account holding no contact group `nope`
- WHEN `cardamum --json card add -k nope c.vcf` runs, or `cardamum --json card list -k nope`
- THEN each exits 1 coded `addressbook-not-found` after one `GET contactGroups/nope`, and no `createContact` is sent

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
