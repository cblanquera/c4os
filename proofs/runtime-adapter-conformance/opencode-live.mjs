import assert from 'node:assert/strict';
import { mkdtemp, realpath } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { delimiter, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createOpencode, createOpencodeClient } from '@opencode-ai/sdk/v2';

const proofDir = dirname(fileURLToPath(import.meta.url));
const isolatedRoot = await mkdtemp(join(tmpdir(), 'c4os-opencode-conformance-'));
const canonicalRoot = await realpath(isolatedRoot);
const originalCwd = process.cwd();
const password = `proof-${crypto.randomUUID()}`;
process.env.XDG_DATA_HOME = join(isolatedRoot, 'data');
process.env.XDG_CONFIG_HOME = join(isolatedRoot, 'config');
process.env.XDG_CACHE_HOME = join(isolatedRoot, 'cache');
process.env.OPENCODE_SERVER_PASSWORD = password;
process.env.PATH = `${join(proofDir, 'node_modules', '.bin')}${delimiter}${process.env.PATH ?? ''}`;
process.chdir(isolatedRoot);

const port = 49173;
let runtime;
try {
  runtime = await createOpencode({ hostname: '127.0.0.1', port, timeout: 15000 });
  const unauthenticated = await runtime.client.global.health();
  assert.equal(unauthenticated.response.status, 401);

  const authorization = `Basic ${Buffer.from(`opencode:${password}`).toString('base64')}`;
  const client = createOpencodeClient({
    baseUrl: runtime.server.url,
    headers: { Authorization: authorization },
  });
  const health = await client.global.health();
  assert.deepEqual(health.data, { healthy: true, version: '1.18.3' });
  const created = await client.session.create({ title: 'C4OS conformance proof' });
  assert.ok(created.data.id);
  assert.equal(await realpath(created.data.directory), canonicalRoot);
  assert.equal((await client.session.abort({ sessionID: created.data.id })).data, true);
  console.log(JSON.stringify({
    version: health.data.version,
    unauthenticatedStatus: unauthenticated.response.status,
    authenticatedHealth: health.data.healthy,
    sessionCreated: true,
    isolatedDirectory: await realpath(created.data.directory) === canonicalRoot,
    abortAccepted: true,
  }, null, 2));
} finally {
  runtime?.server.close();
  process.chdir(originalCwd);
}
