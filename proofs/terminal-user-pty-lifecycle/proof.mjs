export function createTerminalManager({ homeDir }) {
  const sessions = new Map();
  const panelEvents = [];

  return {
    panelEvents,
    openForChat({ chatId, projectPath, env = {} }) {
      if (sessions.has(chatId)) return sessions.get(chatId);
      const session = {
        id: `pty-${chatId}`,
        chatId,
        cwd: projectPath ?? homeDir,
        env: {
          C4OS_TERMINAL_KIND: 'user-pty',
          ...env
        },
        terminated: false
      };
      sessions.set(chatId, session);
      panelEvents.push({
        kind: 'pty:create',
        chatId,
        cwd: session.cwd,
        source: 'terminal-panel'
      });
      return session;
    },
    removeChat(chatId) {
      const session = sessions.get(chatId);
      if (!session) return { terminated: false, deletedState: false };
      session.terminated = true;
      sessions.delete(chatId);
      panelEvents.push({
        kind: 'pty:terminate',
        chatId,
        source: 'terminal-panel'
      });
      return { terminated: true, deletedState: true };
    },
    get(chatId) {
      return sessions.get(chatId);
    }
  };
}

export function createRuntimeToolGateway() {
  const events = [];

  return {
    events,
    runTerminalTool({ chatId, command, cwd }) {
      const result = {
        chatId,
        command,
        cwd,
        visibleInTerminalPanel: false,
        eventSources: ['thread-context', 'chat-debug']
      };
      events.push({
        kind: 'tool_call_requested',
        toolId: 'terminal.run',
        chatId,
        command,
        cwd,
        source: 'tool-gateway'
      });
      events.push({
        kind: 'tool_call_completed',
        toolId: 'terminal.run',
        chatId,
        source: 'tool-gateway'
      });
      return result;
    }
  };
}
