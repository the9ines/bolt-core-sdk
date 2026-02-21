import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import * as boltCore from '../src/index.js';

const __dirname = dirname(fileURLToPath(import.meta.url));
const snapshotPath = join(__dirname, '..', 'scripts', 'export-snapshot.json');

describe('Public API surface', () => {
  it('matches the export snapshot', () => {
    const snapshot = JSON.parse(readFileSync(snapshotPath, 'utf-8'));
    const actual = Object.keys(boltCore).sort();
    expect(actual).toEqual(snapshot.exports);
  });

  it('has no unexpected removals', () => {
    const snapshot = JSON.parse(readFileSync(snapshotPath, 'utf-8'));
    const actual = new Set(Object.keys(boltCore));
    const missing = snapshot.exports.filter((name: string) => !actual.has(name));
    expect(missing).toEqual([]);
  });
});
