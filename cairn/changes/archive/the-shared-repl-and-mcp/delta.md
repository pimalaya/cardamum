---
cairn: delta
change: the-shared-repl-and-mcp
---

# Delta

## ADDED Requirements

### Requirement: A session runs the CLI over one account
(repl) `cardamum repl` SHALL start a session bound to one account and one `--backend`, read one command per stdin line, split as a shell splits it, parse it with the one-shot grammar and run it, and loop until `quit`, `exit` or EOF, the unified client connecting once for the session and again after a command that failed. A parse error or a command error SHALL be printed, on stdout as the failure object when the line asks for JSON, and the loop SHALL continue.

### Requirement: A session refuses what cannot finish inside it
(repl) Inside a session, `repl` and `mcp` SHALL be refused, the composer (`-i`) SHALL be refused by name, its editor needing the terminal the session reads, and a command that would read its input off stdin SHALL be refused with `stdin-taken`.

### Requirement: Tools are the classified unified API
(mcp) `cardamum mcp` SHALL expose as tools `account_capability`, `addressbook_list`, `card_list` and `card_read`, and, under `--allow-write` only, `addressbook_create`, `addressbook_update`, `addressbook_delete`, `card_create`, `card_update` and `card_delete`, each tool's input being its command's grammar without the composer flags, its output schema the command's own. A tool the account's backend does not serve at all SHALL NOT be listed.

### Requirement: A tool reads no local file
(mcp) Inside the MCP server, a vCard source SHALL be taken as text, a local path naming a file being no way in, and stdin SHALL never be read by a tool.

### Requirement: Tool annotations say what a tool does
(mcp) A reading tool SHALL be `readOnlyHint`, an update or a delete `destructiveHint`, and every tool `openWorldHint: false`, no cardamum tool reaching people beyond the account's own data.

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
