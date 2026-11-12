---
cairn: delta
change: every-text-output-is-printable
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: Plain output is printable
(commands) A field taken from a card or a server (a name, an address, an id, an error a server wrote) SHALL have its control characters and bidi controls replaced before it is printed as text (pimalaya-cli `sanitize::text`), the write confirmations included. The `card read` view SHALL replace them in the card too, keeping tabs and line feeds and reading CRLF as LF. JSON strings and the raw bytes a command is asked for SHALL remain unchanged.

## REMOVED Requirements

None.
