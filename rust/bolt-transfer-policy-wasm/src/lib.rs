//! WASM bindings for bolt-transfer-core policy decisions.
//!
//! Thin wrapper crate — exposes policy functions to JavaScript via
//! wasm-bindgen. All policy logic lives in bolt-transfer-core; this
//! crate only handles boundary serialization.
//!
//! # Export Surface
//!
//! | Export | Purpose |
//! |--------|---------|
//! | `policy_decide` | Chunk scheduling decision |
//! | `policy_detect_stall` | Stall classification |
//! | `policy_progress_cadence` | Progress emission gate |
//! | Enums: `DeviceClass`, `FairnessMode`, `PressureState`, `Backpressure` | Configuration constants |

use wasm_bindgen::prelude::*;

use bolt_transfer_core::policy::{
    decide, detect_stall, progress_cadence, Backpressure, DeviceClass, FairnessMode, LinkStats,
    PolicyInput, PressureState, ProgressConfig, StallInput, TransferConstraints,
};

// ─── Enum re-exports with wasm_bindgen ────────────────────────────────

/// Device performance tier.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmDeviceClass {
    Desktop = 0,
    Mobile = 1,
    LowPower = 2,
    Unknown = 3,
}

/// Scheduling fairness mode.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmFairnessMode {
    Balanced = 0,
    Throughput = 1,
    Latency = 2,
}

/// Backpressure state input.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmPressureState {
    Clear = 0,
    Elevated = 1,
    Pressured = 2,
}

/// Backpressure signal output.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmBackpressure {
    Pause = 0,
    Resume = 1,
    NoChange = 2,
}

// ─── Enum conversion helpers ──────────────────────────────────────────

impl From<WasmDeviceClass> for DeviceClass {
    fn from(v: WasmDeviceClass) -> Self {
        match v {
            WasmDeviceClass::Desktop => DeviceClass::Desktop,
            WasmDeviceClass::Mobile => DeviceClass::Mobile,
            WasmDeviceClass::LowPower => DeviceClass::LowPower,
            WasmDeviceClass::Unknown => DeviceClass::Unknown,
        }
    }
}

impl From<WasmFairnessMode> for FairnessMode {
    fn from(v: WasmFairnessMode) -> Self {
        match v {
            WasmFairnessMode::Balanced => FairnessMode::Balanced,
            WasmFairnessMode::Throughput => FairnessMode::Throughput,
            WasmFairnessMode::Latency => FairnessMode::Latency,
        }
    }
}

impl From<WasmPressureState> for PressureState {
    fn from(v: WasmPressureState) -> Self {
        match v {
            WasmPressureState::Clear => PressureState::Clear,
            WasmPressureState::Elevated => PressureState::Elevated,
            WasmPressureState::Pressured => PressureState::Pressured,
        }
    }
}

impl From<Backpressure> for WasmBackpressure {
    fn from(v: Backpressure) -> Self {
        match v {
            Backpressure::Pause => WasmBackpressure::Pause,
            Backpressure::Resume => WasmBackpressure::Resume,
            Backpressure::NoChange => WasmBackpressure::NoChange,
        }
    }
}

// ─── DTO types for boundary crossing ──────────────────────────────────

/// Schedule decision result — returned from `policy_decide`.
///
/// `next_chunk_ids` is accessed via the `next_chunk_ids()` method
/// which returns a `Uint32Array`.
#[wasm_bindgen]
pub struct WasmScheduleDecision {
    chunk_ids: Vec<u32>,
    pacing_delay_ms: u32,
    window_suggestion_chunks: u16,
    backpressure: WasmBackpressure,
    effective_chunk_size: u32,
}

#[wasm_bindgen]
impl WasmScheduleDecision {
    /// Chunk IDs to send this round, as a Uint32Array.
    #[wasm_bindgen(js_name = nextChunkIds)]
    pub fn next_chunk_ids(&self) -> js_sys::Uint32Array {
        js_sys::Uint32Array::from(&self.chunk_ids[..])
    }

    /// Suggested delay (ms) before next decision round.
    #[wasm_bindgen(js_name = pacingDelayMs, getter)]
    pub fn pacing_delay_ms(&self) -> u32 {
        self.pacing_delay_ms
    }

    /// Suggested send window size in chunks.
    #[wasm_bindgen(js_name = windowSuggestionChunks, getter)]
    pub fn window_suggestion_chunks(&self) -> u16 {
        self.window_suggestion_chunks
    }

    /// Backpressure signal.
    #[wasm_bindgen(getter)]
    pub fn backpressure(&self) -> WasmBackpressure {
        self.backpressure
    }

