---
cairn: delta
change: a-request-off-its-api-is-coded
---

# Delta

Each requirement names its capability file in parentheses.

## MODIFIED Requirements

### Requirement: A raw request stays on the provider's API origin
(commands) The refusal says `Cannot send a request to <origin>: it is off the API origin <base origin>, which alone receives the account's token`, coded `request-origin` (pimalaya-cli's `error::request_origin`), exit 1.

### Requirement: A JSON error carries a stable code
(commands) The table gains `request-origin`: `msgraph request` or `gpeople request` was given a URL off the API origin; nothing was sent. The failure object's schema types `code` by pimalaya-cli's common codes too (`usage-invalid`, `request-origin`).
