import { createHash } from 'node:crypto';
import { cp, mkdir, readFile, readdir, realpath, rename, rm, stat } from 'node:fs/promises';
import { isAbsolute, join, relative, resolve } from 'node:path';

async function filesUnder(root, current = root) {
  const entries = await readdir(current, { withFileTypes: true });
  const files = [];
  for (const entry of entries.sort((a, b) => a.name.localeCompare(b.name))) {
    const path = join(current, entry.name);
    if (entry.isDirectory()) files.push(...await filesUnder(root, path));
    else if (entry.isFile()) files.push(path);
  }
  return files;
}

export async function digestDirectory(root) {
  const canonical = await realpath(root);
  const hash = createHash('sha256');
  for (const path of await filesUnder(canonical)) {
    hash.update(relative(canonical, path));
    hash.update('\0');
    hash.update(await readFile(path));
    hash.update('\0');
  }
  return hash.digest('hex');
}

function assertImmutable(source) {
  if (source.kind === 'git' && !/^[0-9a-f]{7,64}$/i.test(source.sha ?? '')) {
    throw new Error('Git source requires an immutable commit SHA');
  }
  if (source.kind === 'npm' && !/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(source.version ?? '')) {
    throw new Error('npm source requires an exact version');
  }
  if (!['git', 'npm', 'local-proof'].includes(source.kind)) throw new Error('Unsupported source kind');
}

async function ensureInside(root, path) {
  const canonicalRoot = await realpath(root);
  const canonicalPath = await realpath(path);
  const rel = relative(canonicalRoot, canonicalPath);
  if (rel.startsWith('..') || isAbsolute(rel)) throw new Error('Path escapes C4OS cache');
  return canonicalPath;
}

export class MarketplaceManager {
  constructor(cacheRoot) {
    this.cacheRoot = resolve(cacheRoot);
    this.records = new Map();
    this.revokedDigests = new Set();
    this.audit = [];
  }

  async initialize() {
    await mkdir(join(this.cacheRoot, 'plugins'), { recursive: true });
    await mkdir(join(this.cacheRoot, 'staging'), { recursive: true });
    this.cacheRoot = await realpath(this.cacheRoot);
  }

  async inspect(source) {
    assertImmutable(source);
    const manifest = JSON.parse(await readFile(join(source.path, '.codex-plugin/plugin.json'), 'utf8'));
    const digest = await digestDirectory(source.path);
    this.audit.push({ step: 'metadata-reviewed', plugin: manifest.name, version: manifest.version, digest });
    return { manifest, digest };
  }

  async install(source, expectedDigest) {
    const { manifest, digest } = await this.inspect(source);
    if (digest !== expectedDigest) throw new Error('Digest verification failed');
    if (this.revokedDigests.has(digest)) throw new Error('Digest is revoked');
    const destination = join(this.cacheRoot, 'plugins', manifest.name, manifest.version, digest);
    await mkdir(destination, { recursive: true });
    await cp(source.path, destination, { recursive: true, force: false });
    await ensureInside(this.cacheRoot, destination);
    const record = {
      name: manifest.name, version: manifest.version, digest, path: destination,
      enabled: false, source: structuredClone(source),
    };
    this.records.set(manifest.name, record);
    this.audit.push({ step: 'installed-disabled', plugin: manifest.name, version: manifest.version });
    return structuredClone(record);
  }

  enable(name) {
    const record = this.records.get(name);
    if (!record) throw new Error('Plugin is not installed');
    if (this.revokedDigests.has(record.digest)) throw new Error('Digest is revoked');
    record.enabled = true;
    this.audit.push({ step: 'enabled', plugin: name, version: record.version });
  }

  async update(name, source, expectedDigest, migrate) {
    const previous = this.records.get(name);
    if (!previous) throw new Error('Plugin is not installed');
    const { manifest, digest } = await this.inspect(source);
    if (manifest.name !== name) throw new Error('Update identity mismatch');
    if (digest !== expectedDigest) throw new Error('Digest verification failed');
    if (this.revokedDigests.has(digest)) throw new Error('Digest is revoked');
    const stage = join(this.cacheRoot, 'staging', `${name}-${manifest.version}-${digest}`);
    await rm(stage, { recursive: true, force: true });
    await cp(source.path, stage, { recursive: true });
    await ensureInside(this.cacheRoot, stage);
    try {
      await migrate({ from: structuredClone(previous), to: { manifest, digest, path: stage } });
      const destination = join(this.cacheRoot, 'plugins', name, manifest.version, digest);
      await mkdir(resolve(destination, '..'), { recursive: true });
      await rename(stage, destination);
      this.records.set(name, {
        name, version: manifest.version, digest, path: destination,
        enabled: false, source: structuredClone(source),
      });
      this.audit.push({ step: 'update-committed-disabled', plugin: name, version: manifest.version });
      return { updated: true, rolledBack: false };
    } catch (error) {
      await rm(stage, { recursive: true, force: true });
      this.records.set(name, previous);
      this.audit.push({ step: 'update-rolled-back', plugin: name, version: previous.version, error: error.message });
      return { updated: false, rolledBack: true, error: error.message };
    }
  }

  revoke(digest) {
    this.revokedDigests.add(digest);
    for (const record of this.records.values()) {
      if (record.digest === digest) record.enabled = false;
    }
    this.audit.push({ step: 'revoked', digest });
  }

  async uninstall(name) {
    const record = this.records.get(name);
    if (!record) return false;
    await ensureInside(this.cacheRoot, record.path);
    await rm(join(this.cacheRoot, 'plugins', name), { recursive: true, force: true });
    this.records.delete(name);
    this.audit.push({ step: 'uninstalled', plugin: name });
    return true;
  }

  get(name) {
    const record = this.records.get(name);
    return record && structuredClone(record);
  }
}
