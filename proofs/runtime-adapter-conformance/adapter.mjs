import {
  consumeSingleUseAuthorization,
  createSingleUseAuthorization,
  evaluatePolicy,
} from '../policy-classification-and-resolution/policy.mjs';

export const requiredCapabilities = [
  'identity', 'availability', 'capabilities', 'workspace.binding', 'session.create',
  'session.resume', 'session.enumerate', 'session.send', 'session.receive',
  'session.cancel', 'session.close', 'events.normalized', 'tools.brokered',
  'models.mapped', 'recovery.generation', 'diagnostics.redacted',
];

const manifests = {
  opencode: {
    adapter: 'OCAdapter', nativeVersion: '1.18.3', transport: 'authenticated-loopback-http-sse',
    optional: [
      'session.fork', 'session.share', 'session.revert', 'prompt.async',
      'context.compact', 'context.skills', 'tools.dynamic',
      'permissions.nativeRequests', 'artifacts.nativeDiff', 'workspace.fileSearch',
      'workspace.symbolSearch', 'workspace.vcs', 'providers.oauth',
      'extensions.native', 'runtime.remoteConnect',
    ],
  },
  pi: {
    adapter: 'PIAdapter', nativeVersion: '0.80.10', transport: 'c4os-node-sdk-sidecar',
    optional: [
      'session.fork', 'session.treeNavigation', 'prompt.steer', 'prompt.followUp',
      'context.compact', 'context.skills', 'context.agentsFiles',
      'context.promptTemplates', 'tools.dynamic', 'tools.progress',
      'extensions.native', 'extensions.uiRequests', 'runtime.resourceReload',
    ],
    alternatives: {
      rpc: {
        available: true,
        selected: false,
        reason: 'RPC has a supported client and stronger process isolation, but SDK-sidecar exposes direct tool interception, resource control, and adapter-owned event correlation.',
      },
    },
  },
};

function redact(value) {
  return String(value)
    .replace(/(bearer\s+)[^\s]+/gi, '$1[REDACTED]')
    .replace(/((?:api[_-]?key|password|token)\s*[=:]\s*)[^\s,;]+/gi, '$1[REDACTED]');
}

export class ControlledRuntimeAdapter {
  constructor(kind) {
    if (!manifests[kind]) throw new Error(`Unsupported runtime: ${kind}`);
    this.kind = kind;
    this.manifest = manifests[kind];
    this.generation = 0;
    this.status = 'stopped';
    this.sessions = new Map();
    this.events = [];
    this.logs = [];
    this.sequence = 0;
  }

  probe() {
    return {
      runtime: this.kind,
      adapter: this.manifest.adapter,
      adapterVersion: 'proof-1',
      nativeVersion: this.manifest.nativeVersion,
      compatible: true,
    };
  }

  start(binding) {
    if (!binding?.workspaceId || !binding?.trustedRoot || !binding?.environment) {
      throw new Error('Explicit workspace, trusted root, and execution environment are required');
    }
    this.binding = structuredClone(binding);
    this.generation += 1;
    this.status = 'ready';
    return this.probe();
  }

  stop() {
    this.status = 'stopped';
  }

  describeCapabilities() {
    return {
      schemaVersion: 1,
      required: Object.fromEntries(requiredCapabilities.map((capability) => [capability, true])),
      optional: Object.fromEntries(this.manifest.optional.map((capability) => [capability, true])),
      transport: this.manifest.transport,
      alternatives: this.manifest.alternatives ?? {},
    };
  }

  createSession(c4osSessionId) {
    if (this.status !== 'ready') throw new Error('Runtime is not ready');
    const session = {
      c4osSessionId,
      nativeSessionId: `${this.kind}-${c4osSessionId}`,
      workspaceId: this.binding.workspaceId,
      generation: this.generation,
      state: 'idle',
    };
    this.sessions.set(c4osSessionId, session);
    return structuredClone(session);
  }

  resumeSession(c4osSessionId) {
    const session = this.sessions.get(c4osSessionId);
    if (!session) throw new Error('Unknown session');
    session.generation = this.generation;
    return structuredClone(session);
  }

  enumerateSessions() {
    return [...this.sessions.values()].map((session) => structuredClone(session));
  }

