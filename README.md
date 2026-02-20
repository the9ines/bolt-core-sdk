# Bolt Protocol

Bolt is an encrypted device-to-device transport protocol for secure file transfer. It defines identity, pairing, session security, message semantics, and state machines independently of any specific transport layer.

## Documents

| Document | Description |
|----------|-------------|
| [PROTOCOL.md](PROTOCOL.md) | Bolt Protocol Core v1. Transport-agnostic specification. |
| [LOCALBOLT_PROFILE.md](LOCALBOLT_PROFILE.md) | LocalBolt Profile v1. Transport binding for WebRTC. |

## Core vs Profiles

Bolt is structured as a **protocol family**:

- **Bolt Core** defines what happens: identity, encryption, message types, state machines, conformance rules. It contains no transport-specific details.
- **Profiles** define how it happens: signaling mechanism, transport channel, encoding format, framing, and transport-specific policies.

This separation allows the same protocol to run over different transports:

- **LocalBolt Profile** -- Browser WebRTC, local network, JSON encoding
- **ByteBolt Profile** (future) -- libdatachannel, any network, binary encoding

A ByteBolt implementation MUST support the LocalBolt Profile when it detects a LocalBolt peer on LAN.

## Versioning

- Protocol versions: Bolt Core v1, v2, etc.
- Profile versions: LocalBolt Profile v1, ByteBolt Profile v1, etc.
- Profiles declare which Core version they implement
- Version negotiation happens during the HELLO handshake

## Non-goals

Bolt is not:

- A transport protocol (it runs on top of transports)
- A network routing protocol
- A NAT traversal specification
- A storage format
- A metadata privacy / traffic analysis resistance system

## Design Principles

- No novel cryptography. Proven primitives only (NaCl box: X25519 + XSalsa20-Poly1305).
- Transport independence. Security does not depend on transport-layer encryption.
- Minimal complexity. The smallest protocol that provides the required security properties.

## Status

| Document | Version | Status |
|----------|---------|--------|
| Bolt Core v1 | 1.0.0 | Draft |
| LocalBolt Profile v1 | 1.0.0 | Draft |
