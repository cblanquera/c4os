import assert from 'node:assert/strict';
import test from 'node:test';

import {
  FINDING_001_PHASE0_GATES,
  evaluateRuntimeControlEvidence,
} from '../tools/phase0/runtime-control-harness.mjs';

test('FINDING-001 Phase 0 harness defines every mandatory runtime-control gate', () => {
  assert.deepEqual(
    FINDING_001_PHASE0_GATES.map((gate) => gate.id),
    [
      'integration-surface',
      'structured-events',
      'pre-execution-interception',
      'structured-denial',
      'stop-behavior',
      'app-owned-record-mapping',
      'runtime-config-isolation',
      'instruction-loading-observability',
    ],
  );

  for (const gate of FINDING_001_PHASE0_GATES) {
    assert.equal(gate.finding, 'FINDING-001');
    assert.equal(gate.phase, 'Phase 0');
    assert.ok(gate.requiredEvidence.length > 0);
  }
});

test('FINDING-001 remains blocked when any mandatory runtime-control gate lacks passing evidence', () => {
  const result = evaluateRuntimeControlEvidence([
    { gateId: 'integration-surface', status: 'passed', evidence: 'source reference' },
    { gateId: 'structured-events', status: 'passed', evidence: 'observed event payload' },
  ]);

  assert.equal(result.finding, 'FINDING-001');
  assert.equal(result.ready, false);
  assert.equal(result.recommendation, 'fallback-required');
  assert.deepEqual(result.missingGateIds, [
    'pre-execution-interception',
    'structured-denial',
    'stop-behavior',
    'app-owned-record-mapping',
    'runtime-config-isolation',
    'instruction-loading-observability',
  ]);
});

test('FINDING-001 can pass only when all mandatory gates have evidence-backed pass status', () => {
  const result = evaluateRuntimeControlEvidence(
    FINDING_001_PHASE0_GATES.map((gate) => ({
      gateId: gate.id,
      status: 'passed',
      evidence: `${gate.id} transcript`,
    })),
  );

  assert.equal(result.ready, true);
  assert.equal(result.recommendation, 'direct-opencode-viable');
  assert.deepEqual(result.missingGateIds, []);
});

test('failed mandatory evidence rejects direct OpenCode even if the rest of the matrix passes', () => {
  const evidence = FINDING_001_PHASE0_GATES.map((gate) => ({
    gateId: gate.id,
    status: 'passed',
    evidence: `${gate.id} transcript`,
  }));
  evidence.find((item) => item.gateId === 'pre-execution-interception').status = 'failed';

  const result = evaluateRuntimeControlEvidence(evidence);

  assert.equal(result.ready, false);
  assert.equal(result.recommendation, 'fallback-required');
  assert.deepEqual(result.missingGateIds, []);
  assert.deepEqual(result.failedGateIds, ['pre-execution-interception']);
});
