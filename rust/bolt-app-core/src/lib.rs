//! bolt-app-core — shared app runtime for LocalBolt native shells.
//!
//! Extracted from localbolt-app Tauri backend (NATIVE-APP-CORE-1).
//! Zero Tauri dependency. Consumable by any Rust-based shell:
//! egui desktop (bolt-ui), SwiftUI/Kotlin mobile (via UniFFI), or Tauri (transitional).
//!
//! # Modules
//!
//! - [`watchdog`] — Daemon process supervision state machine (N3 spec)
//! - [`ipc_types`] — IPC message contract (NDJSON wire format)
//! - [`ipc_transport`] — Cross-platform IPC stream (Unix socket / Windows named pipe)
//! - [`ipc_client`] — Daemon readiness probe (version handshake + status check)
//! - [`platform`] — Platform-aware path defaults and process management
//! - [`daemon_log`] — Stderr ring buffer and crash snapshot persistence
//! - [`signal_monitor`] — Signal server health probe state machine (N8 spec)

pub mod daemon_lifecycle;
pub mod daemon_log;
pub mod ipc_bridge_core;
pub mod ipc_client;
pub mod ipc_transport;
pub mod ipc_types;
pub mod platform;
pub mod signal_monitor;
pub mod signaling_client;
pub mod watchdog;
