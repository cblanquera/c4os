import { createHash, randomBytes } from 'node:crypto';
import { readFile, realpath, mkdtemp } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, isAbsolute, join, relative, resolve } from 'node:path';
import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';

function redact(value) {
  return String(value)
    .replace(/(bearer\s+)[^\s]+/gi, '$1[REDACTED]')
    .replace(/((?:api[_-]?key|password|token)\s*[=:]\s*)[^\s,;]+/gi, '$1[REDACTED]');
}

async function sha256(path) {
  return createHash('sha256').update(await readFile(path)).digest('hex');
}

export class RuntimeSupervisor {
  constructor({ bundleRoot, manifest, maximumRestarts = 2 }) {
    this.bundleRoot = resolve(bundleRoot);
    this.manifest = structuredClone(manifest);
    this.maximumRestarts = maximumRestarts;
    this.restartCount = 0;
    this.generation = 0;
    this.logs = [];
    this.status = 'stopped';
  }

  async discover() {
    if (!this.manifest.signed) throw new Error('Unsigned sidecar manifest');
    const candidate = resolve(this.bundleRoot, this.manifest.relativePath);
    const canonicalBundle = await realpath(this.bundleRoot);
    const canonicalCandidate = await realpath(candidate);
    const rel = relative(canonicalBundle, canonicalCandidate);
    if (!isAbsolute(canonicalBundle) || rel.startsWith('..') || isAbsolute(rel)) {
      throw new Error('Sidecar escapes bundle root');
    }
    if (await sha256(canonicalCandidate) !== this.manifest.sha256) {
      throw new Error('Sidecar digest mismatch');
    }
    return canonicalCandidate;
  }

  async start({ expectedVersion = this.manifest.version } = {}) {
    const executable = await this.discover();
    this.token = randomBytes(32).toString('base64url');
    this.stateRoot = await mkdtemp(join(tmpdir(), `c4os-${this.manifest.runtime}-`));
    this.generation += 1;
    this.status = 'starting';
    this.child = spawn(process.execPath, [executable, this.token, this.stateRoot, this.manifest.version], {
      cwd: dirname(executable),
      detached: process.platform !== 'win32',
      env: { LANG: 'C', PATH: process.env.PATH ?? '' },
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    this.child.stderr.on('data', (chunk) => this.log(chunk));
    const ready = await this.waitForReady();
    if (ready.host !== '127.0.0.1') {
      await this.stop();
      throw new Error('Sidecar did not bind loopback');
    }
    if (ready.version !== expectedVersion) {
      await this.stop();
      throw new Error(`Version mismatch: expected ${expectedVersion}, received ${ready.version}`);
    }
    this.url = `http://${ready.host}:${ready.port}`;
    this.status = 'ready';
    return { ...ready, generation: this.generation, url: this.url };
  }

  waitForReady() {
    return new Promise((resolveReady, rejectReady) => {
      const timeout = setTimeout(() => rejectReady(new Error('Readiness timeout')), 5000);
      const lines = createInterface({ input: this.child.stdout });
      lines.on('line', (line) => {
        this.log(line);
        try {
          const message = JSON.parse(line);
          if (message.type === 'ready') {
            clearTimeout(timeout);
            lines.close();
            resolveReady(message);
          }
        } catch {}
      });
      this.child.once('exit', (code) => {
        if (this.status === 'starting') {
          clearTimeout(timeout);
          rejectReady(new Error(`Sidecar exited before ready: ${code}`));
        }
      });
    });
  }

  async health(token = this.token) {
    const response = await fetch(`${this.url}/health`, {
      headers: token ? { 'x-c4os-runtime-token': token } : {},
    });
    return { status: response.status, data: response.ok ? await response.json() : undefined };
  }

  async recordCrash() {
    await this.stop();
    this.restartCount += 1;
    if (this.restartCount > this.maximumRestarts) {
      this.status = 'degraded';
      return false;
    }
    return true;
  }

  log(value) {
    const safe = redact(value);
    this.logs.push(safe);
    return safe;
  }

  async stop() {
    const child = this.child;
    if (!child || child.exitCode !== null) {
      if (this.status !== 'degraded') this.status = 'stopped';
      return;
    }
    this.signalProcessTree(child, 'SIGTERM');
    await new Promise((resolveExit) => {
      const timer = setTimeout(() => {
        this.signalProcessTree(child, 'SIGKILL');
      }, 1500);
      child.once('exit', () => {
        clearTimeout(timer);
        resolveExit();
      });
    });
    if (this.status !== 'degraded') this.status = 'stopped';
  }

  signalProcessTree(child, signal) {
    try {
      if (process.platform === 'win32') child.kill(signal);
      else process.kill(-child.pid, signal);
    } catch {
      // the supervised process group already exited
    }
  }
}
