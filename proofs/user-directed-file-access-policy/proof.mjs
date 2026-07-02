export function evaluateFileAccessPolicy() {
  return {
    userDirectedOutsideRead: decide({ kind: 'read', outsideProject: true, userDirected: true }),
    agentInitiatedOutsideRead: decide({ kind: 'read', outsideProject: true, userDirected: false }),
    trustedProjectWrite: decide({ kind: 'write', outsideProject: false, userDirected: false }),
    destructiveTrustedProjectDelete: decide({ kind: 'delete', outsideProject: false, destructive: true }),
    outsideProjectWriteWithoutExplicitRequest: decide({ kind: 'write', outsideProject: true, userDirected: false }),
    outsideProjectWriteWithExplicitRequest: decide({ kind: 'write', outsideProject: true, userDirected: true })
  };
}

function decide(request) {
  if (request.kind === 'read' && request.outsideProject) {
    return request.userDirected ? 'allow' : 'ask';
  }
  if (request.kind === 'delete' && request.destructive) return 'ask';
  if (request.kind === 'write' && request.outsideProject) {
    return request.userDirected ? 'allow' : 'ask';
  }
  if (request.kind === 'write') return 'allow';
  return 'ask';
}
