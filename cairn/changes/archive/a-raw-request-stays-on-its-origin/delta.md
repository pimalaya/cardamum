---
cairn: delta
change: a-raw-request-stays-on-its-origin
---

# Delta

## ADDED Requirements

### Requirement: A raw request stays on the provider's API origin (commands)

`msgraph request` and `gpeople request` refuse, before sending, a URL off the API base's origin (`https://graph.microsoft.com`, `https://people.googleapis.com`); `jmap request` posts to the session's `apiUrl` alone.

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
