---
cairn: change
id: the-live-night-findings
status: active
created: 2026-11-24
---

# The live night findings

The live test night of 2026-11-23 (workspace .lanes/live/report.md) ran every cardamum command on 17 accounts. 7f7043c fixed a stale People etag overwriting a newer contact, JMAP `notFound` ids dropped, and People custom methods refused.

## What

1. **`contact-groups list --sync-token` fails** on Google's `"totalItems": -1` (io-gpeople, its cairn/changes/the-live-night-findings). cardamum bumps io-gpeople once fixed.
2. **JMAP capability overstates address book writes.** `account capability` says create and update are `full`; Fastmail's session says `mayCreateAddressBook: false` and refuses both. Read the session flag and report `none` (or `server`, as CardDAV does) when it is false.
3. **People group delete keeps the contacts** (`deleteContacts=false`) while the help says "and every card it contains". Ruling recommended: keep the safe behaviour, a contact may sit in other groups; fix the help and the capability detail.
4. **`jmap address-book create --json`** prints `name: null` and every right false: io-jmap fills what the /set reply omits with defaults. Print an id receipt, as himalaya's `jmap mailbox create` does.
5. **`vdir rename` onto an existing collection** prints the raw "Directory not empty (os error 39)". Refuse it coded, naming the target.

## Not tested, for the next live run

iCloud CardDAV writes (iCloud refuses a new address book; writing a run contact into the default `card` book and deleting it needs the owner's leave), microsoft-pimdir contacts (the live neverest rc syncs Graph mail only), People `profile get` and other contacts (the token lacks the scopes), the JMAP copy success path (a second contacts account), the google and microsoft live suites.
