import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import {
  legacyScenarioGroups,
  legacyScenarioKeys,
  scenarioCorpus,
} from './scenario-corpus.mjs';
import {
  consumeSingleUseAuthorization,
  createSingleUseAuthorization,
  evaluatePolicy,
  matchesRule,
} from './policy.mjs';

const intent = (overrides = {}) => ({
  surface: 'file', effects: ['read'], scope: 'workspace', initiator: 'agent',
  sensitivity: 'ordinary', reversibility: 'reversible', confidence: 'known',
  ...overrides,
});

test('the complete r012 compatibility corpus converts to composable facts', () => {
  const wireframeSource = readFileSync(
    new URL('../../wireframes/r012-cleanup/script.js', import.meta.url),
    'utf8',
  );
  const policyBlock = wireframeSource.slice(
    wireframeSource.indexOf('//advanced policy grouping'),
    wireframeSource.indexOf('  const options = [', wireframeSource.indexOf('//advanced policy grouping')),
  );
  const sourceKeys = [...policyBlock.matchAll(/'([a-z]+(?:\.[a-z]+)*)',/g)]
    .map((match) => match[1])
    .filter((key) => key.includes('.') || key === 'unknown');

  assert.equal(legacyScenarioKeys.length, 71);
  assert.equal(new Set(legacyScenarioKeys).size, 71);
  assert.deepEqual(legacyScenarioKeys, sourceKeys);
  assert.equal(Object.keys(legacyScenarioGroups).length, 9);
  assert.equal(scenarioCorpus.length, 71);
  for (const scenario of scenarioCorpus) {
    assert.ok(scenario.surface);
    assert.ok(scenario.effects.length);
    assert.ok(scenario.scope);
    assert.ok(scenario.initiator);
    assert.ok(scenario.sensitivity);
    assert.ok(scenario.reversibility);
    assert.ok(scenario.confidence);
    if (scenario.legacyKey.includes('unknown') || scenario.legacyKey === 'unknown') {
      assert.equal(scenario.confidence, 'ambiguous');
      assert.deepEqual(scenario.effects, ['unknown']);
    }
  }
});

test('the four presets are bounded and unknown never silently allows', () => {
  assert.equal(evaluatePolicy(intent(), { preset: 'ask' }).decision, 'allow');
  assert.equal(evaluatePolicy(intent({ effects: ['modify'] }), { preset: 'ask' }).decision, 'ask');
  assert.equal(evaluatePolicy(intent({ effects: ['modify'] }), { preset: 'safe' }).decision, 'allow');
  assert.equal(evaluatePolicy(intent({ scope: 'external-local' }), { preset: 'safe' }).decision, 'ask');
  assert.equal(evaluatePolicy(intent({ effects: ['execute'], surface: 'terminal' }), { preset: 'approve-for-me' }).decision, 'allow');
  assert.equal(evaluatePolicy(intent(), { preset: 'custom' }).decision, 'ask');
  assert.equal(evaluatePolicy(intent({ confidence: 'ambiguous', effects: ['unknown'] }), { preset: 'approve-for-me' }).decision, 'ask');
});

test('deny overrides ask and allow across every applicable category', () => {
  const action = intent({ surface: 'terminal', effects: ['execute', 'publish'], scope: 'remote' });
  const result = evaluatePolicy(action, {
    preset: 'approve-for-me',
    categoryRules: [
      { id: 'terminal', decision: 'allow', match: { surface: 'terminal' } },
      { id: 'publish', decision: 'ask', match: { effects: 'publish' } },
      { id: 'remote', decision: 'deny', match: { scope: 'remote' } },
    ],
  });
  assert.equal(result.decision, 'deny');
  assert.deepEqual(result.sources, ['rule:remote']);
});

test('non-bypassable ceilings constrain category rules and Approve for me', () => {
  const credential = intent({ surface: 'credential', effects: ['reveal'], sensitivity: 'credential' });
  assert.equal(evaluatePolicy(credential, {
    preset: 'approve-for-me',
    categoryRules: [{ id: 'secrets', decision: 'allow', match: { surface: 'credential' } }],
  }).decision, 'deny');

  const outsideWrite = intent({ effects: ['modify'], scope: 'external-local' });
  assert.equal(evaluatePolicy(outsideWrite, { preset: 'approve-for-me' }).decision, 'ask');
  assert.equal(evaluatePolicy({ ...outsideWrite, grantedScope: true }, { preset: 'approve-for-me' }).decision, 'allow');

  const managed = intent({ surface: 'network', effects: ['publish'], scope: 'remote' });
  assert.equal(evaluatePolicy(managed, {
    preset: 'approve-for-me',
    managedRules: [{ id: 'no-publish', decision: 'deny', match: { effects: 'publish' } }],
  }).decision, 'deny');
});

test('authenticated publishing must resolve its destination before automatic approval', () => {
  const publish = intent({
    surface: 'git', effects: ['publish'], scope: 'remote',
    sensitivity: 'authenticated', authenticated: true,
  });
  assert.equal(evaluatePolicy(publish, { preset: 'approve-for-me' }).decision, 'ask');
  assert.equal(evaluatePolicy({ ...publish, targetResolved: true }, { preset: 'approve-for-me' }).decision, 'allow');
});

test('remembered exceptions remain narrow and expire', () => {
  const status = intent({
    surface: 'git', tool: 'git.status', target: 'workspace:/repo', runtime: 'opencode',
  });
  const exception = {
    id: 'status-one-repo', kind: 'exception', decision: 'allow',
    match: {
      surface: 'git', tool: 'git.status', target: 'workspace:/repo', runtime: 'opencode',
    },
    expiresAt: '2026-07-19T00:00:00.000Z',
  };
  assert.equal(matchesRule(status, exception, Date.parse('2026-07-18T00:00:00.000Z')), true);
  assert.equal(matchesRule({ ...status, target: 'workspace:/other' }, exception), false);
  assert.equal(matchesRule({ ...status, runtime: 'pi' }, exception), false);
  assert.equal(matchesRule(status, exception, Date.parse('2026-07-20T00:00:00.000Z')), false);
});

test('authorization is exact, generation-bound, and single-use', () => {
  const action = intent({
    runtimeGeneration: 'oc-4', toolCallId: 'call-9', tool: 'file.write',
    argumentsHash: 'sha256:abc', target: 'workspace:/repo/a.txt',
  });
  const authorization = createSingleUseAuthorization(action, 'allow', 'nonce-1');
  assert.equal(consumeSingleUseAuthorization(authorization, {
    ...authorization.binding,
    target: 'workspace:/repo/b.txt',
  }), false);
  assert.equal(consumeSingleUseAuthorization(authorization, authorization.binding), true);
  assert.equal(consumeSingleUseAuthorization(authorization, authorization.binding), false);
});
