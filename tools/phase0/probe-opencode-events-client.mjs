import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { createOpencodeClient } from '@opencode-ai/sdk';

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function basic(password) {
  return `Basic ${Buffer.from(`:${password}`).toString('base64')}`;
}

function summarizeEvent(event) {
  const properties = event.properties ?? {};
  return {
    type: event.type,
    propertyKeys: Object.keys(properties).sort(),
    properties,
  };
}

const baseUrl = process.env.OPENCODE_BASE_URL ?? 'http://127.0.0.1:4196';
const password = process.env.OPENCODE_SERVER_PASSWORD ?? 'phase0-event-probe';
const noReply = process.env.OPENCODE_PROBE_NO_REPLY !== 'false';
const projectDir = await mkdtemp(join(tmpdir(), 'c4os-opencode-events-'));
const captured = [];
const abort = new AbortController();

try {
  const client = createOpencodeClient({
    baseUrl,
    headers: {
      Authorization: basic(password),
    },
    directory: projectDir,
  });

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
        if (captured.length >= 80) {
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
  if (created.error) {
    throw new Error(JSON.stringify({
      step: 'session.create',
      status: created.response?.status,
      statusText: created.response?.statusText,
      error: created.error,
    }));
  }
  const sessionID = created.data.id;

  await sleep(300);

  const prompted = await client.session.prompt({
    path: {
      id: sessionID,
    },
    query: {
      directory: projectDir,
    },
    body: {
      noReply,
      parts: [
        {
          type: 'text',
          text: noReply
            ? 'Phase 0 structured event probe. Do not reply.'
            : 'Phase 0 structured event probe. Reply with OK only.',
        },
      ],
    },
  });
  if (prompted.error) {
    throw new Error(JSON.stringify({
      step: 'session.prompt',
      status: prompted.response?.status,
      statusText: prompted.response?.statusText,
      error: prompted.error,
    }));
  }

  await sleep(2500);
  abort.abort();
  await reader;

  console.log(JSON.stringify({
    baseUrl,
    noReply,
    projectDir,
    sessionID,
    eventCount: captured.length,
    eventTypes: [...new Set(captured.map((event) => event.type))],
    events: captured,
  }, null, 2));
} finally {
  abort.abort();
  await rm(projectDir, { recursive: true, force: true });
}