    /// Effective chunk size after transport cap (bytes).
    #[wasm_bindgen(js_name = effectiveChunkSize, getter)]
    pub fn effective_chunk_size(&self) -> u32 {
        self.effective_chunk_size
    }

    /// Number of chunk IDs in this round.
    #[wasm_bindgen(js_name = chunkCount, getter)]
    pub fn chunk_count(&self) -> u32 {
        self.chunk_ids.len() as u32
    }
}

/// Stall detection result — returned from `policy_detect_stall`.
///
/// Flattened DTO for the `StallClassification` enum (wasm-bindgen
/// does not support enums with data payloads).
///
/// Tag values: 0 = Healthy, 1 = Warning, 2 = Stalled, 3 = Complete.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct WasmStallResult {
    tag: u8,
    ms_since_progress: u64,
}

#[wasm_bindgen]
impl WasmStallResult {
    /// Classification tag: 0=Healthy, 1=Warning, 2=Stalled, 3=Complete.
    #[wasm_bindgen(getter)]
    pub fn tag(&self) -> u8 {
        self.tag
    }

    /// Milliseconds since progress (meaningful for tag 1 and 2).
    #[wasm_bindgen(js_name = msSinceProgress, getter)]
    pub fn ms_since_progress(&self) -> u64 {
        self.ms_since_progress
    }
}

/// Progress cadence result — returned from `policy_progress_cadence`.
///
/// `should_emit` is true when both time and percentage thresholds are met.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct WasmProgressResult {
    should_emit: bool,
    percent: u8,
    bytes_transferred: u64,
    total_bytes: u64,
}

#[wasm_bindgen]
impl WasmProgressResult {
    /// Whether a progress event should be emitted.
    #[wasm_bindgen(js_name = shouldEmit, getter)]
    pub fn should_emit(&self) -> bool {
        self.should_emit
    }

    /// Percentage complete (0-100).
    #[wasm_bindgen(getter)]
    pub fn percent(&self) -> u8 {
        self.percent
    }

    /// Bytes transferred so far.
    #[wasm_bindgen(js_name = bytesTransferred, getter)]
    pub fn bytes_transferred(&self) -> u64 {
        self.bytes_transferred
    }

    /// Total bytes in transfer.
    #[wasm_bindgen(js_name = totalBytes, getter)]
    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }
}

// ─── Exported functions ───────────────────────────────────────────────

/// Compute a scheduling decision from flattened policy inputs.
///
/// Accepts `pending_chunk_ids` as a `&[u32]` (maps from JS Uint32Array).
/// All other parameters are scalar/enum.
#[wasm_bindgen(js_name = policyDecide)]
pub fn policy_decide(
    pending_chunk_ids: &[u32],
    rtt_ms: u32,
    loss_ppm: u32,
    device_class: WasmDeviceClass,
    max_parallel_chunks: u16,
    max_in_flight_bytes: u32,
    priority: u8,
    fairness_mode: WasmFairnessMode,
    configured_chunk_size: u32,
    transport_max_message_size: u32,
    pressure: WasmPressureState,
) -> WasmScheduleDecision {
    let input = PolicyInput {
        pending_chunk_ids: pending_chunk_ids.to_vec(),
        link_stats: LinkStats { rtt_ms, loss_ppm },
        device_class: device_class.into(),
        constraints: TransferConstraints {
            max_parallel_chunks,
            max_in_flight_bytes,
            priority,
            fairness_mode: fairness_mode.into(),
            configured_chunk_size,
            transport_max_message_size,
        },
        pressure: pressure.into(),
    };

    let result = decide(&input);

    WasmScheduleDecision {
        chunk_ids: result.next_chunk_ids,
        pacing_delay_ms: result.pacing_delay_ms,
        window_suggestion_chunks: result.window_suggestion_chunks,
        backpressure: result.backpressure.into(),
        effective_chunk_size: result.effective_chunk_size,
    }
}

/// Classify the current stall state of a transfer.
#[wasm_bindgen(js_name = policyDetectStall)]
pub fn policy_detect_stall(
    bytes_acked: u64,
    total_bytes: u64,
    ms_since_progress: u64,
    stall_threshold_ms: u64,
    warn_threshold_ms: u64,
) -> WasmStallResult {
    let input = StallInput {
        bytes_acked,
        total_bytes,
        ms_since_progress,
        stall_threshold_ms,
        warn_threshold_ms,
    };

    let result = detect_stall(&input);

    use bolt_transfer_core::policy::StallClassification;
    match result {
        StallClassification::Healthy => WasmStallResult {
            tag: 0,
            ms_since_progress: 0,
        },
        StallClassification::Warning { ms_since_progress } => WasmStallResult {
            tag: 1,
            ms_since_progress,
        },
        StallClassification::Stalled { ms_since_progress } => WasmStallResult {
            tag: 2,
            ms_since_progress,
        },
        StallClassification::Complete => WasmStallResult {
            tag: 3,
            ms_since_progress: 0,
        },
    }
}

