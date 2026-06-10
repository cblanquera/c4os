//client
import { createAppCommandClient } from './backend/tauri-adapter.js';

const commandClient = createAppCommandClient();
const appElement = document.querySelector('#app');

/**
 * Renders the initial application shell.
 */
async function renderAppShell() {
  const status = await commandClient.getAppStatus();

  appElement.innerHTML = `
    <section class="app-shell" aria-labelledby="app-title">
      <header class="top-bar">
        <div>
          <p class="eyebrow">Local desktop control center</p>
          <h1 id="app-title">C4OS</h1>
        </div>
        <span class="status-pill">Backend ready</span>
      </header>

      <section class="workspace-panel" aria-label="Foundation status">
        <div>
          <span class="label">Boundary</span>
          <strong>Backend command invocation active</strong>
        </div>
        <div>
          <span class="label">Telemetry</span>
          <strong>${status.telemetryEnabled ? 'Enabled' : 'Disabled'}</strong>
        </div>
        <div>
          <span class="label">Diagnostics</span>
          <strong>${status.diagnostics.storageMode}</strong>
        </div>
        <div>
          <span class="label">Provider</span>
          <strong>${status.provider.ready ? 'Ready' : 'Setup required'}</strong>
        </div>
        <div>
          <span class="label">Data flow</span>
          <strong>${status.provider.disclosure}</strong>
        </div>
        <div>
          <span class="label">Project</span>
          <strong>${status.project.active ? status.project.rootPath : 'Not selected'}</strong>
        </div>
        <div>
          <span class="label">Tool ledger</span>
          <strong>${status.timeline.toolActivityVisible ? 'Visible' : 'Hidden'}</strong>
        </div>
        <div>
          <span class="label">Session</span>
          <strong>${status.session.runtimeState}</strong>
        </div>
        <div>
          <span class="label">Approvals</span>
          <strong>${status.approvals.pendingCount} pending</strong>
        </div>
        <div>
          <span class="label">Recovery</span>
          <strong>${status.recovery.canRetry ? 'Retry ready' : 'Idle'}</strong>
        </div>
        <div>
          <span class="label">Changes</span>
          <strong>${status.changes.changedFileCount} files</strong>
        </div>
        <div>
          <span class="label">Artifacts</span>
          <strong>${status.artifacts.visibleRecords ? 'Text viewer' : 'Unavailable'}</strong>
        </div>
      </section>
    </section>
  `;
}

renderAppShell().catch((error) => {
  appElement.innerHTML = `
    <section class="app-shell error-state">
      <h1>C4OS</h1>
      <p>${error.message}</p>
    </section>
  `;
});
