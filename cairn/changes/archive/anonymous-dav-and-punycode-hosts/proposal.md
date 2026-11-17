---
cairn: change
id: anonymous-dav-and-punycode-hosts
status: landed
created: 2026-11-17
---

# Anonymous DAV and Punycode hosts

## Why

Audit line COH-6 of the CLI family analysis (2026-11-15) and two owner rulings of 2026-11-17. A `[carddav]` block with no `auth` loaded in carillon and failed "missing field `auth`" in cardamum, calendula and neverest; a non-ASCII host was refused in some apps and accepted in others. The rulings: `auth` is optional everywhere, absent sending no `Authorization` header, a 401 then saying the account configures no `auth`; a non-ASCII host is refused everywhere, the message pointing at its Punycode form.

## What

- `carddav.auth` optional (`#[serde(default)]`, `CarddavAuthConfig::None` the default), `auth = "none"` still accepted.
- A 401 to a block with no `auth` fails with "CardDAV server answered 401: it asks for credentials, and the account configures no `auth`", its code unchanged (`null`), on the unified commands, the `carddav` namespace and `account check`.
- A CardDAV `server` or `home` and a JMAP `server` whose host is not ASCII are refused with "Host `<host>` is not ASCII: write it in its Punycode form (xn--...)". A full `https://` CardDAV `server` with such a host was read as the host `https` once prefixed; it is refused instead.
- `carddav.home` is read as a string and parsed by cardamum, so url no longer needs its `serde` feature.
