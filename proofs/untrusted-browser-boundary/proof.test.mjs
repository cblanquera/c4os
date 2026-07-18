import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { UntrustedBrowserBoundary } from './controller.mjs';

const proofDir = dirname(fileURLToPath(import.meta.url));
const boundary = () => {
  const value = new UntrustedBrowserBoundary({ downloadRoot: '/safe/downloads' });
  value.createProfile('private');
  value.createProfile('signed-in', { persistent: true });
  return value;
};

test('current raw-Wry hostile-page evidence has no Tauri bridge or leaked secret', async () => {
  const evidence = await readFile(join(
    proofDir, '../native-browser-wry/native-browser-wry-evidence-2026-06-20.md',
  ), 'utf8');
  assert.match(evidence, /Status: passed-with-wry-ipc-warning/);
  assert.match(evidence, /"tauri_internals": false/);
  assert.match(evidence, /"leaked_secret": false/);
  assert.match(evidence, /"wry_ipc": true/);
  assert.match(evidence, /Wry window\.ipc was visible but unbound/);
});

test('allows only web navigation and mediates every popup', () => {
  const browser = boundary();
  assert.equal(browser.navigate('private', 'https://example.com/').type, 'navigation.allowed');
  for (const url of [
    'file:///etc/passwd', 'javascript:alert(1)', 'data:text/html,hello', 'tauri://localhost/',
  ]) assert.equal(browser.navigate('private', url).type, 'navigation.blocked');
  assert.equal(browser.requestPopup('private', 'https://example.com/new').type, 'popup.blocked');
});

test('permissions are explicit, origin/profile-bound, and single-use', () => {
  const browser = boundary();
  assert.equal(browser.grantOnce({
    profileId: 'signed-in', origin: 'https://meet.example', permission: 'camera', userApproved: false,
  }), false);
  assert.equal(browser.grantOnce({
    profileId: 'signed-in', origin: 'https://meet.example', permission: 'camera', userApproved: true,
  }), true);
  assert.equal(browser.consumePermission({
    profileId: 'private', origin: 'https://meet.example', permission: 'camera',
  }), false);
  assert.equal(browser.consumePermission({
    profileId: 'signed-in', origin: 'https://evil.example', permission: 'camera',
  }), false);
  assert.equal(browser.consumePermission({
    profileId: 'signed-in', origin: 'https://meet.example', permission: 'camera',
  }), true);
  assert.equal(browser.consumePermission({
    profileId: 'signed-in', origin: 'https://meet.example', permission: 'camera',
  }), false);
});

test('downloads require policy, stay in the granted root, and use exact authorization', () => {
  const browser = boundary();
  assert.deepEqual(browser.authorizeDownload({
    profileId: 'private', sourceUrl: 'https://example.com/a', filename: 'a.txt', policyDecision: 'ask',
  }), { allowed: false, reason: 'policy' });
  assert.deepEqual(browser.authorizeDownload({
    profileId: 'private', sourceUrl: 'https://example.com/a', filename: '../../escape.txt', policyDecision: 'allow',
  }), { allowed: false, reason: 'outside-download-root' });
  const authorization = browser.authorizeDownload({
    profileId: 'private', sourceUrl: 'https://example.com/a', filename: 'a.txt', policyDecision: 'allow',
  });
  assert.equal(browser.consumeDownload(authorization, {
    profileId: 'private', sourceUrl: 'https://example.com/a', target: '/safe/downloads/b.txt',
  }), false);
  assert.equal(browser.consumeDownload(authorization, authorization), true);
  assert.equal(browser.consumeDownload(authorization, authorization), false);
});

test('profiles do not share cookies and crash clears transient authority and stale events', () => {
  const browser = boundary();
  browser.setCookie('private', 'session', 'private-cookie');
  browser.setCookie('signed-in', 'session', 'account-cookie');
  assert.equal(browser.getCookie('private', 'session'), 'private-cookie');
  assert.equal(browser.getCookie('signed-in', 'session'), 'account-cookie');
  browser.grantOnce({
    profileId: 'signed-in', origin: 'https://meet.example', permission: 'camera', userApproved: true,
  });
  const oldGeneration = browser.generation;
  const download = browser.authorizeDownload({
    profileId: 'private', sourceUrl: 'https://example.com/a', filename: 'a.txt', policyDecision: 'allow',
  });
  browser.crash();
  assert.equal(browser.getCookie('private', 'session'), undefined);
  assert.equal(browser.getCookie('signed-in', 'session'), 'account-cookie');
  assert.equal(browser.consumePermission({
    profileId: 'signed-in', origin: 'https://meet.example', permission: 'camera',
  }), false);
  assert.equal(browser.consumeDownload(download, download), false);
  assert.deepEqual(browser.ingest({ type: 'page.loaded' }, oldGeneration), {
    accepted: false, reason: 'stale-generation',
  });
  browser.recover();
  assert.equal(browser.ingest({ type: 'page.loaded' }, browser.generation).accepted, true);
});
