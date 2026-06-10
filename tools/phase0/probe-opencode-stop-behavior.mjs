import { access, mkdtemp, readFile, rm } from 'node:fs/promises';
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

async function exists(path) {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

function pidAlive(pid) {
  if (!pid) return false;
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

function summarizeToolPart(event) {
  const part = event.properties?.part;
  if (part?.type !== 'tool') return undefined;
  return {
    tool: part.tool,
    status: part.state?.status,
    input: part.state?.input,
    error: part.state?.error,
    output: part.state?.output,
    time: part.state?.time,
  };
}

const projectDir = await mkdtemp(join(tmpdir(), 'c4os-opencode-stop-'));
process.env.XDG_CONFIG_HOME = join(projectDir, 'xdg-config');
process.env.XDG_DATA_HOME = join(projectDir, 'xdg-data');
process.env.XDG_STATE_HOME = join(projectDir, 'xdg-state');
process.env.XDG_CACHE_HOME = join(projectDir, 'xdg-cache');

const pidFile = join(projectDir, 'phase0-long-running.pid');
const startedFile = join(projectDir, 'phase0-long-running.started');
const trapFile = join(projectDir, 'phase0-long-running.trap');
const doneFile = join(projectDir, 'phase0-long-running.done');
const command = [
  'printf "$$" > phase0-long-running.pid',
  'touch phase0-long-running.started',
  'trap "echo trapped > phase0-long-running.trap; exit 0" TERM INT HUP',
  'sleep 60',
  'echo done > phase0-long-running.done',
].join('; ');

const captured = [];
const permissionEvents = [];
const permissionReplies = [];
const toolEvents = [];
const statusEvents = [];
const sessionErrors = [];
const abort = new AbortController();
let server;
let sessionID;
let abortResponse;
let childPid;

try {
  const runtime = await createOpencode({
    hostname: '127.0.0.1',
    port: 4199,
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
    query: { directory: projectDir },
  });

  const reader = (async () => {
    try {
      for await (const event of events.stream) {
        captured.push({
          type: event.type,
          properties: event.properties,
        });

        if (event.properties?.sessionID !== sessionID) continue;

        if (event.type === 'session.status') {
          statusEvents.push(event.properties.status);
        }

        if (event.type === 'session.error') {
          sessionErrors.push(event.properties.error);
        }

        const tool = summarizeToolPart(event);
        if (tool) {
          toolEvents.push(tool);
        }

        if (event.type === 'permission.asked' || event.type === 'permission.updated') {
          permissionEvents.push({
            type: event.type,
            id: event.properties.id,
            permission: event.properties.permission ?? event.properties.type,
            patterns: event.properties.patterns ?? event.properties.pattern,
            metadata: event.properties.metadata,
            tool: event.properties.tool,
            startedExistsBeforeReply: await exists(startedFile),
            pidExistsBeforeReply: await exists(pidFile),
            doneExistsBeforeReply: await exists(doneFile),
          });

          const reply = await client.postSessionIdPermissionsPermissionId({
            path: {
              id: event.properties.sessionID,
              permissionID: event.properties.id,
            },
            query: { directory: projectDir },
            body: { response: 'once' },
          });

          permissionReplies.push({
            response: 'once',
            status: reply.response?.status,
            error: reply.error,
          });
        }

        if (event.type === 'permission.replied') {
          permissionReplies.push({
            event: 'permission.replied',
            reply: event.properties.reply ?? event.properties.response,
            requestID: event.properties.requestID ?? event.properties.permissionID,
          });
        }
      }
    } catch (error) {
      if (!abort.signal.aborted) throw error;
    }
  })();

  await sleep(300);

  const created = await client.session.create({
    query: { directory: projectDir },
  });
  if (created.error) throw new Error(JSON.stringify(created.error));
  sessionID = created.data.id;

  const prompt = await client.session.promptAsync({
    path: { id: sessionID },
    query: { directory: projectDir },
    body: {
      parts: [
        {
          type: 'text',
          text: `Use the bash tool to run exactly this command: ${command}. Do not ask for clarification.`,
        },
      ],
    },
  });
  if (prompt.error) throw new Error(JSON.stringify(prompt.error));

  const deadline = Date.now() + 15000;
  while (Date.now() < deadline) {
    if (await exists(pidFile)) break;
    await sleep(250);
  }

  if (await exists(pidFile)) {
    childPid = Number((await readFile(pidFile, 'utf8')).trim());
  }

  const startedBeforeAbort = await exists(startedFile);
  const doneBeforeAbort = await exists(doneFile);
  const trapBeforeAbort = await exists(trapFile);
  const aliveBeforeAbort = pidAlive(childPid);

  abortResponse = await client.session.abort({
    path: { id: sessionID },
    query: { directory: projectDir },
  });

  await sleep(2500);

  const aliveAfterAbort = pidAlive(childPid);
  const doneAfterAbort = await exists(doneFile);
  const trapAfterAbort = await exists(trapFile);

  if (aliveAfterAbort && childPid) {
    try {
      process.kill(childPid, 'TERM');
    } catch {
      // The probe reports liveness above; cleanup is best-effort.
    }
    await sleep(500);
  }

  const aliveAfterCleanup = pidAlive(childPid);

  abort.abort();
  await reader;

  console.log(JSON.stringify({
    serverUrl: server.url,
    projectDir,
    sessionID,
    command,
    childPid,
    permissionEvents,
    permissionReplies,
    abortResponse: {
      status: abortResponse.response?.status,
      error: abortResponse.error,
    },
    processState: {
      startedBeforeAbort,
      doneBeforeAbort,
      trapBeforeAbort,
      aliveBeforeAbort,
      aliveAfterAbort,
      doneAfterAbort,
      trapAfterAbort,
      aliveAfterCleanup,
    },
    toolEvents,
    statusEvents,
    sessionErrors,
    eventTypes: [...new Set(captured.map((event) => event.type))],
  }, null, 2));
} finally {
  abort.abort();
  server?.close();
  if (childPid && pidAlive(childPid)) {
    try {
      process.kill(childPid, 'KILL');
    } catch {
      // best-effort cleanup
    }
  }
  await rm(projectDir, { recursive: true, force: true });
}
