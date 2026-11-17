---
cairn: delta
change: anonymous-dav-and-punycode-hosts
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: A DAV server may ask for no credentials (config)

`carddav.auth` is optional, absent reading as `auth = "none"`; a 401 to such a block says the account configures no `auth`, its code unchanged. Two scenarios added.

### Requirement: Hosts are ASCII (packaging)

A CardDAV `server` or `home` and a JMAP `server` with a non-ASCII host are refused naming the host and its Punycode form; a CardDAV `server` is never prefixed into host `https`. One scenario added.

## REMOVED Requirements

None.
