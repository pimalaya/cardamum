---
cairn: log
change: people-honest-writes
landed: 2026-08-09
---

# Google People: a count nobody asked for, an error page, and an update that kept what it dropped

Three findings from the 2026-08-09 Google People run, the last account of the round.

**A member count that was never the server's.** `contact-group list` showed `MEMBERS 0` for a group holding a contact, and `contact-group get` said `0` too, while the raw API reported `memberCount: 1`. Two causes, one on top of the other: the commands passed an empty `groupFields` mask, so People left `memberCount` out, and the table then counted `memberResourceNames`, which People fills only on a `get` and only up to the requested maximum. Both commands now ask for `name`, `groupType` and `memberCount`, and both renderers read the count the server sent, leaving the cell blank when it was not asked for rather than printing a zero that reads as an empty group.

**An error page as an error message.** io-people's `parse_api_error` fell back to the raw body whenever the response was not a Google JSON envelope, and Google answers some 404s with an HTML page. It now summarises: the page's own `title` when it has one, else the markup stripped, the whitespace collapsed and the length capped, the same treatment io-webdav gained earlier today. `People API returned HTTP 404: <!DOCTYPE html>…` reads `People API returned HTTP 404: Error 404 (Not Found)!!1`.

**An update that kept what it dropped.** A vCard property People cannot model lives in a `clientData` entry. An update carrying a new value replaces it, but an update that drops it leaves the old value standing, while every first-class property is removed by that same request. The limit is Google's, established before touching anything: the update mask does carry `clientData`, and a raw `people request patch` with `updatePersonFields=clientData` and an explicit empty list leaves the entry there too. Microsoft Graph, whose stash works the same way, clears it correctly.

Nothing can make Google forget, so the fix is to stop claiming otherwise. The shared `update_card` now returns a `CardUpdateOutcome` naming the properties the backend had to leave behind, every backend but People returning it empty, and `card update` appends them to its message: "Card `x` successfully updated, except X-CUSTOM-FIELD, which the server will not let go". The capability [commands](../spec/commands.md) gained a requirement for that and for the count.

Verified live on the `people` account, with a regression pass on Fastmail CardDAV for the changed shared signature.
