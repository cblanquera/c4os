export const FINDING_001_PHASE0_GATES = Object.freeze([
  {
    id: 'integration-surface',
    finding: 'FINDING-001',
    phase: 'Phase 0',
    requiredEvidence: [
      'Supported OpenCode API, protocol, source, or subprocess contract for direct runtime control.',
    ],
  },
  {
    id: 'structured-events',
    finding: 'FINDING-001',
    phase: 'Phase 0',
    requiredEvidence: [
      'Structured session, assistant, model, tool proposal, tool result, approval, denial, error, and stop events without terminal scraping.',
    ],
  },
  {
    id: 'pre-execution-interception',
    finding: 'FINDING-001',
    phase: 'Phase 0',
    requiredEvidence: [
      'File writes cannot execute before backend approval.',
      'Shell commands cannot execute before backend approval.',
      'Runtime-proposed Git state changes cannot execute before backend approval.',
    ],
  },
  {
    id: 'structured-denial',
    finding: 'FINDING-001',
    phase: 'Phase 0',
    requiredEvidence: [
      'Denied or blocked action is returned to OpenCode as a structured denial result.',
    ],
  },
  {
    id: 'stop-behavior',
    finding: 'FINDING-001',
    phase: 'Phase 0',
    requiredEvidence: [
      'Active model/runtime work cancels and app-supervised child processes terminate while partial records survive.',
    ],
  },
  {
    id: 'app-owned-record-mapping',
    finding: 'FINDING-001',
    phase: 'Phase 0',
    requiredEvidence: [
      'OpenCode session IDs, logs, and persistence map to app-owned canonical records without becoming source of truth.',
    ],
  },
  {
    id: 'runtime-config-isolation',
    finding: 'FINDING-001',
    phase: 'Phase 0',
    requiredEvidence: [
      'Existing OpenCode config cannot silently override provider, model, tool, permission, session, or instruction behavior.',
    ],
  },
  {
    id: 'instruction-loading-observability',
    finding: 'FINDING-001',
    phase: 'Phase 0',
    requiredEvidence: [
      'Runtime-native instruction loading is disabled, observable, or disclosed.',
    ],
  },
]);

const passingStatuses = new Set(['passed']);
const failingStatuses = new Set(['failed', 'fallback-required']);

export function evaluateRuntimeControlEvidence(evidenceItems) {
  const evidenceByGateId = new Map(
    evidenceItems.map((item) => [item.gateId, item]),
  );

  const missingGateIds = [];
  const failedGateIds = [];

  for (const gate of FINDING_001_PHASE0_GATES) {
    const evidence = evidenceByGateId.get(gate.id);

    if (evidence && failingStatuses.has(evidence.status)) {
      failedGateIds.push(gate.id);
      continue;
    }

    if (!evidence || !evidence.evidence || !passingStatuses.has(evidence.status)) {
      missingGateIds.push(gate.id);
    }
  }

  const ready = missingGateIds.length === 0 && failedGateIds.length === 0;

  return {
    finding: 'FINDING-001',
    ready,
    recommendation: ready ? 'direct-opencode-viable' : 'fallback-required',
    missingGateIds,
    failedGateIds,
  };
}
