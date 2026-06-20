#!/usr/bin/env node
import { createServer } from 'node:http';
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { chromium } from 'playwright';

const startedAt = new Date().toISOString();
const repoRoot = new URL('../../', import.meta.url).pathname.replace(/\/$/, '');
const evidencePath = join(repoRoot, '.agents/poc/native-browser-plugin-evidence-2026-06-15.md');

const localPreview = await startFixtureServer('local-preview');
const remotePreview = await startFixtureServer('remote-preview');
const shellServer = await startShellServer();

const result = {
  startedAt,
  localPreview: localPreview.url,
  remotePreview: remotePreview.url,
  shell: shellServer.url,
  checks: [],
  policy: {
    allowedProtocols: ['http:', 'https:'],
    blockedProtocols: ['file:', 'data:', 'javascript:'],
    cookies: 'defer persistent session support; sandboxed pages must not share app-shell cookies',
    downloads: 'defer downloads; native plugin must require explicit user destination and approval',
    externalBrowserHandoff: 'defer until user-facing browser policy is accepted'
  }
};

let browser;
try {
  browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  await page.goto(shellServer.url, { waitUntil: 'domcontentloaded' });

  await runCheck('shell exposes app bridge only to host page', async () => {
    const bridgeVisible = await page.evaluate(() => Boolean(globalThis.__C4OS_APP_BRIDGE__));
    assert(bridgeVisible, 'host shell bridge sentinel missing');
    return { bridgeVisible };
  });

  await runNavigationCheck(page, 'local preview loads in isolated surface', localPreview.url);
  await runNavigationCheck(page, 'remote preview loads in isolated surface', remotePreview.url);

  await runCheck('page scripts cannot access host app bridge or secret sentinel', async () => {
    const messages = await page.evaluate(() => globalThis.pocMessages);
    const latest = messages[messages.length - 1];
    assert(latest.bridgeVisible === false, 'framed page observed host app bridge');
    assert(latest.secretVisible === false, 'framed page observed host secret sentinel');
    assert(latest.topAccessDenied === true, 'top access was not denied');
    return latest;
  });

  await runCheck('navigation policy blocks file URL', async () => {
    const blocked = await page.evaluate(() => globalThis.pocNavigate('file:///etc/passwd'));
    assert(blocked.status === 'blocked', 'file URL was not blocked');
    assert(blocked.reason.includes('protocol'), 'block reason did not mention protocol');
    return blocked;
  });

  await runCheck('navigation policy blocks javascript URL', async () => {
    const blocked = await page.evaluate(() => globalThis.pocNavigate('javascript:alert(1)'));
    assert(blocked.status === 'blocked', 'javascript URL was not blocked');
    assert(blocked.reason.includes('protocol'), 'block reason did not mention protocol');
    return blocked;
  });

  await runCheck('reload and stop are observable through constrained shell state', async () => {
    const reloaded = await page.evaluate(() => globalThis.pocReload());
    const stopped = await page.evaluate(() => globalThis.pocStop());
    assert(reloaded.status === 'reloaded', 'reload state not observed');
    assert(stopped.status === 'stopped', 'stop state not observed');
    return { reloaded, stopped };
  });

  result.finishedAt = new Date().toISOString();
  result.summary = summarize(result.checks);
} catch (error) {
  result.finishedAt = new Date().toISOString();
  result.error = String(error?.stack || error);
  result.summary = summarize(result.checks);
  await writeEvidence(result);
  throw error;
} finally {
  await browser?.close();
  await closeServer(localPreview.server);
  await closeServer(remotePreview.server);
  await closeServer(shellServer.server);
}

await writeEvidence(result);
console.log(`native-browser-plugin-poc:${result.summary.status}`);
console.log(`evidence:${evidencePath}`);

async function runNavigationCheck(page, name, url) {
  await runCheck(name, async () => {
    const navigated = await page.evaluate((nextUrl) => globalThis.pocNavigate(nextUrl), url);
    assert(navigated.status === 'loaded', `navigation did not load: ${navigated.status}`);
    assert(navigated.currentUrl === url, `unexpected current URL: ${navigated.currentUrl}`);
    const latest = await page.evaluate(() => globalThis.pocMessages.at(-1));
    assert(latest?.kind === 'browser-page-report', 'framed page did not report over message boundary');
    return { navigated, latest };
  });
}

async function runCheck(name, fn) {
  const started = Date.now();
  try {
    const evidence = await fn();
    result.checks.push({ name, status: 'pass', durationMs: Date.now() - started, evidence });
  } catch (error) {
    result.checks.push({
      name,
      status: 'fail',
      durationMs: Date.now() - started,
      error: String(error?.message || error)
    });
    throw error;
  }
}

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function startFixtureServer(label) {
  const server = createServer((request, response) => {
    if (request.url === '/favicon.ico') {
      response.writeHead(404).end();
      return;
    }
    response.writeHead(200, {
      'content-type': 'text/html; charset=utf-8',
      'cache-control': 'no-store',
      'set-cookie': `${label}=fixture; SameSite=Lax`
    });
    response.end(fixturePage(label));
  });
  return listen(server);
}

async function startShellServer() {
  const server = createServer((request, response) => {
    if (request.url === '/favicon.ico') {
      response.writeHead(404).end();
      return;
    }
    response.writeHead(200, {
      'content-type': 'text/html; charset=utf-8',
      'cache-control': 'no-store'
    });
    response.end(shellPage());
  });
  return listen(server);
}

function listen(server) {
  return new Promise((resolve) => {
    server.listen(0, '127.0.0.1', () => {
      const { port } = server.address();
      resolve({ server, url: `http://127.0.0.1:${port}/` });
    });
  });
}