  send(c4osSessionId, input) {
    const session = this.sessions.get(c4osSessionId);
    if (!session) throw new Error('Unknown session');
    session.state = 'running';
    return this.emit('lifecycle.started', { c4osSessionId, input });
  }

  cancel(c4osSessionId) {
    const session = this.sessions.get(c4osSessionId);
    if (!session) throw new Error('Unknown session');
    if (session.state === 'cancelled') return false;
    session.state = 'cancelled';
    this.emit('lifecycle.cancelled', { c4osSessionId });
    return true;
  }

  closeSession(c4osSessionId) {
    return this.sessions.delete(c4osSessionId);
  }

  emit(category, detail = {}, nativeType = category) {
    const event = {
      runtime: this.kind,
      generation: this.generation,
      sequence: ++this.sequence,
      category,
      nativeType,
      receivedAt: new Date().toISOString(),
      ...detail,
    };
    this.events.push(event);
    return structuredClone(event);
  }

  ingestNativeEvent(nativeEvent, generation) {
    if (generation !== this.generation) return { accepted: false, reason: 'stale-generation' };
    const session = nativeEvent.c4osSessionId && this.sessions.get(nativeEvent.c4osSessionId);
    if (session?.state === 'cancelled') return { accepted: false, reason: 'cancelled-session' };
    return { accepted: true, event: this.emit(nativeEvent.category ?? 'diagnostic.unknown', nativeEvent, nativeEvent.type) };
  }

  buildActionIntent(request) {
    return {
      nativeTool: request.nativeTool,
      nativeArguments: structuredClone(request.nativeArguments),
      tool: request.tool,
      toolCallId: request.toolCallId,
      argumentsHash: request.argumentsHash,
      target: request.target,
      resolvedTargets: [...(request.resolvedTargets ?? [])],
      runtime: this.kind,
      runtimeGeneration: this.generation,
      workspaceId: this.binding.workspaceId,
      sessionId: request.sessionId,
      executionEnvironment: this.binding.environment,
      initiator: request.initiator ?? 'agent',
      plugin: request.plugin,
      mcp: request.mcp,
      authenticated: request.authenticated ?? false,
      credentialInvolvement: request.credentialInvolvement ?? false,
      surface: request.surface,
      effects: [...request.effects],
      scope: request.scope,
      sensitivity: request.sensitivity,
      reversibility: request.reversibility,
      confidence: request.confidence,
      targetResolved: request.targetResolved,
      grantedScope: request.grantedScope,
      declarationExceeded: request.declarationExceeded,
    };
  }

  async requestTool(request, policy, execute) {
    const actionIntent = this.buildActionIntent(request);
    this.emit('tool.requested', { c4osSessionId: request.sessionId, toolCallId: request.toolCallId, actionIntent });
    const policyResult = evaluatePolicy(actionIntent, policy);
    this.emit('approval.resolved', {
      c4osSessionId: request.sessionId,
      toolCallId: request.toolCallId,
      decision: policyResult.decision,
    });
    if (policyResult.decision !== 'allow') {
      return { executed: false, actionIntent, policyResult };
    }
    const authorization = createSingleUseAuthorization(actionIntent, 'allow', `${this.kind}-${request.toolCallId}`);
    if (!consumeSingleUseAuthorization(authorization, authorization.binding)) {
      throw new Error('Authorization binding failure');
    }
    const result = await execute(actionIntent);
    this.emit('tool.completed', { c4osSessionId: request.sessionId, toolCallId: request.toolCallId });
    return { executed: true, actionIntent, policyResult, result };
  }

  crash(message) {
    this.status = 'crashed';
    this.log(`runtime error: ${message}`);
    return this.generation;
  }

  recover() {
    if (this.status !== 'crashed') throw new Error('Runtime has not crashed');
    this.generation += 1;
    this.status = 'ready';
    for (const session of this.sessions.values()) session.generation = this.generation;
    return this.generation;
  }

  log(message) {
    const safe = redact(message);
    this.logs.push(safe);
    return safe;
  }

  diagnostics() {
    return {
      ...this.probe(), status: this.status, generation: this.generation,
      transport: this.manifest.transport, logs: [...this.logs],
    };
  }
}

export class OCAdapter extends ControlledRuntimeAdapter {
  constructor() { super('opencode'); }
}

export class PIAdapter extends ControlledRuntimeAdapter {
  constructor() { super('pi'); }
}
