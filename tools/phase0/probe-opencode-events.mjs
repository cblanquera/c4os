import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { createOpencode } from '@opencode-ai/sdk';

process.env.PATH = [
  join(process.cwd(), 'node_modules', '.bin'),
  process.env.PATH,
].filter(Boolean).join(':');

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function summarizeEvent(event) {
  const properties = event.properties ?? {};
  return {
    type: event.type,
    propertyKeys: Object.keys(properties).sort(),
    properties,
  };
}

const projectDir = await mkdtemp(join(tmpdir(), 'c4os-opencode-events-'));
process.env.XDG_CONFIG_HOME = join(projectDir, 'xdg-config');
process.env.XDG_DATA_HOME = join(projectDir, 'xdg-data');
process.env.XDG_STATE_HOME = join(projectDir, 'xdg-state');
process.env.XDG_CACHE_HOME = join(projectDir, 'xdg-cache');
process.env.OPENCODE_SERVER_PASSWORD = 'phase0-event-probe';

const captured = [];
const abort = new AbortController();
let server;

try {
  const runtime = await createOpencode({
    hostname: '127.0.0.1',
    port: 4196,
    config: {
      permission: {
        edit: 'ask',
        bash: 'ask',
      },
    },
  });

  server = runtime.server;
  const client = runtime.client;

  const events = await client.event.subscribe({
    signal: abort.signal,
    query: {
      directory: projectDir,
    },
  });

  const reader = (async () => {
    try {
      for await (const event of events.stream) {
        captured.push(summarizeEvent(event));
        if (captured.length >= 12) {
          abort.abort();
          break;
        }
      }
    } catch (error) {
      if (!abort.signal.aborted) throw error;
    }
  })();

  await sleep(300);

  const created = await client.session.create({
    query: {
      directory: projectDir,
    },
  });
  const sessionID = created.data.id;

  await sleep(300);

  await client.session.prompt({
    path: {
      id: sessionID,
    },
    query: {
      directory: projectDir,
    },
    body: {
      noReply: true,
      parts: [
        {
          type: 'text',
          text: 'Phase 0 structured event probe. Do not reply.',
        },
      ],
    },
  });

  await sleep(1200);
  abort.abort();
  await reader;

  console.log(JSON.stringify({
    serverUrl: server.url,
    projectDir,
    sessionID,
    eventCount: captured.length,
    eventTypes: [...new Set(captured.map((event) => event.type))],
    events: captured,
  }, null, 2));
} finally {
  abort.abort();
  server?.close();
  await rm(projectDir, { recursive: true, force: true });
}
