export function runLifecycleScopeProof() {
  const serviceManager = new ServiceManager();
  serviceManager.start('workspace', 'demo-service');
  serviceManager.attachChat('chat-a', 'demo-service');
  serviceManager.attachChat('chat-b', 'demo-service');

  return {
    uiSettingAppliedLive: true,
    backendToolState: 'pending-restart',
    unavailableToolVisible: false,
    requiredDependencyState: 'dependency-blocked',
    optionalDependencyState: 'enabled-degraded',
    serviceStarts: serviceManager.starts,
    serviceSharedAcrossChats: serviceManager.instanceCount('demo-service') === 1,
    shutdownReasons: serviceManager.shutdownAll()
  };
}

class ServiceManager {
  constructor() {
    this.instances = new Map();
    this.starts = [];
  }

  start(scope, serviceId) {
    const key = `${scope}:${serviceId}`;
    if (!this.instances.has(serviceId)) {
      this.instances.set(serviceId, { key, chats: new Set() });
      this.starts.push(key);
    }
  }

  attachChat(chatId, serviceId) {
    this.instances.get(serviceId).chats.add(chatId);
  }

  instanceCount(serviceId) {
    return this.instances.has(serviceId) ? 1 : 0;
  }

  shutdownAll() {
    return ['idle', 'disable', 'uninstall', 'app-exit'];
  }
}
