//client
import { createAppCommandClient } from './backend/tauri-adapter.js';
import {
  canSubmitPrompt,
  errorMessage,
  mcpCapabilityLabel,
  skillCapabilityLabel,
  sessionActivityMessage,
} from './ui/state.js';

const commandClient = createAppCommandClient();
const appElement = document.querySelector('#app');

const models = [
  'anthropic/claude-3.5-sonnet',
  'openai/gpt-4.1',
  'google/gemini-2.5-pro',
  'meta-llama/llama-3.1-70b-instruct',
];

let appStatus = null;
let notice = null;
let workingAction = null;
let statusPollTimer = null;

/**
 * Loads status and renders the interactive MVP shell.
 */
async function renderAppShell() {
  appStatus = await commandClient.getAppStatus();

  appElement.innerHTML = `
    <section class="app-shell ${workingAction ? 'is-working' : ''}" aria-labelledby="app-title">
      <header class="top-bar">
        <div>
          <p class="eyebrow">Local desktop control center</p>
          <h1 id="app-title">C4OS</h1>
        </div>
        <span class="status-pill">${statusLabel(appStatus)}</span>
      </header>

      ${noticeMarkup()}

      <section class="workflow-grid" aria-label="MVP first-run workflow">
        ${providerPanel()}
        ${projectPanel()}
        ${sessionPanel()}
      </section>

      <section class="status-grid" aria-label="Workspace status">
        ${statusTile('Boundary', 'Backend command invocation active')}
        ${statusTile('Telemetry', appStatus.telemetryEnabled ? 'Enabled' : 'Disabled')}
        ${statusTile('Diagnostics', appStatus.diagnostics.storageMode)}
        ${statusTile('Provider', appStatus.provider.ready ? 'Ready' : 'Setup required')}
        ${statusTile('Data flow', appStatus.provider.disclosure)}
        ${statusTile('Project', appStatus.project.active ? appStatus.project.rootPath : 'Not selected')}
        ${statusTile('Project selector', selectorLabel(appStatus))}
        ${statusTile('Instructions', appStatus.project.instructionResolution.supportTier)}
        ${statusTile('Skills', skillCapabilityLabel(appStatus))}
        ${statusTile('MCP', mcpCapabilityLabel(appStatus))}
        ${statusTile('Tool ledger', appStatus.timeline.toolActivityVisible ? 'Visible' : 'Hidden')}
        ${statusTile('Session', appStatus.session.runtimeState)}
        ${statusTile('Approvals', `${appStatus.approvals.pendingCount} pending`)}
        ${statusTile('Recovery', appStatus.recovery.canRetry ? 'Retry ready' : 'Idle')}
        ${statusTile('Changes', `${appStatus.changes.changedFileCount} files`)}
        ${statusTile('Artifacts', artifactLabel(appStatus))}
      </section>
    </section>
  `;

  bindFormHandlers();
  updateStatusPolling();
}

/**
 * Renders provider setup controls.
 */
function providerPanel() {
  const selectedModel = appStatus.provider.selectedModel ?? models[0];
  const disabled = workingAction ? 'disabled' : '';

  return `
    <form class="workflow-panel" data-form="provider">
      <div class="panel-heading">
        <span class="step-index">1</span>
        <div>
          <h2>Provider</h2>
          <p>${appStatus.provider.ready ? 'OpenRouter ready' : 'Setup required'}</p>
        </div>
      </div>

      <label>
        <span>OpenRouter key</span>
        <input
          autocomplete="off"
          ${disabled}
          name="apiKey"
          placeholder="sk-or-..."
          type="password"
        />
      </label>

      <label>
        <span>Model</span>
        <select ${disabled} name="selectedModel">
          ${models
            .map((model) => `
              <option value="${escapeHtml(model)}" ${model === selectedModel ? 'selected' : ''}>
                ${escapeHtml(model)}
              </option>
            `)
            .join('')}
        </select>
      </label>

      <button ${disabled} type="submit">${workingAction === 'provider' ? 'Saving...' : 'Save provider'}</button>
    </form>
  `;
}

/**
 * Renders project registration controls.
 */
function projectPanel() {
  const rootPath = appStatus.project.rootPath ?? '';
  const disabled = workingAction ? 'disabled' : '';

  return `
    <form class="workflow-panel" data-form="project">
      <div class="panel-heading">
        <span class="step-index">2</span>
        <div>
          <h2>Project</h2>
          <p>${appStatus.project.active ? 'Selected' : 'Not selected'}</p>
        </div>
      </div>

      <label>
        <span>Git project path</span>
        <input
          ${disabled}
          name="rootPath"
          placeholder="/Users/name/project"
          type="text"
          value="${escapeHtml(rootPath)}"
        />
      </label>

      <button ${disabled} type="submit">${workingAction === 'project' ? 'Selecting...' : 'Use project'}</button>
    </form>
  `;
}

/**
 * Renders prompt submission and transcript state.
 */
function sessionPanel() {
  const canSubmit = canSubmitPrompt(appStatus, workingAction);
  const disabled = canSubmit ? '' : 'disabled';
  const activityMessage = sessionActivityMessage(appStatus, workingAction);

  return `
    <form class="workflow-panel session-panel" data-form="session">
      <div class="panel-heading">
        <span class="step-index">3</span>
        <div>
          <h2>Session</h2>
          <p>${sessionDescription(appStatus)}</p>
        </div>
      </div>

      <label>
        <span>Prompt</span>
        <textarea
          ${disabled}
          name="prompt"
          placeholder="Ask the agent to inspect or change the selected project."
          rows="5"
        ></textarea>
      </label>

      <button ${disabled} type="submit">${workingAction === 'session' ? 'Starting...' : 'Start session'}</button>

      <div class="transcript" aria-label="Latest transcript">
        ${activityMessage ? `<p class="pending-state">${escapeHtml(activityMessage)}</p>` : ''}
        ${transcriptMarkup()}
      </div>
    </form>
  `;
}

