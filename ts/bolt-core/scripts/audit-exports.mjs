#!/usr/bin/env node
/**
 * Audit the public export surface of @the9ines/bolt-core.
 *
 * Reads the built dist/index.js, collects all named exports,
 * and writes a sorted snapshot to scripts/export-snapshot.json.
 *
 * Usage: node scripts/audit-exports.mjs
 */

import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, '..');
const snapshotPath = join(root, 'scripts', 'export-snapshot.json');

const mod = await import(join(root, 'dist', 'index.js'));
const exports = Object.keys(mod).sort();

const snapshot = {
  description: 'Public API surface of @the9ines/bolt-core. Do not edit manually.',
  exports,
};

writeFileSync(snapshotPath, JSON.stringify(snapshot, null, 2) + '\n');
console.log(`Snapshot written: ${exports.length} exports`);
for (const name of exports) {
  console.log(`  ${name}`);
}
