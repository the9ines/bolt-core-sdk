# Bolt Protocol

Bolt is a protocol layer for encrypted device-to-device file transfer. It defines identity, pairing, message protection, message semantics, and state machines independently of any specific transport.

---

## Documents

| Document | Description |
|----------|-------------|
| [PROTOCOL.md](PROTOCOL.md) | Bolt Protocol Core v1. Transport-agnostic specification. |
| [LOCALBOLT_PROFILE.md](LOCALBOLT_PROFILE.md) | LocalBolt Profile v1. Transport binding for the browser peer channel. |

---

## Core vs Profiles

Bolt is structured as a **protocol family**:

- **Bolt Core** defines what happens: identity, encryption, message types, state machines, conformance rules. It contains no transport-specific details.
- **Profiles** define how it happens: rendezvous mechanism, transport channel, encoding format, framing, and transport-specific policies.

This separation allows the same protocol to run over different transports:

- **LocalBolt Profile** -- Browser peer channel, local scope policy, `json-envelope-v1`
- **ByteBolt Profile** (future) -- Broader scope, binary envelope encoding

---

## Bolt Core SDK

**Bolt Core SDK** is the reference implementation layer that applications embed to speak Bolt Core.

Goals:

- One codepath for all apps: pairing, TOFU pinning, handshake gating, envelope encryption, transfer state machine
- Pluggable Profile bindings: apps provide Profile adapters (rendezvous + peer channel + encoding hooks)
- Deterministic behavior: canonical serialization per Profile, stable state machine, strict conformance checks
- Small surface area: minimal public API, strong invariants, reliable defaults

Non-goals:

- The SDK does not implement discovery or routing across networks by itself
- The SDK does not define UI or product policy beyond the Core requirements
- The SDK does not require a daemon; daemon usage is optional and Profile/app-specific

---

## Bolt Daemon

The **Bolt Daemon** is a minimal Rust service that provides reliable, low-resource background capability for Bolt-based apps on supported platforms.

Primary responsibilities (planned):

- Maintain device identity key storage and pinned peer store (TOFU)
- Provide a stable local API for apps to initiate or accept transfers
- Enforce resource limits and policy defaults consistently
- Implement optional background behaviors: keepalive, retry scheduling, rate limiting, logging

Design constraints:

- Low memory footprint
- Low idle CPU usage (event-driven, no busy polling)
- Reliable under network churn and app restarts
- Clear failure modes (crash-only acceptable if supervised, with clean state)

Repo inclusion policy:

- Included in: `localbolt` repo, `localbolt-app` repo
- Not included in: `localbolt-v3` repo (web app; daemon not applicable)

---

## Versioning

- Core versions: Bolt Core v1, v2, etc.
- Profile versions: LocalBolt Profile v1, ByteBolt Profile v1, etc.
- Profiles declare which Core version they implement
- Version and capability negotiation happens during the HELLO handshake

---

## Non-goals

Bolt is not:

- A transport protocol (it runs on top of transports)
- A network routing protocol
- A NAT traversal specification
- A storage format
- A metadata privacy or traffic analysis resistance system

---

## Design Principles

- No novel cryptography. Proven primitives only (NaCl box: X25519 + XSalsa20-Poly1305).
- Transport independence. Security does not depend on transport-layer encryption.
- Minimal complexity. The smallest protocol that provides the required security properties.

---

## Status

| Document | Version | Status |
|----------|---------|--------|
| Bolt Core v1 | 1.0.0 | Draft |
| LocalBolt Profile v1 | 1.0.0 | Draft |
