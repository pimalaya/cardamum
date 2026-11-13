---
cairn: spec
capability: repl
status: current
---

# REPL

`cardamum repl` runs the CLI's own commands over one account, a line at a time, on pimalaya-cli's shared `Repl` (src/repl.rs, src/session.rs).

### Requirement: A session runs the CLI over one account
`cardamum repl` SHALL start a session bound to one account and one `--backend`, read one command per stdin line, split as a shell splits it, parse it with the one-shot grammar and run it, and loop until `quit`, `exit` or EOF, the unified client connecting once for the session and again after a command that failed. A parse error or a command error SHALL be printed, on stdout as the failure object when the line asks for JSON, a parse error coded `usage-invalid`, and the loop SHALL continue.

### Requirement: A session refuses what cannot finish inside it
Inside a session, `repl` and `mcp` SHALL be refused, the composer (`-i`) SHALL be refused by name, its editor needing the terminal the session reads, and a command that would read its input off stdin SHALL be refused with `stdin-taken`.
