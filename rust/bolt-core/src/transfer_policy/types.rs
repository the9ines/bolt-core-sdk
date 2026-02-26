//! Transfer policy types — greenfield performance infrastructure.
//!
//! These types define the interface for a pure, deterministic transfer
//! scheduling policy. The policy core has NO IO, NO clocks, NO global
//! state. It receives observed link/transfer state and returns scheduling
//! decisions.
//!
//! # WASM Consumption Path
//!
//! These types are designed to be WASM-friendly (no references, no
//! lifetimes, simple enums). A future `wasm-bindgen` build will expose
//! them to the TypeScript transfer runtime (`bolt-transport-web`).
//! That integration is out of scope for S2A.

/// Opaque chunk identifier, caller-owned.
/// The policy never creates or inspects chunk content — it only
/// schedules chunk IDs provided by the caller.
pub type ChunkId = u32;

/// Observed link statistics, provided by the caller each decision round.
///
/// All values are point-in-time observations. The policy does not
/// maintain history — callers may smooth/filter before providing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinkStats {
    /// Round-trip time in milliseconds.
    pub rtt_ms: u32,
    /// Packet loss in parts-per-million (0 = no loss, 1_000_000 = 100%).
    pub loss_ppm: u32,
    /// Bytes currently in-flight (sent but not yet acknowledged).
    pub in_flight_bytes: u32,
}

/// Performance-tier classification of the local device.
///
/// This is independent of `bolt-rendezvous-protocol::DeviceType`
/// (Phone | Tablet | Laptop | Desktop). The TypeScript caller is
/// responsible for mapping any runtime signal into `DeviceClass`.
/// Rust does not derive or inspect device type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceClass {
    Desktop,
    Mobile,
    LowPower,
    Unknown,
}

/// Constraints for a single transfer, provided by the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransferConstraints {
    /// Maximum number of chunks that may be in-flight simultaneously.
    pub max_parallel_chunks: u16,
    /// Maximum total bytes allowed in-flight.
    pub max_in_flight_bytes: u32,
    /// Transfer priority (0 = lowest, 255 = highest).
    pub priority: u8,
    /// Scheduling fairness mode.
    pub fairness_mode: FairnessMode,
}

/// Scheduling fairness mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FairnessMode {
    /// Balance throughput and latency equally.
    Balanced,
    /// Maximize throughput (larger windows, less pacing).
    Throughput,
    /// Minimize latency (smaller windows, more pacing).
    Latency,
}

/// Input to the policy decision function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyInput {
    /// Chunk IDs available to send (caller-provided, not yet in-flight).
    pub pending_chunk_ids: Vec<ChunkId>,
    /// Current link observations.
    pub link_stats: LinkStats,
    /// Device performance tier.
    pub device_class: DeviceClass,
    /// Transfer constraints.
    pub constraints: TransferConstraints,
}

/// Backpressure signal from the policy to the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backpressure {
    /// Caller should pause sending.
    Pause,
    /// Caller should resume sending.
    Resume,
    /// No change to current send/pause state.
    NoChange,
}

/// Output of the policy decision function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleDecision {
    /// Chunk IDs to send in this round (ordered).
    pub next_chunk_ids: Vec<ChunkId>,
    /// Suggested delay before next decision round (milliseconds).
    pub pacing_delay_ms: u32,
    /// Suggested send window size in chunks.
    pub window_suggestion_chunks: u16,
    /// Backpressure signal.
    pub backpressure: Backpressure,
}

/// Maximum allowed pacing delay (milliseconds).
/// Any policy implementation MUST NOT return a pacing_delay_ms
/// exceeding this value.
pub const MAX_PACING_DELAY_MS: u32 = 5_000;
