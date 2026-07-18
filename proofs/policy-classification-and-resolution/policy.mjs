const weight = { allow: 0, ask: 1, deny: 2 };
const validDecisions = new Set(Object.keys(weight));

function strictest(decisions) {
  return decisions.reduce((current, decision) =>
    weight[decision] > weight[current] ? decision : current, 'allow');
}

function hasEffect(intent, effect) {
  return (intent.effects ?? []).includes(effect);
}

function matchValue(actual, expected) {
  if (Array.isArray(expected)) return expected.includes(actual);
  return actual === expected;
}

export function matchesRule(intent, rule, now = Date.now()) {
  if (!validDecisions.has(rule.decision)) return false;
  if (rule.expiresAt && new Date(rule.expiresAt).getTime() <= now) return false;

  const match = rule.match ?? {};
  for (const [field, expected] of Object.entries(match)) {
    if (field === 'effects') {
      const actual = intent.effects ?? [];
      const expectedEffects = Array.isArray(expected) ? expected : [expected];
      if (!expectedEffects.every((effect) => actual.includes(effect))) return false;
      continue;
    }
    if (!matchValue(intent[field], expected)) return false;
  }
  return true;
}

function ceilingDecisions(intent, managed = []) {
  const decisions = managed
    .filter((rule) => matchesRule(intent, rule))
    .map((rule) => ({ decision: rule.decision, source: `managed:${rule.id ?? 'rule'}` }));

  if (intent.confidence !== 'known' || hasEffect(intent, 'unknown')) {
    decisions.push({ decision: 'ask', source: 'ceiling:unknown' });
  }
  if (intent.sensitivity === 'credential' && hasEffect(intent, 'reveal')) {
    decisions.push({ decision: 'deny', source: 'ceiling:credential-reveal' });
  }
  if (intent.scope === 'system' && intent.reversibility === 'destructive') {
    decisions.push({ decision: 'deny', source: 'ceiling:destructive-system' });
  }
  const mutates = (intent.effects ?? []).some((effect) =>
    ['create', 'modify', 'delete', 'execute', 'control', 'publish', 'listen'].includes(effect));
  if (intent.scope === 'external-local' && mutates && !intent.grantedScope) {
    decisions.push({ decision: 'ask', source: 'ceiling:ungranted-external-write' });
  }
  if (intent.scope === 'remote' && hasEffect(intent, 'publish') &&
      (intent.sensitivity === 'authenticated' || intent.authenticated) && !intent.targetResolved) {
    decisions.push({ decision: 'ask', source: 'ceiling:unresolved-authenticated-publish' });
  }
  if (intent.declarationExceeded) {
    decisions.push({ decision: 'deny', source: 'ceiling:undeclared-authority' });
  }
  return decisions;
}

function presetDecision(intent, preset) {
  const effects = intent.effects ?? [];
  const readOnly = effects.length > 0 && effects.every((effect) =>
    ['read', 'capture'].includes(effect));
  const trustedRead = readOnly && intent.confidence === 'known' &&
    intent.scope === 'workspace' && intent.sensitivity === 'ordinary';
  const safeWorkspace = intent.confidence === 'known' && intent.scope === 'workspace' &&
    intent.sensitivity === 'ordinary' && intent.reversibility === 'reversible' &&
    effects.every((effect) => ['read', 'capture', 'create', 'modify'].includes(effect));

  if (preset === 'ask') return trustedRead ? 'allow' : 'ask';
  if (preset === 'safe') return safeWorkspace ? 'allow' : 'ask';
  if (preset === 'approve-for-me') return 'allow';
  if (preset === 'custom') return 'ask';
  throw new Error(`Unknown preset: ${preset}`);
}

export function evaluatePolicy(intent, policy = {}) {
  const preset = policy.preset ?? 'ask';
  const ceiling = ceilingDecisions(intent, policy.managedRules ?? []);
  const matched = [...(policy.categoryRules ?? []), ...(policy.exceptions ?? [])]
    .filter((rule) => matchesRule(intent, rule, policy.now))
    .map((rule) => ({ decision: rule.decision, source: `${rule.kind ?? 'rule'}:${rule.id ?? 'anonymous'}` }));
  const applicable = ceiling.length || matched.length
    ? [...ceiling, ...matched]
    : [{ decision: presetDecision(intent, preset), source: `preset:${preset}` }];
  const decision = strictest(applicable.map((item) => item.decision));
  return {
    decision,
    sources: applicable.filter((item) => item.decision === decision).map((item) => item.source),
    matched: applicable,
  };
}

export function createSingleUseAuthorization(intent, decision, nonce) {
  if (decision !== 'allow') throw new Error('Only allowed actions can be authorized');
  const binding = {
    runtimeGeneration: intent.runtimeGeneration,
    toolCallId: intent.toolCallId,
    tool: intent.tool,
    argumentsHash: intent.argumentsHash,
    target: intent.target,
  };
  return { nonce, binding, consumed: false };
}

export function consumeSingleUseAuthorization(authorization, attempt) {
  if (authorization.consumed) return false;
  for (const [key, expected] of Object.entries(authorization.binding)) {
    if (attempt[key] !== expected) return false;
  }
  authorization.consumed = true;
  return true;
}
