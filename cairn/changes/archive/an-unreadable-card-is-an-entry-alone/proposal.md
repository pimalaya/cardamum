---
cairn: change
id: an-unreadable-card-is-an-entry-alone
status: landed
created: 2026-11-15
---

# An unreadable card is an entry alone

## Why

`card list` printed a card it could not read twice: as a row of `cards` and as an `errors` entry. On a vdir holding a broken .vcf, `cards` held `{"id":"broken","fullName":null,...}` and `errors` `{"id":"broken","code":"card-unreadable",...}`, the text a row of dashes above the `Error:` line. A pimdir card whose body could not be read listed its summary preview beside its entry the same way. calendula's `event list` names an item it cannot read in `errors` alone, and pimalaya-cli's output contract prints "the two items read and an `errors` list holding the third's item error". Finding F13, task C2 of the family's cli-alignment-2026-11-15 plan.

## What

- The page seam every backend's listing goes through (`CardPage::of`, src/shared/client.rs) leaves out of `cards` each card it names in `errors`: one whose body does not parse or is not UTF-8 (vdir, pimdir, CardDAV, JMAP and the others alike, checked there), and one its backend already named, the pimdir card whose body could not be read.
- The window a backend returns stays positional: a pimdir card whose body could not be read still holds its place there with its preview, so the page is cut first and the card leaves after, and the next page starts after it. vdir files that cannot be read, CardDAV rows without data and JMAP cards that do not project were already entries alone.
- The text prints no row of empty cells for such a card; the `Page:` line counts the cards printed, as calendula's counts the events read.
- Spec: commands ("A listing names the items it could not read" modified), backends ("Each backend names the cards it could not read" modified).
