export function prepareBrowserAnnotationBundle() {
  const annotations = [
    annotation(1, '#summary-card', 'Use this total as evidence.'),
    annotation(2, '#risk-table tr:nth-child(2)', 'This row needs follow-up.'),
    annotation(3, 'button[data-action="approve"]', 'Confirm wording before approval.')
  ];

  const promptAttachmentBundle = {
    id: 'browser-evidence-1',
    kind: 'browser-evidence-bundle',
    sourcePlugin: 'browser-plugin',
    chatId: 'chat-1',
    attachments: annotations
  };

  return {
    promptAttachmentBundle,
    activeAnnotationsAfterSend: [],
    events: [
      ...annotations.map((item) => ({
        kind: 'browser.annotation.attached',
        marker: item.marker,
        attachmentId: item.id,
        parsedFromProse: false
      })),
      {
        kind: 'browser.annotations.clearedAfterSend',
        bundleId: promptAttachmentBundle.id,
        parsedFromProse: false
      }
    ]
  };
}

function annotation(marker, selectorPath, comment) {
  return {
    id: `annotation-${marker}`,
    kind: 'browser-annotation',
    marker,
    comment,
    url: 'https://example.test/dashboard',
    title: 'Operations Dashboard',
    frame: 'main',
    selectorPath,
    viewport: '1365x768@2x',
    screenshot: {
      id: `screenshot-${marker}`,
      mime: 'image/png',
      scope: 'viewport-target-evidence',
      bytes: 42_000
    },
    targetOutline: {
      style: 'codex-style-outline',
      marker
    },
    memoryCapBytes: 2_000_000,
    redaction: 'no-raw-secret-values',
    providerCompatibility: 'openai-compatible'
  };
}
