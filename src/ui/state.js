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
