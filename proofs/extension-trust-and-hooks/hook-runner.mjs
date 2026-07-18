import { spawn } from 'node:child_process';
import { access, mkdtemp, realpath, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';

/**
 * Escapes a path for a macOS sandbox profile literal.
 */
function escapeProfilePath(path) {
  return path.replaceAll('\\', '\\\\').replaceAll('"', '\\"');
}

/**
 * Runs an explicitly trusted hook with a constrained macOS process boundary.
 */
export async function runHook(options) {
  if (!options.enabled || !options.explicitTrust || !options.signatureVerified) {
    throw new Error('Hook requires enabled, explicitly trusted, verified extension');
  }
  if (process.platform !== 'darwin') throw new Error('This proof runner requires macOS sandbox-exec');

  const createdWorkspace = await mkdtemp(join(tmpdir(), 'c4os-hook-workspace-'));
  const workspace = await realpath(createdWorkspace);
  const hookPath = resolve(options.hookPath);
  const nodePath = resolve(process.execPath);
  const nodeInstallRoot = dirname(dirname(nodePath));
  const nodeRuntimeRoot = nodePath.startsWith('/opt/homebrew/') ? '/opt/homebrew' : nodeInstallRoot;
  const profile = [
    '(version 1)',
    '(import "system.sb")',
    '(allow file-read-metadata (subpath "/Users"))',
    '(allow file-read-metadata (subpath "/private"))',
    '(allow file-read-metadata (subpath "/var"))',
    `(allow process-exec (literal "${escapeProfilePath(nodePath)}"))`,
    // Homebrew Node dynamically loads libnode from ../lib; self-contained
    // distributions simply receive a slightly broader read-only install root.
    `(allow file-read* (subpath "${escapeProfilePath(nodeRuntimeRoot)}"))`,
    `(allow file-read* (subpath "${escapeProfilePath(dirname(hookPath))}"))`,
    `(allow file-read* (subpath "${escapeProfilePath(workspace)}"))`,
    '(deny network*)',
    '(deny file-write*)',
    `(allow file-write* (subpath "${escapeProfilePath(workspace)}"))`,
  ].join('\n');
  const environment = {
    C4OS_EVENT: JSON.stringify(options.event),
    HOME: workspace,
    PATH: '/usr/bin:/bin',
    TMPDIR: workspace,
  };
  const child = spawn('/usr/bin/sandbox-exec', ['-p', profile, nodePath, hookPath], {
    cwd: workspace,
    detached: true,
    env: environment,
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  const maximumOutput = options.maximumOutput ?? 8192;
  let output = '';
  let errorOutput = '';
  let timedOut = false;
  child.stdout.on('data', (chunk) => {
    output = `${output}${chunk}`.slice(0, maximumOutput);
  });
  child.stderr.on('data', (chunk) => {
    errorOutput = `${errorOutput}${chunk}`.slice(0, maximumOutput);
  });
  const timer = setTimeout(() => {
    timedOut = true;
    try {
      process.kill(-child.pid, 'SIGKILL');
    } catch {
      // the process group already exited
    }
  }, options.timeoutMs ?? 1000);
  const result = await new Promise((resolveResult, rejectResult) => {
    child.once('error', rejectResult);
    child.once('exit', (code, signal) => resolveResult({ code, signal }));
  });
  clearTimeout(timer);
  const workspaceFile = join(workspace, 'hook-output.json');
  let workspaceWrite = false;
  try {
    await access(workspaceFile);
    workspaceWrite = true;
  } catch {
    workspaceWrite = false;
  }
  await rm(workspace, { recursive: true, force: true });
  return { ...result, errorOutput, output, timedOut, workspaceWrite };
}
