//! Bolt Transfer Ratchet (BTR) — reference implementation.
//!
//! Implements the Bolt Transfer Ratchet protocol extension (PROTOCOL.md §16):
//! per-transfer DH ratchet + per-chunk symmetric chain for forward secrecy
//! and transfer isolation.
//!
//! # Architecture
//!
//! BTR sits between the handshake layer (bolt-core) and the transfer state
//! machine (bolt-transfer-core). It is transport-agnostic: no I/O, no async,
//! no network dependencies.
//!
//! # Module Map
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`constants`] | HKDF info strings and key lengths (§14) |
//! | [`errors`] | BTR error types (§16.7) |
//! | [`key_schedule`] | HKDF-SHA256 derivation chain (§16.3) |
//! | [`ratchet`] | Inter-transfer DH ratchet (§16.3) |
//! | [`encrypt`] | NaCl secretbox with BTR message keys (§16.4) |
//! | [`state`] | Session/transfer engine and lifecycle (§16.5) |
//! | [`replay`] | Replay rejection guard (§11) |
//! | [`negotiate`] | Capability negotiation matrix (§4) |
//! | [`vectors`] | Golden vector generator (test-only, feature-gated) |
//!
//! # Security Properties
//!
//! - **REPLAY-BTR**: (transfer_id, generation, chain_index) triple prevents replay
//! - **ISOLATION-BTR**: Each transfer has independent root key
//! - **ORDER-BTR**: No skipped-key buffer; chain_index must be monotonic
//! - **EPOCH-BTR**: DH ratchet at transfer boundary provides self-healing
//! - All key material is memory-only (no persistence)
//! - All secret-holding structs implement zeroize-on-drop

/// BTR-specific constants — HKDF info strings (§14).
pub mod constants;

/// BTR error types (§16.7).
pub mod errors;

/// HKDF-SHA256 key schedule (§16.3).
pub mod key_schedule;

/// Inter-transfer DH ratchet (§16.3).
pub mod ratchet;

/// NaCl secretbox encryption with BTR message keys (§16.4).
pub mod encrypt;

/// Session and transfer state engine (§16.5).
pub mod state;

/// Replay rejection guard (§11).
pub mod replay;

/// Capability negotiation matrix (§4).
pub mod negotiate;

/// BTR golden vector generator (test-only).
/// Requires the `vectors` feature: `cargo test --features vectors`.
#[cfg(feature = "vectors")]
pub mod vectors;

// Re-exports for convenience.
pub use errors::BtrError;
pub use negotiate::{negotiate_btr, BtrMode};
pub use state::{BtrEngine, BtrTransferContext};
