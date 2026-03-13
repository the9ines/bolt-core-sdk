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
//! - State-machine authority (AC-RC-10: transfer SM, BTR SM, backpressure)
//!
//! TS-owned invariants (NOT tested here — see AAR):
//! - Handshake gating (WebRTCService state machine) — AC-RC-07 scope
//! - Downgrade resistance (capability negotiation) — AC-RC-07 scope
//! - HELLO exactly-once enforcement (WebRTCService) — AC-RC-07 scope
//! - Appendix A error frame codes (transport-level)

#[cfg(feature = "vectors")]
mod envelope_validation;

#[cfg(feature = "vectors")]
mod sas_determinism;

mod error_code_mapping;
mod state_machine_authority;
mod wire_error_registry;
