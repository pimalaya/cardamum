---
cairn: change
id: a-text-output-ends-with-a-newline
status: landed
created: 2026-11-15
---

# A text output ends with a newline

## Why

Live testing of 2026-11-15 (reports/cardamum/part2-pimdir.md F2, jmap-fastmail.md J3, low): `card read` printed a card whose last line lacks a line break (a CardDAV `REPORT` body, io-webdav F3) as is, and `jmap request` printed its pretty JSON with no final newline, the shell prompt landing on the last line. commands.md "Text output is plain text" already asks for the newline. Task C28 of cli-alignment-2026-11-15.

## What

- `CardReadOutput` adds a newline when the card's text does not end with one; `--json` keeps the bytes.
- `RawJsonOutput`, shared by the `jmap`, `msgraph` and `gpeople` `request` commands, ends with a newline.
- Left as they are: `carddav get`, `vdir item get` and `card build`, which print the bytes asked for, raw.
