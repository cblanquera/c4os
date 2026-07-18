import { readFile, realpath, stat } from 'node:fs/promises';
import { isAbsolute, relative, resolve } from 'node:path';
import { parseTomlSubset } from './toml-subset.mjs';

const accepted = new Set([
  'model', 'model_reasoning_effort', 'approval_policy', 'sandbox_mode', 'web_search',
]);
const translated = new Set(['mcp_servers', 'model_providers', 'skills', 'hooks']);
const projectIgnored = new Set([
  'openai_base_url', 'chatgpt_base_url', 'apps_mcp_product_sku', 'model_provider',
  'model_providers', 'notify', 'profile', 'profiles',
  'experimental_realtime_ws_base_url', 'otel',
]);

export async function readToml(path) {
  return parseTomlSubset(await readFile(path, 'utf8'));
}

function applyLayer(target, report, layerName, config, { project = false } = {}) {
  for (const [key, value] of Object.entries(config)) {
    if (project && projectIgnored.has(key)) {
      report.push({ layer: layerName, key, disposition: 'ignored', reason: 'machine-local-project-key' });
      continue;
    }
    if (/api[_-]?key|access[_-]?token|password|secret/i.test(key) ||
        (typeof value === 'string' && /^(?:sk-|Bearer\s)/i.test(value))) {
      report.push({ layer: layerName, key, disposition: 'rejected', reason: 'raw-secret' });
      continue;
    }
    if (accepted.has(key)) {
      target[key] = structuredClone(value);
      report.push({ layer: layerName, key, disposition: 'accepted' });
    } else if (translated.has(key)) {
      target[key] = structuredClone(value);
      report.push({ layer: layerName, key, disposition: 'translated' });
    } else if (key === 'notify') {
      report.push({ layer: layerName, key, disposition: 'rejected', reason: 'machine-local-command' });
    } else {
      report.push({ layer: layerName, key, disposition: 'ignored', reason: 'unknown-field' });
    }
  }
}

export function importConfig({ system, user, profile, projects = [], cli = {}, trusted = true, requirements }) {
  const imported = {};
  const report = [];
  applyLayer(imported, report, 'system', system ?? {});
  applyLayer(imported, report, 'user', user ?? {});
  applyLayer(imported, report, 'profile', profile ?? {});
  if (trusted) {
    projects.forEach((config, index) => applyLayer(imported, report, `project:${index}`, config, { project: true }));
  } else if (projects.length) {
    report.push({ layer: 'project:*', key: '*', disposition: 'ignored', reason: 'untrusted-project' });
  }
  applyLayer(imported, report, 'cli', cli);
  if (requirements) {
    report.push({ layer: 'requirements', key: '*', disposition: 'rejected', reason: 'managed-policy-is-not-importable-config' });
  }
  return { imported, report, oneWay: true, schemaVersion: 'codex-0.145-c4os-import-v1' };
}

const portableManifest = new Set([
  'name', 'version', 'description', 'author', 'homepage', 'repository', 'license',
  'keywords', 'skills', 'mcpServers', 'apps', 'hooks', 'interface',
]);

async function resolveComponent(root, path) {
  if (typeof path !== 'string' || !path.startsWith('./')) throw new Error('Component path must start with ./');
  const canonicalRoot = await realpath(root);
  const candidate = await realpath(resolve(root, path));
  const rel = relative(canonicalRoot, candidate);
  if (rel.startsWith('..') || isAbsolute(rel)) throw new Error('Component escapes plugin root');
  await stat(candidate);
  return candidate;
}

export async function inspectPlugin(root) {
  const manifest = JSON.parse(await readFile(resolve(root, '.codex-plugin/plugin.json'), 'utf8'));
  if (!manifest.name || !manifest.version || !manifest.description) throw new Error('Missing plugin identity');
  const report = [];
  for (const [key, value] of Object.entries(manifest)) {
    if (portableManifest.has(key)) {
      if (['skills', 'mcpServers', 'apps', 'hooks'].includes(key)) await resolveComponent(root, value);
      report.push({ key, disposition: key === 'hooks' ? 'accepted-disabled' : 'accepted' });
    } else if (key === 'settings') {
      await resolveComponent(root, value);
      report.push({ key, disposition: 'translated', reason: 'c4os-extension-not-codex-portable-manifest' });
    } else {
      report.push({ key, disposition: 'ignored', reason: 'unknown-manifest-field' });
    }
  }
  if (manifest.skills) {
    const skill = await readFile(resolve(root, manifest.skills, 'hello/SKILL.md'), 'utf8');
    const frontmatter = skill.match(/^---\n([\s\S]*?)\n---/);
    if (!frontmatter || !/^name:\s*\S+/m.test(frontmatter[1]) ||
        !/^description:\s*.+/m.test(frontmatter[1])) {
      throw new Error('Invalid SKILL.md frontmatter');
    }
  }
  return { name: manifest.name, report, enabled: false, hookTrustRequired: Boolean(manifest.hooks) };
}

export async function inspectMarketplace(path) {
  const root = resolve(path, '../../..');
  const marketplace = JSON.parse(await readFile(path, 'utf8'));
  if (!marketplace.name || !marketplace.interface?.displayName || !Array.isArray(marketplace.plugins)) {
    throw new Error('Invalid marketplace identity');
  }
  for (const plugin of marketplace.plugins) {
    if (!plugin.name || !plugin.policy?.installation || !plugin.policy?.authentication || !plugin.category) {
      throw new Error('Incomplete marketplace plugin policy');
    }
    const sourcePath = typeof plugin.source === 'string' ? plugin.source : plugin.source?.path;
    await resolveComponent(root, sourcePath);
  }
  return { name: marketplace.name, pluginCount: marketplace.plugins.length, metadataOnly: true };
}
