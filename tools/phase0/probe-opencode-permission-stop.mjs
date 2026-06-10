import { access, mkdtemp, rm } from 'node:fs/promises';
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

function partState(event) {
  const part = event.properties?.part;
  if (part?.type !== 'tool') return undefined;
  return {
    tool: part.tool,
    status: part.state?.status,
    input: part.state?.input,
    error: part.state?.error,
    output: part.state?.output,
  };
}

const projectDir = await mkdtemp(join(tmpdir(), 'c4os-opencode-permission-'));
const permissionResponse = process.env.OPENCODE_PROBE_PERMISSION_RESPONSE ?? 'reject';
process.env.XDG_CONFIG_HOME = join(projectDir, 'xdg-config');
process.env.XDG_DATA_HOME = join(projectDir, 'xdg-data');
process.env.XDG_STATE_HOME = join(projectDir, 'xdg-state');
process.env.XDG_CACHE_HOME = join(projectDir, 'xdg-cache');

const captured = [];
const permissionResponses = [];
const abort = new AbortController();
let server;
let toolSessionID;
let stopSessionID;
let stopAbortResponse;

try {
  const runtime = await createOpencode({
    hostname: '127.0.0.1',
    port: 4197,
    config: {
      permission: {
        edit: 'ask',
        bash: 'ask',
        webfetch: 'ask',
        external_directory: 'deny',
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

        if (event.type === 'permission.updated' || event.type === 'permission.asked') {
          const response = await client.postSessionIdPermissionsPermissionId({
            path: {
              id: event.properties.sessionID,
              permissionID: event.properties.id,
            },
            query: {
              directory: projectDir,
            },
            body: {
              response: permissionResponse,
            },
          });
          permissionResponses.push({
            permissionID: event.properties.id,
            sessionID: event.properties.sessionID,
            eventType: event.type,
            responseStatus: response.response?.status,
            responseError: response.error,
          });
        }
      }
    } catch (error) {
      if (!abort.signal.aborted) throw error;
    }
  })();

  await sleep(300);

  const toolSession = await client.session.create({
    query: {
      directory: projectDir,
    },
  });
  if (toolSession.error) throw new Error(JSON.stringify(toolSession.error));
  toolSessionID = toolSession.data.id;

  const toolPrompt = await client.session.promptAsync({
    path: {
      id: toolSessionID,
    },
    query: {
      directory: projectDir,
    },
    body: {
      parts: [
        {
          type: 'text',
          text: 'Use the bash tool to run exactly: echo phase0-denied > phase0-denied.txt. Do not ask me for clarification. If the tool is denied, report the denial.',
        },
      ],
    },
  });
  if (toolPrompt.error) throw new Error(JSON.stringify(toolPrompt.error));

  await sleep(9000);

  const stopSession = await client.session.create({
    query: {
      directory: projectDir,
    },
  });
  if (stopSession.error) throw new Error(JSON.stringify(stopSession.error));
  stopSessionID = stopSession.data.id;

  const stopPrompt = await client.session.promptAsync({
    path: {
      id: stopSessionID,
    },
    query: {
      directory: projectDir,
    },
    body: {
      tools: {},
      parts: [
        {
          type: 'text',
          text: 'Write a long response of at least 500 words about validation evidence. Continue until interrupted.',
        },
      ],
    },
  });
  if (stopPrompt.error) throw new Error(JSON.stringify(stopPrompt.error));

  await sleep(1200);
  stopAbortResponse = await client.session.abort({
    path: {
      id: stopSessionID,
    },
    query: {
      directory: projectDir,
    },
  });

  await sleep(2500);
  abort.abort();
  await reader;

  let deniedFileExists = true;
  try {
    await access(join(projectDir, 'phase0-denied.txt'));
  } catch {
    deniedFileExists = false;
  }

  const toolEvents = captured
    .map(partState)
    .filter(Boolean);

  console.log(JSON.stringify({
    serverUrl: server.url,
    permissionResponse,
    projectDir,
    toolSessionID,
    stopSessionID,
    eventCount: captured.length,
    eventTypes: [...new Set(captured.map((event) => event.type))],
    permissionResponses,
    stopAbort: {
      status: stopAbortResponse.response?.status,
      error: stopAbortResponse.error,
    },
    deniedFileExists,
    toolEvents,
    events: captured,
  }, null, 2));
} finally {
  abort.abort();
  server?.close();
  await rm(projectDir, { recursive: true, force: true });
}
