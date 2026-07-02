import test from 'node:test';
import assert from 'node:assert/strict';
import { prepareBrowserAnnotationBundle } from './proof.mjs';

test('many Browser annotations attach with screenshot metadata and clear active annotations after send', () => {
  const result = prepareBrowserAnnotationBundle();

  assert.equal(result.promptAttachmentBundle.kind, 'browser-evidence-bundle');
  assert.equal(result.promptAttachmentBundle.attachments.length, 3);
  assert.deepEqual(result.promptAttachmentBundle.attachments.map((item) => item.marker), [1, 2, 3]);
  assert.equal(result.promptAttachmentBundle.attachments.every((item) => item.kind === 'browser-annotation'), true);
  assert.equal(result.promptAttachmentBundle.attachments.every((item) => item.screenshot.scope === 'viewport-target-evidence'), true);
  assert.equal(result.promptAttachmentBundle.attachments.every((item) => item.url === 'https://example.test/dashboard'), true);
  assert.equal(result.promptAttachmentBundle.attachments.every((item) => item.frame === 'main'), true);
  assert.equal(result.promptAttachmentBundle.attachments.every((item) => item.selectorPath.length > 0), true);
  assert.equal(result.promptAttachmentBundle.attachments.every((item) => item.viewport === '1365x768@2x'), true);
  assert.equal(result.promptAttachmentBundle.attachments.every((item) => item.providerCompatibility === 'openai-compatible'), true);
  assert.deepEqual(result.activeAnnotationsAfterSend, []);
  assert.equal(result.events.at(-1).kind, 'browser.annotations.clearedAfterSend');
});
