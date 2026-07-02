import test from 'node:test';
import assert from 'node:assert/strict';
import { runPiRuntimeAppLayerProof } from './proof.mjs';

test('pi runtime app layer streams, intercepts tools, denies approval, and resumes', () => {
  const result = runPiRuntimeAppLayerProof();

  assert.deepEqual(result.stream, ['thinking', 'tool_call_requested', 'approval_denied', 'resume_requested', 'final_response']);
  assert.equal(result.interceptedTool, 'files.write');
  assert.equal(result.deniedToolExecuted, false);
  assert.equal(result.resumePreservedTraceId, true);
});
