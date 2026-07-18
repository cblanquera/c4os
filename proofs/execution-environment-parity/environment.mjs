import { createHash } from 'node:crypto';
import { spawn } from 'node:child_process';
import { mkdtemp, readFile, realpath, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, posix, resolve } from 'node:path';

const roots = {
  local: (workspace) => resolve(workspace),
  docker: () => '/workspace',
  ssh: () => '/srv/c4os/workspace',
};

export class ExecutionEnvironmentAdapter {
  constructor({ kind, id, host, workspaceRoot }) {
    if (!roots[kind]) throw new Error(`Unsupported environment: ${kind}`);
    this.kind = kind;
    this.id = id;
    this.host = host;
    this.localWorkspaceRoot = resolve(workspaceRoot);
    this.runtimeWorkspaceRoot = roots[kind](workspaceRoot);
    this.generation = 1;
    this.sequence = 0;
  }

  mapWorkspacePath(relativePath) {
    if (relativePath.startsWith('/') || relativePath.split('/').includes('..')) {
      throw new Error('Path must be workspace-relative');
    }
    return this.kind === 'local'
      ? resolve(this.runtimeWorkspaceRoot, relativePath)
      : posix.join(this.runtimeWorkspaceRoot, relativePath);
  }

  environmentIdentity() {
    return {
      kind: this.kind, id: this.id, host: this.host,
      runtimeWorkspaceRoot: this.runtimeWorkspaceRoot,
      generation: this.generation,
    };
  }

  authorize(action) {
    return {
      environment: `${this.kind}:${this.id}`,
      generation: this.generation,
      actionHash: createHash('sha256').update(JSON.stringify(action)).digest('hex'),
      consumed: false,
    };
  }

  consumeAuthorization(authorization, action) {
    if (authorization.consumed || authorization.generation !== this.generation) return false;
    if (authorization.environment !== `${this.kind}:${this.id}`) return false;
    const hash = createHash('sha256').update(JSON.stringify(action)).digest('hex');
    if (authorization.actionHash !== hash) return false;
    authorization.consumed = true;
    return true;
  }

  event(type, detail = {}) {
    return {
      type, sequence: ++this.sequence, environment: this.environmentIdentity(), ...detail,
    };
  }

  contractJourney({ credentialRef = 'secret://provider/default' } = {}) {
    const file = this.mapWorkspacePath('notes/result.txt');
    const action = {
      surface: 'file', effects: ['modify'], scope: 'workspace', target: file,
      executionEnvironment: this.environmentIdentity(), credentialRef,
    };
    const authorization = this.authorize(action);
    const approved = this.consumeAuthorization(authorization, action);
    const artifact = {
      id: `artifact:${this.kind}:${this.id}:result`,
      environment: this.environmentIdentity(),
      runtimePath: file,
      workspaceRelativePath: 'notes/result.txt',
      provenance: { operation: 'file.write', approved },
    };
    return {
      events: [
        this.event('file.requested', { action }),
        this.event('approval.resolved', { decision: approved ? 'allow' : 'deny' }),
        this.event('terminal.started', { commandId: 'command-1' }),
        this.event('terminal.cancelled', { commandId: 'command-1' }),
        this.event('artifact.created', { artifact }),
      ],
      action,
      artifact,
      approved,
    };
  }
}

export async function runActualLocalJourney() {
  const workspace = await mkdtemp(join(tmpdir(), 'c4os-local-parity-'));
  const adapter = new ExecutionEnvironmentAdapter({
    kind: 'local', id: 'local-proof', host: 'localhost', workspaceRoot: workspace,
  });
  const outputPath = adapter.mapWorkspacePath('result.txt');
  await writeFile(outputPath, 'local journey');
  const content = await readFile(outputPath, 'utf8');

  const child = spawn(process.execPath, ['-e', 'setTimeout(() => {}, 10000)'], {
    stdio: 'ignore',
  });
  const cancelled = await new Promise((resolveCancelled) => {
    child.once('spawn', () => child.kill('SIGTERM'));
    child.once('exit', (_code, signal) => resolveCancelled(signal === 'SIGTERM'));
  });
  return {
    environment: adapter.environmentIdentity(),
    fileWritten: content === 'local journey',
    commandCancelled: cancelled,
    artifact: {
      id: 'artifact:local:local-proof:result',
      runtimePath: outputPath,
      workspaceRelativePath: 'result.txt',
      environment: adapter.environmentIdentity(),
    },
  };
}

