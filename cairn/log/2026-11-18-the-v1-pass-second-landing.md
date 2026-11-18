---
cairn: log
change: the-v1-pass
date: 2026-11-18
---

# The v1.0.0 pass, second landing

- **CAR-12** (375e73f, 1b010e5): a field flag refuses anything after the first card and ends an appended line as the card's `BEGIN` does; a `tel:` or `mailto:` prefix is the value's scheme, never a type; an invalid vdir id (`a/b`) is `card-not-found`; `jmap contact-card query` prints `queryState`; `msgraph contacts create|update` send the JSON as written (`MsgraphClient::raw_send`) and print Graph's answer raw.
- **CAR-6** (d27ad07): `card list` reads the page it prints on JMAP (window and chunks of `maxObjectsInGet`, `total` read), Graph (`$skip`, `$top`) and People (stops once it holds the window); page 1 is one request on each.
- **CAR-7** (d27ad07): `card add -k <unknown>` is `addressbook-not-found` on CardDAV and JMAP, a stale People `etag` `precondition-failed`, a JMAP method error inside a typed call coded.
- **CAL-14's twin and the kind-scoped account** (2d3653b, 553a3cf), the https-only `.well-known` probe (666bd25), install.sh `curl -f` (f3d8b22).

Each change carries a test that failed on the code before it, but the `--if-none-match` value check and the HTTP 403/429 half of CAR-7, which already held. Spec: backends ("A network listing reads the page it prints", "A refused add names a missing address book" added; vdir, pimdir account), commands (field flags, `queryState`, protocol JSON writes, listing errors). CHANGELOG (Changed, Fixed).

Left: CAR-12 (7), Graph address books at every folder depth; CAR-14 (pimalaya-cli release); io-pimdir's next release (C1, C2, C4, draft-14, the card matcher for card-search); the Graph default-folder check of CAR-5 on the Microsoft test account.
