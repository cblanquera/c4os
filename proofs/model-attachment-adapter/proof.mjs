export function adaptAttachments() {
  const prompt = 'summarize attachments';
  const attachments = [
    { filename: 'screen.png', mime: 'image/png', data: 'AAA' },
    { filename: 'notes.txt', mime: 'text/plain', data: 'SGk=' },
    { filename: 'archive.zip', mime: 'application/zip', data: 'UEs=' }
  ];
  const openAiCompatibleParts = [{ type: 'input_text', text: prompt }];
  const degradations = [];

  for (const attachment of attachments) {
    if (attachment.mime.startsWith('image/')) {
      openAiCompatibleParts.push({
        type: 'input_image',
        image_url: `data:${attachment.mime};base64,${attachment.data}`
      });
    } else if (attachment.mime === 'text/plain') {
      openAiCompatibleParts.push({
        type: 'input_file',
        filename: attachment.filename,
        file_data: `data:${attachment.mime};base64,${attachment.data}`
      });
    } else {
      degradations.push({
        filename: attachment.filename,
        reason: 'unsupported attachment type for provider path'
      });
    }
  }

  return {
    openAiCompatibleParts,
    degradations,
    redactedLog: JSON.stringify({ prompt, attachments: attachments.map(({ filename, mime }) => ({ filename, mime })) })
  };
}
