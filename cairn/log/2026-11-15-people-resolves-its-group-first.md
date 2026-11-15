---
cairn: log
change: people-resolves-its-group-first
date: 2026-11-15
---

# People resolves its group first

Every unified card command on Google People reads the contact group it names before touching anything: an unknown group fails `addressbook-not-found` after that one request, where `card add -k nope` created the contact in myContacts then failed on its membership with a bare 404, and `card list -k nope` listed as empty with exit 0. Tasks C24 (high) and C25 of cli-alignment-2026-11-15, from live findings G1 and G2. Capability **backends** moved.

- **src/gpeople/backend.rs**: `resolve_group` (`GET contactGroups/<id>`, myContacts skipped) at the head of `list_cards`, `get_card`, `create_card` (after the local vCard parse, before `createContact`), `update_card` and `delete_card`; `group_error` codes a 404 `addressbook-not-found` there and on `addressbook update` and `delete`; a membership refused after the create says the card stays in Contacts.
- **Checked**: Graph and JMAP create in one request naming the folder or address book, so nothing is written for an unknown one.
- **Evidence**: `an_unknown_group_is_not_found_before_anything_is_written` scripts a People server answering 404: for list, read, add, update and delete, one `GET /v1/contactGroups/nope` and the code; with the resolution disabled it fails (the create sent `createContact` first). `a_create_into_a_group_adds_the_membership_or_says_what_stays` checks the order GET, `createContact`, `members:modify`, and the sentence when the last fails; `a_missing_person_is_a_card_not_found` now asserts the group's code. Not run live.

CHANGELOG (Fixed). Spec: backends ("People resolves its contact group before acting" added).
