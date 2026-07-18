import test from 'node:test';
import assert from 'node:assert/strict';
import { access, mkdtemp } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { MarketplaceManager, digestDirectory } from './marketplace.mjs';

const proofDir = dirname(fileURLToPath(import.meta.url));
const v1 = join(proofDir, 'fixtures/plugin-v1');
const v2 = join(proofDir, 'fixtures/plugin-v2');
const source = (path, sha) => ({ kind: 'git', sha, path });

async function manager() {
  const cache = await mkdtemp(join(tmpdir(), 'c4os-marketplace-'));
  const value = new MarketplaceManager(cache);
  await value.initialize();
  return value;
}

test('requires immutable selectors and verified content digests', async () => {
  const store = await manager();
  const digest = await digestDirectory(v1);
  await assert.rejects(store.inspect({ kind: 'git', sha: 'main', path: v1 }), /immutable/);
  await assert.rejects(store.inspect({ kind: 'npm', version: '^1.0.0', path: v1 }), /exact version/);
  await assert.rejects(store.install(source(v1, 'a1b2c3d'), '0'.repeat(64)), /Digest verification failed/);
  assert.equal(store.get('proof-plugin'), undefined);
  assert.ok(digest.length === 64);
});

test('reviews metadata first, installs to C4OS cache disabled, and never executes package code', async () => {
  const store = await manager();
  const digest = await digestDirectory(v1);
  const installed = await store.install(source(v1, 'a1b2c3d'), digest);
  assert.equal(installed.enabled, false);
  assert.match(installed.path, /c4os-marketplace-/);
  assert.deepEqual(store.audit.slice(0, 2).map((item) => item.step), [
    'metadata-reviewed', 'installed-disabled',
  ]);
  await assert.rejects(access(join(proofDir, 'fixtures/ACTIVATED')));
});

test('failed migration rolls back atomically to the enabled prior version', async () => {
  const store = await manager();
  const digest1 = await digestDirectory(v1);
  const digest2 = await digestDirectory(v2);
  await store.install(source(v1, 'a1b2c3d'), digest1);
  store.enable('proof-plugin');
  const result = await store.update('proof-plugin', source(v2, 'd4e5f6a'), digest2, async () => {
    throw new Error('migration fixture failed');
  });
  assert.deepEqual(result, {
    updated: false, rolledBack: true, error: 'migration fixture failed',
  });
  assert.equal(store.get('proof-plugin').version, '1.0.0');
  assert.equal(store.get('proof-plugin').enabled, true);
});

test('successful update commits disabled and revocation blocks re-enable', async () => {
  const store = await manager();
  const digest1 = await digestDirectory(v1);
  const digest2 = await digestDirectory(v2);
  await store.install(source(v1, 'a1b2c3d'), digest1);
  store.enable('proof-plugin');
  assert.deepEqual(await store.update('proof-plugin', source(v2, 'd4e5f6a'), digest2, async () => {}), {
    updated: true, rolledBack: false,
  });
  assert.equal(store.get('proof-plugin').version, '2.0.0');
  assert.equal(store.get('proof-plugin').enabled, false);
  store.revoke(digest2);
  await assert.rejects(async () => store.enable('proof-plugin'), /revoked/);
  assert.equal(store.get('proof-plugin').enabled, false);
});

test('uninstall removes executable cache state and its install record', async () => {
  const store = await manager();
  const digest = await digestDirectory(v1);
  const installed = await store.install(source(v1, 'a1b2c3d'), digest);
  assert.equal(await store.uninstall('proof-plugin'), true);
  assert.equal(store.get('proof-plugin'), undefined);
  await assert.rejects(access(installed.path));
});
