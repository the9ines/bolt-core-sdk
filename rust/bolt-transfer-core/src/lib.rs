//! Bolt Transfer Core — transport-agnostic transfer state machine.
//!
//! This crate implements the canonical transfer state machine from
//! PROTOCOL.md §9. It is consumed by bolt-daemon (and in future by
//! localbolt-app and WASM targets).
//!
//! # Design Principles
//!
//! - **Transport-agnostic**: No network I/O, no WebRTC, no IPC.
//!   Transport observation is via the [`TransportQuery`] trait.
//! - **Crypto-free**: No hash or encryption dependencies.
//!   Integrity verification is optional via [`IntegrityVerifier`].
//! - **Pure state machines**: Deterministic transitions, no async,
//!   no side effects.
//! - **WASM-compatible**: Compiles to `wasm32-unknown-unknown`.
//!
//! # Module Map
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`state`] | Canonical §9 state enums |
//! | [`error`] | Transfer error types |
//! | [`send`] | Send-side state machine |
//! | [`receive`] | Receive-side state machine |
//! | [`backpressure`] | Watermark-based flow control |
//! | [`transport`] | Transport/integrity trait interfaces |
//! | [`policy`] | Pure deterministic scheduling policy (S2A) |

/// Canonical transfer state enums (PROTOCOL.md §9).
pub mod state;

/// Transfer error types.
pub mod error;

/// Send-side transfer state machine.
pub mod send;

/// Receive-side transfer state machine.
pub mod receive;

/// Backpressure — high/low watermark pause-resume.
pub mod backpressure;

/// Transport and integrity trait interfaces.
pub mod transport;

/// Transfer policy — pure, deterministic chunk scheduling (S2A).
pub mod policy;

// Re-export primary types for convenience.
pub use backpressure::{BackpressureConfig, BackpressureController};
pub use error::TransferError;
pub use policy::{
    decide, Backpressure, ChunkId, DeviceClass, FairnessMode, LinkStats, PolicyInput,
    PressureState, ScheduleDecision, TransferConstraints,
};
pub use receive::ReceiveSession;
pub use send::{SendChunk, SendOffer, SendSession};
pub use state::{CancelReason, TransferState};
pub use transport::{IntegrityVerifier, TransportQuery};
