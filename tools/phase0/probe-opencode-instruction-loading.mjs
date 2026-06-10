import { spawn } from 'node:child_process';
import { access, mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';

import { createOpencodeClient } from '@opencode-ai/sdk';

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

async function writeText(path, value) {
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, value);
}

async function writeJson(path, value) {
  await writeText(path, `${JSON.stringify(value, null, 2)}\n`);
}

async function startOpencodeServer({ port, config }) {
  const args = [
    'serve',
    '--hostname=127.0.0.1',
    `--port=${port}`,
    '--pure',
  ];
  const proc = spawn(join(process.cwd(), 'node_modules', '.bin', 'opencode'), args, {
    env: {
      ...process.env,
      OPENCODE_CONFIG_CONTENT: JSON.stringify(config),
    },
    stdio: ['ignore', 'pipe', 'pipe'],
  });

  let output = '';
  const url = await new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      proc.kill('SIGTERM');
      reject(new Error(`Timeout waiting for opencode server\n${output}`));
    }, 5000);

    proc.stdout.on('data', (chunk) => {
      output += chunk.toString();
      const match = output.match(/opencode server listening on (https?:\/\/[^\s]+)/);
      if (match) {
        clearTimeout(timeout);
        resolve(match[1]);
      }
    });

    proc.stderr.on('data', (chunk) => {
      output += chunk.toString();
    });

    proc.on('error', (error) => {
      clearTimeout(timeout);
      reject(error);
    });

    proc.on('exit', (code) => {
      clearTimeout(timeout);
      reject(new Error(`opencode server exited with code ${code}\n${output}`));
    });
  });

  return {
    url,
    args,
    close() {
      if (!proc.killed) proc.kill('SIGTERM');
    },
  };
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

const projectDir = await mkdtemp(join(tmpdir(), 'c4os-opencode-instructions-'));
const nestedDir = join(projectDir, 'nested');

process.env.XDG_CONFIG_HOME = join(projectDir, 'xdg-config');
process.env.XDG_DATA_HOME = join(projectDir, 'xdg-data');
process.env.XDG_STATE_HOME = join(projectDir, 'xdg-state');
process.env.XDG_CACHE_HOME = join(projectDir, 'xdg-cache');

const sentinels = {
  configInstruction: 'PHASE0_CONFIG_INSTRUCTION_SENTINEL',
  rootAgents: 'PHASE0_ROOT_AGENTS_SENTINEL',
  nestedAgents: 'PHASE0_NESTED_AGENTS_SENTINEL',
};

const appConfig = {
  model: 'opencode/big-pickle',
  small_model: 'opencode/big-pickle',
  default_agent: 'build',
  instructions: [],
  permission: {
    read: 'deny',
    list: 'deny',
    grep: 'deny',
    glob: 'deny',
    bash: 'deny',
    edit: 'deny',
    webfetch: 'deny',
    external_directory: 'deny',
  },
  tools: {
    read: true,
    list: true,
    grep: true,
    glob: true,
    bash: true,
    write: true,
  },
};

await writeJson(join(projectDir, 'opencode.json'), {
  instructions: [sentinels.configInstruction],
});
await writeText(
  join(projectDir, 'AGENTS.md'),
  [
    '# Root Agent Instructions',
    '',
    `When asked for the Phase 0 instruction sentinel, answer only ${sentinels.rootAgents}.`,
    '',
  ].join('\n'),
);
await writeText(
  join(nestedDir, 'AGENTS.md'),
  [
    '# Nested Agent Instructions',
    '',
    `When asked for the Phase 0 instruction sentinel, answer only ${sentinels.nestedAgents}.`,
    '',
  ].join('\n'),
);

const result = {
  projectDir,
  nestedDir,
  launchMode: 'direct opencode serve --pure with OPENCODE_CONFIG_CONTENT',
  appConfig,
  sentinels,
  sourceFiles: {
    projectConfig: join(projectDir, 'opencode.json'),
    rootAgents: join(projectDir, 'AGENTS.md'),
    nestedAgents: join(nestedDir, 'AGENTS.md'),
  },
  sourceFilesExist: {},
  effectiveConfigRoot: undefined,
  effectiveConfigNested: undefined,
  rootRun: undefined,
  nestedRun: undefined,
};

