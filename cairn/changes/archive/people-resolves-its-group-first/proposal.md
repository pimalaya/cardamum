---
cairn: change
id: people-resolves-its-group-first
status: landed
created: 2026-11-15
---

# People resolves its group first

## Why

Live testing on the Google test account (reports/cardamum/gpeople-google.md, G1 and G2, 2026-11-15):

- `card create -k nope c.vcf` exited 1 with a bare "People API returned HTTP 404" after creating the person in myContacts: `create_card` created the contact, then added the membership, which 404'd. A retry creates a second one, and nothing told the user of the first. Task C24 of cli-alignment-2026-11-15, high.
- `card list -k nope` exited 0 with an empty list: `list_cards` lists every connection and keeps the members of the group, so a typo matches nothing. Task C25.

Both break commands "A command resolves its addressbook before acting", fixed on vdir and pimdir before.

## What

- `resolve_group`: one `GET contactGroups/<id>` before `card list`, `read`, `add`, `update` and `delete` touch anything, myContacts skipped; a 404 is `addressbook-not-found`, cardamum's sentence, People's text in `sources`. `update` and `delete` ignored the group before; they check it now, as every other backend names an unknown address book.
- A membership refused after the create says the card stays in Contacts (`Card `<id>` was created in Contacts but not added to addressbook `<id>``), the race of a group deleted in between.
- `addressbook update` and `delete` code a 404 the same way.
- Checked: Graph creates a contact with one `POST contactFolders/<id>/contacts` and JMAP with one `ContactCard/set` naming the address book, so an unknown one creates nothing there; their uncoded refusal of an unknown one is the low findings of msgraph-microsoft.md (its `card list` row) and jmap-fastmail.md J2, not this change. No other People write takes a group in two steps: the protocol commands are one request each.
