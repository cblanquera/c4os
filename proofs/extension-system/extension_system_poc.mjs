#!/usr/bin/env node
import { spawn } from 'node:child_process';
import { mkdtemp, mkdir, readFile, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import readline from 'node:readline';

const repoRoot = new URL('../../', import.meta.url).pathname.replace(/\/$/, '');
const evidencePath = join(repoRoot, 'poc/extension-system/extension-system-evidence-2026-06-15.md');
const fixtureRoot = await mkdtemp(join(tmpdir(), 'c4os-extension-poc-'));
const sourceRoot = new URL('.', import.meta.url).pathname;

const result = {
  startedAt: new Date().toISOString(),
  fixtureRoot,
  checks: [],
  policy: {
    defaultState: 'disabled',
    inventoryMode: 'static manifests before execution',
    provenanceRequired: true,
    unknownTools: 'approval_required',
    highRiskActions: 'blocked_or_approval_required',
    outputTaint: 'extension-output-untrusted'
  }
};

try {
  await createFixtures();

  await runCheck('static inventory reads manifests without executing extension code', async () => {
    const inventory = await buildInventory();
    assert(inventory.length === 4, `expected 4 inventory entries, got ${inventory.length}`);
    assert(inventory.every((entry) => entry.enabled === false), 'entries were not disabled by default');
    assert(inventory.every((entry) => entry.trust.state === 'untrusted'), 'entries were not untrusted by default');
    assert(inventory.every((entry) => entry.provenance.source), 'missing provenance source');
    assert(!globalThis.__PLUGIN_EXECUTED__, 'plugin code executed during inventory');
    assert(!globalThis.__HOOK_EXECUTED__, 'hook code executed during inventory');
    return { inventory };
  });

  await runCheck('skill is instruction content, not executable code', async () => {
    const inventory = await buildInventory();
    const skill = findEntry(inventory, 'skill', 'review-local-docs');
    assert(skill.execution.kind === 'instruction-only', 'skill was not instruction-only');
    assert(skill.permissions.length === 0, 'skill requested executable permissions');
    assert(skill.enabled === false, 'skill was enabled by default');
    return skill;
  });

  await runCheck('plugin manifest metadata is visible without executing plugin module', async () => {
    const inventory = await buildInventory();
    const plugin = findEntry(inventory, 'plugin', 'sample-code-indexer');
    assert(plugin.execution.kind === 'executable-plugin', 'plugin kind mismatch');
    assert(plugin.manifest.entrypoint === './plugin-code.mjs', 'plugin entrypoint missing');
    assert(plugin.permissions.includes('read:workspace'), 'plugin permission missing');
    assert(plugin.enabled === false, 'plugin was enabled by default');
    assert(!globalThis.__PLUGIN_EXECUTED__, 'plugin code executed during manifest inventory');
    return plugin;
  });

  await runCheck('hook proposal is represented without automatic execution', async () => {
    const inventory = await buildInventory();
    const hook = findEntry(inventory, 'hook', 'before-run-policy-note');
    assert(hook.execution.kind === 'event-binding-proposal', 'hook kind mismatch');
    assert(hook.event === 'beforeRun', 'hook event mismatch');
    assert(hook.enabled === false, 'hook was enabled by default');
    assert(hook.requiresApproval === true, 'hook did not require approval');
    assert(!globalThis.__HOOK_EXECUTED__, 'hook code executed during inventory');
    return hook;
  });

  await runCheck('harmless MCP server connects, lists tools, calls safe tool, and disconnects', async () => {
    const mcp = await connectMcp();
    try {
      const initialized = await mcp.request('initialize', {
        protocolVersion: '2025-03-26',
        capabilities: {},
        clientInfo: { name: 'c4os-extension-poc', version: '0.0.0' }
      });
      const listed = await mcp.request('tools/list', {});
      assert(listed.tools.length === 1, 'expected one harmless tool');
      assert(listed.tools[0].name === 'echo_safe', 'unexpected tool name');
      const called = await mcp.request('tools/call', {
        name: 'echo_safe',
        arguments: { message: 'hello' }
      });
      assert(called.content[0].text === 'echo:hello', 'safe tool call returned unexpected output');
      return {
        initialized,
        listed,
        called,
        trust: {
          state: 'untrusted',
          toolPolicy: 'approval_required_before_model_use',
          filesystemAccess: 'none-granted'
        }
      };
    } finally {
      await mcp.disconnect();
    }
  });

  await runCheck('unknown or high-risk extension actions route through policy', async () => {
    const unknownTool = approvalPolicy({
      extensionType: 'mcp',
      action: 'tool:unknown',
      risk: 'unknown'
    });
    const highRiskHook = approvalPolicy({
      extensionType: 'hook',
      action: 'beforeRun:shell',
      risk: 'high'
    });
    const safeSkillRead = approvalPolicy({
      extensionType: 'skill',
      action: 'read-instructions',
      risk: 'low'
    });
    assert(unknownTool.decision === 'approval_required', 'unknown tool did not require approval');
    assert(highRiskHook.decision === 'blocked_until_enabled', 'high-risk hook was not blocked');
    assert(safeSkillRead.decision === 'allow_inventory_only', 'skill inventory was not allowed');
    return { unknownTool, highRiskHook, safeSkillRead };
  });

  result.finishedAt = new Date().toISOString();
  result.summary = summarize(result.checks);
  await writeEvidence();
  console.log(`extension-system-poc:${result.summary.status}`);
  console.log(`evidence:${evidencePath}`);
} catch (error) {
  result.finishedAt = new Date().toISOString();
  result.error = String(error?.stack || error);
  result.summary = summarize(result.checks);
  await writeEvidence();
  throw error;
}

async function createFixtures() {
  await mkdir(join(fixtureRoot, 'skills', 'review-local-docs'), { recursive: true });
  await mkdir(join(fixtureRoot, 'plugins', 'sample-code-indexer'), { recursive: true });
  await mkdir(join(fixtureRoot, 'hooks'), { recursive: true });
  await mkdir(join(fixtureRoot, 'mcp'), { recursive: true });

  await writeFile(join(fixtureRoot, 'skills', 'review-local-docs', 'SKILL.md'), [
    '---',
    'name: review-local-docs',
    'description: Read local documentation and summarize risks.',
    '---',
    '',
    '# Review Local Docs',
    '',
    'Instruction-only fixture. It must never execute during inventory.',
    ''
  ].join('\n'));

  await writeJson(join(fixtureRoot, 'plugins', 'sample-code-indexer', 'plugin.json'), {
    id: 'sample-code-indexer',
    name: 'Sample Code Indexer',
    version: '0.0.0',
    entrypoint: './plugin-code.mjs',
    permissions: ['read:workspace'],
    provenance: {
      source: 'local-fixture',
      installSource: 'file://poc-fixture/plugins/sample-code-indexer'
    }
  });
  await writeFile(join(fixtureRoot, 'plugins', 'sample-code-indexer', 'plugin-code.mjs'), [
    'globalThis.__PLUGIN_EXECUTED__ = true;',
    'throw new Error("plugin code must not execute during inventory");',
    ''
  ].join('\n'));

  await writeJson(join(fixtureRoot, 'hooks', 'before-run-policy-note.json'), {
    id: 'before-run-policy-note',
    name: 'Before Run Policy Note',
    event: 'beforeRun',
    action: 'append-policy-note',
    enabled: false,
    requiresApproval: true,
    permissions: ['write:session-context'],
    provenance: {
      source: 'local-fixture',
      installSource: 'file://poc-fixture/hooks/before-run-policy-note.json'
    }
  });

  await writeJson(join(fixtureRoot, 'mcp', 'harmless-server.json'), {
    id: 'harmless-echo',
    name: 'Harmless Echo MCP',
    command: process.execPath,
    args: [join(sourceRoot, 'harmless-mcp-server.mjs')],
    permissions: [],
    provenance: {
      source: 'local-fixture',
      installSource: 'file://poc-fixture/mcp/harmless-server.json'
    }
  });
}

async function buildInventory() {
  const skillPath = join(fixtureRoot, 'skills', 'review-local-docs', 'SKILL.md');
  const pluginPath = join(fixtureRoot, 'plugins', 'sample-code-indexer', 'plugin.json');
  const hookPath = join(fixtureRoot, 'hooks', 'before-run-policy-note.json');
  const mcpPath = join(fixtureRoot, 'mcp', 'harmless-server.json');

  const skillContent = await readFile(skillPath, 'utf8');
  const plugin = JSON.parse(await readFile(pluginPath, 'utf8'));
  const hook = JSON.parse(await readFile(hookPath, 'utf8'));
  const mcp = JSON.parse(await readFile(mcpPath, 'utf8'));

  return [
    {
      type: 'skill',
      id: 'review-local-docs',
      name: 'review-local-docs',
      enabled: false,
      trust: { state: 'untrusted', reason: 'local fixture not reviewed' },
      provenance: { source: 'local-fixture', installSource: `file://${skillPath}` },
      permissions: [],
      execution: { kind: 'instruction-only' },
      contentDigest: `chars:${skillContent.length}`,
      taint: 'instruction-untrusted'
    },
    {
      type: 'plugin',
      id: plugin.id,
      name: plugin.name,
      enabled: false,
      trust: { state: 'untrusted', reason: 'executable package not reviewed' },
      provenance: plugin.provenance,
      permissions: plugin.permissions,
      execution: { kind: 'executable-plugin', entrypoint: plugin.entrypoint },
      manifest: plugin,
      taint: 'plugin-manifest-untrusted'
    },
    {
      type: 'hook',
      id: hook.id,
      name: hook.name,
      enabled: false,
      trust: { state: 'untrusted', reason: 'event binding requires explicit approval' },
      provenance: hook.provenance,
      permissions: hook.permissions,
      execution: { kind: 'event-binding-proposal', action: hook.action },
      event: hook.event,
      requiresApproval: hook.requiresApproval,
      taint: 'hook-proposal-untrusted'
    },
    {
      type: 'mcp',
      id: mcp.id,
      name: mcp.name,
      enabled: false,
      trust: { state: 'untrusted', reason: 'server not connected until user approval' },
      provenance: mcp.provenance,
      permissions: mcp.permissions,
      execution: { kind: 'stdio-mcp-server', command: mcp.command, args: mcp.args },
      taint: 'mcp-metadata-untrusted'
    }
  ];
}

async function connectMcp() {
  const config = JSON.parse(await readFile(join(fixtureRoot, 'mcp', 'harmless-server.json'), 'utf8'));
  const child = spawn(config.command, config.args, {
    stdio: ['pipe', 'pipe', 'pipe'],
    env: { PATH: process.env.PATH || '' }
  });
  const rl = readline.createInterface({ input: child.stdout });
  const pending = new Map();
  let id = 0;
  rl.on('line', (line) => {
    const message = JSON.parse(line);
    const handlers = pending.get(message.id);
    if (!handlers) {
      return;
    }
    pending.delete(message.id);
    if (message.error) {
      handlers.reject(new Error(message.error.message));
    } else {
      handlers.resolve(message.result);
    }
  });

  child.stderr.on('data', (chunk) => {
    result.mcpStderr = `${result.mcpStderr || ''}${chunk}`;
  });

  return {
    request(method, params) {
      id += 1;
      const requestId = id;
      const payload = JSON.stringify({ jsonrpc: '2.0', id: requestId, method, params });
      child.stdin.write(`${payload}\n`);
      return new Promise((resolvePromise, reject) => {
        const timer = setTimeout(() => {
          pending.delete(requestId);
          reject(new Error(`MCP request timed out: ${method}`));
        }, 1000);
        pending.set(requestId, {
          resolve(value) {
            clearTimeout(timer);
            resolvePromise(value);
          },
          reject(error) {
            clearTimeout(timer);
            reject(error);
          }
        });
      });
    },
    async disconnect() {
      rl.close();
      child.kill('SIGTERM');
      await new Promise((resolvePromise) => {
        child.once('exit', resolvePromise);
        setTimeout(resolvePromise, 300);
      });
    }
  };
}

function approvalPolicy(request) {
  if (request.extensionType === 'skill' && request.action === 'read-instructions') {
    return { decision: 'allow_inventory_only', reason: 'static instruction inventory only' };
  }
  if (request.extensionType === 'hook' && request.risk === 'high') {
    return { decision: 'blocked_until_enabled', reason: 'hooks require explicit enablement' };
  }
  if (request.risk === 'unknown' || request.risk === 'high') {
    return { decision: 'approval_required', reason: 'unknown or high-risk extension action' };
  }
  return { decision: 'approval_required', reason: 'extensions default to approval gate' };
}

async function runCheck(name, fn) {
  const started = Date.now();
  try {
    const evidence = await fn();
    result.checks.push({ name, status: 'pass', durationMs: Date.now() - started, evidence });
  } catch (error) {
    result.checks.push({
      name,
      status: 'fail',
      durationMs: Date.now() - started,
      error: String(error?.message || error)
    });
    throw error;
  }
}

function findEntry(inventory, type, id) {
  const entry = inventory.find((candidate) => candidate.type === type && candidate.id === id);
  assert(entry, `missing ${type}:${id}`);
  return entry;
}

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function writeJson(path, value) {
  await writeFile(path, `${JSON.stringify(value, null, 2)}\n`);
}

function summarize(checks) {
  const failed = checks.filter((check) => check.status === 'fail').length;
  return {
    status: failed ? 'failed' : 'passed',
    passed: checks.filter((check) => check.status === 'pass').length,
    failed
  };
}

async function writeEvidence() {
  const summary = result.summary || summarize(result.checks);
  const lines = [
    '# Extension System POC Evidence: 2026-06-15',
    '',
    `Status: ${summary.status}`,
    `Started: ${result.startedAt}`,
    `Finished: ${result.finishedAt || ''}`,
    '',
    '## Scope',
    '',
    'Disposable static inventory and harmless MCP connection POC. This does not',
    'enable production Plugins, Skills, MCP Servers, or Hooks.',
    '',
    '## Checks',
    '',
    ...result.checks.map((check) => (
      `- ${check.status.toUpperCase()}: ${check.name} (${check.durationMs}ms)`
    )),
    '',
    '## Security Findings',
    '',
    `- Default state: ${result.policy.defaultState}`,
    `- Inventory mode: ${result.policy.inventoryMode}`,
    `- Unknown tools: ${result.policy.unknownTools}`,
    `- High-risk actions: ${result.policy.highRiskActions}`,
    `- Output taint: ${result.policy.outputTaint}`,
    '',
    '## Limits',
    '',
    '- This POC uses local fixture manifests and a harmless stdio MCP fixture.',
    '- It proves a shared inventory/trust schema is feasible for local sources.',
    '- Git/npm install flows, signatures, update policies, and real external MCP',
    '  servers still require follow-up POCs or implementation design.',
    '',
    '## Raw Result',
    '',
    '```json',
    JSON.stringify(result, null, 2),
    '```',
    ''
  ];
  await writeFile(evidencePath, lines.join('\n'));
}
