---
cairn: change
id: the-protocol-views-print-what-they-hold
status: landed
created: 2026-11-15
---

# The protocol views print what they hold

## Why

Live testing of 2026-11-15, three low findings against commands.md "A command prints what the server returned":

- `carddav propfind` printed empty `Name:` and `Color:` lines on iCloud and never the description Fastmail sends, which `--json` carries (carddav-icloud.md C3). Task C31.
- `msgraph contacts` read the mobile phone alone, so a contact with business phones showed none, and printed empty `Email:` and `Phone:` lines (msgraph-microsoft.md M1). Task C34.
- `vdir list` printed the name and the path, never the description and colour its JSON carries (vdir.md V1). Task C35.

## What

- Each view prints a field it holds and no empty one; `carddav propfind` and `vdir list` add the description and the colour.
- A Graph contact prints every email and every phone, mobile then business then home, each kind on one line.
