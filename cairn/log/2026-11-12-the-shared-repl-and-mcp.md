---
cairn: log
change: the-shared-repl-and-mcp
date: 2026-11-12
---

# The shared REPL and MCP server

Every command runs over a `Session` (src/session.rs) keeping the unified `AddressbookClient`, so `cardamum repl` and `cardamum mcp` connect it once. The REPL runs the full command tree on pimalaya-cli's `Repl`, refusing `repl`, `mcp`, the composer (`-i`) and stdin input (`stdin-taken`) by name while the session goes on. The MCP server, copied from Himalaya's, lists `account_capability`, `addressbook_list`, `card_list` and `card_read`, the six writes under `--allow-write`, and no tool the account's backend does not serve; no tool takes the composer flags or reads a local path, and every tool is `openWorldHint: false`. tests/repl.rs and tests/mcp.rs drive the binary over a scratch vdir (a card given as a path to a local file was refused and nothing read from it, one given as text was written); 95 unit tests and every integration suite pass, and clippy is clean on all features and on `rustls-ring` with no backend, pimdir and carddav.

Capabilities moved: repl (new: "A session runs the CLI over one account", "A session refuses what cannot finish inside it"), mcp (new: "Tools are the classified unified API", "A tool reads no local file", "Tool annotations say what a tool does"), commands (the code table gains `stdin-taken`).