const alpineImage = 'alpine:3.20@sha256:d9e853e87e55526f6b2917df91a2115c36dd7c696a35be12163d44e6e2a4b6bc';

/**
 * Runs the file, credential-reference, network, and cancellation journey in Docker.
 */
export async function runActualDockerJourney() {
  const createdWorkspace = await mkdtemp(join(tmpdir(), 'c4os-docker-parity-'));
  const workspace = await realpath(createdWorkspace);
  const containerName = `c4os-parity-${crypto.randomUUID()}`;
  const credentialRef = 'secret://ssh/proof-host';
  try {
    const fileResult = await runProcess('docker', [
      'run', '--rm', '--network', 'none',
      '--mount', `type=bind,source=${workspace},target=/workspace`,
      '--env', `C4OS_CREDENTIAL_REF=${credentialRef}`,
      alpineImage,
      'sh', '-c',
      'printf "docker journey" > /workspace/result.txt; printf "%s" "$C4OS_CREDENTIAL_REF" > /workspace/credential-ref.txt',
    ]);
    if (fileResult.code !== 0) throw new Error(`Docker file journey failed: ${fileResult.stderr}`);
    const [content, credential] = await Promise.all([
      readFile(join(workspace, 'result.txt'), 'utf8'),
      readFile(join(workspace, 'credential-ref.txt'), 'utf8'),
    ]);

    const child = spawn('docker', [
      'run', '--rm', '--name', containerName, '--network', 'none', alpineImage, 'sleep', '30',
    ], { stdio: 'ignore' });
    await new Promise((resolveReady, rejectReady) => {
      child.once('spawn', () => setTimeout(resolveReady, 500));
      child.once('error', rejectReady);
    });
    const stopResult = await runProcess('docker', ['stop', '--time', '1', containerName]);
    if (stopResult.code !== 0) throw new Error(`Docker cancellation failed: ${stopResult.stderr}`);
    const cancelled = await new Promise((resolveCancelled) => {
      child.once('exit', (code) => resolveCancelled(code !== 0));
    });
    const inspection = await runProcess('docker', ['container', 'inspect', containerName]);

    return {
      artifact: {
        environment: { kind: 'docker', id: containerName },
        runtimePath: '/workspace/result.txt',
        workspaceRelativePath: 'result.txt',
      },
      commandCancelled: cancelled,
      containerRemoved: inspection.code !== 0,
      credentialReferenceOnly: credential === credentialRef,
      fileWritten: content === 'docker journey',
      image: alpineImage,
      networkDisabled: true,
    };
  } finally {
    await runProcess('docker', ['rm', '--force', containerName]);
    await rm(workspace, { recursive: true, force: true });
  }
}

/**
 * Runs a child process and captures bounded proof output.
 */
async function runProcess(command, argumentsList) {
  const child = spawn(command, argumentsList, { stdio: ['ignore', 'pipe', 'pipe'] });
  let stdout = '';
  let stderr = '';
  child.stdout.on('data', (chunk) => {
    stdout = `${stdout}${chunk}`.slice(0, 8192);
  });
  child.stderr.on('data', (chunk) => {
    stderr = `${stderr}${chunk}`.slice(0, 8192);
  });
  return new Promise((resolveResult, rejectResult) => {
    child.once('error', rejectResult);
    child.once('exit', (code, signal) => resolveResult({ code, signal, stderr, stdout }));
  });
}
