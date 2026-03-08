//! Transport-facing interface (logic boundary only).
//!
//! The transfer core never performs I/O. This trait provides read-only
//! observation of the transport layer for backpressure decisions.

/// Read-only observation of the underlying transport channel.
///
/// Implementations are provided by the caller (daemon, app, WASM host).
/// The transfer core uses this to make backpressure decisions without
/// owning any transport I/O.
pub trait TransportQuery {
    /// Whether the transport channel is currently open.
    fn is_open(&self) -> bool;
    /// Number of bytes buffered/queued for sending.
    fn buffered_bytes(&self) -> usize;
    /// Maximum size of a single message the transport can carry.
    fn max_message_size(&self) -> usize;
}

/// Optional integrity verifier, caller-injected.
///
/// The transfer core has NO crypto dependencies. When hash verification
/// is needed (e.g., bolt.file-hash capability), the caller provides a
/// concrete verifier. When `None`, integrity checks are skipped.
pub trait IntegrityVerifier {
    /// Verify that `data` matches `expected_hash`.
    /// Returns `true` if the hash matches.
    fn verify(&self, data: &[u8], expected_hash: &str) -> bool;
}
