# @the9ines/bolt-core

Core crypto primitives and utilities for the Bolt Protocol.

## Scope

This package provides transport-agnostic building blocks consumed by all Bolt ecosystem products:

- **Crypto**: NaCl box seal/open, ephemeral keypair generation
- **Peer codes**: Generation and validation using unambiguous base32 alphabet
- **Hashing**: SHA-256, file hashing, hex encoding
- **SAS**: Short Authentication String computation per Bolt Protocol spec
- **Errors**: Core error types (BoltError, EncryptionError, ConnectionError, TransferError)
- **Constants**: Protocol constants (nonce length, chunk size, key sizes, etc.)
- **Encoding**: Base64 encode/decode helpers

## What this package does NOT include

- WebRTC, WebSocket, or any transport logic
- Signaling or discovery
- UI components
- Handshake state machines or HELLO message handling
- Identity persistence or TOFU pinning

## Usage

```typescript
import {
  sealBoxPayload,
  openBoxPayload,
  generateEphemeralKeyPair,
  generateSecurePeerCode,
  computeSas,
  DEFAULT_CHUNK_SIZE,
} from '@the9ines/bolt-core';

// Generate ephemeral keys for a connection
const myKeys = generateEphemeralKeyPair();

// Encrypt a chunk
const sealed = sealBoxPayload(plaintext, remotePubKey, myKeys.secretKey);

// Decrypt a chunk
const opened = openBoxPayload(sealed, remotePubKey, myKeys.secretKey);
```

## Build

```bash
npm install
npm run build   # tsc -> dist/
npm run test    # vitest
```
