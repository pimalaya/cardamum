---
cairn: change
id: every-text-output-is-printable
status: landed
created: 2026-11-12
---

# Every text output is printable

## Why

the-shared-kit added "Plain output is printable" but left two text outputs printing what a server wrote as is: the `card read` view printed the vCard raw, and the write confirmations printed a server-given id raw, so a card or an id carrying an escape sequence could drive the terminal. Found by the audit against Himalaya the owner asked for on 2026-11-12, which also restored the first-run test of an MCP server missing its configuration, dropped while cardamum had no server.

## What

- `card read`'s text view keeps the card's layout, CRLF read as LF, and replaces any other control character and the bidi controls with U+FFFD, as Himalaya's `message read` does; `--json` keeps the bytes.
- The write receipts, `card create`, `card update` and `addressbook create` sanitize the id they print.
- tests/first_run.rs: an MCP server missing its configuration leaves stdout empty and names the path on stderr.
