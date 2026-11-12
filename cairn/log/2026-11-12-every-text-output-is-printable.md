---
cairn: log
change: every-text-output-is-printable
date: 2026-11-12
---

# Every text output is printable

`card read` prints the card with its control and bidi characters replaced (CRLF read as LF), and the write confirmations sanitize the id they print; `the_text_view_defuses_what_could_drive_the_terminal` would fail on the raw view. tests/first_run.rs regained Himalaya's MCP case. 113 tests pass.

Capabilities moved: commands ("Plain output is printable").
