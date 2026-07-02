export function preparePromptAttachments() {
  const records = [
    attachmentRecord('att-file', 'file', 'notes.txt', 'text/plain', 900, {
      source: 'files-plugin',
      target: { path: '/repo/notes.txt' }
    }),
    attachmentRecord('att-screen', 'browser-screenshot', 'screen.png', 'image/png', 1200, {
      source: 'browser-plugin',
      target: { url: 'https://example.test', viewport: '1280x720' }
    }),
    attachmentRecord('att-ann-1', 'browser-annotation', 'annotation-1.png', 'image/png', 1100, {
      source: 'browser-plugin',
      target: { marker: 1, selector: '#hero', comment: 'Check headline' }
    }),
    attachmentRecord('att-ann-2', 'browser-annotation', 'annotation-2.png', 'image/png', 1100, {
      source: 'browser-plugin',
      target: { marker: 2, selector: '.cta', comment: 'Check action text' }
    }),
    attachmentRecord('att-archive', 'archive', 'archive.zip', 'application/zip', 800, {
      source: 'files-plugin',
      target: { path: '/repo/archive.zip' }
    })
  ];

  const openAiCompatibleParts = [{ type: 'input_text', text: 'Review these attachments.' }];
  const visibleWarnings = [];
  const degradations = [];

  for (const record of records) {
    if (record.mime === 'text/plain') {
      openAiCompatibleParts.push({
        type: 'input_file',
        filename: record.filename,
        file_data: `data:${record.mime};encoded`
      });
    } else if (record.mime.startsWith('image/')) {
      openAiCompatibleParts.push({
        type: 'input_image',
        image_url: `data:${record.mime};encoded`
      });
      if (record.kind === 'browser-annotation') {
        openAiCompatibleParts.push({
          type: 'input_text',
          text: `Annotation ${record.target.marker}: ${record.target.comment}`
        });
      }
    } else {
      visibleWarnings.push(`${record.filename} cannot be sent directly to this model; it will be listed as an unsupported attachment.`);
      degradations.push({
        attachmentId: record.id,
        reason: 'unsupported attachment type for provider path'
      });
    }
  }

  return {
    records,
    openAiCompatibleParts,
    visibleWarnings,
    degradations,
    redactedLog: JSON.stringify({
      attachments: records.map(({ id, kind, filename, mime }) => ({ id, kind, filename, mime }))
    }),
    activeBrowserAnnotationsAfterSend: []
  };
}

function attachmentRecord(id, kind, filename, mime, sizeBytes, extra) {
  return {
    id,
    kind,
    filename,
    mime,
    sizeBytes,
    memoryCapBytes: 2_000_000,
    providerCompatibility: 'openai-compatible',
    fallback: kind === 'archive' ? 'unsupported-listing' : 'direct-or-safe-adapter',
    redaction: 'no-raw-secret-values',
    ...extra
  };
}
