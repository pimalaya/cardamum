---
cairn: log
change: account-proxy
landed: 2026-09-30
---

# Proxy per account and per backend

An account's `proxy` block now reaches every network backend naming none of its own, and `<backend>.proxy` overrides it, as in Himalaya. Neither falls back to the `all_proxy` and `https_proxy` environment variables, which is also what the io-jmap and io-msgraph 0.4 bump started doing for those two backends.

JMAP and Graph take the proxy through their connect options. io-webdav 0.4 and io-people 0.3 take none, so CardDAV and People open the stream themselves and hand it to the libraries' stream constructors; the CardDAV `.well-known` probes follow the same proxy. CardDAV `discover` lookups do not, io-pim-discovery exposing no proxy.

Verified with 79 tests passing and clippy clean over all features and over each backend alone. No live account or proxy was reached.

The [config](../spec/config.md) capability moved: one requirement added.
