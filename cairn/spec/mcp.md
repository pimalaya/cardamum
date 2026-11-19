---
cairn: spec
capability: mcp
status: current
---

# MCP

`cardamum mcp` is a Model Context Protocol server over stdio exposing the unified API as tools, on pimalaya-cli's shared `mcp` module (src/mcp.rs).

### Requirement: Tools are the classified unified API
`cardamum mcp` SHALL expose as tools `account_capability`, `addressbook_list`, `card_list`, `card_read`, `card_search`, and, under `--allow-write` only, `addressbook_create`, `addressbook_update`, `addressbook_delete`, `card_add`, `card_update` and `card_delete`, each tool's input being its command's grammar without the composer flags, its output schema the command's own. Every leaf of `addressbook` and `card` SHALL be either a tool or excluded by name (`card build`, a local formatter writing files), and the instructions SHALL name no tool the server does not define, each checked by a test. A tool the account's backend does not serve at all SHALL NOT be listed.

### Requirement: A tool reads no local file
Inside the MCP server, a vCard source SHALL be taken as text, a local path naming a file being no way in, and stdin SHALL never be read by a tool.

### Requirement: Tool annotations say what a tool does
A reading tool SHALL be `readOnlyHint`, an update or a delete `destructiveHint`, and every tool `openWorldHint: false`, no cardamum tool reaching people beyond the account's own data.

### Requirement: A partial outcome keeps its result
A tool whose command ends with a partial outcome (a listing whose `errors` is not empty, an `account check` with a failed backend) SHALL return its result as `structuredContent` beside `isError: true`, so an agent reads the items and their `errors` together.

### Requirement: A call the grammar refuses is a usage failure
A tool call whose arguments the tool's input schema admits but its command's grammar refuses SHALL answer a result with `isError`, its failure object coded `usage-invalid` and worded by the tool's grammar (`pimalaya_cli::mcp::grammar::Grammar::usage_error`), which names each argument as the tool's input does, never clap's usage text: a missing argument as ``Missing argument `card-id` ``, a refused value as ``Invalid value `-1` for `page-size`: <reason>``. The server SHALL go on.

#### Scenario: A card read missing its id
- GIVEN a server over a vdir account
- WHEN `card_read` is called with `addressbook` alone
- THEN the result is an error whose failure object is coded `usage-invalid` and reads ``Missing argument `card-id` ``
