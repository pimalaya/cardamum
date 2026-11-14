---
cairn: delta
change: the-list-frame-is-shared
---

# Delta

Every requirement lives in commands.md.

## ADDED Requirements

## MODIFIED Requirements

### Requirement: A card page names its context and its next page
`card list` SHALL print as a pimalaya-cli `listing::Listing`: above the cards `Account: <name> (<backend>)`, `Addressbook: <id>`, `Page: <n>, cards <a> to <b>` (`card <a>` for one, `empty` for none, `all` when unpaged) followed by `, the last` when no card follows, and `Next page: --page <n>` while one does, the values aligned in one column; then a blank line and one line per card with no column header (`id  email  phone  name`, the free-text name last, `-` for an empty cell). Below the cards SHALL print only the command's errors ("A command's errors print at its end").

### Requirement: A command's errors print at its end
In the text output, each `errors` entry SHALL print as one `Error:` line, `<id>  <code>  <error>` with `-` for a null id or code, last in the command's output: after the items and one blank line, the `Error:` lines aligned among themselves (pimalaya-cli `error::ItemErrors`). Nothing SHALL print, the blank line included, when `errors` is empty.

### Requirement: An account lists what it serves
`account capability` SHALL print pimalaya-cli's `capability::AccountCapabilityOutput`, connecting nowhere: `{"account", "capabilities": [{"name", "backend", "level", "detail"}]}`, one row per capability, `backend` the backend the unified commands resolve to (`--backend` honoured), `level` one of `full`, `partial`, `server` and `none`, `detail` what the level leaves out or `null`. When the account configures no backend `--backend` allows, every row SHALL read `backend` `null` and level `none`, and the command SHALL succeed. The text SHALL be a list headed by `Account: <name>`, then one line per capability: the name, the backend, the level and the detail.

#### Scenario: A backend the account lacks
- GIVEN an account configuring vdir alone
- WHEN `cardamum --json --backend pimdir account capability` runs
- THEN every row has `"backend": null` and `"level": "none"`, and the exit status is 0

## REMOVED Requirements
