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

| Target | Package Name | Registry |
|--------|-------------|----------|
| Rust | `bolt-core` | [crates.io](https://crates.io) |
| TypeScript | `@the9ines/bolt-core` | [npm](https://www.npmjs.com) |

Both packages follow the same version number. A single SDK release produces both artifacts.

## Publishing Strategy

1. All changes land on `main` via PR.
2. Tag with `sdk-vX.Y.Z` following semver.
3. CI publishes both crate and npm package from the same tag.
4. Breaking protocol changes require a major version bump.
5. Profile-only changes (no Core impact) are minor or patch.

## Compatibility Matrix

| SDK Version | Bolt Core Spec | Products |
|------------|----------------|----------|
| sdk-v1.0.x | Core v1 | localbolt, localbolt-app, localbolt-v3, bytebolt-app |

Products declare a minimum SDK version in their dependency manifest. The SDK guarantees backward compatibility within the same major version.

---

## License

MIT
