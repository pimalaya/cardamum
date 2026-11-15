---
cairn: log
change: next-page-prints-after-the-listing
date: 2026-11-15
---

# Next page prints after the listing

cardamum moves to pimalaya-cli 0.6.0, io-gpeople 0.8.1 and io-msgraph 0.9.2 (task X14 of cli-alignment-2026-11-15). Capability **commands** moved: `Next page: --page <n>` prints below the items of `card list` and `pimdir search`, one blank line apart and above the `Error:` lines, and every listing's header values sit two spaces past its longest label (`Account:  perso (vdir)`). No cardamum code changed; the snapshots in src/shared/card/list.rs, src/shared/addressbook/list.rs, src/pimdir/search.rs and tests/capability.rs follow. io-gpeople 0.8.1 stops `keptProperties` naming what a stashed contact's update keeps; io-msgraph 0.9.2 adds calendar series cardamum does not use.
