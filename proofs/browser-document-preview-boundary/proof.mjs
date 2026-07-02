export function routeDocumentPreview() {
  const browserHostedPreviews = [
    hostedPreview('docx-preview-1', 'documents-plugin', 'docx', '<article>Rendered document</article>'),
    hostedPreview('xlsx-preview-1', 'spreadsheets-plugin', 'xlsx', '<table><tr><td>Rendered sheet</td></tr></table>'),
    hostedPreview('pdf-preview-1', 'browser-native', 'pdf', 'native-pdf-viewer')
  ];

  return {
    browserOwnsParsing: false,
    documentFamilyOwners: {
      docx: 'documents-plugin',
      xlsx: 'spreadsheets-plugin',
      pdf: 'browser-native'
    },
    browserHostedPreviews,
    unsupportedPreview: {
      extension: 'pages',
      action: 'show-boundary-message',
      visibleBoundaryMessage:
        'Install or enable a document-family plugin before Browser can host this rendered preview.'
    },
    events: browserHostedPreviews.map((preview) => ({
      kind: 'browser.documentPreview.handoff',
      sourcePlugin: preview.sourcePlugin,
      hostedBy: preview.hostedBy,
      format: preview.format,
      parsedFromProse: false
    }))
  };
}

function hostedPreview(id, sourcePlugin, format, renderedOutput) {
  return {
    id,
    sourcePlugin,
    format,
    renderedOutput,
    hostedBy: 'browser-plugin',
    sandboxed: true,
    browserBridgeExposedToDocument: false
  };
}
