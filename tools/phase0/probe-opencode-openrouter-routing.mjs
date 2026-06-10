import { createServer } from 'node:http';
import { spawn } from 'node:child_process';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { createOpencodeClient } from '@opencode-ai/sdk';

process.env.PATH = [
  join(process.cwd(), 'node_modules', '.bin'),
  process.env.PATH,
].filter(Boolean).join(':');

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function readBody(req) {
  const chunks = [];
  for await (const chunk of req) chunks.push(chunk);
  const raw = Buffer.concat(chunks).toString('utf8');
  try {
    return { raw, json: raw ? JSON.parse(raw) : undefined };
  } catch {
    return { raw, json: undefined };
  }
}

async function startFakeOpenRouter({ port, selectedModel }) {
  const requests = [];
  const server = createServer(async (req, res) => {
    const body = await readBody(req);
    const record = {
      method: req.method,
      url: req.url,
      headers: {
        authorization: req.headers.authorization,
        contentType: req.headers['content-type'],
        referer: req.headers['http-referer'],
        title: req.headers['x-openrouter-title'],
      },
      body: body.json ?? body.raw,
    };
    requests.push(record);

    if (req.method === 'GET' && req.url?.endsWith('/models')) {
      res.writeHead(200, { 'content-type': 'application/json' });
      res.end(JSON.stringify({
        data: [
          {
            id: selectedModel,
            name: 'Phase 0 fake OpenRouter model',
            context_length: 8192,
            pricing: { prompt: '0', completion: '0' },
            supported_parameters: ['tools'],
          },
        ],
      }));
      return;
    }

    if (req.method === 'POST' && req.url?.includes('/chat/completions')) {
      const stream = body.json?.stream !== false;
      if (stream) {
        res.writeHead(200, {
          'content-type': 'text/event-stream',
          'cache-control': 'no-cache',
          connection: 'keep-alive',
        });
        res.write(`data: ${JSON.stringify({
          id: 'gen-phase0-openrouter',
          object: 'chat.completion.chunk',
          created: Math.floor(Date.now() / 1000),
          model: body.json?.model ?? selectedModel,
          choices: [
            { index: 0, delta: { role: 'assistant', content: 'phase0-openrouter-ok' }, finish_reason: null },
          ],
        })}\n\n`);
        res.write(`data: ${JSON.stringify({
          id: 'gen-phase0-openrouter',
          object: 'chat.completion.chunk',
          created: Math.floor(Date.now() / 1000),
          model: body.json?.model ?? selectedModel,
          choices: [
            { index: 0, delta: {}, finish_reason: 'stop' },
          ],
          usage: {
            prompt_tokens: 1,
            completion_tokens: 1,
            total_tokens: 2,
          },
        })}\n\n`);
        res.write('data: [DONE]\n\n');
        res.end();
        return;
      }

      res.writeHead(200, { 'content-type': 'application/json' });
      res.end(JSON.stringify({
        id: 'gen-phase0-openrouter',
        object: 'chat.completion',
        created: Math.floor(Date.now() / 1000),
        model: body.json?.model ?? selectedModel,
        choices: [
          {
            index: 0,
            message: { role: 'assistant', content: 'phase0-openrouter-ok' },
            finish_reason: 'stop',
          },
        ],
        usage: {
          prompt_tokens: 1,
          completion_tokens: 1,
          total_tokens: 2,
        },
      }));
      return;
    }

    res.writeHead(404, { 'content-type': 'application/json' });
    res.end(JSON.stringify({ error: { message: `Unhandled fake OpenRouter path ${req.method} ${req.url}` } }));
  });

  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(port, '127.0.0.1', resolve);
  });

  return {
    baseUrl: `http://127.0.0.1:${port}/api/v1`,
    requests,
    close() {
      return new Promise((resolve) => server.close(resolve));
    },
  };
}

