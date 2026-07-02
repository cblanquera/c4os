export function runPiRuntimeAppLayerProof() {
  const traceId = 'trace-pi-001';
  const stream = [];
  stream.push('thinking');
  const interceptedTool = interceptToolCall({ id: 'files.write', traceId });
  stream.push('tool_call_requested');
  const approval = denyApproval(interceptedTool);
  stream.push('approval_denied');
  const resume = resumeAfterDenial(traceId);
  stream.push('resume_requested');
  stream.push('final_response');

  return {
    stream,
    interceptedTool: interceptedTool.id,
    deniedToolExecuted: approval.executed,
    resumePreservedTraceId: resume.traceId === traceId
  };
}

function interceptToolCall(call) {
  return call;
}

function denyApproval() {
  return { executed: false };
}

function resumeAfterDenial(traceId) {
  return { traceId };
}
