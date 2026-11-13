---
cairn: spec
capability: mcp
status: current
---

# MCP

`cardamum mcp` is a Model Context Protocol server over stdio exposing the unified API as tools, on pimalaya-cli's shared `mcp` module (src/mcp.rs).

### Requirement: Tools are the classified unified API
`cardamum mcp` SHALL expose as tools `account_capability`, `addressbook_list`, `card_list` and `card_read`, and, under `--allow-write` only, `addressbook_create`, `addressbook_update`, `addressbook_delete`, `card_create`, `card_update` and `card_delete`, each tool's input being its command's grammar without the composer flags, its output schema the command's own. A tool the account's backend does not serve at all SHALL NOT be listed.

### Requirement: A tool reads no local file
Inside the MCP server, a vCard source SHALL be taken as text, a local path naming a file being no way in, and stdin SHALL never be read by a tool.

### Requirement: Tool annotations say what a tool does
A reading tool SHALL be `readOnlyHint`, an update or a delete `destructiveHint`, and every tool `openWorldHint: false`, no cardamum tool reaching people beyond the account's own data.

### Requirement: A call the grammar refuses is a usage failure
A tool call whose arguments the tool's input schema admits but its command's grammar refuses (a `page-size` of `-1`) SHALL answer a result with `isError`, its failure object coded `usage-invalid` (`pimalaya_cli::error::usage_error`), the server going on.
