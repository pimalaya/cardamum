//! # MCP command
//!
//! The `mcp` subcommand, a Model Context Protocol server over stdin and
//! stdout exposing the unified API as tools, one per leaf command.
//!
//! pimalaya-cli's [`mcp`] module answers the protocol and checks each call
//! against its tool's input schema; this one lists the tools and runs the
//! calls. A tool's input is its clap command's [`Grammar`], and a call
//! turns back into an argv parsed by that same command, so the CLI, the
//! [`crate::repl`] and MCP share one grammar. Commands run over the
//! [`Session`] the server keeps and print into a [`CapturePrinter`], whose
//! one JSON value becomes the call's structured result.
//!
//! [`mcp`]: pimalaya_cli::mcp

use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Result, anyhow};
use clap::{CommandFactory, Parser};
use pimalaya_cli::{
    mcp::{ServerInfo, ToolCall, ToolResult, grammar::Grammar, serve, tool_output_schema},
    printer::CapturePrinter,
};
use serde_json::{Value, json};

use crate::{
    capability,
    cli::{Cli, Command, resolve_account},
    error::ErrorCode,
    json_schema,
    session::Session,
    shared::client::AddressbookClient,
};

/// What the server tells the agent at initialization.
const INFO: ServerInfo = ServerInfo {
    name: "cardamum",
    version: env!("CARGO_PKG_VERSION"),
    instructions: Some(INSTRUCTIONS),
};

/// How to use the tools, for the client to hand its model.
const INSTRUCTIONS: &str = "Reads, and when the server allows it changes, \
the contacts of one account. Find an addressbook with addressbook_list and \
pass its id as `addressbook`, the account's default one when left out. \
List its cards with card_list, a page at a time: give `nextPage` back as \
`page-token` for the next one. A card's `id` names it, in that \
addressbook, for card_read, card_update and card_delete. card_create and \
card_update take the card as vCard text in `vcard`, or as field flags \
(`full-name`, `email`, `phone`), never as a path. account_capability \
says what this account's backend serves; a tool it cannot serve is not \
listed. The tools that change the addressbooks or the cards are listed \
only when the server runs with --allow-write. A failed call answers \
isError with a cardamum-error object, its `code` naming the failure when \
it has one. A listing names what it could not read in `errors`, one \
cardamum-item-error object (`id`, `code`, `error`, `sources`) per card or \
addressbook; when that list is not empty the call answers isError beside \
its result, the items it read included.";

/// The leaves exposed as tools, with what each may do.
///
/// A leaf missing here is not exposed, so a new command is listed only
/// once it is classified. `card build` (a local formatter writing files)
/// and the protocol commands are left out.
const TOOLS: &[McpTool] = &[
    McpTool::read("account", "capability"),
    McpTool::read("addressbook", "list"),
    McpTool::write("addressbook", "create", false),
    McpTool::write("addressbook", "update", true),
    McpTool::write("addressbook", "delete", true),
    McpTool::read("card", "list"),
    McpTool::read("card", "read"),
    McpTool::write("card", "create", false),
    McpTool::write("card", "update", true),
    McpTool::write("card", "delete", true),
];

/// The arguments no tool takes: the composer spawns an editor, which a
/// server has no terminal for.
const COMPOSER_ARGS: &[&str] = &["interactive", "composer"];

/// Whether local file paths given as command input are read as such.
static PATHS_CLOSED: AtomicBool = AtomicBool::new(false);

/// Start an MCP server over stdio for one account.
///
/// Each tool is a unified API command (`card_list`, `card_read`) run over
/// a client connected once for the server's lifetime; the account and
/// `--backend` are the server's. Without flags only the tools that read
/// are listed, and a tool the account's backend does not serve is never
/// listed.
#[derive(Debug, Parser)]
pub struct McpCommand {
    /// List the tools that change the addressbooks and the cards too.
    ///
    /// Adds the addressbook create, update and delete tools and the card
    /// create, update and delete tools.
    #[arg(long)]
    pub allow_write: bool,
}

impl McpCommand {
    /// Serves JSON-RPC requests off stdin until EOF, answering each on
    /// stdout.
    pub fn execute(self, session: &mut Session) -> Result<()> {
        let account = session.account;

        // NOTE: resolved once up front, so a missing configuration or
        // account fails the server rather than its first call, and the
        // tools listed are the ones the account's backend serves.
        let (config, name, account_config) = resolve_account(account.config_paths, account.name)?;
        let client = AddressbookClient::new(config, name, account_config, account.backend)?;
        let backend = client.backend().to_string();

        PATHS_CLOSED.store(true, Ordering::Relaxed);

        let tools = McpToolSpec::all(self.allow_write, Some(&backend));
        let definitions: Vec<Value> = tools.iter().map(|tool| tool.definition.clone()).collect();

        let mut server = McpServer { session, tools };

        serve(&INFO, &definitions, |call| server.call(call))
    }
}

/// Whether a local file path given as command input must be taken as the
/// input itself, as it is inside an MCP server.
pub fn paths_closed() -> bool {
    PATHS_CLOSED.load(Ordering::Relaxed)
}

