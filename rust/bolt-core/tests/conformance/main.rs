//! S1 Conformance Harness — Core Protocol Invariant Tests
//!
//! Enforces MUST-level core protocol invariants from PROTOCOL.md and
//! PROTOCOL_ENFORCEMENT.md using bolt-core-sdk's own H3 golden vectors.
//!
//! Invariant coverage:
//! - Envelope roundtrip determinism (PROTO-01, PROTO-07)
//! - MAC verification enforcement (SEC-06)
//! - Nonce / replay sanity (SEC-01, SEC-02)
//! - SAS determinism (PROTO-06)
//! - Error code mapping (Appendix A, Rust-surface only)
//!
//! TS-owned invariants (NOT tested here — see AAR):
//! - Handshake gating (WebRTCService state machine)
//! - Downgrade resistance (capability negotiation)
//! - HELLO exactly-once enforcement (WebRTCService)
//! - Appendix A error frame codes (transport-level)

#[cfg(feature = "vectors")]
mod envelope_validation;

#[cfg(feature = "vectors")]
mod sas_determinism;

mod error_code_mapping;
mod wire_error_registry;
