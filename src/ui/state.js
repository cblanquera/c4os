/**
 * Normalizes rejected Tauri/browser command values into visible notice text.
 */
export function errorMessage(error) {
  if (typeof error === 'string' && error.trim()) {
    return error;
  }

  if (error instanceof Error && error.message.trim()) {
    return error.message;
  }

  if (error && typeof error === 'object') {
    if (typeof error.message === 'string' && error.message.trim()) {
      return error.message;
    }

    return JSON.stringify(error);
  }

  return 'The command failed without an error message.';
}

/**
 * Determines whether the prompt form can start a new backend run.
 */
export function canSubmitPrompt(status, workingAction) {
  return Boolean(
    status.provider.ready
      && status.project.active
      && !status.session.active
      && !workingAction,
  );
}

/**
 * Returns the transcript-level activity message for pending/running sessions.
 */
export function sessionActivityMessage(status, workingAction) {
  if (workingAction === 'session') {
    return 'Starting OpenCode through OpenRouter...';
  }

  if (status.session.active) {
    return 'OpenCode is running through OpenRouter. Waiting for a response...';
  }

  return null;
}

/**
 * Summarizes the accepted Agent Skills support tier for compact status UI.
 */
export function skillCapabilityLabel(status) {
  if (status.skills?.supportTier !== 'explicit_discovery_and_invocation_only') {
    return 'Unavailable';
  }

  if (
    status.skills.autoInvocationAvailable
      || status.skills.scriptExecutionAvailable
  ) {
    return 'Unsupported scope';
  }

  return 'Explicit skills only';
}

/**
 * Summarizes the accepted local stdio MCP support tier for compact status UI.
 */
export function mcpCapabilityLabel(status) {
  if (status.mcp?.supportTier !== 'local_stdio_explicit_approval_only') {
    return 'Unavailable';
  }

  if (
    !status.mcp.localStdioAvailable
      || status.mcp.remoteAvailable
      || status.mcp.autoStartFromProjectFiles
  ) {
    return 'Unsupported scope';
  }

  return 'Local MCP approval only';
}

/**
 * Returns the bounded workflow purposes accepted for the foundation slice.
 */
export function workflowPurposeOptions(status) {
  const purposes = status.workflowOrganization?.allowedPurposes;

  if (!Array.isArray(purposes)) {
    return [];
  }

  return purposes.filter((purpose) => (
    purpose === 'research'
      || purpose === 'writing'
      || purpose === 'documentation'
      || purpose === 'analysis'
  ));
}

/**
 * Summarizes promoted project/session navigation without broader claims.
 */
export function workspaceNavigationLabel(status) {
  const projectSearch = Boolean(status.project?.selector?.searchAvailable);
  const projectWorkflow = Boolean(
    status.project?.selector?.workflowPurposeFilterAvailable,
  );
  const sessionSearch = Boolean(status.session?.catalog?.searchAvailable);
  const sessionWorkflow = Boolean(
    status.session?.catalog?.workflowPurposeFilterAvailable,
  );

  if (projectSearch && projectWorkflow && sessionSearch && sessionWorkflow) {
    return 'Project and session filters';
  }

  if (projectSearch || sessionSearch) {
    return 'Partial filters';
  }

  return 'Unavailable';
}

/**
 * Summarizes workflow labels while preserving no-hidden-context semantics.
 */
export function workflowContextLabel(status) {
  if (status.workflowOrganization?.modelContextEffect !== 'none') {
    return 'Review context behavior';
  }

  if (status.workflowOrganization?.autoContextInjection) {
    return 'Review context behavior';
  }

  return 'Labels only';
}

/**
 * Summarizes session resume state for compact workspace overview copy.
 */
export function resumableSessionLabel(status) {
  if (status.session?.active) {
    return status.session.runtimeState;
  }

  if (status.session?.id) {
    return status.session.runtimeState;
  }

  return 'No session';
}
