import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const proofDir = path.dirname(fileURLToPath(import.meta.url));
const source = fs.readFileSync(path.join(proofDir, 'src/main.rs'), 'utf8');
const cargo = fs.readFileSync(path.join(proofDir, 'Cargo.toml'), 'utf8');
const resultPath = path.join(proofDir, 'macos-wkwebview-production-boundary-result.json');

test('locks the selected public WebKit boundary', () => {
  assert.match(cargo, /tauri = "=2\.11\.2"/);
  assert.match(cargo, /objc2-web-kit = (?:"0\.3\.2"|\{ version = "0\.3\.2")/);
  assert.match(source, /WKPermissionDecision::Prompt/);
  assert.match(source, /WKWebsiteDataStore::dataStoreForIdentifier/);
  assert.match(source, /WKWebsiteDataStore::nonPersistentDataStore/);
  assert.doesNotMatch(source, /addScriptMessageHandler|with_ipc_handler|setURLSchemeHandler/);
  assert.doesNotMatch(source, /private_api|_killWebContentProcess|custom_data_directory/);
});

test('recorded native run satisfies every proof check', () => {
  assert.equal(fs.existsSync(resultPath), true, 'run the native proof before this assertion');
  const result = JSON.parse(fs.readFileSync(resultPath, 'utf8'));
  assert.equal(result.status, 'passed');
  assert.equal(result.checks.length >= 7, true);
  for (const check of result.checks) {
    assert.equal(check.status, 'pass', `${check.name}: ${JSON.stringify(check.evidence)}`);
  }
  assert.equal(result.reports.some(report => report.harnessError), false);
  assert.equal(result.target.tauri, '2.11.2');
  assert.equal(result.target.objc2_web_kit, '0.3.2');
  assert.equal(result.target.permission_decision, 'Prompt');
  assert.equal(result.target.page_ipc_handler, false);
  assert.equal(result.target.custom_profile_path, false);
});
