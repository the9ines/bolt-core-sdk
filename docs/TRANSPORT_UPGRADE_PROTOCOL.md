# Transport Web Upgrade Protocol

Defines the process for upgrading `@the9ines/bolt-transport-web` across the ecosystem.

## Current Published Version

`0.1.0`

## Enforcement Rules

1. **Exact pin only.** Product repos must use bare semver (`"0.1.0"`) with no prefix (`^`, `~`, `*`, `>=`, `latest`). CI guards enforce this.
2. **No deep imports.** All imports must use the barrel: `import { ... } from '@the9ines/bolt-transport-web'`. CI guards enforce this (Phase 4G).
3. **No duplicate local sources.** The 17 extracted files must not be reintroduced in product repos. CI guards enforce this (Phase 4G).
4. **Single install.** Each product repo must resolve exactly one instance of the package. CI guards enforce this.

## Version Source of Truth

Each product repo contains a `.transport-web-version` file at the repo root with the expected version string (e.g., `0.1.0`). The single-install CI guard validates the installed version matches this file.

## Upgrade Steps

### 1. Publish new version (bolt-core-sdk)

```bash
cd ts/bolt-transport-web
# Make changes, bump version in package.json
npm run build
npm test  # bolt-core tests must pass
git add -A && git commit -m "feat: bump transport-web to vX.Y.Z"
git tag transport-web-vX.Y.Z
git push origin main && git push origin transport-web-vX.Y.Z
# GitHub Actions publishes to GitHub Packages
```

### 2. Upgrade each product repo

Each product repo provides an upgrade script:

```bash
# localbolt
cd localbolt
bash scripts/upgrade-transport-web.sh X.Y.Z

# localbolt-app
cd localbolt-app
bash scripts/upgrade-transport-web.sh X.Y.Z

# localbolt-v3 (run from workspace root)
cd localbolt-v3
bash scripts/upgrade-transport-web.sh X.Y.Z
```

The script:
- Updates `package.json` with the new version
- Updates `.transport-web-version`
- Runs clean install + build (+ tests for localbolt)
- Reports PASS/FAIL
- Does NOT auto-commit or auto-tag

### 3. Commit and tag (manual)

After the upgrade script reports PASS:
- Review changes with `git diff`
- Commit following repo conventions
- Tag following repo scheme
- Push

### 4. Verify CI

All repos must pass CI after push:
- Version pin guard
- Single-install guard
- Drift guard (no deep imports, no duplicate sources)
- Build
- Tests (localbolt)

## STOP Conditions

Any of the following blocks the upgrade:
- Build failure in any product repo
- Test failure in localbolt
- Version mismatch between `package.json` and `.transport-web-version`
- Multiple instances of the package in the dependency tree
- Deep imports detected
- Duplicate source files detected

## Compatibility Table

| Transport Web Version | bolt-core Version | Status |
|-----------------------|-------------------|--------|
| 0.1.0 | 0.0.5 | Active — all 3 products |

## CI Guards (per product repo)

| Script | What it checks |
|--------|---------------|
| `scripts/check-transport-version-pin.sh` | Exact semver pin in package.json |
| `scripts/check-transport-single-install.sh` | Single instance, version matches `.transport-web-version` |
| `scripts/check-transport-drift.sh` | No deep imports, no duplicate source files |