function closeServer(server) {
  return new Promise((resolve, reject) => {
    server.close((error) => error ? reject(error) : resolve());
  });
}

function fixturePage(label) {
  return `<!doctype html>
    <html>
      <head><meta charset="utf-8"><title>${escapeHtml(label)}</title></head>
      <body>
        <h1>${escapeHtml(label)}</h1>
        <script>
          const report = {
            kind: 'browser-page-report',
            label: ${JSON.stringify(label)},
            href: location.href,
            bridgeVisible: false,
            secretVisible: false,
            topAccessDenied: false,
            cookieWriteSucceeded: false,
            cookieValue: ''
          };
          try {
            report.bridgeVisible = Boolean(parent.__C4OS_APP_BRIDGE__);
          } catch (error) {
            report.topAccessDenied = true;
          }
          try {
            report.secretVisible = Boolean(parent.__C4OS_PROVIDER_SECRET__);
          } catch (error) {
            report.topAccessDenied = true;
          }
          try {
            document.cookie = 'poc_${label}=1; SameSite=Lax';
            report.cookieValue = document.cookie;
            report.cookieWriteSucceeded = document.cookie.includes('poc_${label}');
          } catch (error) {
            report.cookieValue = 'cookie-error:' + error.name;
          }
          parent.postMessage(report, '*');
        </script>
      </body>
    </html>`;
}

function shellPage() {
  return `<!doctype html>
    <html>
      <head><meta charset="utf-8"><title>Native Browser Plugin POC</title></head>
      <body>
        <iframe
          id="browser-surface"
          title="Isolated browser surface"
          sandbox="allow-scripts allow-forms"
          referrerpolicy="no-referrer"
        ></iframe>
        <script>
          globalThis.__C4OS_APP_BRIDGE__ = { invoke: () => 'host-only' };
          globalThis.__C4OS_PROVIDER_SECRET__ = 'must-not-leak';
          globalThis.pocMessages = [];
          globalThis.pocState = {
            status: 'idle',
            currentUrl: '',
            lastError: '',
            events: []
          };
          const allowedProtocols = new Set(['http:', 'https:']);
          const frame = document.getElementById('browser-surface');

          addEventListener('message', (event) => {
            globalThis.pocMessages.push(event.data);
            globalThis.pocState.events.push({
              kind: 'message',
              label: event.data && event.data.label,
              href: event.data && event.data.href
            });
          });

          frame.addEventListener('load', () => {
            globalThis.pocState.status = frame.src === 'about:blank' ? 'stopped' : 'loaded';
            globalThis.pocState.currentUrl = frame.src;
            globalThis.pocState.events.push({ kind: 'load', href: frame.src });
          });

          globalThis.pocNavigate = async (url) => {
            const parsed = new URL(url, location.href);
            if (!allowedProtocols.has(parsed.protocol)) {
              globalThis.pocState.status = 'blocked';
              globalThis.pocState.lastError = 'blocked protocol: ' + parsed.protocol;
              return { ...globalThis.pocState, reason: globalThis.pocState.lastError };
            }
            globalThis.pocState.status = 'loading';
            globalThis.pocState.currentUrl = parsed.href;
            frame.src = parsed.href;
            await new Promise((resolve) => setTimeout(resolve, 150));
            return { ...globalThis.pocState };
          };

          globalThis.pocReload = async () => {
            if (!globalThis.pocState.currentUrl) {
              return { ...globalThis.pocState, status: 'no-url' };
            }
            frame.src = globalThis.pocState.currentUrl;
            await new Promise((resolve) => setTimeout(resolve, 100));
            globalThis.pocState.status = 'reloaded';
            return { ...globalThis.pocState };
          };

          globalThis.pocStop = async () => {
            frame.src = 'about:blank';
            globalThis.pocState.status = 'stopped';
            globalThis.pocState.currentUrl = 'about:blank';
            return { ...globalThis.pocState };
          };
        </script>
      </body>
    </html>`;
}

async function writeEvidence(nextResult) {
  const lines = [
    '# Native Browser Plugin POC Evidence: 2026-06-15',
    '',
    `Status: ${nextResult.summary.status}`,
    `Started: ${nextResult.startedAt}`,
    `Finished: ${nextResult.finishedAt || ''}`,
    '',
    '## Scope',
    '',
    'Disposable loopback POC for an isolated browser surface, navigation policy,',
    'and constrained message boundary. This is not product Browser implementation.',
    '',
    '## Endpoints',
    '',
    `- Shell: ${nextResult.shell}`,
    `- Local preview: ${nextResult.localPreview}`,
    `- Remote-like preview: ${nextResult.remotePreview}`,
    '',
    '## Checks',
    '',
    ...nextResult.checks.map((check) => (
      `- ${check.status.toUpperCase()}: ${check.name} (${check.durationMs}ms)`
    )),
    '',
    '## Policy Findings',
    '',
    `- Allowed protocols: ${nextResult.policy.allowedProtocols.join(', ')}`,
    `- Blocked protocols: ${nextResult.policy.blockedProtocols.join(', ')}`,
    `- Cookies: ${nextResult.policy.cookies}`,
    `- Downloads: ${nextResult.policy.downloads}`,
    `- External browser handoff: ${nextResult.policy.externalBrowserHandoff}`,
    '',
    '## Raw Result',
    '',
    '```json',
    JSON.stringify(nextResult, null, 2),
    '```',
    ''
  ];
  await writeFile(evidencePath, lines.join('\n'));
}

function summarize(checks) {
  const failed = checks.filter((check) => check.status === 'fail').length;
  return {
    status: failed ? 'failed' : 'passed',
    passed: checks.filter((check) => check.status === 'pass').length,
    failed
  };
}

function escapeHtml(value) {
  return String(value)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;');
}
