import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { importConfig, inspectMarketplace, inspectPlugin, readToml } from './importer.mjs';

const proofDir = dirname(fileURLToPath(import.meta.url));
const configDir = join(proofDir, 'fixtures/config');
const pluginsDir = join(proofDir, 'fixtures/plugins');
const codexBin = '/Applications/ChatGPT.app/Contents/Resources/codex';

async function configs() {
  return {
    system: await readToml(join(configDir, 'system.toml')),
    user: await readToml(join(configDir, 'user.toml')),
    profile: await readToml(join(configDir, 'review.config.toml')),
    projects: [
      await readToml(join(configDir, 'project-root.toml')),
      await readToml(join(configDir, 'project-nested.toml')),
    ],
    requirements: await readToml(join(configDir, 'requirements.toml')),
  };
}

test('pins the locally validated Codex release', () => {
  const version = execFileSync(codexBin, ['--version'], { encoding: 'utf8' });
  assert.match(version, /codex-cli 0\.145\.0-alpha\.18/);
});

test('the pinned Codex strict parser accepts known config and rejects an unknown key', () => {
  const codexHome = mkdtempSync(join(tmpdir(), 'c4os-codex-config-'));
  const env = { ...process.env, CODEX_HOME: codexHome };
  writeFileSync(join(codexHome, 'config.toml'), [
    'model = "gpt-5.6"',
    'approval_policy = "on-request"',
    'sandbox_mode = "workspace-write"',
    'web_search = "cached"',
  ].join('\n'));
  const known = spawnSync(codexBin, ['app-server', '--strict-config', '--listen', 'off'], {
    env, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'],
  });
  assert.match(known.stderr, /no transport configured/);
  assert.doesNotMatch(known.stderr, /unknown (?:field|key)/i);
  writeFileSync(join(codexHome, 'config.toml'), 'not_a_codex_key = true\n');
  const unknown = spawnSync(codexBin, ['app-server', '--strict-config', '--listen', 'off'], {
    env, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'],
  });
  assert.notEqual(unknown.status, 0);
  assert.match(unknown.stderr, /not_a_codex_key|unknown (?:field|key)/i);
});

test('imports trusted layers in documented precedence with explicit dispositions', async () => {
  const result = importConfig({
    ...await configs(),
    cli: { approval_policy: 'never' },
    trusted: true,
  });
  assert.equal(result.imported.model, 'gpt-nested');
  assert.equal(result.imported.sandbox_mode, 'workspace-write');
  assert.equal(result.imported.approval_policy, 'never');
  assert.equal(result.imported.model_reasoning_effort, 'high');
  assert.equal(result.imported.mcp_servers.docs.command, 'node');
  assert.equal(result.oneWay, true);
  assert.ok(result.report.some((item) => item.key === 'notify' && item.layer === 'project:0' && item.disposition === 'ignored'));
  assert.ok(result.report.some((item) => item.key === 'raw_api_key' && item.disposition === 'rejected'));
  assert.ok(result.report.some((item) => item.layer === 'requirements' && item.disposition === 'rejected'));
  assert.ok(result.report.some((item) => item.key === 'future_key' && item.disposition === 'ignored'));
});

test('suppresses every project layer when the project is untrusted', async () => {
  const result = importConfig({ ...await configs(), trusted: false });
  assert.equal(result.imported.model, 'gpt-profile');
  assert.equal(result.imported.sandbox_mode, 'read-only');
  assert.ok(result.report.some((item) => item.reason === 'untrusted-project'));
  assert.equal(result.report.some((item) => item.layer.startsWith('project:') && item.disposition !== 'ignored'), false);
});

test('preserves environment-key references while rejecting raw secret values', async () => {
  const result = importConfig({ ...await configs(), trusted: true });
  assert.equal(result.imported.model_providers.proxy.env_key, 'PROXY_API_KEY');
  assert.equal(JSON.stringify(result.imported).includes('sk-do-not-import'), false);
});

test('classifies skills, app/MCP, settings-extension, and hook bundles', async () => {
  const skills = await inspectPlugin(join(pluginsDir, 'skills-only'));
  const appMcp = await inspectPlugin(join(pluginsDir, 'app-mcp'));
  const settings = await inspectPlugin(join(pluginsDir, 'settings-extension'));
  const hooks = await inspectPlugin(join(pluginsDir, 'hooks'));
  assert.ok(skills.report.some((item) => item.key === 'skills' && item.disposition === 'accepted'));
  assert.ok(appMcp.report.some((item) => item.key === 'apps' && item.disposition === 'accepted'));
  assert.ok(appMcp.report.some((item) => item.key === 'mcpServers' && item.disposition === 'accepted'));
  assert.ok(settings.report.some((item) => item.key === 'settings' && item.disposition === 'translated'));
  assert.equal(hooks.hookTrustRequired, true);
  assert.ok(hooks.report.some((item) => item.key === 'hooks' && item.disposition === 'accepted-disabled'));
  for (const plugin of [skills, appMcp, settings, hooks]) assert.equal(plugin.enabled, false);
});

test('accepts marketplace metadata without activating plugin code', async () => {
  const result = await inspectMarketplace(join(proofDir, 'fixtures/.agents/plugins/marketplace.json'));
  assert.deepEqual(result, {
    name: 'c4os-proof-marketplace', pluginCount: 4, metadataOnly: true,
  });
});

test('the pinned Codex marketplace loader lists the fixture without installing it', () => {
  const codexHome = mkdtempSync(join(tmpdir(), 'c4os-codex-marketplace-'));
  const env = { ...process.env, CODEX_HOME: codexHome };
  const fixtureRoot = join(proofDir, 'fixtures');
  const added = JSON.parse(execFileSync(codexBin, [
    'plugin', 'marketplace', 'add', fixtureRoot, '--json',
  ], { env, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }));
  const listed = JSON.parse(execFileSync(codexBin, [
    'plugin', 'list', '--available', '--json',
  ], { env, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }));
  assert.equal(added.marketplaceName, 'c4os-proof-marketplace');
  assert.equal(listed.installed.length, 0);
  assert.deepEqual(listed.available.map((plugin) => plugin.pluginId).sort(), [
    'app-mcp@c4os-proof-marketplace',
    'hooks@c4os-proof-marketplace',
    'settings-extension@c4os-proof-marketplace',
    'skills-only@c4os-proof-marketplace',
  ]);
  assert.equal(listed.available.every((plugin) => plugin.enabled === false), true);
});
