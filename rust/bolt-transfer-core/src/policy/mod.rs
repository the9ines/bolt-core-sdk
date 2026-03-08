//! Transfer policy — pure, deterministic chunk scheduling.
//!
//! Migrated from `bolt-core::transfer_policy` in S2A. This module is the
//! single authoritative location for transfer scheduling decisions.
//!
//! # Design
//!
//! - **Pure**: No IO, no clocks, no global state.
//! - **Deterministic**: Identical inputs always produce identical outputs.
//! - **Transport-agnostic**: Decision engine only; callers own transport I/O.
//! - **WASM-compatible**: All types are wasm-bindgen-friendly.
//!
//! # Backpressure Authority
//!
//! The [`BackpressureController`](crate::BackpressureController) is the sole
//! evaluator of transport pressure (watermark hysteresis). Its output feeds
//! into [`PolicyInput::pressure`] as a [`PressureState`]. The policy emits
//! a single [`Backpressure`] signal — no dual-path overlap.

pub mod decide;
pub mod progress;
pub mod stall;
pub mod types;

pub use decide::decide;
pub use progress::{progress_cadence, ProgressConfig, ProgressReport};
pub use stall::{detect_stall, StallClassification, StallInput};
pub use types::{
    Backpressure, ChunkId, DeviceClass, FairnessMode, LinkStats, PolicyInput, PressureState,
    ScheduleDecision, TransferConstraints, MAX_PACING_DELAY_MS,
};
