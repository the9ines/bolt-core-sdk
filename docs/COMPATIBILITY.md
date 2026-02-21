# Bolt Core SDK — Compatibility Contract

## Versioning

Pre-1.0 semver (`0.y.z`):

- **Minor** (`y`): MAY introduce breaking changes to the public API. Consumers must review changelogs before upgrading.
- **Patch** (`z`): MUST NOT break runtime behavior. Bug fixes and documentation only.

Post-1.0 follows standard semver.

## Public API Surface

The public API is defined strictly by the named exports of `src/index.ts`.

The current surface is recorded in `scripts/export-snapshot.json` and guarded by `__tests__/exports.test.ts`. Any change to the export list causes a test failure.

### Rules

- Only symbols exported from `index.ts` are part of the public contract.
- Internal modules (`src/constants.ts`, `src/encoding.ts`, etc.) are implementation details.
- Consumers MUST NOT deep-import (e.g. `@the9ines/bolt-core/dist/crypto.js`). These paths may change without notice.
- The `exports` field in `package.json` enforces the single entry point.

## Adding Exports

1. Add the export to `src/index.ts`.
2. Run `npm run audit-exports` to update `scripts/export-snapshot.json`.
3. Run `npm run test` to confirm the snapshot test passes.
4. Document the addition in the commit message.

## Removing or Renaming Exports

This is a breaking change under pre-1.0 policy. It requires a minor version bump (`0.y+1.0`).

1. Remove or rename in `src/index.ts`.
2. Run `npm run audit-exports` to update the snapshot.
3. Update all consumer repos in the same release cycle.
4. Bump minor version. Tag accordingly.
