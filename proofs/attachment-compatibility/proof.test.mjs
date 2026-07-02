import test from 'node:test';
import assert from 'node:assert/strict';
import { preparePromptAttachments } from './proof.mjs';

test('prompt attachments become C4OS records and adapt or degrade safely for OpenAI-compatible models', () => {
  const result = preparePromptAttachments();

  assert.deepEqual(result.records.map((record) => record.kind), [
    'file',
    'browser-screenshot',
    'browser-annotation',
    'browser-annotation',
    'archive'
  ]);
  assert.equal(result.records.every((record) => record.redaction === 'no-raw-secret-values'), true);
  assert.equal(result.records.every((record) => record.sizeBytes <= record.memoryCapBytes), true);
  assert.deepEqual(result.openAiCompatibleParts.map((part) => part.type), [
    'input_text',
    'input_file',
    'input_image',
    'input_image',
    'input_text',
    'input_image',
    'input_text'
  ]);
  assert.deepEqual(result.visibleWarnings, [
    'archive.zip cannot be sent directly to this model; it will be listed as an unsupported attachment.'
  ]);
  assert.deepEqual(result.degradations, [
    { attachmentId: 'att-archive', reason: 'unsupported attachment type for provider path' }
  ]);
  assert.equal(result.redactedLog.includes('base64'), false);
  assert.equal(result.activeBrowserAnnotationsAfterSend.length, 0);
});
