import test from 'node:test';
import assert from 'node:assert/strict';
import { adaptAttachments } from './proof.mjs';

test('attachment adapter translates supported attachments and records degradation for unsupported ones', () => {
  const result = adaptAttachments();

  assert.deepEqual(result.openAiCompatibleParts, [
    { type: 'input_text', text: 'summarize attachments' },
    { type: 'input_image', image_url: 'data:image/png;base64,AAA' },
    { type: 'input_file', filename: 'notes.txt', file_data: 'data:text/plain;base64,SGk=' }
  ]);
  assert.deepEqual(result.degradations, [
    { filename: 'archive.zip', reason: 'unsupported attachment type for provider path' }
  ]);
  assert.equal(result.redactedLog.includes('SGk='), false);
});
