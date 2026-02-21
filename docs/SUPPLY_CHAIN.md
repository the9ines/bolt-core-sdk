# Bolt Core SDK — Supply Chain

## Package

| Field | Value |
|-------|-------|
| Name | `@the9ines/bolt-core` |
| Registry | GitHub Packages (`https://npm.pkg.github.com`) |
| Source repo | `the9ines/bolt-core-sdk` |
| Package path | `ts/bolt-core/` |

## Publishing

Publishing is automated via GitHub Actions.

- **Trigger**: Push a git tag matching `sdk-v*` (e.g. `sdk-v0.0.5`).
- **Workflow**: `.github/workflows/publish-bolt-core.yml`
- **Steps**: checkout → setup-node → npm ci → build → test → npm publish
- **Version**: The npm version in `ts/bolt-core/package.json` must match the tag (e.g. tag `sdk-v0.0.5` publishes `@the9ines/bolt-core@0.0.5`).
- **Auth**: `GITHUB_TOKEN` with `packages: write` scope (granted by the workflow's `permissions` block).

Do not publish manually. Do not move or delete pushed tags.

## CI Auth (Consumer Repos)

Product repos (localbolt, localbolt-app, localbolt-v3) install `@the9ines/bolt-core` during CI.

Required CI workflow configuration:

```yaml
permissions:
  contents: read
  packages: read

steps:
  - uses: actions/setup-node@v4
    with:
      node-version: 20
      registry-url: https://npm.pkg.github.com
      scope: '@the9ines'
  - run: npm ci
    env:
      NODE_AUTH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

`GITHUB_TOKEN` is automatically available in GitHub Actions. No manual secrets are needed for repos within the `the9ines` organization.

## Local Development

GitHub Packages requires authentication for all npm operations, including install.

### Setup

1. Create a Personal Access Token (classic) at https://github.com/settings/tokens with `read:packages` scope only.
2. Add to your global `~/.npmrc`:
   ```
   //npm.pkg.github.com/:_authToken=YOUR_TOKEN
   @the9ines:registry=https://npm.pkg.github.com
   ```
3. Do **not** commit tokens. Each product repo has a project-level `.npmrc` that sets the registry scope but contains no credentials.

### Verify

```
npm view @the9ines/bolt-core version
```

## Rollout Playbook

When updating the SDK:

1. Make changes in `bolt-core-sdk/ts/bolt-core/`.
2. Run `npm run build && npm run test` locally.
3. Bump version in `package.json`.
4. Commit, tag `sdk-v<VERSION>`, push main and tag.
5. Wait for the publish workflow to succeed.
6. In each product repo:
   - Update `@the9ines/bolt-core` version in web `package.json`.
   - Delete `node_modules` and `package-lock.json`.
   - Run `npm install` to regenerate the lockfile with the correct `resolved` URL.
   - Run tests and build.
   - Commit lockfile + package.json, tag, push.
7. Confirm all CI workflows pass.

## Lockfile Requirements

Product lockfiles must contain `resolved` and `integrity` fields for `@the9ines/bolt-core`. Entries without these fields (e.g. from a `file:` install) will fail in CI. Always regenerate from a clean install with registry auth configured.
