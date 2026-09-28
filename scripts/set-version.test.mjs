import assert from 'node:assert/strict';
import { cpSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';
import { normalizeVersion, setVersion } from './set-version.mjs';

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');

test('release version updates all package metadata without changing dependencies', () => {
  const temp = mkdtempSync(join(tmpdir(), 'sunshine-release-version-'));
  try {
    for (const path of [
      'package.json',
      'src-tauri/tauri.conf.json',
      'src-tauri/Cargo.toml',
      'src-tauri/Cargo.lock',
    ]) {
      cpSync(join(projectRoot, path), join(temp, path), { recursive: true });
    }
    const originalLock = readFileSync(join(temp, 'src-tauri/Cargo.lock'), 'utf8');
    assert.equal(setVersion(temp, 'v2.3.4'), '2.3.4');
    assert.equal(JSON.parse(readFileSync(join(temp, 'package.json'))).version, '2.3.4');
    assert.equal(JSON.parse(readFileSync(join(temp, 'src-tauri/tauri.conf.json'))).version, '2.3.4');
    assert.match(readFileSync(join(temp, 'src-tauri/Cargo.toml'), 'utf8'), /^version = "2\.3\.4"$/m);
    const updatedLock = readFileSync(join(temp, 'src-tauri/Cargo.lock'), 'utf8');
    assert.equal(
      updatedLock,
      originalLock.replace(
        /(\[\[package\]\]\r?\nname = "sunshine-library-sync"\r?\nversion = ")[^"]+(")/,
        '$12.3.4$2',
      ),
    );
  } finally {
    rmSync(temp, { recursive: true, force: true });
  }
});

test('release version rejects invalid input', () => {
  for (const value of ['', '1.2', '1.2.3-beta', '01.2.3', '1.2.3; echo unexpected']) {
    assert.throws(() => normalizeVersion(value));
  }
});
