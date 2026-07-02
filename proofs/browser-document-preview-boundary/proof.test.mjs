import test from 'node:test';
import assert from 'node:assert/strict';
import { routeDocumentPreview } from './proof.mjs';

test('Browser hosts rendered document output but document-family plugins own parsing', () => {
  const result = routeDocumentPreview();

  assert.equal(result.browserOwnsParsing, false);
  assert.deepEqual(result.documentFamilyOwners, {
    docx: 'documents-plugin',
    xlsx: 'spreadsheets-plugin',
    pdf: 'browser-native'
  });
  assert.deepEqual(result.browserHostedPreviews.map((preview) => preview.sourcePlugin), [
    'documents-plugin',
    'spreadsheets-plugin',
    'browser-native'
  ]);
  assert.equal(result.browserHostedPreviews.every((preview) => preview.hostedBy === 'browser-plugin'), true);
  assert.equal(result.browserHostedPreviews.every((preview) => preview.sandboxed === true), true);
  assert.equal(result.unsupportedPreview.visibleBoundaryMessage.includes('document-family plugin'), true);
  assert.equal(result.events.some((event) => event.kind === 'browser.documentPreview.handoff'), true);
});
