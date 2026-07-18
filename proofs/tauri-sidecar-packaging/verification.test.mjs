import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { access, readFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const proofDir = dirname(fileURLToPath(import.meta.url));
const app = join(
  proofDir,
  'target/release/bundle/macos/C4OS Sidecar Packaging Proof.app',
);
const sidecar = join(app, 'Contents/MacOS/c4os-sidecar');
const host = join(app, 'Contents/MacOS/c4os-tauri-sidecar-packaging-proof');

test('real Tauri app bundle contains and executes the target-qualified external binary', async () => {
  const config = JSON.parse(await readFile(join(proofDir, 'tauri.conf.json'), 'utf8'));
  assert.deepEqual(config.bundle.externalBin, ['binaries/c4os-sidecar']);
  await Promise.all([access(host), access(sidecar)]);
  assert.equal(execFileSync(sidecar, { encoding: 'utf8' }).trim(), 'c4os-sidecar-proof 1.0.0');
});

test('ad-hoc macOS signature verifies the host and nested sidecar as one strict bundle', () => {
  execFileSync('/usr/bin/codesign', ['--verify', '--deep', '--strict', '--verbose=2', app]);
  const inspection = spawnSync('/usr/bin/codesign', ['-dv', '--verbose=2', sidecar], {
    encoding: 'utf8',
  });
  assert.equal(inspection.status, 0);
  const details = `${inspection.stdout}${inspection.stderr}`;
  assert.match(details, /Signature=adhoc/);
  assert.match(details, /TeamIdentifier=not set/);
});
