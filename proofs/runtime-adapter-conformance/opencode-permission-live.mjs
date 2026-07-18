import assert from 'node:assert/strict';
import { access, copyFile, mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { homedir, tmpdir } from 'node:os';
import { delimiter, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createOpencode, createOpencodeClient } from '@opencode-ai/sdk/v2';

const proofDir = dirname(fileURLToPath(import.meta.url));
const isolatedRoot = await mkdtemp(join(tmpdir(), 'c4os-opencode-permission-'));
const workspaceRoot = join(isolatedRoot, 'workspace');
const configRoot = join(isolatedRoot, 'config');
const dataRoot = join(isolatedRoot, 'data');
const deniedPath = join(workspaceRoot, 'should-not-exist.txt');
const authSource = process.env.C4OS_OPENCODE_AUTH_SOURCE
  ?? join(homedir(), '.local', 'share', 'opencode', 'auth.json');
const password = `proof-${crypto.randomUUID()}`;
const originalCwd = process.cwd();
let runtime;

function selectModel(providerData) {
  for (const providerId of providerData.connected) {
    const provider = providerData.all.find((item) => item.id === providerId);
    const models = Object.values(provider?.models ?? {}).filter((model) => (
      model.capabilities.toolcall && model.status === 'active'
    ));
    const requested = process.env.C4OS_OPENCODE_MODEL;
    const priorities = [requested, 'openai/gpt-4o-mini', 'openai/gpt-4.1-mini'].filter(Boolean);
    const selected = priorities
      .map((id) => models.find((model) => model.id === id))
      .find(Boolean)
      ?? models.sort((left, right) => left.cost.input - right.cost.input)[0];
    if (provider && selected) return { providerID: provider.id, modelID: selected.id };
  }
  throw new Error('No authenticated tool-capable OpenCode provider/model is available');
}

async function waitForPermission(client, directory, sessionID) {
  const deadline = Date.now() + 90000;
  while (Date.now() < deadline) {
    const [legacy, current] = await Promise.all([
      client.permission.list({ directory }),
      client.v2.permission.request.list({ location: { directory } }),
    ]);
    const legacyRequest = legacy.data.find((item) => item.sessionID === sessionID);
    if (legacyRequest) return { ...legacyRequest, api: 'legacy' };
    const currentRequest = current.data.data.find((item) => item.sessionID === sessionID);
    if (currentRequest) return { ...currentRequest, api: 'v2' };
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  const messages = await client.session.messages({ directory, sessionID, limit: 10 });
  const summary = messages.data.map((message) => ({
    role: message.info.role,
    error: message.info.error,
    parts: message.parts.map((part) => ({
      type: part.type,
      tool: part.tool,
      state: part.state,
    })),
  }));
  throw new Error(`Timed out waiting for the OpenCode permission request: ${JSON.stringify(summary)}`);
}

try {
  await Promise.all([
    mkdir(workspaceRoot, { recursive: true }),
    mkdir(join(configRoot, 'opencode'), { recursive: true }),
    mkdir(join(dataRoot, 'opencode'), { recursive: true }),
  ]);
  await copyFile(authSource, join(dataRoot, 'opencode', 'auth.json'));
  await writeFile(join(configRoot, 'opencode', 'opencode.json'), JSON.stringify({
    $schema: 'https://opencode.ai/config.json',
    permission: { '*': 'ask' },
  }, null, 2));

  process.env.XDG_DATA_HOME = dataRoot;
  process.env.XDG_CONFIG_HOME = configRoot;
  process.env.XDG_CACHE_HOME = join(isolatedRoot, 'cache');
  process.env.OPENCODE_SERVER_PASSWORD = password;
  process.env.PATH = `${join(proofDir, 'node_modules', '.bin')}${delimiter}${process.env.PATH ?? ''}`;
  process.chdir(workspaceRoot);

  runtime = await createOpencode({
    hostname: '127.0.0.1',
    port: 49174,
    timeout: 15000,
    config: { permission: { '*': 'ask' } },
  });
  const authorization = `Basic ${Buffer.from(`opencode:${password}`).toString('base64')}`;
  const client = createOpencodeClient({
    baseUrl: runtime.server.url,
    headers: { Authorization: authorization },
  });
  const loadedConfig = await client.config.get({ directory: workspaceRoot });
  assert.equal(loadedConfig.data.permission['*'], 'ask');
  const model = selectModel((await client.provider.list({ directory: workspaceRoot })).data);
  const session = await client.session.create({
    directory: workspaceRoot,
    title: 'C4OS deny-before-side-effect proof',
  });

  await client.session.promptAsync({
    directory: workspaceRoot,
    sessionID: session.data.id,
    model,
    parts: [{
      type: 'text',
      text: 'Use the write tool exactly once to create should-not-exist.txt containing denied. Do not explain first.',
    }],
  });

  const permission = await waitForPermission(client, workspaceRoot, session.data.id);
  await assert.rejects(access(deniedPath));
  if (permission.api === 'legacy') {
    assert.equal((await client.permission.reply({
      directory: workspaceRoot,
      requestID: permission.id,
      reply: 'reject',
      message: 'C4OS policy denied this proof action',
    })).data, true);
  } else {
    await client.v2.session.permission.reply({
      sessionID: session.data.id,
      requestID: permission.id,
      reply: 'reject',
      message: 'C4OS policy denied this proof action',
    });
  }
  await client.session.abort({ directory: workspaceRoot, sessionID: session.data.id });
  await assert.rejects(access(deniedPath));

  process.stdout.write(`${JSON.stringify({
    runtime: 'opencode',
    version: '1.18.3',
    provider: model.providerID,
    model: model.modelID,
    permission: permission.permission ?? permission.action,
    permissionApi: permission.api,
    reply: 'reject',
    deniedBeforeSideEffect: true,
    deniedPathAbsent: true,
  }, null, 2)}\n`);
} finally {
  runtime?.server.close();
  process.chdir(originalCwd);
  await rm(isolatedRoot, { recursive: true, force: true });
}
