# Bolt Core SDK — Release Playbook

## Prerequisites

- Working tree clean (`git status` shows nothing).
- All existing tests pass (`npm run build && npm run test`).
- `~/.npmrc` configured with `read:packages` PAT for local verification.

## Steps

### 1. Make changes

Edit source files in `ts/bolt-core/src/`. No changes outside this directory.

### 2. Build and test

```
cd ts/bolt-core
npm run build
npm run test
```

All 36+ tests must pass.

### 3. Update export snapshot (if exports changed)

```
npm run audit-exports
npm run test
```

If exports were not changed, skip this step. The snapshot test will catch unintended changes.

### 4. Bump version

Edit `ts/bolt-core/package.json`:

- Patch (`z`): bug fixes, no API changes.
- Minor (`y`): new exports or breaking changes (pre-1.0).

### 5. Commit

```
git add ts/bolt-core/
git commit -m "feat: <description>

Files changed:
- <list>

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

### 6. Tag

```
git tag sdk-v<VERSION>
```

Version in tag must match `package.json` version exactly.

### 7. Push

```
git push origin main
git push origin sdk-v<VERSION>
```

The tag push triggers `.github/workflows/publish-bolt-core.yml`.

### 8. Wait for publish

```
gh run list --repo the9ines/bolt-core-sdk --limit 1
gh run watch <RUN_ID> --repo the9ines/bolt-core-sdk
```

The workflow builds, tests, publishes to GitHub Packages, then runs a consumer smoke test.

### 9. Verify

Confirm the smoke test step passed in the workflow output. If it failed, the package may have a runtime issue — do not upgrade consumers.

### 10. Upgrade consumers

For each product repo (localbolt, localbolt-app, localbolt-v3):

1. Update `@the9ines/bolt-core` version in the web `package.json`.
2. Delete `node_modules` and `package-lock.json`.
3. Run `npm install` to regenerate the lockfile with `resolved` + `integrity`.
4. Run tests and build.
5. Commit, tag, push.
6. Confirm CI passes.

See `docs/SUPPLY_CHAIN.md` for auth and lockfile requirements.
