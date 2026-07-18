import { spawn } from 'node:child_process';
import { chmod, mkdtemp, readFile, realpath, rm, writeFile } from 'node:fs/promises';
import { createServer } from 'node:net';
import { tmpdir, userInfo } from 'node:os';
import { join } from 'node:path';

/**
 * Reserves a loopback port for the isolated SSH fixture.
 */
async function reservePort() {
  const server = createServer();
  await new Promise((resolveReady, rejectReady) => {
    server.once('error', rejectReady);
    server.listen(0, '127.0.0.1', resolveReady);
  });
  const port = server.address().port;
  await new Promise((resolveClose) => server.close(resolveClose));
  return port;
}

/**
 * Runs a command and returns its bounded result.
 */
async function runProcess(command, argumentsList, options = {}) {
  const child = spawn(command, argumentsList, {
    env: options.env ?? process.env,
    stdio: ['ignore', 'pipe', 'pipe'],
  });
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

/**
 * Waits until a generated proof file exists.
 */
async function waitForFile(path, timeoutMs = 5000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      return await readFile(path, 'utf8');
    } catch {
      await new Promise((resolveWait) => setTimeout(resolveWait, 50));
    }
  }
  throw new Error(`Timed out waiting for ${path}`);
}

/**
 * Waits for the disconnected SSH command process to leave the host table.
 */
async function waitForProcessExit(pid, timeoutMs = 5000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      process.kill(pid, 0);
    } catch {
      return true;
    }
    await new Promise((resolveWait) => setTimeout(resolveWait, 50));
  }
  return false;
}

/**
 * Runs a real OpenSSH transport journey against an isolated loopback sshd.
 */
export async function runActualSshJourney() {
  const createdRoot = await mkdtemp(join(tmpdir(), 'c4os-ssh-parity-'));
  const root = await realpath(createdRoot);
  const workspace = join(root, 'workspace');
  const hostKey = join(root, 'host-key');
  const clientKey = join(root, 'client-key');
  const authorizedKeys = join(root, 'authorized-keys');
  const knownHosts = join(root, 'known-hosts');
  const config = join(root, 'sshd-config');
  const port = await reservePort();
  const username = userInfo().username;
  let server;
  let cancellationClient;
  try {
    await Promise.all([
      runProcess('/bin/mkdir', ['-p', workspace]),
      runProcess('/usr/bin/ssh-keygen', ['-q', '-t', 'ed25519', '-N', '', '-f', hostKey]),
      runProcess('/usr/bin/ssh-keygen', ['-q', '-t', 'ed25519', '-N', '', '-f', clientKey]),
    ]);
    await writeFile(authorizedKeys, await readFile(`${clientKey}.pub`));
    await chmod(authorizedKeys, 0o600);
    await writeFile(config, [
      `Port ${port}`,
      'ListenAddress 127.0.0.1',
      `HostKey ${hostKey}`,
      `AuthorizedKeysFile ${authorizedKeys}`,
      `PidFile ${join(root, 'sshd.pid')}`,
      'PasswordAuthentication no',
      'KbdInteractiveAuthentication no',
      'UsePAM no',
      'PermitRootLogin no',
      'StrictModes no',
      'AcceptEnv C4OS_CREDENTIAL_REF',
      `AllowUsers ${username}`,
      'LogLevel ERROR',
    ].join('\n'));
    const validation = await runProcess('/usr/sbin/sshd', ['-t', '-f', config]);
    if (validation.code !== 0) throw new Error(`sshd config failed: ${validation.stderr}`);

    server = spawn('/usr/sbin/sshd', ['-D', '-e', '-f', config], {
      detached: true,
      stdio: ['ignore', 'ignore', 'pipe'],
    });
    let serverError = '';
    server.stderr.on('data', (chunk) => {
      serverError = `${serverError}${chunk}`.slice(0, 8192);
    });
    await new Promise((resolveReady, rejectReady) => {
      server.once('spawn', resolveReady);
      server.once('error', rejectReady);
    });

    let scanned;
    for (let attempt = 0; attempt < 20; attempt += 1) {
      scanned = await runProcess('/usr/bin/ssh-keyscan', ['-p', String(port), '127.0.0.1']);
      if (scanned.code === 0 && scanned.stdout.includes('ssh-ed25519')) break;
      await new Promise((resolveWait) => setTimeout(resolveWait, 100));
    }
    if (!scanned?.stdout.includes('ssh-ed25519')) {
      throw new Error(`sshd did not become ready: ${serverError}`);
    }
    await writeFile(knownHosts, scanned.stdout);

    const clientBase = [
      '-F', '/dev/null', '-i', clientKey, '-p', String(port),
      '-o', 'BatchMode=yes', '-o', 'IdentitiesOnly=yes',
      '-o', 'StrictHostKeyChecking=yes', '-o', `UserKnownHostsFile=${knownHosts}`,
      '-o', 'SendEnv=C4OS_CREDENTIAL_REF', `${username}@127.0.0.1`,
    ];
    const credentialRef = 'secret://ssh/loopback-proof';
    const remoteCommand = [
      `printf 'ssh journey' > '${join(workspace, 'result.txt')}'`,
      `printf '%s' "$C4OS_CREDENTIAL_REF" > '${join(workspace, 'credential-ref.txt')}'`,
    ].join('; ');
    const fileResult = await runProcess('/usr/bin/ssh', [...clientBase, remoteCommand], {
      env: { ...process.env, C4OS_CREDENTIAL_REF: credentialRef },
    });
    if (fileResult.code !== 0) throw new Error(`SSH file journey failed: ${fileResult.stderr}`);

    const pidPath = join(workspace, 'remote.pid');
    cancellationClient = spawn('/usr/bin/ssh', [
      ...clientBase,
      `echo $$ > '${pidPath}'; exec sleep 30`,
    ], { stdio: 'ignore' });
    const remotePid = Number((await waitForFile(pidPath)).trim());
    cancellationClient.kill('SIGTERM');
    await new Promise((resolveExit) => cancellationClient.once('exit', resolveExit));
    cancellationClient = undefined;
    const cancelResult = await runProcess('/usr/bin/ssh', [
      ...clientBase,
      `kill -TERM ${remotePid}`,
    ]);
    if (cancelResult.code !== 0) {
      throw new Error(`SSH remote cancellation failed: ${cancelResult.stderr}`);
    }
    const remoteProcessStopped = await waitForProcessExit(remotePid);

    const [content, credential] = await Promise.all([
      readFile(join(workspace, 'result.txt'), 'utf8'),
      readFile(join(workspace, 'credential-ref.txt'), 'utf8'),
    ]);
    return {
      artifact: {
        environment: { kind: 'ssh', id: 'loopback-openssh-proof', host: '127.0.0.1' },
        runtimePath: '/srv/c4os/workspace/result.txt',
        workspaceRelativePath: 'result.txt',
      },
      credentialReferenceOnly: credential === credentialRef,
      fileWritten: content === 'ssh journey',
      hostKeyVerified: true,
      remoteProcessStopped,
      transport: 'OpenSSH loopback',
    };
  } finally {
    cancellationClient?.kill('SIGKILL');
    if (server?.pid) {
      try {
        process.kill(-server.pid, 'SIGTERM');
      } catch {
        // the isolated sshd process group already exited
      }
    }
    await rm(root, { recursive: true, force: true });
  }
}
