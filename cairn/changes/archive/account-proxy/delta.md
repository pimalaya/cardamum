---
cairn: change
change: account-proxy
---

# Delta

## ADDED Requirements

### Requirement: A proxy is set per account and per backend
An account's `proxy` SHALL apply to every network backend of that account (CardDAV, JMAP, Microsoft Graph, Google People) whose own block names no `proxy`, and a backend's own `proxy` SHALL win over it. With neither, the connection SHALL read the `all_proxy` and `https_proxy` environment variables, honouring `no_proxy`. CardDAV `discover` lookups SHALL read the environment only, io-pim-discovery taking no proxy.

`proxy.url` SHALL take `socks5://`, `socks5h://` or `http://`. `proxy.username` and `proxy.password` SHALL override the URL's user info, the password being a secret like any credential; a password without a username SHALL be rejected.

## MODIFIED Requirements

## REMOVED Requirements
