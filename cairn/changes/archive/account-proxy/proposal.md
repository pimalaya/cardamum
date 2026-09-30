---
cairn: change
id: account-proxy
status: landed
created: 2026-09-30
---

# Proxy per account and per backend

## Why

pimalaya-stream 0.3 routes every connection through a `Proxy`, io-jmap and io-msgraph 0.4 expose it on their connect options, and Himalaya already configures it per account. Cardamum had no way to name a proxy, so its network backends could only follow the environment.

## What

Himalaya's model, key for key: an account-level `proxy` (`url`, `username`, `password`) inherited by every network backend naming none, and `<backend>.proxy` overriding it for one. JMAP and Graph pass it through their connect options. io-webdav and io-people take no proxy, so Cardamum opens the stream itself and hands it to their stream constructors; the CardDAV `.well-known` probes go through it too.

## Scope / non-goals

- CardDAV `discover` lookups stay on the environment proxy: io-pim-discovery takes none.
- The wizard writes no proxy.
