# bolt-core (Rust)

Canonical reference implementation of the Bolt Protocol core.

## Role

This crate is the **canonical** SDK for Bolt protocol crypto primitives,
constants, and payload sealing. It defines the authoritative behavior
that all implementations (including the TypeScript `@the9ines/bolt-core`
adapter) MUST match.

## Status

**Scaffold.** Crypto primitives are not yet implemented. The initial
deliverable is:

1. Constants matching the TypeScript SDK.
2. Vector compatibility tests that parse and validate the existing
   golden test vectors from `ts/bolt-core/__tests__/vectors/`.

## Authority

See [docs/SDK_AUTHORITY.md](../../docs/SDK_AUTHORITY.md) for the full
authority model and interop gate requirements.

## License

MIT
