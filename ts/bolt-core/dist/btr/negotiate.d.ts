/**
 * Capability negotiation — BTR mode decision matrix (§4).
 *
 * Maps the 6-cell negotiation matrix from the BTR-0 spec.
 * Must match Rust bolt-btr/src/negotiate.rs exactly.
 */
/** BTR negotiation result. */
export declare const BtrMode: {
    /** Both peers support BTR. Full per-transfer DH ratchet + per-chunk chain. */
    readonly FullBtr: "FULL_BTR";
    /** One peer supports BTR, the other does not. Fall back to static ephemeral. */
    readonly Downgrade: "DOWNGRADE";
    /** Neither peer supports BTR. Current v1 static ephemeral. */
    readonly StaticEphemeral: "STATIC_EPHEMERAL";
    /** Malformed BTR metadata detected. RATCHET_DOWNGRADE_REJECTED + disconnect. */
    readonly Reject: "REJECT";
};
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
export declare function negotiateBtr(localSupports: boolean, remoteSupports: boolean, remoteWellFormed: boolean): BtrModeValue;
/** Returns the log token for a given BTR mode, or null if none. */
export declare function btrLogToken(mode: BtrModeValue): string | null;