/// Determine whether a progress event should be emitted.
#[wasm_bindgen(js_name = policyProgressCadence)]
pub fn policy_progress_cadence(
    bytes_transferred: u64,
    total_bytes: u64,
    elapsed_since_last_report_ms: u64,
    last_reported_percent: u8,
    min_interval_ms: u64,
    min_percent_delta: u8,
) -> WasmProgressResult {
    let config = ProgressConfig {
        min_interval_ms,
        min_percent_delta,
    };

    match progress_cadence(
        bytes_transferred,
        total_bytes,
        elapsed_since_last_report_ms,
        last_reported_percent,
        &config,
    ) {
        Some(report) => WasmProgressResult {
            should_emit: true,
            percent: report.percent,
            bytes_transferred: report.bytes_transferred,
            total_bytes: report.total_bytes,
        },
        None => WasmProgressResult {
            should_emit: false,
            percent: 0,
            bytes_transferred,
            total_bytes,
        },
    }
}

// ─── Native parity tests ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decide_parity_clear_pressure() {
        let chunks: Vec<u32> = vec![0, 1, 2, 3, 4];
        let wasm_result = policy_decide(
            &chunks,
            10, // rtt_ms
            0,  // loss_ppm
            WasmDeviceClass::Desktop,
            4,     // max_parallel_chunks
            65536, // max_in_flight_bytes
            128,   // priority
            WasmFairnessMode::Balanced,
            16384, // configured_chunk_size
            65536, // transport_max_message_size
            WasmPressureState::Clear,
        );

        // Native call for comparison
        let native_input = PolicyInput {
            pending_chunk_ids: vec![0, 1, 2, 3, 4],
            link_stats: LinkStats {
                rtt_ms: 10,
                loss_ppm: 0,
            },
            device_class: DeviceClass::Desktop,
            constraints: TransferConstraints {
                max_parallel_chunks: 4,
                max_in_flight_bytes: 65536,
                priority: 128,
                fairness_mode: FairnessMode::Balanced,
                configured_chunk_size: 16384,
                transport_max_message_size: 65536,
            },
            pressure: PressureState::Clear,
        };
        let native_result = decide(&native_input);

        assert_eq!(wasm_result.chunk_ids, native_result.next_chunk_ids);
        assert_eq!(wasm_result.pacing_delay_ms, native_result.pacing_delay_ms);
        assert_eq!(
            wasm_result.window_suggestion_chunks,
            native_result.window_suggestion_chunks
        );
        assert_eq!(
            wasm_result.effective_chunk_size,
            native_result.effective_chunk_size
        );
    }

    #[test]
    fn decide_parity_pressured() {
        let chunks: Vec<u32> = vec![0, 1, 2];
        let wasm_result = policy_decide(
            &chunks,
            50,
            0,
            WasmDeviceClass::Mobile,
            4,
            65536,
            128,
            WasmFairnessMode::Throughput,
            16384,
            65536,
            WasmPressureState::Pressured,
        );

        assert!(wasm_result.chunk_ids.is_empty());
        assert_eq!(wasm_result.backpressure, WasmBackpressure::Pause);
        assert_eq!(wasm_result.window_suggestion_chunks, 0);
    }

    #[test]
    fn decide_parity_elevated_latency() {
        let chunks: Vec<u32> = vec![0, 1, 2, 3];
        let wasm_result = policy_decide(
            &chunks,
            100, // rtt_ms
            0,
            WasmDeviceClass::Desktop,
            4,
            65536,
            128,
            WasmFairnessMode::Latency,
            16384,
            65536,
            WasmPressureState::Elevated,
        );

        let native_input = PolicyInput {
            pending_chunk_ids: vec![0, 1, 2, 3],
            link_stats: LinkStats {
                rtt_ms: 100,
                loss_ppm: 0,
            },
            device_class: DeviceClass::Desktop,
            constraints: TransferConstraints {
                max_parallel_chunks: 4,
                max_in_flight_bytes: 65536,
                priority: 128,
                fairness_mode: FairnessMode::Latency,
                configured_chunk_size: 16384,
                transport_max_message_size: 65536,
            },
            pressure: PressureState::Elevated,
        };
        let native_result = decide(&native_input);

        assert_eq!(wasm_result.chunk_ids, native_result.next_chunk_ids);
        assert_eq!(wasm_result.pacing_delay_ms, native_result.pacing_delay_ms);
        assert_eq!(
            wasm_result.window_suggestion_chunks,
            native_result.window_suggestion_chunks
        );
    }

    #[test]
    fn decide_parity_low_power_device() {
        let chunks: Vec<u32> = vec![0, 1, 2, 3, 4, 5];
        let wasm_result = policy_decide(
            &chunks,
            20,
            0,
            WasmDeviceClass::LowPower,
            6,
            65536,
            128,
            WasmFairnessMode::Balanced,
            16384,
            65536,
            WasmPressureState::Clear,
        );

        let native_input = PolicyInput {
            pending_chunk_ids: vec![0, 1, 2, 3, 4, 5],
            link_stats: LinkStats {
                rtt_ms: 20,
                loss_ppm: 0,
            },
            device_class: DeviceClass::LowPower,
            constraints: TransferConstraints {
                max_parallel_chunks: 6,
                max_in_flight_bytes: 65536,
                priority: 128,
                fairness_mode: FairnessMode::Balanced,
                configured_chunk_size: 16384,
                transport_max_message_size: 65536,
            },
            pressure: PressureState::Clear,
        };
        let native_result = decide(&native_input);

        assert_eq!(wasm_result.chunk_ids, native_result.next_chunk_ids);
        assert_eq!(
            wasm_result.window_suggestion_chunks,
            native_result.window_suggestion_chunks
        );
    }

    #[test]
    fn decide_parity_empty_chunks() {
        let chunks: Vec<u32> = vec![];
        let wasm_result = policy_decide(
            &chunks,
            10,
            0,
            WasmDeviceClass::Desktop,
            4,
            65536,
            128,
            WasmFairnessMode::Balanced,
            16384,
            65536,
            WasmPressureState::Clear,
        );

        assert!(wasm_result.chunk_ids.is_empty());
        assert_eq!(wasm_result.window_suggestion_chunks, 0);
    }

    #[test]
    fn decide_parity_chunk_cap() {
        let chunks: Vec<u32> = vec![0, 1];
        let wasm_result = policy_decide(
            &chunks,
            10,
            0,
            WasmDeviceClass::Desktop,
            4,
            65536,
            128,
            WasmFairnessMode::Balanced,
            16384,
            8192, // transport_max < configured
            WasmPressureState::Clear,
        );

        assert_eq!(wasm_result.effective_chunk_size, 8192);
    }

    #[test]
    fn stall_parity_healthy() {
        let result = policy_detect_stall(1000, 10000, 0, 10000, 5000);
        assert_eq!(result.tag, 0); // Healthy
    }

    #[test]
    fn stall_parity_warning() {
        let result = policy_detect_stall(1000, 10000, 5000, 10000, 5000);
        assert_eq!(result.tag, 1); // Warning
        assert_eq!(result.ms_since_progress, 5000);
    }

    #[test]
    fn stall_parity_stalled() {
        let result = policy_detect_stall(1000, 10000, 10000, 10000, 5000);
        assert_eq!(result.tag, 2); // Stalled
        assert_eq!(result.ms_since_progress, 10000);
    }

    #[test]
    fn stall_parity_complete() {
        let result = policy_detect_stall(10000, 10000, 999999, 10000, 5000);
        assert_eq!(result.tag, 3); // Complete
    }

    #[test]
    fn progress_parity_emit() {
        let result = policy_progress_cadence(5000, 10000, 300, 0, 250, 1);
        assert!(result.should_emit);
        assert_eq!(result.percent, 50);
        assert_eq!(result.bytes_transferred, 5000);
        assert_eq!(result.total_bytes, 10000);
    }

    #[test]
    fn progress_parity_suppress_time() {
        let result = policy_progress_cadence(5000, 10000, 100, 0, 250, 1);
        assert!(!result.should_emit);
    }

    #[test]
    fn progress_parity_suppress_delta() {
        let result = policy_progress_cadence(5500, 10000, 1000, 50, 0, 10);
        assert!(!result.should_emit);
    }

    #[test]
    fn progress_parity_zero_total() {
        let result = policy_progress_cadence(0, 0, 1000, 0, 250, 1);
        assert!(!result.should_emit);
    }
}
