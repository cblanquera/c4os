import test from 'node:test';
import assert from 'node:assert/strict';
import { generateKeyPairSync, sign } from 'node:crypto';
import { access } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { runHook } from './hook-runner.mjs';
import {
  canonicalRelease,
  digestPayload,
  ExtensionTrustStore,
} from './trust-store.mjs';

const proofDir = dirname(fileURLToPath(import.meta.url));
const safeHook = join(proofDir, 'fixtures/safe-hook.mjs');
const slowHook = join(proofDir, 'fixtures/slow-hook.mjs');

/**
 * Creates a locally signed immutable release fixture.
 */
function releaseFixture(overrides = {}) {
  const keys = generateKeyPairSync('ed25519');
  const payload = Buffer.from('verified extension payload');
  const release = {
    digest: digestPayload(payload),
    id: 'proof-extension',
    keyId: 'proof-authority',
    origin: 'https://plugins.c4os.test',
    version: '1.0.0',
    ...overrides,
  };
  release.signature = sign(null, canonicalRelease(release), keys.privateKey);
  const authority = {
    id: release.keyId,
    origin: release.origin,
    publicKey: keys.publicKey,
  };
  return { authority, payload, release };
}

test('verifies immutable origin, digest, and Ed25519 signature before disabled install', () => {
  const fixture = releaseFixture();
  const store = new ExtensionTrustStore([fixture.authority]);
  const installed = store.install(fixture.release, fixture.payload);
  assert.equal(installed.enabled, false);
  assert.equal(installed.quarantined, true);
  assert.throws(() => store.enable(installed.id, false), /Explicit review/);
  assert.equal(store.enable(installed.id, true).enabled, true);

  const wrongOrigin = { ...fixture.release, origin: 'https://evil.example' };
  assert.throws(() => store.install(wrongOrigin, fixture.payload), /Untrusted/);
  assert.throws(() => store.install(fixture.release, Buffer.from('tampered')), /digest mismatch/i);
});

test('revocation immediately disables content and signing authorities', () => {
  const first = releaseFixture();
  const firstStore = new ExtensionTrustStore([first.authority]);
  firstStore.install(first.release, first.payload);
  firstStore.enable(first.release.id, true);
  firstStore.revokeDigest(first.release.digest);
  assert.equal(firstStore.installed.get(first.release.id).enabled, false);
  assert.throws(() => firstStore.enable(first.release.id, true), /revoked/);

  const second = releaseFixture();
  const secondStore = new ExtensionTrustStore([second.authority]);
  secondStore.install(second.release, second.payload);
  secondStore.enable(second.release.id, true);
  secondStore.revokeKey(second.release.keyId);
  assert.equal(secondStore.installed.get(second.release.id).enabled, false);
});

test('hook requires verified explicit trust and runs with sandboxed effects', async () => {
  await assert.rejects(runHook({
    enabled: true,
    event: { type: 'turn.completed' },
    explicitTrust: false,
    hookPath: safeHook,
    signatureVerified: true,
  }), /explicitly trusted/);

  const result = await runHook({
    enabled: true,
    event: { type: 'turn.completed' },
    explicitTrust: true,
    hookPath: safeHook,
    signatureVerified: true,
    timeoutMs: 3000,
  });
  assert.equal(result.code, 0, result.errorOutput);
  const output = JSON.parse(result.output);
  assert.equal(result.workspaceWrite, true);
  assert.equal(output.outsideWriteDenied, true);
  assert.equal(output.networkDenied, true);
  assert.deepEqual(output.environmentKeys.filter((key) => !key.startsWith('__CF_')), [
    'C4OS_EVENT', 'HOME', 'PATH', 'TMPDIR',
  ]);
  await assert.rejects(access('/tmp/c4os-hook-escape.txt'));
});

test('hook timeout kills the isolated process group', async () => {
  const result = await runHook({
    enabled: true,
    event: { type: 'turn.completed' },
    explicitTrust: true,
    hookPath: slowHook,
    signatureVerified: true,
    timeoutMs: 100,
  });
  assert.equal(result.timedOut, true, result.errorOutput);
  assert.equal(result.signal, 'SIGKILL');
});
