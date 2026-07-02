const SENSITIVE_VALUES = [
  'sk-live-provider-key',
  'Bearer secret-token',
  'raw-cookie-value',
  'database-password'
];

export function buildChatDebugHistory() {
  const runs = [
    run('run-001', 'chat-1', []),
    run('run-002', 'chat-1', [
      event('runtime.tool.requested', 'Browser navigation requested', {
        toolId: 'browser.open',
        target: 'https://example.test',
        authorization: 'Bearer secret-token'
      }),
      event('approval.decision', 'Allowed browser.open for example.test', {
        toolId: 'browser.open',
        decision: 'allow',
        targetScope: 'https://example.test',
        visibleIn: ['thread-context', 'chat-debug']
      })
    ]),
    run('run-003', 'chat-1', [
      event('plugin.setting.changed', 'Updated plugin setting', {
        pluginId: 'provider-openrouter',
        key: 'apiKey',
        value: 'sk-live-provider-key',
        sensitive: true
      }),
      event('terminal.tool.completed', 'Runtime terminal command completed', {
        toolId: 'terminal.run',
        exitCode: 0,
        env: { DATABASE_PASSWORD: 'database-password' }
      }),
      event('attachment.adapted', 'Attachment adapted for provider', {
        attachmentId: 'att-1',
        cookie: 'raw-cookie-value'
      }),
      event('structured.error', 'Provider request failed with redacted details', {
        code: 'provider.auth',
        providerKey: 'sk-live-provider-key'
      })
    ])
  ];

  const historyLimit = 2;
  const visibleRuns = runs.slice(-historyLimit).reverse().map((item) => ({
    ...item,
    events: item.events.map(redactEvent)
  }));
  const events = [...visibleRuns].reverse().flatMap((item) => item.events);
  const persistedJson = JSON.stringify(visibleRuns);

  return {
    visibleRuns,
    prunedRunIds: runs.slice(0, -historyLimit).map((item) => item.id),
    exportAvailable: false,
    approvalSurfaces: ['thread-context', 'chat-debug'],
    events,
    persistedJson,
    displayRows: events.map((item) => ({
      kind: item.kind,
      summary: item.summary,
      details: item.details
    }))
  };
}

function run(id, chatId, events) {
  return { id, chatId, events };
}

function event(kind, summary, details) {
  return { kind, summary, details, parsedFromProse: false };
}

function redactEvent(item) {
  return {
    ...item,
    details: redactValue(item.details)
  };
}

function redactValue(value) {
  if (typeof value === 'string') {
    return SENSITIVE_VALUES.includes(value) ? '[redacted]' : value;
  }
  if (Array.isArray(value)) {
    return value.map(redactValue);
  }
  if (value && typeof value === 'object') {
    return Object.fromEntries(
      Object.entries(value).map(([key, entry]) => {
        const sensitiveKey = /authorization|cookie|password|providerKey|apiKey|token|secret/i.test(key);
        return [key, sensitiveKey ? '[redacted]' : redactValue(entry)];
      })
    );
  }
  return value;
}