async function startOpencodeServer({ port, config, env }) {
  const args = [
    'serve',
    '--hostname=127.0.0.1',
    `--port=${port}`,
    '--pure',
  ];
  const proc = spawn(join(process.cwd(), 'node_modules', '.bin', 'opencode'), args, {
    env: {
      ...process.env,
      ...env,
      OPENCODE_CONFIG_CONTENT: JSON.stringify(config),
    },
    stdio: ['ignore', 'pipe', 'pipe'],
  });

  let output = '';
  const url = await new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      proc.kill('SIGTERM');
      reject(new Error(`Timeout waiting for opencode server\n${output}`));
    }, 7000);

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
    output,
    close() {
      if (!proc.killed) proc.kill('SIGTERM');
    },
  };
}

function summarizeMessages(messages) {
  return (messages ?? []).map((message) => ({
    role: message.role,
    contentTypes: Array.isArray(message.content)
      ? message.content.map((part) => part.type)
      : [typeof message.content],
    contentPreview: typeof message.content === 'string'
      ? message.content.slice(0, 160)
      : undefined,
  }));
}

const projectDir = await mkdtemp(join(tmpdir(), 'c4os-opencode-openrouter-'));
process.env.XDG_CONFIG_HOME = join(projectDir, 'xdg-config');
process.env.XDG_DATA_HOME = join(projectDir, 'xdg-data');
process.env.XDG_STATE_HOME = join(projectDir, 'xdg-state');
process.env.XDG_CACHE_HOME = join(projectDir, 'xdg-cache');

const selectedProvider = 'openrouter';
const selectedModel = 'phase0/fake-openrouter-model';
const selectedFullModel = `${selectedProvider}/${selectedModel}`;
const startingCredentialRef = 'keychain://openrouter/phase0-starting-ref';
const startingApiKey = 'phase0-starting-api-key';
const changedApiKey = 'phase0-changed-api-key';

await writeFile(join(projectDir, 'opencode.json'), JSON.stringify({
  model: 'openai/hostile-model',
  small_model: 'openai/hostile-small-model',
  enabled_providers: ['openai'],
}, null, 2));

const fakeOpenRouter = await startFakeOpenRouter({
  port: 4312,
  selectedModel,
});

const appConfig = {
  model: selectedFullModel,
  small_model: selectedFullModel,
  enabled_providers: ['openrouter'],
  disabled_providers: ['openai', 'anthropic', 'google', 'gemini', 'opencode'],
  provider: {
    openrouter: {
      options: {
        apiKey: '{env:OPENROUTER_API_KEY}',
        baseURL: fakeOpenRouter.baseUrl,
        timeout: 30000,
      },
      models: {
        [selectedModel]: {
          name: 'Phase 0 fake OpenRouter model',
          tool_call: true,
          limit: { context: 8192, output: 1024 },
          cost: { input: 0, output: 0 },
        },
      },
    },
  },
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
};

const result = {
  projectDir,
  selectedProvider,
  selectedModel,
  selectedFullModel,
  selectedCredentialRef: startingCredentialRef,
  fakeOpenRouterBaseUrl: fakeOpenRouter.baseUrl,
  hostileProjectConfig: join(projectDir, 'opencode.json'),
  appConfig,
  effectiveConfig: undefined,
  session: undefined,
  sessionCredentialRef: startingCredentialRef,
  changedEnvironmentAfterSessionStart: false,
  modelEvents: [],
  statusEvents: [],
  text: '',
  sessionErrors: [],
  fakeOpenRouterRequests: undefined,
};

let opencodeServer;
const abort = new AbortController();

