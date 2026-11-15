---
cairn: delta
change: a-text-output-ends-with-a-newline
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: Text output is plain text
(commands) A text output SHALL be plain text: no border, no colour, nothing sized to or truncated for the terminal, and it SHALL end with a newline: the `card read` view of a card whose last line lacks one, and the pretty JSON of the `jmap`, `msgraph` and `gpeople` `request` commands, included. A list of short rows prints as aligned columns, a list of wide items as records, a single item as `Label:  value` fields (pimalaya-cli `Columns`, `Records`, `Fields`).

#### Scenario: A card read through a REPORT
- GIVEN a pimdir card whose body ends `END:VCARD` with no line break, as a CardDAV `REPORT` stored it
- WHEN `cardamum card read` prints it
- THEN the text ends `END:VCARD` and one newline, while `--json` keeps the bytes

## REMOVED Requirements

None.
