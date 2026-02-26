//! Transfer policy — pure, deterministic chunk scheduling.
//!
//! This module is greenfield S2 infrastructure. The Rust policy core
//! makes scheduling decisions without IO, clocks, or global state.
//! A future WASM build will expose `decide()` to the TypeScript
//! transfer runtime (`bolt-transport-web / WebRTCService`).
//!
//! # S2 Scope
//!
//! - S2A (this phase): Policy types + stub + contract tests.
//! - S2B (future): WASM build, TS adapter, measurement harness.

pub mod policy;
pub mod types;

// Re-export the canonical entrypoint and core types.
pub use policy::decide;
pub use types::{
    Backpressure, ChunkId, DeviceClass, FairnessMode, LinkStats, PolicyInput, ScheduleDecision,
    TransferConstraints, MAX_PACING_DELAY_MS,
};
