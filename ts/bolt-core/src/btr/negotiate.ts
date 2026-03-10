/**
 * Capability negotiation — BTR mode decision matrix (§4).
 *
 * Maps the 6-cell negotiation matrix from the BTR-0 spec.
 * Must match Rust bolt-btr/src/negotiate.rs exactly.
 */

/** BTR negotiation result. */
export const BtrMode = {
  /** Both peers support BTR. Full per-transfer DH ratchet + per-chunk chain. */
  FullBtr: 'FULL_BTR',
  /** One peer supports BTR, the other does not. Fall back to static ephemeral. */
  Downgrade: 'DOWNGRADE',
  /** Neither peer supports BTR. Current v1 static ephemeral. */
  StaticEphemeral: 'STATIC_EPHEMERAL',
  /** Malformed BTR metadata detected. RATCHET_DOWNGRADE_REJECTED + disconnect. */
  Reject: 'REJECT',
} as const;

export type BtrModeValue = (typeof BtrMode)[keyof typeof BtrMode];

/**
 * Negotiate BTR mode from capability advertisement.
 *
 * Implements the 6-cell matrix from §4:
 *
 * | Local | Remote | Well-formed | Result          |
 * |-------|--------|-------------|-----------------|
 * | YES   | YES    | YES         | FullBtr         |
 * | YES   | NO     | -           | Downgrade       |
 * | NO    | YES    | -           | Downgrade       |
 * | NO    | NO     | -           | StaticEphemeral |
 * | YES   | YES    | NO          | Reject          |
 */
export function negotiateBtr(
  localSupports: boolean,
  remoteSupports: boolean,
  remoteWellFormed: boolean,
): BtrModeValue {
  if (localSupports && remoteSupports) {
    return remoteWellFormed ? BtrMode.FullBtr : BtrMode.Reject;
  }
  if (localSupports || remoteSupports) {
    return BtrMode.Downgrade;
  }
  return BtrMode.StaticEphemeral;
}

/** Returns the log token for a given BTR mode, or null if none. */
export function btrLogToken(mode: BtrModeValue): string | null {
  switch (mode) {
    case BtrMode.Downgrade:
      return '[BTR_DOWNGRADE]';
    case BtrMode.Reject:
      return '[BTR_DOWNGRADE_REJECTED]';
    default:
      return null;
  }
}
