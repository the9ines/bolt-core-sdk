# Bolt Core SDK

Open reference implementation of the [Bolt Protocol](https://github.com/the9ines/bolt-protocol) for encrypted device-to-device file transfer.

---

## What This Is

The SDK that all Bolt-based applications embed to speak Bolt Core. One codepath for pairing, TOFU pinning, handshake gating, envelope encryption, and the transfer state machine.

The protocol specification lives in [bolt-protocol](https://github.com/the9ines/bolt-protocol). This repository contains only the implementation.

---

## SDK Goals

- Pluggable Profile bindings: apps provide Profile adapters (rendezvous + peer channel + encoding hooks)
- Deterministic behavior: canonical serialization per Profile, stable state machine, strict conformance checks
- Small surface area: minimal public API, strong invariants, reliable defaults
- Transport-agnostic at Core level

## SDK Non-Goals

- Does not implement discovery or routing
- Does not define UI or product policy
- Does not require a daemon

---

## Role in the Ecosystem

| Relationship | Repository |
|-------------|-----------|
| Specification | [bolt-protocol](https://github.com/the9ines/bolt-protocol) |
| Depends on this | [localbolt](https://github.com/the9ines/localbolt), [localbolt-app](https://github.com/the9ines/localbolt-app), [localbolt-v3](https://github.com/the9ines/localbolt-v3), [bytebolt-app](https://github.com/the9ines/bytebolt-app), [bolt-daemon](https://github.com/the9ines/bolt-daemon) |

## Packaging

- **Rust crate**: `bolt-core`
- **TypeScript package**: `@the9ines/bolt-core`

---

## License

MIT
