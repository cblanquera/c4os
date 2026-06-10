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

async function exists(path) {
  try {
    await access(path);
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
  };
}

const projectDir = await mkdtemp(join(tmpdir(), 'c4os-opencode-preexecution-'));
process.env.XDG_CONFIG_HOME = join(projectDir, 'xdg-config');
process.env.XDG_DATA_HOME = join(projectDir, 'xdg-data');
process.env.XDG_STATE_HOME = join(projectDir, 'xdg-state');
process.env.XDG_CACHE_HOME = join(projectDir, 'xdg-cache');

const scenarios = [
  {
    name: 'shell-deny',
    response: 'reject',
    target: join(projectDir, 'phase0-shell-denied.txt'),
    prompt: 'Use the bash tool to run exactly: echo phase0-shell-denied > phase0-shell-denied.txt. Do not ask for clarification.',
  },
  {
    name: 'shell-allow',
    response: 'once',
    target: join(projectDir, 'phase0-shell-allowed.txt'),
    prompt: 'Use the bash tool to run exactly: echo phase0-shell-allowed > phase0-shell-allowed.txt. Do not ask for clarification.',
  },
  {
    name: 'git-deny',
    response: 'reject',
    target: join(projectDir, '.git'),
    prompt: 'Use the bash tool to run exactly: git init && git checkout -b phase0-denied-branch. Do not ask for clarification.',
  },
  {
    name: 'git-allow',
    response: 'once',
    target: join(projectDir, '.git'),
    prompt: 'Use the bash tool to run exactly: git init && git checkout -b phase0-allowed-branch. Do not ask for clarification.',
  },
  {
    name: 'file-write-deny',
    response: 'reject',
    target: join(projectDir, 'phase0-edit-denied.txt'),
    prompt: 'Use the edit or write tool, not bash, to create phase0-edit-denied.txt with exactly this content: phase0-edit-denied. Do not ask for clarification.',
  },
  {
    name: 'file-write-allow',
    response: 'once',
    target: join(projectDir, 'phase0-edit-allowed.txt'),
    prompt: 'Use the edit or write tool, not bash, to create phase0-edit-allowed.txt with exactly this content: phase0-edit-allowed. Do not ask for clarification.',
  },
];

const bySession = new Map();
const results = scenarios.map((scenario) => ({
  name: scenario.name,
  requestedResponse: scenario.response,
  target: scenario.target,
  permissionEvents: [],
  permissionReplies: [],
  toolEvents: [],
  statusEvents: [],
  sessionErrors: [],
  targetExistsBeforePrompt: undefined,
  targetExistsAtPermission: undefined,
  targetExistsAfterRun: undefined,
}));

const abort = new AbortController();
let server;

try {
  const runtime = await createOpencode({
    hostname: '127.0.0.1',
    port: 4198,
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
        const sessionID = event.properties?.sessionID;
        const index = bySession.get(sessionID);
        const result = index === undefined ? undefined : results[index];

        if (result && event.type === 'session.status') {
          result.statusEvents.push(event.properties.status);
        }

        if (result && event.type === 'session.error') {
          result.sessionErrors.push(event.properties.error);
        }

        const tool = summarizeToolPart(event);
        if (result && tool) {
          result.toolEvents.push(tool);
        }

        if (result && (event.type === 'permission.asked' || event.type === 'permission.updated')) {
          const beforeReply = await exists(scenarios[index].target);
          result.targetExistsAtPermission = beforeReply;
          result.permissionEvents.push({
            type: event.type,
            id: event.properties.id,
            permission: event.properties.permission ?? event.properties.type,
            patterns: event.properties.patterns ?? event.properties.pattern,
            metadata: event.properties.metadata,
            tool: event.properties.tool,
            targetExistsBeforeReply: beforeReply,
          });

          const reply = await client.postSessionIdPermissionsPermissionId({
            path: {
              id: event.properties.sessionID,
              permissionID: event.properties.id,
            },
            query: { directory: projectDir },
            body: { response: scenarios[index].response },
          });

          result.permissionReplies.push({
            response: scenarios[index].response,
            status: reply.response?.status,
            error: reply.error,
          });
        }

        if (result && event.type === 'permission.replied') {
          result.permissionReplies.push({
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

  for (const [index, scenario] of scenarios.entries()) {
    results[index].targetExistsBeforePrompt = await exists(scenario.target);

    const created = await client.session.create({
      query: { directory: projectDir },
    });
    if (created.error) throw new Error(JSON.stringify(created.error));
    bySession.set(created.data.id, index);

    const prompt = await client.session.promptAsync({
      path: { id: created.data.id },
      query: { directory: projectDir },
      body: {
        parts: [
          {
            type: 'text',
            text: scenario.prompt,
          },
        ],
      },
    });
    if (prompt.error) throw new Error(JSON.stringify(prompt.error));

    await sleep(9000);
    results[index].targetExistsAfterRun = await exists(scenario.target);
  }

  abort.abort();
  await reader;

  console.log(JSON.stringify({
    serverUrl: server.url,
    projectDir,
    results,
    summary: results.map((result) => ({
      name: result.name,
      requestedResponse: result.requestedResponse,
      permissionTypes: result.permissionEvents.map((event) => event.permission),
      permissionCount: result.permissionEvents.length,
      targetExistsBeforePrompt: result.targetExistsBeforePrompt,
      targetExistsAtPermission: result.targetExistsAtPermission,
      targetExistsAfterRun: result.targetExistsAfterRun,
      toolStates: result.toolEvents.map((event) => `${event.tool}:${event.status}`),
      sessionErrors: result.sessionErrors.map((error) => error.name),
    })),
  }, null, 2));
} finally {
  abort.abort();
  server?.close();
  await rm(projectDir, { recursive: true, force: true });
}
