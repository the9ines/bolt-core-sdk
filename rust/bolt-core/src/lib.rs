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
//! | [`encoding`] | `encoding.ts` | Complete |
//! | [`crypto`] | `crypto.ts` | Complete |
//! | [`hash`] | `hash.ts` | Complete |
//! | [`identity`] | `identity.ts` | Complete |
//! | [`sas`] | `sas.ts` | Complete |
//! | [`peer_code`] | `peer-code.ts` | Complete |
//! | [`session`] | WebRTCService (TS-owned) | Rust-canonical (AC-RC-07) |
//! | [`vectors`] | N/A | Complete (test-only) |
//!
//! Transfer policy has been moved to `bolt-transfer-core::policy` (S2A).
//!
//! # Parity Strategy
//!
//! Rust is the canonical source of all golden test vectors (AC-RC-08).
//! Vector files live in `test-vectors/core/` and `test-vectors/btr/`.
//! Both Rust and TS test suites consume from these locations.
//! TS vector generation is deprecated (AC-RC-09).
//! See `VECTOR_AUTHORITY.md` for the migration details.

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

/// Session authority — transport-agnostic handshake lifecycle primitives.
pub mod session;

/// Deterministic golden vector generator (test use only).
/// Requires the `vectors` feature: `cargo test --features vectors`.
#[cfg(feature = "vectors")]
pub mod vectors;