try {
  opencodeServer = await startOpencodeServer({
    port: 4212,
    config: appConfig,
    env: {
      OPENROUTER_API_KEY: startingApiKey,
    },
  });

  const client = createOpencodeClient({
    baseUrl: opencodeServer.url,
  });

  const configResponse = await client.config.get({
    query: { directory: projectDir },
  });
  if (configResponse.error) throw new Error(JSON.stringify(configResponse.error));
  result.effectiveConfig = configResponse.data;

  const events = await client.event.subscribe({
    signal: abort.signal,
    query: { directory: projectDir },
  });

  let sessionID;
  const reader = (async () => {
    try {
      for await (const event of events.stream) {
        if (event.type === 'session.next.model.switched') {
          result.modelEvents.push(event.properties.model);
        }
        if (sessionID && event.properties?.sessionID !== sessionID) continue;
        if (event.type === 'session.status') result.statusEvents.push(event.properties.status);
        if (event.type === 'session.error') result.sessionErrors.push(event.properties.error);
        if (event.type === 'message.part.delta' && event.properties.field === 'text') {
          result.text += event.properties.delta;
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
  result.session = created.data;

  process.env.OPENROUTER_API_KEY = changedApiKey;
  result.changedEnvironmentAfterSessionStart = true;

  const prompt = await client.session.promptAsync({
    path: { id: sessionID },
    query: { directory: projectDir },
    body: {
      parts: [
        {
          type: 'text',
          text: 'Reply with exactly phase0-openrouter-ok. Do not use tools. Do not read files.',
        },
      ],
    },
  });
  if (prompt.error) throw new Error(JSON.stringify(prompt.error));

  await sleep(7000);
  abort.abort();
  await reader;

  result.fakeOpenRouterRequests = fakeOpenRouter.requests.map((request) => ({
    ...request,
    body: request.body && typeof request.body === 'object'
      ? {
        model: request.body.model,
        stream: request.body.stream,
        messages: summarizeMessages(request.body.messages),
        toolsPresent: Array.isArray(request.body.tools),
        provider: request.body.provider,
        models: request.body.models,
        route: request.body.route,
        plugins: request.body.plugins,
      }
      : request.body,
  }));

  const chatRequests = fakeOpenRouter.requests.filter((request) => request.url?.includes('/chat/completions'));
  const firstChat = chatRequests[0];
  const authHeader = firstChat?.headers.authorization;

  result.summary = {
    opencodeServerUrl: opencodeServer.url,
    opencodeLaunchArgs: opencodeServer.args,
    effectiveModel: result.effectiveConfig?.model,
    effectiveSmallModel: result.effectiveConfig?.small_model,
    effectiveEnabledProviders: result.effectiveConfig?.enabled_providers,
    effectiveDisabledProviders: result.effectiveConfig?.disabled_providers,
    modelEvents: result.modelEvents,
    assistantText: result.text,
    fakeOpenRouterRequestCount: fakeOpenRouter.requests.length,
    fakeOpenRouterChatRequestCount: chatRequests.length,
    firstChatUrl: firstChat?.url,
    firstChatModel: firstChat?.body?.model,
    firstChatUsesStartingCredential: authHeader === `Bearer ${startingApiKey}`,
    firstChatUsesChangedCredential: authHeader === `Bearer ${changedApiKey}`,
    firstChatAuthorizationPresent: typeof authHeader === 'string',
    routeFallbackRequested: firstChat?.body?.route === 'fallback',
    alternateModelsRequested: Array.isArray(firstChat?.body?.models),
    pluginsRequested: Array.isArray(firstChat?.body?.plugins),
    contextMessageSummary: summarizeMessages(firstChat?.body?.messages),
    passed:
      result.effectiveConfig?.model === selectedFullModel &&
      result.effectiveConfig?.small_model === selectedFullModel &&
      result.effectiveConfig?.enabled_providers?.includes('openrouter') &&
      !result.effectiveConfig?.enabled_providers?.includes('openai') &&
      chatRequests.length > 0 &&
      firstChat?.body?.model === selectedModel &&
      authHeader === `Bearer ${startingApiKey}` &&
      authHeader !== `Bearer ${changedApiKey}` &&
      !Array.isArray(firstChat?.body?.models) &&
      firstChat?.body?.route !== 'fallback',
  };

  console.log(JSON.stringify(result, null, 2));
} finally {
  abort.abort();
  opencodeServer?.close();
  await fakeOpenRouter.close();
  await sleep(500);
  await rm(projectDir, {
    recursive: true,
    force: true,
    maxRetries: 5,
    retryDelay: 200,
  });
}