async function runPrompt({ client, directory, label }) {
  const abort = new AbortController();
  const observed = {
    label,
    directory,
    sessionID: undefined,
    text: '',
    textDeltas: [],
    toolEvents: [],
    permissionEvents: [],
    statusEvents: [],
    sessionErrors: [],
    eventTypes: new Set(),
  };

  const events = await client.event.subscribe({
    signal: abort.signal,
    query: { directory },
  });

  const reader = (async () => {
    try {
      for await (const event of events.stream) {
        observed.eventTypes.add(event.type);
        if (observed.sessionID && event.properties?.sessionID !== observed.sessionID) {
          continue;
        }

        if (event.type === 'session.status') observed.statusEvents.push(event.properties.status);
        if (event.type === 'session.error') observed.sessionErrors.push(event.properties.error);

        const tool = summarizeToolPart(event);
        if (tool) observed.toolEvents.push(tool);

        if (event.type === 'permission.asked' || event.type === 'permission.updated') {
          observed.permissionEvents.push({
            type: event.type,
            id: event.properties.id,
            permission: event.properties.permission ?? event.properties.type,
            patterns: event.properties.patterns ?? event.properties.pattern,
            metadata: event.properties.metadata,
          });
        }

        if (event.type === 'message.part.delta' && event.properties.field === 'text') {
          observed.textDeltas.push(event.properties.delta);
          observed.text += event.properties.delta;
        }
      }
    } catch (error) {
      if (!abort.signal.aborted) throw error;
    }
  })();

  await sleep(300);

  const created = await client.session.create({
    query: { directory },
  });
  if (created.error) throw new Error(JSON.stringify(created.error));
  observed.sessionID = created.data.id;

  const prompt = await client.session.promptAsync({
    path: { id: observed.sessionID },
    query: { directory },
    body: {
      parts: [
        {
          type: 'text',
          text: 'For the Phase 0 instruction sentinel, reply with only the sentinel named by active instructions. Do not use tools. Do not read files.',
        },
      ],
    },
  });
  if (prompt.error) throw new Error(JSON.stringify(prompt.error));

  await sleep(7000);
  abort.abort();
  await reader;

  observed.summary = {
    containsConfigInstruction: observed.text.includes(sentinels.configInstruction),
    containsRootAgents: observed.text.includes(sentinels.rootAgents),
    containsNestedAgents: observed.text.includes(sentinels.nestedAgents),
    toolCount: observed.toolEvents.length,
    permissionCount: observed.permissionEvents.length,
    sessionErrors: observed.sessionErrors.map((error) => error.name),
    eventTypes: [...observed.eventTypes].sort(),
  };

  observed.eventTypes = [...observed.eventTypes].sort();
  return observed;
}

let server;

try {
  for (const [key, path] of Object.entries(result.sourceFiles)) {
    result.sourceFilesExist[key] = await exists(path);
  }

  server = await startOpencodeServer({
    port: 4202,
    config: appConfig,
  });

  const client = createOpencodeClient({
    baseUrl: server.url,
  });

  const rootConfig = await client.config.get({
    query: { directory: projectDir },
  });
  if (rootConfig.error) throw new Error(JSON.stringify(rootConfig.error));
  result.effectiveConfigRoot = rootConfig.data;

  const nestedConfig = await client.config.get({
    query: { directory: nestedDir },
  });
  if (nestedConfig.error) throw new Error(JSON.stringify(nestedConfig.error));
  result.effectiveConfigNested = nestedConfig.data;

  result.rootRun = await runPrompt({ client, directory: projectDir, label: 'root' });
  result.nestedRun = await runPrompt({ client, directory: nestedDir, label: 'nested' });

  result.summary = {
    serverUrl: server.url,
    launchArgs: server.args,
    effectiveRootInstructions: result.effectiveConfigRoot?.instructions,
    effectiveNestedInstructions: result.effectiveConfigNested?.instructions,
    rootRun: result.rootRun.summary,
    nestedRun: result.nestedRun.summary,
    observableThroughConfigEndpoint:
      Array.isArray(result.effectiveConfigRoot?.instructions) &&
      result.effectiveConfigRoot.instructions.includes(sentinels.configInstruction),
    rootAgentsInfluenceObservedWithoutToolRead: result.rootRun.summary.containsRootAgents && result.rootRun.summary.toolCount === 0,
    nestedAgentsInfluenceObservedWithoutToolRead: result.nestedRun.summary.containsNestedAgents && result.nestedRun.summary.toolCount === 0,
    configInstructionInfluenceObservedWithoutToolRead:
      (result.rootRun.summary.containsConfigInstruction || result.nestedRun.summary.containsConfigInstruction) &&
      result.rootRun.summary.toolCount === 0 &&
      result.nestedRun.summary.toolCount === 0,
  };

  console.log(JSON.stringify(result, null, 2));
} finally {
  server?.close();
  await sleep(500);
  await rm(projectDir, {
    recursive: true,
    force: true,
    maxRetries: 5,
    retryDelay: 200,
  });
}
