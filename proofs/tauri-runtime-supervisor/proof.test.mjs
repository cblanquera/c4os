import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { RuntimeSupervisor } from './supervisor.mjs';

const proofDir = dirname(fileURLToPath(import.meta.url));
const manifest = JSON.parse(await readFile(join(proofDir, 'sidecar-manifest.json'), 'utf8'));

test('discovers a digest-pinned bundled sidecar and rejects trust/version failures', async () => {
  await new RuntimeSupervisor({ bundleRoot: proofDir, manifest }).discover();
  await assert.rejects(
    new RuntimeSupervisor({ bundleRoot: proofDir, manifest: { ...manifest, signed: false } }).discover(),
    /Unsigned/,
  );
  await assert.rejects(
    new RuntimeSupervisor({ bundleRoot: proofDir, manifest: { ...manifest, sha256: '0'.repeat(64) } }).discover(),
    /digest mismatch/,
  );
  const supervisor = new RuntimeSupervisor({ bundleRoot: proofDir, manifest });
  await assert.rejects(supervisor.start({ expectedVersion: '2.0.0' }), /Version mismatch/);
  assert.equal(supervisor.child.exitCode === null, false);
});

test('authenticates loopback health and isolates launch state', async () => {
  const first = new RuntimeSupervisor({ bundleRoot: proofDir, manifest });
  const second = new RuntimeSupervisor({ bundleRoot: proofDir, manifest });
  try {
    const firstReady = await first.start();
    const secondReady = await second.start();
    assert.equal(firstReady.host, '127.0.0.1');
    assert.equal((await first.health()).status, 200);
    assert.equal((await first.health('wrong-token')).status, 401);
    assert.equal((await first.health('')).status, 401);
    assert.notEqual(first.token, second.token);
    assert.notEqual(firstReady.stateRoot, secondReady.stateRoot);
    assert.equal((await first.health()).data.stateRoot, firstReady.stateRoot);
  } finally {
    await Promise.all([first.stop(), second.stop()]);
  }
  assert.equal(first.child.exitCode === null, false);
  assert.equal(second.child.exitCode === null, false);
});

test('shutdown terminates the sidecar descendant process group on macOS', async () => {
  const supervisor = new RuntimeSupervisor({ bundleRoot: proofDir, manifest });
  const ready = await supervisor.start();
  assert.ok(ready.descendantPid > 0);
  await supervisor.stop();
  await new Promise((resolveWait) => setTimeout(resolveWait, 100));
  assert.throws(() => process.kill(ready.descendantPid, 0), /ESRCH/);
});

test('bounds restart attempts and redacts diagnostic logs', async () => {
  const supervisor = new RuntimeSupervisor({ bundleRoot: proofDir, manifest, maximumRestarts: 2 });
  await supervisor.start();
  assert.equal(await supervisor.recordCrash(), true);
  await supervisor.start();
  assert.equal(await supervisor.recordCrash(), true);
  await supervisor.start();
  assert.equal(await supervisor.recordCrash(), false);
  assert.equal(supervisor.status, 'degraded');
  supervisor.log('token=secret-value password=hunter2 Bearer abc.def');
  assert.equal(supervisor.logs.at(-1).includes('secret-value'), false);
  assert.equal(supervisor.logs.at(-1).includes('hunter2'), false);
  assert.equal(supervisor.logs.at(-1).includes('abc.def'), false);
});
