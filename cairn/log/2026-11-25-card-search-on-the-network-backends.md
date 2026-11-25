---
cairn: log
change: card-search
date: 2026-11-25
---

# Card search on the network backends

CardDAV, JMAP, Microsoft Graph and Google People now serve `card search`, `card.search` `partial` (`SEARCH_MATCHED`, the matcher vdir reads with), where they refused it `capability-missing` until a search over a large address book was measured (owner ruling of 2026-11-19).

Measured on 2026-11-25 with a release build over 3,000 generated cards a backend, each address book created for the run and deleted after: on Graph and on the local Stalwart (JMAP, CardDAV) a search costs the listing it reads, the matching adding a few milliseconds (CardDAV: list 55 ms, search 60 ms over 2,000 cards). Graph searched one folder in 3.2 s and every folder in 3.4 s; JMAP in about 100 ms.

People listed its connections 100 a request, every address book walking all of them, so one search over three address books of 3,033 contacts sent 92 requests and People refused it 429 under its per-minute read quota. The listing now reads 1,000 connections a request, the API's largest: a 3,000-contact group lists in 1.5 to 1.8 s where it took 4.9 to 5.4 s, and a search over every address book answers in 3.4 to 4.9 s. Five such searches a minute still meet the quota on the sixth. A contact in a People group is listed under the group and under `myContacts` alike, so a search over every address book prints it once for each.

Seen, left as they are: Stalwart truncates a CardDAV listing at 2,000 cards, which `card list` and `card search` report as an error entry, exiting 3; `card search -s 0` pages one card where `card list -s 0` lists every card.

Spec: commands.md (the unified card search, every backend answering the same hit set), backends.md (People reads 1,000 connections a request).
