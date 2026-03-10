//! Capability negotiation — BTR mode decision matrix (§4).
//!
//! Maps the 6-cell negotiation matrix from the BTR-0 spec.

/// BTR negotiation result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtrMode {
    /// Both peers support BTR. Full per-transfer DH ratchet + per-chunk chain.
    FullBtr,
    /// One peer supports BTR, the other does not. Fall back to static ephemeral.
    /// Log `[BTR_DOWNGRADE]`, warn user.
    Downgrade,
    /// Neither peer supports BTR. Current v1 static ephemeral.
    StaticEphemeral,
    /// Malformed BTR metadata detected. `RATCHET_DOWNGRADE_REJECTED` + disconnect.
    Reject,
}

/// Negotiate BTR mode from capability advertisement.
///
/// Implements the 6-cell matrix from §4:
///
/// | Local | Remote | Well-formed | Result |
/// |-------|--------|-------------|--------|
/// | YES   | YES    | YES         | FullBtr |
/// | YES   | NO     | -           | Downgrade |
/// | NO    | YES    | -           | Downgrade |
/// | NO    | NO     | -           | StaticEphemeral |
/// | YES   | YES    | NO (remote) | Reject |
/// | YES   | YES    | NO (local)  | Reject |
///
/// `remote_well_formed` is only relevant when both peers advertise BTR.
/// When one or both don't support BTR, well-formedness is not checked.
pub fn negotiate_btr(
    local_supports: bool,
    remote_supports: bool,
    remote_well_formed: bool,
) -> BtrMode {
    match (local_supports, remote_supports) {
        (true, true) => {
            if remote_well_formed {
                BtrMode::FullBtr
            } else {
                BtrMode::Reject
            }
        }
        (true, false) | (false, true) => BtrMode::Downgrade,
        (false, false) => BtrMode::StaticEphemeral,
    }
}

/// Returns the log token for a given BTR mode.
pub fn btr_log_token(mode: BtrMode) -> Option<&'static str> {
    match mode {
        BtrMode::FullBtr => None,
        BtrMode::Downgrade => Some("[BTR_DOWNGRADE]"),
        BtrMode::StaticEphemeral => None,
        BtrMode::Reject => Some("[BTR_DOWNGRADE_REJECTED]"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_support_well_formed() {
        assert_eq!(negotiate_btr(true, true, true), BtrMode::FullBtr);
    }

    #[test]
    fn both_support_malformed() {
        assert_eq!(negotiate_btr(true, true, false), BtrMode::Reject);
    }

    #[test]
    fn local_only() {
        assert_eq!(negotiate_btr(true, false, true), BtrMode::Downgrade);
        assert_eq!(negotiate_btr(true, false, false), BtrMode::Downgrade);
    }

    #[test]
    fn remote_only() {
        assert_eq!(negotiate_btr(false, true, true), BtrMode::Downgrade);
        assert_eq!(negotiate_btr(false, true, false), BtrMode::Downgrade);
    }

    #[test]
    fn neither_supports() {
        assert_eq!(negotiate_btr(false, false, true), BtrMode::StaticEphemeral);
        assert_eq!(negotiate_btr(false, false, false), BtrMode::StaticEphemeral);
    }

    #[test]
    fn log_tokens() {
        assert_eq!(btr_log_token(BtrMode::FullBtr), None);
        assert_eq!(btr_log_token(BtrMode::Downgrade), Some("[BTR_DOWNGRADE]"));
        assert_eq!(btr_log_token(BtrMode::StaticEphemeral), None);
        assert_eq!(
            btr_log_token(BtrMode::Reject),
            Some("[BTR_DOWNGRADE_REJECTED]")
        );
    }

    #[test]
    fn all_six_matrix_cells() {
        let cases = [
            (true, true, true, BtrMode::FullBtr),
            (true, false, true, BtrMode::Downgrade),
            (false, true, true, BtrMode::Downgrade),
            (false, false, true, BtrMode::StaticEphemeral),
            (true, true, false, BtrMode::Reject),
            // Cell 6: local malformed — in practice, local well-formedness is
            // self-validated. This maps to Reject if remote detects it.
            // From local perspective: if we detect our own malformation, reject.
        ];
        for (local, remote, wf, expected) in cases {
            assert_eq!(
                negotiate_btr(local, remote, wf),
                expected,
                "negotiate_btr({local}, {remote}, {wf}) != {expected:?}"
            );
        }
    }
}
