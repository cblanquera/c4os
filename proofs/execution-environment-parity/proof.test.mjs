import test from 'node:test';
import assert from 'node:assert/strict';
import {
  ExecutionEnvironmentAdapter,
  runActualDockerJourney,
  runActualLocalJourney,
} from './environment.mjs';
import { runActualSshJourney } from './ssh-live.mjs';

const workspaceRoot = '/host/private/project';
const adapters = [
  new ExecutionEnvironmentAdapter({ kind: 'local', id: 'desktop', host: 'localhost', workspaceRoot }),
  new ExecutionEnvironmentAdapter({ kind: 'docker', id: 'container-1', host: 'docker-desktop', workspaceRoot }),
  new ExecutionEnvironmentAdapter({ kind: 'ssh', id: 'server-1', host: 'dev.example', workspaceRoot }),
];

test('all environment adapters produce the same journey categories', () => {
  const journeys = adapters.map((adapter) => adapter.contractJourney());
  const expected = journeys[0].events.map((event) => event.type);
  for (const journey of journeys) {
    assert.deepEqual(journey.events.map((event) => event.type), expected);
    assert.equal(journey.approved, true);
    assert.equal(journey.artifact.provenance.approved, true);
  }
});

test('paths and artifact provenance are qualified by execution environment', () => {
  const [local, docker, ssh] = adapters.map((adapter) => adapter.contractJourney());
  assert.equal(local.artifact.runtimePath, '/host/private/project/notes/result.txt');
  assert.equal(docker.artifact.runtimePath, '/workspace/notes/result.txt');
  assert.equal(ssh.artifact.runtimePath, '/srv/c4os/workspace/notes/result.txt');
  assert.equal(docker.artifact.runtimePath.includes('/host/private'), false);
  assert.equal(ssh.artifact.runtimePath.includes('/host/private'), false);
  assert.notEqual(local.artifact.id, docker.artifact.id);
  assert.notEqual(docker.artifact.id, ssh.artifact.id);
});

test('approval cannot replay across environment, generation, action, or use', () => {
  const local = adapters[0];
  const docker = adapters[1];
  const action = { tool: 'file.write', target: local.mapWorkspacePath('a.txt') };
  const authorization = local.authorize(action);
  assert.equal(docker.consumeAuthorization(authorization, action), false);
  assert.equal(local.consumeAuthorization(authorization, { ...action, target: local.mapWorkspacePath('b.txt') }), false);
  assert.equal(local.consumeAuthorization(authorization, action), true);
  assert.equal(local.consumeAuthorization(authorization, action), false);
  const next = local.authorize(action);
  local.generation += 1;
  assert.equal(local.consumeAuthorization(next, action), false);
});

test('credential references cross the contract without raw secret material', () => {
  for (const adapter of adapters) {
    const journey = adapter.contractJourney({ credentialRef: 'secret://github/publish' });
    const serialized = JSON.stringify(journey);
    assert.match(serialized, /secret:\/\/github\/publish/);
    assert.doesNotMatch(serialized, /ghp_|sk-|Bearer\s|rawSecret/);
  }
});

test('the Local Desktop file and cancellation journey executes for real', async () => {
  const result = await runActualLocalJourney();
  assert.equal(result.fileWritten, true);
  assert.equal(result.commandCancelled, true);
  assert.equal(result.environment.kind, 'local');
  assert.equal(result.artifact.environment.id, 'local-proof');
});

test('the Docker file, credential-reference, isolation, and cancellation journey executes for real', async () => {
  const result = await runActualDockerJourney();
  assert.equal(result.fileWritten, true);
  assert.equal(result.credentialReferenceOnly, true);
  assert.equal(result.networkDisabled, true);
  assert.equal(result.commandCancelled, true);
  assert.equal(result.containerRemoved, true);
  assert.equal(result.artifact.runtimePath, '/workspace/result.txt');
  assert.match(result.image, /^alpine:3\.20@sha256:/);
});

test('the OpenSSH file, credential-reference, host-key, and cancellation journey executes for real', async () => {
  const result = await runActualSshJourney();
  assert.equal(result.fileWritten, true);
  assert.equal(result.credentialReferenceOnly, true);
  assert.equal(result.hostKeyVerified, true);
  assert.equal(result.remoteProcessStopped, true);
  assert.equal(result.artifact.environment.kind, 'ssh');
  assert.equal(result.artifact.runtimePath, '/srv/c4os/workspace/result.txt');
});
