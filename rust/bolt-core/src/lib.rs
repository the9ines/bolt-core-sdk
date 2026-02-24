//! Bolt Core — canonical reference implementation.
//!
//! This crate is the canonical source of truth for Bolt protocol crypto
//! primitives and constants. The TypeScript SDK (`@the9ines/bolt-core`)
//! is a supported adapter implementation that MUST produce identical
//! outputs for identical inputs, verified by shared golden test vectors.
//!
//! # Module Map
//!
//! | Module | TS Equivalent | Status |
//! |--------|---------------|--------|
//! | [`constants`] | `constants.ts` | Complete |
//! | [`errors`] | `errors.ts` | Complete |
//! | [`encoding`] | `encoding.ts` | Stub (R1) |
//! | [`crypto`] | `crypto.ts` | Stub (R1) |
//! | [`hash`] | `hash.ts` | Stub (R2) |
//! | [`identity`] | `identity.ts` | Stub (R2) |
//! | [`sas`] | `sas.ts` | Stub (R2) |
//! | [`peer_code`] | `peer-code.ts` | Partial (R3) |
//! | [`vectors`] | N/A | Complete (test-only) |
//!
//! # Parity Strategy
//!
//! TS generates golden test vectors. Rust consumes them. The vector
//! files in `ts/bolt-core/__tests__/vectors/` are the single source
//! of truth. See `RUST_CORE_PLAN.md` for the full parity strategy.

/// Protocol constants — values shared with TypeScript SDK.
pub mod constants;

/// Error types for bolt-core operations.
pub mod errors;

/// Encoding utilities — base64 and hex.
pub mod encoding;

/// Crypto primitives — NaCl box (XSalsa20-Poly1305).
pub mod crypto;

/// Hashing utilities — SHA-256.
pub mod hash;

/// Identity — long-lived keypairs and TOFU error.
pub mod identity;

/// SAS — Short Authentication String computation.
pub mod sas;

/// Peer code generation and validation.
pub mod peer_code;

/// Deterministic golden vector generator (test use only).
/// Requires the `vectors` feature: `cargo test --features vectors`.
#[cfg(feature = "vectors")]
pub mod vectors;