/// A leaf as a tool, and what it may do.
struct McpTool {
    group: &'static str,
    leaf: &'static str,
    write: bool,
    destructive: bool,
}

impl McpTool {
    const fn read(group: &'static str, leaf: &'static str) -> Self {
        Self {
            group,
            leaf,
            write: false,
            destructive: false,
        }
    }

    const fn write(group: &'static str, leaf: &'static str, destructive: bool) -> Self {
        Self {
            group,
            leaf,
            write: true,
            destructive,
        }
    }
}

/// A listed tool: its definition and the grammar a call maps onto.
struct McpToolSpec {
    grammar: Grammar,
    definition: Value,
}

impl McpToolSpec {
    /// The tools the gate allows, from the commands this build carries.
    ///
    /// With a backend, a tool it does not serve at all is left out, as
    /// the capability tables know without connecting.
    fn all(allow_write: bool, backend: Option<&str>) -> Vec<Self> {
        let mut cli = Cli::command();
        cli.build();

        let schemas = json_schema::registry().schemas;
        let mut tools = Vec::new();

        for tool in TOOLS {
            if tool.write && !allow_write {
                continue;
            }

            let served =
                backend.is_none_or(|backend| capability::serves(backend, tool.group, tool.leaf));

            if !served {
                continue;
            }

            let Some(cmd) = cli
                .find_subcommand(tool.group)
                .and_then(|group| group.find_subcommand(tool.leaf))
            else {
                continue;
            };

            let grammar = Grammar::new(&[tool.group, tool.leaf], cmd)
                .retain(|name| !COMPOSER_ARGS.contains(&name));

            let description = cmd
                .get_long_about()
                .or(cmd.get_about())
                .map(ToString::to_string)
                .unwrap_or_default();

            // NOTE: no tool reaches people beyond the account's own data,
            // which is what `openWorldHint` means across the family.
            let mut definition = json!({
                "name": format!("{}_{}", tool.group, tool.leaf),
                "description": description,
                "inputSchema": grammar.input_schema(),
                "annotations": {
                    "readOnlyHint": !tool.write,
                    "destructiveHint": tool.destructive,
                    "openWorldHint": false,
                },
            });

            let schema = schemas.get(&format!("cardamum-{}-{}", tool.group, tool.leaf));

            if let Some(schema) = schema.and_then(tool_output_schema) {
                definition["outputSchema"] = schema.clone();
            }

            tools.push(Self {
                grammar,
                definition,
            });
        }

        tools
    }
}

/// The server: the tools it lists and the session they run over.
struct McpServer<'a, 'b> {
    session: &'a mut Session<'b>,
    tools: Vec<McpToolSpec>,
}

impl McpServer<'_, '_> {
    /// Runs a tool call, listed and holding only the arguments its tool
    /// declares, a failure being a result with `isError` rather than a
    /// protocol error, so the caller reads it.
    fn call(&mut self, call: &ToolCall) -> ToolResult {
        let Some(tool) = self
            .tools
            .iter()
            .find(|tool| tool.definition["name"] == *call.name)
        else {
            return ToolResult::error::<ErrorCode>(anyhow!("Unknown tool: {}", call.name));
        };

        let mut printer = CapturePrinter::default();

        let result = tool
            .grammar
            .argv(&call.arguments)
            .and_then(|argv| {
                McpLine::try_parse_from(argv).map_err(|err| tool.grammar.usage_error(&err))
            })
            .and_then(|line| line.cmd.execute(&mut printer, self.session));

        ToolResult::printed::<ErrorCode>(printer, result)
    }
}

/// One tool call parsed back, the CLI grammar minus the binary name and the
/// global flags.
#[derive(Debug, Parser)]
#[command(no_binary_name = true)]
struct McpLine {
    #[command(subcommand)]
    cmd: Command,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(tools: &[McpToolSpec]) -> Vec<String> {
        tools
            .iter()
            .map(|tool| tool.definition["name"].as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn writes_are_listed_under_their_flag_alone() {
        let reads = names(&McpToolSpec::all(false, None));
        assert_eq!(
            reads,
            [
                "account_capability",
                "addressbook_list",
                "card_list",
                "card_read"
            ]
        );

        let all = names(&McpToolSpec::all(true, None));
        assert!(all.contains(&String::from("card_delete")), "{all:?}");
        assert_eq!(all.len(), TOOLS.len());
    }

    /// pimdir takes no addressbook write, so a server over it lists none.
    #[test]
    fn a_tool_the_backend_cannot_serve_is_not_listed() {
        let tools = names(&McpToolSpec::all(true, Some("pimdir")));
        assert!(
            !tools.iter().any(|name| name.starts_with("addressbook_c")),
            "{tools:?}"
        );
        assert!(tools.contains(&String::from("card_create")), "{tools:?}");
    }

    #[test]
    fn no_tool_takes_the_composer_nor_reaches_the_open_world() {
        for tool in McpToolSpec::all(true, None) {
            let properties = &tool.definition["inputSchema"]["properties"];
            assert!(
                properties.get("interactive").is_none(),
                "{}",
                tool.definition
            );
            assert!(properties.get("composer").is_none(), "{}", tool.definition);
            assert_eq!(tool.definition["annotations"]["openWorldHint"], false);
        }
    }
}