/**
 * Binds form handlers after each render.
 */
function bindFormHandlers() {
  const providerForm = appElement.querySelector('[data-form="provider"]');
  const projectForm = appElement.querySelector('[data-form="project"]');
  const sessionForm = appElement.querySelector('[data-form="session"]');

  providerForm.addEventListener('submit', handleProviderSubmit);
  projectForm.addEventListener('submit', handleProjectSubmit);
  sessionForm.addEventListener('submit', handleSessionSubmit);
}

/**
 * Saves OpenRouter setup through the backend command boundary.
 */
async function handleProviderSubmit(event) {
  event.preventDefault();

  const formData = new FormData(event.currentTarget);

  await runAction('provider', 'Saving provider...', async () => {
    await commandClient.configureOpenRouter({
      apiKey: formData.get('apiKey'),
      selectedModel: formData.get('selectedModel'),
    });
    notice = { tone: 'success', message: 'Provider saved.' };
  });
}

/**
 * Registers and selects the local Git project path.
 */
async function handleProjectSubmit(event) {
  event.preventDefault();

  const formData = new FormData(event.currentTarget);

  await runAction('project', 'Selecting project...', async () => {
    await commandClient.registerProject({
      rootPath: formData.get('rootPath'),
    });
    notice = { tone: 'success', message: 'Project selected.' };
  });
}

/**
 * Starts the visible MVP session from the user's prompt.
 */
async function handleSessionSubmit(event) {
  event.preventDefault();

  const formData = new FormData(event.currentTarget);

  await runAction('session', 'Starting session. OpenCode may take up to two minutes to return...', async () => {
    await commandClient.submitPrompt({
      prompt: formData.get('prompt'),
    });
    notice = { tone: 'success', message: 'Session started.' };
  });
}

/**
 * Runs a workflow command and refreshes the shell.
 */
async function runAction(actionName, pendingMessage, action) {
  workingAction = actionName;
  notice = { tone: 'info', message: pendingMessage };
  await renderAppShell();
  await waitForPaint();

  try {
    await action();
  } catch (error) {
    notice = { tone: 'error', message: errorMessage(error) };
  } finally {
    workingAction = null;
  }

  await renderAppShell();
}

/**
 * Keeps active backend sessions visible without blocking the app shell.
 */
function updateStatusPolling() {
  if (appStatus?.session?.active && !statusPollTimer) {
    statusPollTimer = window.setInterval(() => {
      renderAppShell().catch((error) => {
        notice = { tone: 'error', message: errorMessage(error) };
      });
    }, 1500);
  }

  if (!appStatus?.session?.active && statusPollTimer) {
    window.clearInterval(statusPollTimer);
    statusPollTimer = null;
  }
}

/**
 * Yields long enough for pending UI to paint before a Tauri command starts.
 */
function waitForPaint() {
  return new Promise((resolve) => {
    requestAnimationFrame(() => requestAnimationFrame(resolve));
  });
}

/**
 * Renders a compact status tile.
 */
function statusTile(label, value) {
  return `
    <div>
      <span class="label">${escapeHtml(label)}</span>
      <strong>${escapeHtml(value)}</strong>
    </div>
  `;
}

/**
 * Summarizes the accepted artifact preview tier.
 */
function artifactLabel(status) {
  if (!status.artifacts.visibleRecords) {
    return 'Unavailable';
  }

  if (status.artifacts.supportTier === 'raster_image_preview_only') {
    return 'Text + raster images';
  }

  return 'Text viewer';
}

/**
 * Renders transient workflow feedback.
 */
function noticeMarkup() {
  if (!notice) {
    return '';
  }

  return `
    <div class="notice ${notice.tone}" role="status">
      ${escapeHtml(notice.message)}
    </div>
  `;
}

/**
 * Renders the current transcript or an empty placeholder.
 */
function transcriptMarkup() {
  const messages = appStatus.session.messages ?? [];

  if (!messages.length) {
    return '<p class="empty-state">No messages yet.</p>';
  }

  return messages
    .map((message) => `
      <article>
        <span>${escapeHtml(message.role)}</span>
        <p>${escapeHtml(message.content)}</p>
      </article>
    `)
    .join('');
}

/**
 * Derives the top-level readiness label.
 */
function statusLabel(status) {
  if (!status.provider.ready) {
    return 'Provider setup required';
  }

  if (!status.project.active) {
    return 'Project setup required';
  }

  return status.session.active ? 'Session running' : 'Ready to run';
}

/**
 * Describes the session form state.
 */
function sessionDescription(status) {
  if (!status.provider.ready) {
    return 'Save provider first';
  }

  if (!status.project.active) {
    return 'Select project first';
  }

  return status.session.active ? status.session.runtimeState : 'Ready';
}

/**
 * Describes project selector capability.
 */
function selectorLabel(status) {
  return status.project.selector.selectExactlyOneActive
    ? 'One active project'
    : 'Unavailable';
}

/**
 * Escapes HTML before interpolating user or backend values.
 */
function escapeHtml(value) {
  return `${value ?? ''}`
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#039;');
}

renderAppShell().catch((error) => {
  appElement.innerHTML = `
    <section class="app-shell error-state">
      <h1>C4OS</h1>
      <p>${escapeHtml(errorMessage(error))}</p>
    </section>
  `;
});
