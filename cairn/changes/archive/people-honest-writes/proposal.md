---
cairn: change
id: people-honest-writes
status: landed
created: 2026-08-09
---

# Google People: a count that was never asked for, an error page, and an update that keeps what it says it dropped

Three findings from the 2026-08-09 Google People run ([google-people.md](../../../spec/testing/google-people.md), [people-specific.md](../../../spec/testing/people-specific.md)).

**S2: `contact-group list` reports a member count of 0 for every group.** A group holding one contact lists as `MEMBERS 0` while `contact-group get` on the same id reports `memberCount: 1`. Google's `contactGroups.list` returns `memberCount` only when the `groupFields` mask asks for it, and the list call passes an empty mask, so the absent value renders as zero, reading as fact rather than as "not asked".

**P7 / S5: a non-JSON error surfaces as a whole HTML page.** io-people's `parse_api_error` falls back to the raw body when the response is not a Google JSON error envelope, and Google answers some 404s with an HTML page. This is the noise io-webdav shed when it learned to summarise a body; the People path never learned.

**P8: a stashed property cannot be removed, and the update says nothing.** A vCard property People cannot model lives in a `clientData` entry. An update carrying a new value replaces it, but an update that drops it leaves the old value on the server, while every first-class property is correctly removed by the same request. The limit is Google's, verified down to a raw `people request patch` with `updatePersonFields=clientData` and an explicit `"clientData": []`, which also leaves the entry standing. Microsoft Graph, whose stash works the same way, clears it correctly. What cardamum still controls is the report: today it prints a plain "successfully updated" for a card it only partly updated.

## What changes

- The `contact-group list` call asks for the fields the table shows (`name`, `groupType`, `memberCount`), so the count is the server's rather than a default.
- io-people summarises a non-envelope error body: the HTML `title` when there is one, else the markup stripped, the whitespace collapsed and the length capped, mirroring io-webdav's `summarize_body`.
- The shared `update_card` returns what it could not do, as a list of vCard property names the backend had to leave behind, and `card update` says so instead of reporting a clean success. Every backend but People returns an empty list, since every other backend either removes the property or has no stash at all.

## What does not change

Nothing about the write itself: the update still lands, and the properties Google keeps are still there afterwards. This makes the outcome legible, it does not make Google forget.
