//workspace shell, conversation, composer, and artifact interactions
(function () {
  'use strict';

  var shell = document.querySelector('[data-panel-shell]');
  var markdown = window.MarkdownRuntime;
  if (!shell || !markdown) return;

  var thread = document.querySelector('[data-thread]');
  var composer = document.querySelector('[data-composer]');
  var composerDock = document.querySelector('.composer-dock');
  var modeControl = document.querySelector('[data-mode-control]');
  var modeTrigger = document.querySelector('[data-mode-trigger]');
  var modePopover = document.querySelector('[data-mode-popover]');
  var replyStrip = document.querySelector('[data-reply-strip]');
  var replyPanel = document.querySelector('[data-reply-panel]');
  var replyInput = document.querySelector('[data-reply-input]');
  var primaryAction = document.querySelector('[data-primary-action]');
  var promptStatus = document.querySelector('[data-prompt-status]');
  var copyNotifier = document.querySelector('[data-copy-notifier]');
  var threadHost = document.querySelector('[data-thread-host]');
  var threadScrollButton = document.querySelector(
    '[data-thread-scroll-bottom]',
  );
  var focusViewer = document.querySelector('[data-focus-viewer]');
  var leftChatPane = document.querySelector('[data-left-chat-pane]');
  var leftChatPaneThread = document.querySelector(
    '[data-left-chat-pane-thread]',
  );
  var leftChatPaneResize = document.querySelector(
    '[data-left-chat-pane-resize]',
  );
  var fileSourcePopover = document.querySelector('[data-file-source-popover]');
  var attachmentInput = document.querySelector('[data-attachment-input]');
  var attachmentTray = document.querySelector('[data-attachment-tray]');
  var appDropzone = document.querySelector('[data-app-dropzone]');
  var narrowQuery = window.matchMedia('(max-width: 991.98px)');
  var minimumWidth = 180;
  var minimumCenterWidth = 420;
  var minimumChatPaneHeight = 220;
  var currentMode =
    ['chat', 'files', 'browser', 'terminal'].indexOf(location.hash.slice(1)) >
    -1
      ? location.hash.slice(1)
      : 'chat';
  var replyTarget = null;
  var messageCounter = 10;
  var artifactCounter = 10;
  var copyTimer;
  var focusedArtifact = null;
  var focusedFolderPath = null;
  var focusedExplorerFile = null;
  var modeBeforeArtifact = null;
  var fileSelectionKind = 'file';
  var chatPaneHeight = null;
  var activePanelResizeCleanup = null;
  var activeChatPaneResizeCleanup = null;
  var terminalStreamTimer = null;
  var terminalStreamTick = 0;
  var agentStreamQueue = Promise.resolve();
  var reducedMotionQuery = window.matchMedia(
    '(prefers-reduced-motion: reduce)',
  );
  var draftAttachments = [];
  var attachmentUrls = [];
  var attachmentCounter = 0;
  var fileDragDepth = 0;
  var pendingModelSwitch = '';
  var modelProfiles = {
    'GPT-5': {
      capabilities: ['vision', 'tools', 'reasoning'],
      context: '400K',
      route: 'OpenAI - Work · OpenCode',
    },
    'GPT-5 fast': {
      capabilities: ['vision', 'tools'],
      context: '128K',
      route: 'OpenAI - Work · OpenCode',
    },
    'Kimi K2': {
      capabilities: ['tools'],
      context: '128K',
      route: 'OpenRouter - Personal · OpenCode',
    },
  };

  var icons = {
    copy: '<svg viewBox="0 0 24 24" aria-hidden="true"><rect x="9" y="9" width="12" height="12" rx="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>',
    reply:
      '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m9 17-5-5 5-5"></path><path d="M20 18v-2a4 4 0 0 0-4-4H4"></path></svg>',
    expand:
      '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7"></path></svg>',
    close:
      '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m18 6-12 12M6 6l12 12"></path></svg>',
    edit: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 20h9"></path><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L8 18l-4 1 1-4Z"></path></svg>',
    save: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m20 6-11 11-5-5"></path></svg>',
    trash:
      '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M3 6h18M8 6V4h8v2M19 6l-1 14H6L5 6M10 10v6M14 10v6"></path></svg>',
    folder:
      '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M3 7h6l2 2h10v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z"></path></svg>',
    file: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"></path><path d="M14 2v6h6M8 13h8M8 17h6"></path></svg>',
    stop: '<svg viewBox="0 0 24 24" aria-hidden="true"><rect x="7" y="7" width="10" height="10" rx="1"></rect></svg>',
    send: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m5 12 7-7 7 7M12 19V5"></path></svg>',
  };

  // Providers declare shell capabilities without owning shared shell markup.
  var artifactProviders = {
    browser: { expandable: true },
    file: { expandable: true },
    folder: { expandable: true },
    terminal: { expandable: true },
  };

  var artifacts = {
    'browser-1': {
      id: 'browser-1',
      type: 'browser',
      title: 'Example Domain',
      url: 'https://example.com',
      history: ['https://example.com'],
      historyIndex: 0,
      status: 'Ready',
    },
    'file-1': {
      id: 'file-1',
      type: 'file',
      title: 'notes.md',
      path: '~/project/docs/notes.md',
      content: sampleFileContent(),
      draftContent: null,
      version: 1,
      status: 'Viewing',
      proposedContent: '',
      editing: false,
    },
    'terminal-1': {
      id: 'terminal-1',
      type: 'terminal',
      title: 'npm run dev',
      sessionId: 'shell-1',
    },
  };

  var terminalSession = {
    id: 'shell-1',
    cwd: '~/project',
    activeEntryId: 'terminal-1',
    entries: [
      {
        id: 'terminal-0',
        command: 'ls',
        output: 'index.html  notes.md  src  wireframes',
        status: 'completed',
        exit: 0,
      },
      {
        id: 'terminal-1',
        command: 'npm run dev',
        output:
          '> workspace@1.0.0 dev\n> vite --host 127.0.0.1\n\nLocal: http://127.0.0.1:4179/\nready in 412 ms\nGET / 200 14ms',
        status: 'running',
        exit: null,
        elapsed: 18,
      },
    ],
  };

  function escapeHtml(value) {
    return String(value).replace(/[&<>'"]/g, function (character) {
      return {
        '&': '&amp;',
        '<': '&lt;',
        '>': '&gt;',
        "'": '&#39;',
        '"': '&quot;',
      }[character];
    });
  }

  /** Returns the Markdown source retained by a rendered message body. */
  function markdownSource(element) {
    if (!element) return '';
    return (
      element.getAttribute('data-markdown-source') ||
      markdown.fromElement(element)
    );
  }

  /** Reads either a native mode input or the source-native Chat editor. */
  function inputValue(input) {
    if (!input) return '';
    if (input.matches('[data-markdown-editor]'))
      return markdown.fromElement(input);
    return input.value || '';
  }

  /** Clears a native input or Markdown editor through its owning state model. */
  function clearComposerInput(input) {
    if (!input) return;
    if (input.matches('[data-markdown-editor]'))
      markdown.setEditorValue(input, '');
    else input.value = '';
  }

  /** Focuses either a native input or its nested CodeMirror editor. */
  function focusComposerInput(input) {
    if (!input) return;
    if (input.matches('[data-markdown-editor]')) markdown.focusEditor(input);
    else input.focus();
  }

  /** Renders one message surface while preserving its original Markdown source. */
  function messageBodyMarkup(source, className) {
    var classes = ['message__body', 'markdown-content'];
    if (className) classes.push(className);
    return (
      '<div class="' +
      classes.join(' ') +
      '" data-markdown-source="' +
      escapeHtml(source) +
      '">' +
      markdown.render(source) +
      '</div>'
    );
  }

  /** Produces a compact plain-text excerpt from Markdown source. */
  function markdownPlainText(source) {
    var container = document.createElement('div');
    container.innerHTML = markdown.render(source);
    return container.textContent.trim();
  }

  /** Upgrades seeded message bubbles to the same Markdown contract as new ones. */
  function hydrateMarkdownMessages() {
    thread
      .querySelectorAll(
        '.message--user .message__body, .message--assistant .message__body',
      )
      .forEach(function (body) {
        var source = markdownSource(body);
        body.classList.add('markdown-content');
        body.setAttribute('data-markdown-source', source);
        body.innerHTML = markdown.render(source);
      });
  }

  /** Formats a file size for compact attachment metadata. */
  function formatFileSize(bytes) {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024)
      return Math.max(1, Math.round(bytes / 1024)) + ' KB';
    return (
      (bytes / (1024 * 1024)).toFixed(bytes < 10 * 1024 * 1024 ? 1 : 0) + ' MB'
    );
  }

  /** Resolves a compact uppercase file extension with a MIME fallback. */
  function getAttachmentExtension(attachment) {
    var nameMatch = String(attachment.name || '').match(/\.([^.]+)$/);
    if (nameMatch) return nameMatch[1].toUpperCase().slice(0, 8);
    var mimeSubtype = String(attachment.type || '').split('/')[1] || '';
    return mimeSubtype
      ? mimeSubtype.split(/[.+-]/)[0].toUpperCase().slice(0, 8)
      : 'FILE';
  }

  /** Formats the extension and size shared by draft and submitted files. */
  function formatAttachmentMetadata(attachment) {
    return (
      getAttachmentExtension(attachment) +
      ' · ' +
      formatFileSize(attachment.size)
    );
  }

  /** Renders the one-based reference shared by draft and submitted files. */
  function attachmentNumberMarkup(index) {
    var number = index + 1;
    return (
      '<span class="attachment-number" aria-label="Attachment ' +
      number +
      '" title="Attachment ' +
      number +
      '">' +
      number +
      '</span>'
    );
  }

  /** Returns attachment/index pairs ordered from newest to oldest. */
  function newestAttachmentEntries(attachments) {
    return (attachments || [])
      .map(function (attachment, index) {
        return { attachment: attachment, index: index };
      })
      .reverse();
  }

  /** Returns true when a file can be previewed as an image in the composer. */
  function isImageFile(file) {
    return (
      /^image\//.test(file.type) ||
      /\.(avif|bmp|gif|ico|jpe?g|png|svg|webp)$/i.test(file.name)
    );
  }

  function isAudioFile(file) {
    return /^audio\//.test(file.type) || /\.(aac|flac|m4a|mp3|ogg|wav)$/i.test(file.name);
  }

  function selectedModelName() {
    var value = document.querySelector('[data-option-value="model"]');
    return value ? value.textContent.trim() : 'GPT-5';
  }

  function modelSupports(model, capability) {
    var profile = modelProfiles[model] || { capabilities: [] };
    return profile.capabilities.indexOf(capability) > -1;
  }

  function attachmentCompatibility(attachment, model) {
    if (attachment.converted) return { state: 'ready', label: 'Converted · Ready' };
    if (isImageFile(attachment) && !modelSupports(model, 'vision'))
      return { state: 'incompatible', label: 'Needs Vision' };
    if (isAudioFile(attachment) && !modelSupports(model, 'audio'))
      return { state: 'incompatible', label: 'Needs Audio' };
    return { state: 'ready', label: 'Ready' };
  }

  function incompatibleAttachments(model) {
    return draftAttachments.filter(function (attachment) {
      return attachmentCompatibility(attachment, model).state === 'incompatible';
    });
  }

  function hideCompatibilityAlert() {
    var alert = document.querySelector('[data-compatibility-alert]');
    if (alert) alert.hidden = true;
    pendingModelSwitch = '';
    window.requestAnimationFrame(syncComposerDockHeight);
  }

  function showCompatibilityAlert(model, context) {
    var alert = document.querySelector('[data-compatibility-alert]');
    var incompatible = incompatibleAttachments(model);
    if (!alert || !incompatible.length) return false;
    pendingModelSwitch = context === 'switch' ? model : '';
    alert.querySelector('[data-compatibility-title]').textContent =
      context === 'switch' ? 'This model does not match the draft' : 'Resolve the draft before sending';
    alert.querySelector('[data-compatibility-message]').textContent =
      model + ' cannot use ' + incompatible.map(function (item) { return item.name; }).join(', ') + '. Nothing will be dropped.';
    alert.hidden = false;
    window.requestAnimationFrame(syncComposerDockHeight);
    return true;
  }

  function syncModelDependentControls() {
    var model = selectedModelName();
    var reasoning = document.querySelector('[data-composer-control="reasoning"]');
    var supported = modelSupports(model, 'reasoning');
    if (reasoning) reasoning.hidden = !supported;
    if (!supported) {
      var value = document.querySelector('[data-option-value="reasoning"]');
      if (value) value.textContent = 'Off';
    }
    renderDraftAttachments();
  }

  function applyModelSelection(option) {
    var menu = option.closest('[data-menu="model"]');
    if (!menu) return;
    menu.querySelectorAll('[data-option]').forEach(function (item) {
      item.setAttribute('aria-checked', String(item === option));
    });
    document.querySelector('[data-option-value="model"]').textContent = option.getAttribute('data-option');
    hideCompatibilityAlert();
    syncModelDependentControls();
  }

  /** Renders a draft attachment with a thumbnail or generic file glyph. */
  function draftAttachmentMarkup(attachment, index) {
    var compatibility = attachmentCompatibility(attachment, selectedModelName());
    var visual = attachment.previewUrl
      ? '<img src="' + escapeHtml(attachment.previewUrl) + '" alt="">'
      : '<span class="composer-attachment__file-icon" aria-hidden="true">' +
        icons.file +
        '</span>';
    return (
      '<div class="composer-attachment" data-draft-attachment="' +
      attachment.id +
      '">' +
      attachmentNumberMarkup(index) +
      visual +
      '<span class="composer-attachment__copy"><strong title="' +
      escapeHtml(attachment.name) +
      '">' +
      escapeHtml(attachment.name) +
      '</strong><small>' +
      escapeHtml(formatAttachmentMetadata(attachment)) +
      '</small><span class="attachment-compatibility attachment-compatibility--' +
      compatibility.state +
      '">' +
      escapeHtml(compatibility.label) +
      '</span></span>' +
      '<button type="button" data-remove-attachment="' +
      attachment.id +
      '" aria-label="Remove ' +
      escapeHtml(attachment.name) +
      '" title="Remove attachment">' +
      icons.close +
      '</button></div>'
    );
  }

  /** Keeps the Chat attachment tray synchronized with draft state and mode. */
  function renderDraftAttachments() {
    if (!attachmentTray) return;
    attachmentTray.innerHTML = newestAttachmentEntries(draftAttachments)
      .map(function (entry) {
        return draftAttachmentMarkup(entry.attachment, entry.index);
      })
      .join('');
    attachmentTray.hidden =
      currentMode !== 'chat' || draftAttachments.length === 0;
    composer.dataset.attachmentCount = String(draftAttachments.length);
    composer.dataset.firstAttachmentName = draftAttachments.length
      ? draftAttachments[0].name
      : '';
    window.requestAnimationFrame(syncComposerDockHeight);
  }

  /** Adds selected or dropped files to the current Chat draft. */
  function addDraftAttachments(files) {
    Array.prototype.forEach.call(files || [], function (file) {
      attachmentCounter += 1;
      var previewUrl = isImageFile(file) ? URL.createObjectURL(file) : '';
      if (previewUrl) attachmentUrls.push(previewUrl);
      draftAttachments.push({
        id: 'attachment-' + attachmentCounter,
        name: file.name,
        size: file.size,
        type: file.type,
        previewUrl: previewUrl,
      });
    });
    renderDraftAttachments();
    syncPrimaryAction();
  }

  /** Removes one file from the draft and releases its unused preview URL. */
  function removeDraftAttachment(id) {
    var removed = draftAttachments.find(function (attachment) {
      return attachment.id === id;
    });
    draftAttachments = draftAttachments.filter(function (attachment) {
      return attachment.id !== id;
    });
    if (removed && removed.previewUrl) {
      URL.revokeObjectURL(removed.previewUrl);
      attachmentUrls = attachmentUrls.filter(function (url) {
        return url !== removed.previewUrl;
      });
    }
    renderDraftAttachments();
    hideCompatibilityAlert();
    syncPrimaryAction();
  }

  /** Moves draft files into a submitted message without invalidating image previews. */
  function takeDraftAttachments() {
    var attachments = draftAttachments.slice();
    draftAttachments = [];
    if (attachmentInput) attachmentInput.value = '';
    renderDraftAttachments();
    return attachments;
  }

  /** Discards draft files and releases previews when a new blank chat starts. */
  function discardDraftAttachments() {
    var discardedUrls = draftAttachments
      .map(function (attachment) {
        return attachment.previewUrl;
      })
      .filter(Boolean);
    draftAttachments.forEach(function (attachment) {
      if (attachment.previewUrl) URL.revokeObjectURL(attachment.previewUrl);
    });
    draftAttachments = [];
    attachmentUrls = attachmentUrls.filter(function (url) {
      return discardedUrls.indexOf(url) === -1;
    });
    if (attachmentInput) attachmentInput.value = '';
    renderDraftAttachments();
  }

  /** Renders submitted attachment previews inside a user message. */
  function messageAttachmentsMarkup(attachments) {
    if (!attachments || !attachments.length) return '';
    return (
      '<div aria-label="Attached files" class="message-attachments" role="group">' +
      newestAttachmentEntries(attachments)
        .map(function (entry) {
          var numberBadge = attachmentNumberMarkup(entry.index);
          if (entry.attachment.previewUrl) {
            return (
              '<figure class="message-attachment message-attachment--image">' +
              numberBadge +
              '<img src="' +
              escapeHtml(entry.attachment.previewUrl) +
              '" alt="Preview of ' +
              escapeHtml(entry.attachment.name) +
              '"><figcaption><strong title="' +
              escapeHtml(entry.attachment.name) +
              '">' +
              escapeHtml(entry.attachment.name) +
              '</strong><small>' +
              escapeHtml(formatAttachmentMetadata(entry.attachment)) +
              '</small></figcaption></figure>'
            );
          }
          return (
            '<div class="message-attachment message-attachment--file">' +
            numberBadge +
            '<span class="message-attachment__file-icon" aria-hidden="true">' +
            icons.file +
            '</span><span><strong title="' +
            escapeHtml(entry.attachment.name) +
            '">' +
            escapeHtml(entry.attachment.name) +
            '</strong><small>' +
            escapeHtml(formatAttachmentMetadata(entry.attachment)) +
            '</small></span></div>'
          );
        })
        .join('') +
      '</div>'
    );
  }

  /** Detects file drags without treating selected text as a file drop. */
  function hasDraggedFiles(event) {
    var types = event.dataTransfer && event.dataTransfer.types;
    return !!types && Array.prototype.indexOf.call(types, 'Files') > -1;
  }

  /** Shows or hides the whole-app Chat drop target. */
  function setDropzoneVisible(visible) {
    if (!appDropzone) return;
    appDropzone.hidden = !visible;
    appDropzone.setAttribute('aria-hidden', String(!visible));
    shell.toggleAttribute('data-file-dragging', visible);
  }

  /** Resolves after a short UI-animation delay. */
  function waitForStream(delay) {
    return new Promise(function (resolve) {
      window.setTimeout(resolve, delay);
    });
  }

  /** Keeps a streaming response visible when the transcript is already following the bottom. */
  function followAgentStream(element) {
    var transcript = element.closest('.thread');
    if (!transcript) return;
    var distance =
      transcript.scrollHeight - transcript.scrollTop - transcript.clientHeight;
    if (distance < 180) transcript.scrollTop = transcript.scrollHeight;
    updateThreadScrollButton();
  }

  /** Updates a typing target as plain text or progressively rendered Markdown. */
  function renderAgentText(element, content) {
    if (element.matches('[data-markdown-source]'))
      element.innerHTML = markdown.render(content);
    else element.textContent = content;
  }

  /** Types text in fast chunks and exposes a cursor only while content is arriving. */
  function typeAgentText(element, text) {
    if (!element) return Promise.resolve();
    var content = String(text || '');
    if (reducedMotionQuery.matches || !content) {
      renderAgentText(element, content);
      return Promise.resolve();
    }
    var index = 0;
    var chunkSize = content.length > 180 ? 4 : 2;
    element.classList.add('agent-typing');
    return new Promise(function (resolve) {
      function typeChunk() {
        index = Math.min(content.length, index + chunkSize);
        renderAgentText(element, content.slice(0, index));
        followAgentStream(element);
        if (index < content.length) {
          window.setTimeout(typeChunk, 9);
          return;
        }
        element.classList.remove('agent-typing');
        resolve();
      }
      typeChunk();
    });
  }

  /** Captures response copy and hides completed UI before a queued stream begins. */
  function prepareAgentStream(node) {
    var thinking = node.querySelector('.thinking-process');
    var summary = thinking && thinking.querySelector('summary span');
    var work =
      thinking && thinking.querySelector('.thinking-process__content p');
    var activity =
      thinking && thinking.querySelector('.thinking-process__activity span');
    var response = node.matches('.message--assistant')
      ? node.querySelector('.message__body')
      : null;
    [summary, work, activity, response].forEach(function (element) {
      if (!element) return;
      var streamText = element.matches('[data-markdown-source]')
        ? markdownSource(element)
        : element.textContent;
      element.setAttribute('data-stream-text', streamText);
      if (element.matches('[data-markdown-source]')) element.innerHTML = '';
      else element.textContent = '';
    });
    var revealSelector = node.matches('.message--assistant')
      ? '.message-model-row, .message__content'
      : '.artifact-shell__identity, .artifact-frame, .artifact-shell__actions';
    node.querySelectorAll(revealSelector).forEach(function (element) {
      element.hidden = true;
    });
    if (thinking) thinking.open = true;
    node.classList.add('agent-stream');
    node.setAttribute('aria-busy', 'true');
  }

  /** Reveals a completed response region with a short entrance transition. */
  function revealAgentStreamPart(element) {
    if (!element) return;
    element.hidden = false;
    element.classList.add('agent-stream__reveal');
    window.setTimeout(function () {
      element.classList.remove('agent-stream__reveal');
    }, 220);
  }

  /** Streams the prepared work summary before revealing and typing the final response. */
  async function runAgentStream(node) {
    var thinking = node.querySelector('.thinking-process');
    var summary = thinking && thinking.querySelector('summary span');
    var work =
      thinking && thinking.querySelector('.thinking-process__content p');
    var activity =
      thinking && thinking.querySelector('.thinking-process__activity span');
    var finalLabel = summary ? summary.getAttribute('data-stream-text') : '';
    await typeAgentText(summary, 'Starting…');
    await typeAgentText(work, work && work.getAttribute('data-stream-text'));
    await typeAgentText(summary, 'Working…');
    await typeAgentText(
      activity,
      activity && activity.getAttribute('data-stream-text'),
    );
    await waitForStream(reducedMotionQuery.matches ? 0 : 120);
    if (summary) summary.textContent = finalLabel;
    if (thinking) thinking.open = false;

    if (node.matches('.message--assistant')) {
      revealAgentStreamPart(node.querySelector('.message-model-row'));
      revealAgentStreamPart(node.querySelector('.message__content'));
      var response = node.querySelector('.message__body');
      await typeAgentText(
        response,
        response && response.getAttribute('data-stream-text'),
      );
    } else {
      revealAgentStreamPart(node.querySelector('.artifact-shell__identity'));
      revealAgentStreamPart(node.querySelector('.artifact-frame'));
      revealAgentStreamPart(node.querySelector('.artifact-shell__actions'));
    }
    node.classList.remove('agent-stream');
    node.removeAttribute('aria-busy');
    followAgentStream(node);
  }

  /** Adds a prepared response to the shared generation queue. */
  function queueAgentStream(node) {
    prepareAgentStream(node);
    agentStreamQueue = agentStreamQueue.then(function () {
      return runAgentStream(node);
    });
  }

  function sampleFileContent() {
    return [
      '# Project notes',
      '',
      'The desktop shell keeps chat at the center.',
      'Artifact modes extend the same conversation.',
      '',
      '## Workspace principles',
      '',
      '- Keep the conversation available while inspecting artifacts.',
      '- Preserve the selected artifact when panel dimensions change.',
      '- Use explicit approval immediately before a file write.',
      '- Keep terminal output immutable and append follow-up commands.',
      '',
      '## File workflow',
      '',
      '1. Select a file or folder from the Files composer mode.',
      '2. Review the response artifact in the conversation.',
      '3. Expand a file into the center workspace when more room is needed.',
      '4. Use breadcrumbs to move back into the project folder.',
      '5. Edit with fixed line numbers and save after approval.',
      '',
      '## Browser workflow',
      '',
      'Browser artifacts retain their current address and navigation history.',
      'The expanded surface uses the center workspace above the composer.',
      '',
      '## Open questions',
      '',
      '- Should File Explorer remember its last folder per conversation?',
      '- Should hidden files be visible by default?',
      '- Should folders support Reply with a natural-language instruction?',
      '',
      '## Next review',
      '',
      'Validate long-file scrolling, breadcrumbs, and folder selection.',
    ].join('\n');
  }

  function panelFor(side) {
    return document.querySelector('.side-panel--' + side);
  }

  function toggleFor(side) {
    return document.querySelector('[data-panel-toggle="' + side + '"]');
  }

  function setPanelOpen(side, isOpen) {
    var panel = panelFor(side);
    var toggle = toggleFor(side);
    shell.setAttribute('data-' + side + '-open', String(isOpen));
    if (toggle) toggle.setAttribute('aria-expanded', String(isOpen));
    if (!panel) return;
    panel.setAttribute('aria-hidden', String(!isOpen));
    if (isOpen) panel.removeAttribute('inert');
    else panel.setAttribute('inert', '');
  }

  function maximumPanelWidth() {
    var width = shell.getBoundingClientRect().width;
    return Math.max(
      minimumWidth,
      Math.min(width * 0.55, width - minimumCenterWidth),
    );
  }

  function setPanelWidth(side, width) {
    var maximumWidth = maximumPanelWidth();
    var value = Math.round(
      Math.max(minimumWidth, Math.min(maximumWidth, width)),
    );
    shell.style.setProperty('--' + side + '-panel-width', value + 'px');
    var handle = document.querySelector('[data-panel-resize="' + side + '"]');
    if (handle) {
      handle.setAttribute('aria-valuemax', String(Math.round(maximumWidth)));
      handle.setAttribute('aria-valuenow', String(value));
    }
  }

  function normalizeForViewport() {
    if (narrowQuery.matches) {
      setPanelOpen('left', false);
    }
    setPanelWidth(
      'left',
      panelFor('left').getBoundingClientRect().width || minimumWidth,
    );
    clampChatPaneHeight();
  }

  shell.addEventListener('pointerdown', function (event) {
    var handle = event.target.closest('[data-panel-resize]');
    if (!handle) return;
    if (activePanelResizeCleanup) activePanelResizeCleanup();
    var side = handle.getAttribute('data-panel-resize');
    var bounds = shell.getBoundingClientRect();
    event.preventDefault();
    shell.setAttribute('data-resizing', side);
    document.documentElement.style.cursor = 'col-resize';

    function movePanel(moveEvent) {
      if (moveEvent.buttons === 0) {
        stopResize();
        return;
      }
      setPanelWidth(
        side,
        side === 'left'
          ? moveEvent.clientX - bounds.left
          : bounds.right - moveEvent.clientX,
      );
    }

    function stopResize() {
      if (!activePanelResizeCleanup) return;
      shell.removeAttribute('data-resizing');
      document.documentElement.style.cursor = '';
      document.removeEventListener('pointermove', movePanel);
      document.removeEventListener('pointerup', stopResize);
      document.removeEventListener('pointercancel', stopResize);
      window.removeEventListener('blur', stopResize);
      activePanelResizeCleanup = null;
    }

    activePanelResizeCleanup = stopResize;
    document.addEventListener('pointermove', movePanel);
    document.addEventListener('pointerup', stopResize);
    document.addEventListener('pointercancel', stopResize);
    window.addEventListener('blur', stopResize);
  });

  shell.addEventListener('keydown', function (event) {
    var handle = event.target.closest('[data-panel-resize]');
    if (handle) {
      var side = handle.getAttribute('data-panel-resize');
      var current = panelFor(side).getBoundingClientRect().width;
      var delta = 0;
      if (event.key === 'ArrowLeft') delta = side === 'left' ? -16 : 16;
      if (event.key === 'ArrowRight') delta = side === 'left' ? 16 : -16;
      if (event.key === 'Home') current = minimumWidth;
      if (event.key === 'End') current = maximumPanelWidth();
      if (delta || event.key === 'Home' || event.key === 'End') {
        event.preventDefault();
        setPanelWidth(side, current + delta);
      }
    }
    if (
      event.key === 'Escape' &&
      narrowQuery.matches &&
      !event.target.closest('[data-menu]')
    ) {
      setPanelOpen('left', false);
    }
  });

  narrowQuery.addEventListener('change', normalizeForViewport);
  window.addEventListener('resize', function () {
    setPanelWidth(
      'left',
      panelFor('left').getBoundingClientRect().width || minimumWidth,
    );
    clampChatPaneHeight();
  });

  function inputForMode(mode) {
    return document.querySelector('[data-mode-input="' + mode + '"]');
  }

  function closeModePopover(restoreFocus) {
    modeTrigger.setAttribute('aria-expanded', 'false');
    modePopover.hidden = true;
    if (restoreFocus) modeTrigger.focus();
  }

  function openModePopover() {
    modeTrigger.setAttribute('aria-expanded', 'true');
    modePopover.hidden = false;
    var selected = modePopover.querySelector('[aria-checked="true"]');
    if (selected) selected.focus();
  }

  function setMode(mode, updateHash) {
    currentMode = mode;
    fileDragDepth = 0;
    setDropzoneVisible(false);
    composer.setAttribute('data-mode', mode);
    document.querySelectorAll('[data-mode-option]').forEach(function (button) {
      button.setAttribute(
        'aria-checked',
        String(button.getAttribute('data-mode-option') === mode),
      );
    });
    var selectedOption = document.querySelector(
      '[data-mode-option="' + mode + '"]',
    );
    document.querySelector('[data-mode-trigger-label]').textContent =
      selectedOption.querySelector('strong').textContent;
    document.querySelector('[data-mode-trigger-icon]').innerHTML =
      selectedOption.querySelector('.mode-option__icon').innerHTML;
    closeModePopover(false);
    document.querySelectorAll('[data-mode-panel]').forEach(function (panel) {
      panel.hidden = panel.getAttribute('data-mode-panel') !== mode;
    });
    var labels = {
      chat: 'Send message',
      files: fileSelectionKind === 'folder' ? 'Open folder' : 'Open file',
      browser: 'Open webpage',
      terminal: 'Run command',
    };
    primaryAction.setAttribute('aria-label', labels[mode]);
    primaryAction.title = labels[mode];
    if (updateHash) history.replaceState(null, '', '#' + mode);
    syncTerminalComposer();
    renderDraftAttachments();
    syncPrimaryAction();
  }

  function syncPrimaryAction() {
    var input = replyTarget ? replyInput : inputForMode(currentMode);
    var hasChatAttachments =
      currentMode === 'chat' && draftAttachments.length > 0;
    primaryAction.disabled =
      !input || (inputValue(input).trim().length === 0 && !hasChatAttachments);
    if (input && input.tagName === 'TEXTAREA') {
      input.style.height = 'auto';
      input.style.height = Math.min(input.scrollHeight, 150) + 'px';
    }
    window.requestAnimationFrame(syncComposerDockHeight);
  }

  function syncComposerDockHeight() {
    if (!composerDock) return;
    shell.style.setProperty(
      '--composer-dock-height',
      Math.ceil(composerDock.getBoundingClientRect().height) + 'px',
    );
    updateThreadScrollButton();
  }

  function startReply(button) {
    replyTarget = {
      kind: button.getAttribute('data-target-kind'),
      id: button.getAttribute('data-target-id'),
      title: button.getAttribute('data-target-title'),
      excerpt: button.getAttribute('data-target-excerpt'),
    };
    modeControl.hidden = true;
    closeModePopover(false);
    document.querySelectorAll('[data-mode-panel]').forEach(function (panel) {
      panel.hidden = true;
    });
    replyStrip.hidden = false;
    replyPanel.hidden = false;
    document.querySelector('[data-reply-title]').textContent =
      'Replying to ' + replyTarget.title;
    document.querySelector('[data-reply-excerpt]').textContent =
      replyTarget.excerpt;
    primaryAction.setAttribute('aria-label', 'Send reply');
    primaryAction.title = 'Send reply';
    clearComposerInput(replyInput);
    syncPrimaryAction();
    focusComposerInput(replyInput);
  }

  function cancelReply() {
    replyTarget = null;
    replyStrip.hidden = true;
    replyPanel.hidden = true;
    modeControl.hidden = false;
    setMode(currentMode, false);
    focusComposerInput(inputForMode(currentMode));
  }

  function actionsMarkup(kind, id, title, excerpt) {
    return (
      '<div class="' +
      (kind === 'message' ? 'message-actions' : 'artifact-shell__actions') +
      '" aria-label="Actions">' +
      '<button type="button" data-copy data-copy-kind="' +
      (kind === 'message' ? 'message' : 'artifact') +
      '" aria-label="Copy" title="Copy">' +
      icons.copy +
      '</button>' +
      '<button type="button" data-reply data-target-kind="' +
      kind +
      '" data-target-id="' +
      escapeHtml(id) +
      '" data-target-title="' +
      escapeHtml(title) +
      '" data-target-excerpt="' +
      escapeHtml(excerpt) +
      '" aria-label="Reply" title="Reply">' +
      icons.reply +
      '</button>' +
      '</div>'
    );
  }

  function ensureUserMessageActions() {
    thread.querySelectorAll('.message--user').forEach(function (message) {
      if (message.querySelector('.message-actions')) return;
      var id =
        message.getAttribute('data-message-id') ||
        'message-' + ++messageCounter;
      var body = message.querySelector('.message__body');
      var text = markdownPlainText(markdownSource(body));
      message.setAttribute('data-message-id', id);
      message.insertAdjacentHTML(
        'beforeend',
        actionsMarkup('message', id, 'You', text.slice(0, 70)),
      );
    });
  }

  hydrateMarkdownMessages();
  ensureUserMessageActions();

  function expandMarkup(id) {
    return (
      '<div class="artifact-frame__actions"><button type="button" data-expand="' +
      escapeHtml(id) +
      '" aria-label="Expand artifact" title="Expand">' +
      icons.expand +
      '</button></div>'
    );
  }

  function browserToolbarMarkup(id, url) {
    return (
      '<button type="button" data-browser-action="back" aria-label="Back" disabled><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m15 18-6-6 6-6"></path></svg></button>' +
      '<button type="button" data-browser-action="forward" aria-label="Forward" disabled><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m9 18 6-6-6-6"></path></svg></button>' +
      '<button type="button" data-browser-action="refresh" aria-label="Refresh"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M20 11a8 8 0 1 0-2.3 5.7L20 14"></path><path d="M20 6v5h-5"></path></svg></button>' +
      '<label class="sr-only">Current web address</label><input data-browser-address value="' +
      escapeHtml(url) +
      '" readonly>' +
      expandMarkup(id)
    );
  }

  function fileHeaderActionsMarkup(id, title) {
    return (
      '<div class="artifact-frame__actions"><button type="button" data-file-edit aria-label="Edit ' +
      escapeHtml(title) +
      '" title="Edit">' +
      icons.edit +
      '</button>' +
      '<button type="button" data-file-cancel aria-label="Discard changes to ' +
      escapeHtml(title) +
      '" title="Discard changes" hidden>' +
      icons.trash +
      '</button>' +
      '<button type="button" data-file-save aria-label="Save ' +
      escapeHtml(title) +
      '" title="Save" hidden>' +
      icons.save +
      '</button>' +
      '<button type="button" data-expand="' +
      escapeHtml(id) +
      '" aria-label="Expand file artifact" title="Expand">' +
      icons.expand +
      '</button></div>'
    );
  }

  function lineNumbers(value) {
    return value
      .split('\n')
      .map(function (_, index) {
        return index + 1;
      })
      .join('\n');
  }

  function editorText(editor) {
    return editor ? editor.innerText.replace(/\r/g, '') : '';
  }

  function setEditorText(editor, value) {
    if (editor) editor.textContent = value;
  }

  function breadcrumbMarkup(path, leafIsFile) {
    var isHome = path.indexOf('~/') === 0;
    var parts = path
      .replace(/^~\//, '')
      .replace(/^\/+/, '')
      .split('/')
      .filter(Boolean);
    var built = isHome ? '~' : '';
    return (
      '<nav class="file-breadcrumbs" aria-label="File path">' +
      parts
        .map(function (part, index) {
          built += '/' + part;
          var current = index === parts.length - 1;
          var segment = current
            ? '<span aria-current="page">' + escapeHtml(part) + '</span>'
            : '<button type="button" data-open-folder="' +
              escapeHtml(built) +
              '">' +
              escapeHtml(part) +
              '</button>';
          if (leafIsFile && current)
            return (
              '<span class="file-breadcrumbs__separator">/</span>' + segment
            );
          return (
            (index
              ? '<span class="file-breadcrumbs__separator">/</span>'
              : '') + segment
          );
        })
        .join('') +
      '</nav>'
    );
  }

  function explorerEntries(path) {
    if (/\/docs$/.test(path))
      return [
        { kind: 'folder', name: 'guides', detail: '3 items' },
        { kind: 'folder', name: 'references', detail: '2 items' },
        { kind: 'file', name: 'notes.md', detail: '3 KB' },
        { kind: 'file', name: 'architecture.md', detail: '5 KB' },
      ];
    if (/\/src$/.test(path))
      return [
        { kind: 'folder', name: 'components', detail: '8 items' },
        { kind: 'folder', name: 'lib', detail: '5 items' },
        { kind: 'file', name: 'app.js', detail: '6 KB' },
        { kind: 'file', name: 'workspace.js', detail: '4 KB' },
        { kind: 'file', name: 'styles.css', detail: '9 KB' },
      ];
    if (/\/wireframes$/.test(path))
      return [
        { kind: 'folder', name: 'r004-artifact-focus-swap', detail: '7 items' },
        { kind: 'folder', name: 'r005-left-chat-pane', detail: '9 items' },
        {
          kind: 'folder',
          name: 'r006-file-explorer-artifact',
          detail: '9 items',
        },
        { kind: 'file', name: 'README.md', detail: '2 KB' },
      ];
    return [
      { kind: 'folder', name: 'src', detail: '12 items' },
      { kind: 'folder', name: 'wireframes', detail: '6 items' },
      { kind: 'folder', name: 'docs', detail: '4 items' },
      { kind: 'file', name: 'index.html', detail: '8 KB' },
      { kind: 'file', name: 'notes.md', detail: '3 KB' },
      { kind: 'file', name: 'package.json', detail: '1 KB' },
    ];
  }

  function sampleExplorerFileContent(name) {
    if (/\.html$/.test(name))
      return [
        '<!doctype html>',
        '<html lang="en">',
        '  <head>',
        '    <meta charset="utf-8">',
        '    <meta name="viewport" content="width=device-width, initial-scale=1">',
        '    <title>Artifact workspace</title>',
        '  </head>',
        '  <body>',
        '    <main id="app"></main>',
        '    <script src="./script.js"><' + '/script>',
        '  </body>',
        '</html>',
      ].join('\n');
    if (name === 'notes.md') return sampleFileContent();
    return (
      '# ' +
      name +
      '\n\nThis file is open from File Explorer.\n\nUse Edit to make changes or the folder breadcrumbs to return to the directory.'
    );
  }

  function explorerFile(path) {
    var title = path.split('/').pop();
    return {
      title: title,
      path: path,
      content: sampleExplorerFileContent(title),
      draftContent: null,
      version: 1,
      editing: false,
    };
  }

  function explorerRowsMarkup(path) {
    var rows = explorerEntries(path);
    return (
      '<div class="file-explorer-list">' +
      rows
        .map(function (row) {
          var targetPath = path.replace(/\/$/, '') + '/' + row.name;
          var action =
            row.kind === 'folder'
              ? ' data-open-folder="' + escapeHtml(targetPath) + '"'
              : ' data-open-explorer-file="' + escapeHtml(targetPath) + '"';
          var icon =
            row.kind === 'folder'
              ? icons.folder
              : artifactAvatarMarkup('file').replace(
                  /^<div[^>]*>|<\/div>$/g,
                  '',
                );
          return (
            '<button class="file-explorer-row" type="button"' +
            action +
            '><span class="file-explorer-row__icon">' +
            icon +
            '</span><span class="file-explorer-row__name">' +
            escapeHtml(row.name) +
            '</span><span class="file-explorer-row__detail">' +
            escapeHtml(row.detail) +
            '</span></button>'
          );
        })
        .join('') +
      '</div>'
    );
  }

  /** Returns the registered provider or a safe non-expandable fallback. */
  function artifactProvider(type) {
    return artifactProviders[type] || { expandable: false };
  }

  /** Builds the shared class contract for an artifact shell context. */
  function artifactShellClass(context, type) {
    return (
      'artifact-shell artifact-shell--' +
      context +
      ' artifact-provider artifact-provider--' +
      type
    );
  }

  /** Wraps provider markup in the shared artifact body slot. */
  function artifactBodyMarkup(content) {
    return '<div class="artifact-frame__body">' + content + '</div>';
  }

  /** Builds the shared framed surface used by response artifacts. */
  function artifactFrameMarkup(options) {
    var className = 'artifact-frame response-container';
    if (options.className) className += ' ' + options.className;
    return (
      '<div class="' +
      className +
      '">' +
      '<header class="artifact-frame__header' +
      (options.headerClass ? ' ' + options.headerClass : '') +
      '">' +
      options.header +
      '</header>' +
      artifactBodyMarkup(options.body) +
      (options.footer || '') +
      '</div>'
    );
  }

  /** Builds the response-shell slots around provider-owned frame markup. */
  function responseArtifactMarkup(options) {
    return (
      '<div class="artifact-shell__preamble">' +
      artifactPreambleMarkup(
        options.duration,
        options.summary,
        options.activity,
        options.type,
      ) +
      '</div>' +
      options.frame +
      actionsMarkup(
        options.type,
        options.id,
        options.replyTitle,
        options.replyExcerpt,
      )
    );
  }

  /** Creates a response artifact node with provider-neutral shell attributes. */
  function createResponseArtifactNode(state, options) {
    var context = thread.getAttribute('data-artifact-context') || 'response';
    var node = document.createElement('article');
    node.className =
      artifactShellClass('response', state.type) +
      (context === 'pane' ? ' artifact-shell--pane' : '');
    node.setAttribute('data-artifact-card', '');
    node.setAttribute('data-artifact-context', context);
    node.setAttribute('data-artifact-id', state.id);
    node.setAttribute('data-artifact-type', state.type);
    node.setAttribute('aria-label', options.label);
    node.tabIndex = 0;
    node.innerHTML = responseArtifactMarkup(options);
    return node;
  }

  /** Wraps a provider focus body in the shared center-workspace shell. */
  function focusArtifactMarkup(type, label, content) {
    return (
      '<section class="' +
      artifactShellClass('focus', type) +
      ' artifact-focus" data-artifact-context="focus" aria-label="' +
      escapeHtml(label) +
      '">' +
      content +
      '</section>'
    );
  }

  function explorerFileActionsMarkup(id, file, focused) {
    var className = focused
      ? 'artifact-shell__controls'
      : 'artifact-frame__actions';
    var buttonClass = focused
      ? ' class="artifact-shell__control artifact-shell__control--icon"'
      : '';
    return (
      '<div class="' +
      className +
      '">' +
      '<button' +
      buttonClass +
      ' type="button" data-explorer-file-edit aria-label="Edit ' +
      escapeHtml(file.title) +
      '" title="Edit"' +
      (file.editing ? ' hidden' : '') +
      '>' +
      icons.edit +
      '</button>' +
      '<button' +
      buttonClass +
      ' type="button" data-explorer-file-cancel aria-label="Discard changes to ' +
      escapeHtml(file.title) +
      '" title="Discard changes"' +
      (file.editing ? '' : ' hidden') +
      '>' +
      icons.trash +
      '</button>' +
      '<button' +
      buttonClass +
      ' type="button" data-explorer-file-save aria-label="Save ' +
      escapeHtml(file.title) +
      '" title="Save"' +
      (file.editing ? '' : ' hidden') +
      '>' +
      icons.save +
      '</button>' +
      (focused
        ? '<button class="artifact-shell__control artifact-shell__control--icon" type="button" data-close-focused-artifact aria-label="Close File artifact" title="Close">' +
          icons.close +
          '</button>'
        : '<button type="button" data-expand="' +
          escapeHtml(id) +
          '" aria-label="Expand file artifact" title="Expand">' +
          icons.expand +
          '</button>') +
      '</div>'
    );
  }

  function explorerFileBodyMarkup(file, focused) {
    var content =
      file.editing && file.draftContent != null
        ? file.draftContent
        : file.content;
    return (
      '<div class="explorer-file-document' +
      (focused ? ' explorer-file-document--focused' : '') +
      '" data-explorer-file-document data-editing="' +
      String(file.editing) +
      '">' +
      '<pre class="explorer-file-document__lines" data-explorer-file-lines aria-hidden="true"' +
      (file.editing ? '' : ' hidden') +
      '>' +
      lineNumbers(content) +
      '</pre>' +
      '<div class="explorer-file-document__editor" data-explorer-file-editor role="textbox" aria-label="' +
      (file.editing ? 'Edit ' : 'View ') +
      escapeHtml(file.title) +
      '" aria-multiline="true" aria-readonly="' +
      String(!file.editing) +
      '" contenteditable="' +
      (file.editing ? 'true' : 'false') +
      '" spellcheck="false">' +
      escapeHtml(content) +
      '</div>' +
      '</div>'
    );
  }

  function explorerSurfaceMarkup(id, options) {
    var focused = Boolean(options.focused);
    var file = options.file || null;
    var headerActions = file
      ? explorerFileActionsMarkup(id, file, focused)
      : focused
        ? '<div class="artifact-shell__controls"><button class="artifact-shell__control artifact-shell__control--icon" type="button" data-close-focused-artifact aria-label="Close File Explorer artifact" title="Close">' +
          icons.close +
          '</button></div>'
        : expandMarkup(id);
    var body = file
      ? explorerFileBodyMarkup(file, focused)
      : explorerRowsMarkup(options.path);
    var footer =
      !focused && !file
        ? '<footer class="artifact-frame__footer"><span>Read only</span><span>' +
          explorerEntries(options.path).length +
          ' items</span></footer>'
        : '';
    var header =
      '<div class="artifact-shell__breadcrumbs">' +
      breadcrumbMarkup(file ? file.path : options.path, Boolean(file)) +
      '</div>' +
      headerActions;
    if (focused) {
      return focusArtifactMarkup(
        file ? 'file' : 'folder',
        file ? 'Open file' : 'File Explorer artifact',
        '<header class="artifact-shell__header">' + header + '</header>' + body,
      );
    }
    return artifactFrameMarkup({
      body: body,
      footer: footer,
      header: header,
      headerClass: 'file-explorer-header',
    });
  }

  function folderSurfaceMarkup(id, path) {
    return explorerSurfaceMarkup(id, { path: path, focused: false });
  }

  function explorerFileSurfaceMarkup(id, file) {
    return explorerSurfaceMarkup(id, { file: file, focused: false });
  }

  function updateFolderCard(id) {
    var state = artifacts[id];
    var card = document.querySelector('[data-artifact-id="' + id + '"]');
    if (!state || !card) return;
    var surface = card.querySelector('.artifact-frame');
    surface.outerHTML = state.previewFile
      ? explorerFileSurfaceMarkup(id, state.previewFile)
      : folderSurfaceMarkup(id, state.path);
  }

  function convertFileCardToFolder(card, state, path) {
    state.type = 'folder';
    state.path = path;
    state.title = path.split('/').filter(Boolean).pop() || 'project';
    state.status = 'Browsing';
    state.previewFile = null;
    state.editing = false;
    card.classList.remove('artifact-provider--file');
    card.classList.add('artifact-provider--folder');
    card.setAttribute('data-artifact-type', 'folder');
    card.setAttribute('aria-label', 'File Explorer artifact ' + state.title);
    var avatar = card.querySelector(
      '.artifact-shell__identity .message__avatar',
    );
    if (avatar) avatar.outerHTML = artifactAvatarMarkup('folder');
    var surface = card.querySelector('.artifact-frame');
    if (surface) surface.outerHTML = folderSurfaceMarkup(state.id, path);
    var reply = card.querySelector('[data-reply]');
    if (reply) {
      reply.setAttribute('data-target-kind', 'folder');
      reply.setAttribute('data-target-title', 'File Explorer · ' + state.title);
      reply.setAttribute('data-target-excerpt', path);
    }
    var actions = card.querySelector('.artifact-shell__actions');
    if (actions)
      actions.setAttribute('aria-label', 'File Explorer artifact actions');
    var copy = card.querySelector('[data-copy]');
    if (copy) copy.setAttribute('aria-label', 'Copy folder path');
  }

  function fileEditorMarkup(title, content) {
    return (
      '<div class="file-editor" data-file-editor hidden><pre class="file-editor__lines" data-file-lines aria-hidden="true">' +
      lineNumbers(content) +
      '</pre><div class="file-editor__content" data-file-editor-input role="textbox" aria-label="Edit ' +
      escapeHtml(title) +
      '" aria-multiline="true" contenteditable="true" spellcheck="false">' +
      escapeHtml(content) +
      '</div></div>'
    );
  }

  function assistantAvatarMarkup() {
    return '<div class="message__avatar" aria-hidden="true"><svg viewBox="0 0 24 24"><path d="M12 3l1.4 4.6L18 9l-4.6 1.4L12 15l-1.4-4.6L6 9l4.6-1.4L12 3Z"></path><path d="M18.5 15l.7 2.3 2.3.7-2.3.7-.7 2.3-.7-2.3-2.3-.7 2.3-.7.7-2.3Z"></path></svg></div>';
  }

  function artifactAvatarMarkup(type) {
    var shapes = {
      generic:
        '<rect x="4" y="4" width="16" height="16" rx="4"></rect><path d="M8 9h8M8 12h8M8 15h5"></path>',
      browser:
        '<circle cx="12" cy="12" r="9"></circle><path d="M3 12h18M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18"></path>',
      file: '<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"></path><path d="M14 2v6h6M8 13h8M8 17h6"></path>',
      folder:
        '<path d="M3 7h6l2 2h10v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z"></path>',
      terminal:
        '<rect x="3" y="4" width="18" height="16" rx="2"></rect><path d="m7 9 3 3-3 3M13 15h4"></path>',
    };
    return (
      '<div class="message__avatar" aria-hidden="true"><svg viewBox="0 0 24 24">' +
      (shapes[type] || shapes.generic) +
      '</svg></div>'
    );
  }

  function thinkingMarkup(duration, summary, activity, className) {
    var reasoningControl = document.querySelector('[data-option-value="reasoning"]');
    var hasReasoningSummary =
      !className &&
      modelSupports(selectedModelName(), 'reasoning') &&
      (!reasoningControl || reasoningControl.textContent.trim() !== 'Off');
    var label = hasReasoningSummary ? 'Reasoning summary · ' : 'Activity · ';
    var visibleSummary = hasReasoningSummary
      ? summary
      : 'Processed the request using the available context and tools.';
    return (
      '<details class="thinking-process' +
      (className ? ' ' + className : '') +
      '"><summary><span>' +
      label +
      escapeHtml(duration) +
      '</span><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m9 18 6-6-6-6"></path></svg></summary><div class="thinking-process__content"><p>' +
      escapeHtml(visibleSummary) +
      '</p><div class="thinking-process__activity"><svg viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="4" width="18" height="14" rx="2"></rect><path d="M8 9l2 2 4-4"></path></svg><span>' +
      escapeHtml(activity) +
      '</span></div></div></details>'
    );
  }

  function artifactPreambleMarkup(duration, summary, activity, type) {
    return (
      thinkingMarkup(duration, summary, activity, 'artifact-shell__thinking') +
      '<div class="artifact-shell__identity">' +
      artifactAvatarMarkup(type) +
      '<div class="message__speaker">C4OS</div></div>'
    );
  }

  function appendUserMessage(text, reference, attachments) {
    messageCounter += 1;
    var node = document.createElement('div');
    node.className = 'message message--user';
    node.setAttribute('data-message-id', 'message-' + messageCounter);
    var bodyText = text ? messageBodyMarkup(text) : '';
    var attachmentNames = (attachments || [])
      .map(function (attachment) {
        return attachment.name;
      })
      .join(', ');
    var excerpt =
      markdownPlainText(text) || attachmentNames || 'Attached files';
    node.innerHTML =
      (reference
        ? '<div class="message-reference">Replying to: ' +
          escapeHtml(reference) +
          '</div>'
        : '') +
      messageAttachmentsMarkup(attachments) +
      bodyText +
      actionsMarkup(
        'message',
        'message-' + messageCounter,
        'You',
        excerpt.slice(0, 70),
      );
    thread.appendChild(node);
    return node;
  }

  function replyReferenceText(target) {
    var source = target.kind === 'message' ? target.excerpt : target.title;
    var words = String(source || '')
      .trim()
      .split(/\s+/)
      .filter(Boolean);
    return words.slice(0, 8).join(' ') + (words.length > 8 ? '…' : '');
  }

  function appendAssistantMessage(text) {
    messageCounter += 1;
    var model = document.querySelector(
      '[data-option-value="model"]',
    ).textContent;
    var node = document.createElement('article');
    node.className = 'message message--assistant';
    node.setAttribute('data-message-id', 'message-' + messageCounter);
    node.innerHTML =
      thinkingMarkup(
        '12 sec',
        'I reviewed the referenced context and prepared the requested next step.',
        'Updated the conversation',
      ) +
      '<div class="message-model-row">' +
      assistantAvatarMarkup() +
      '<div class="message__speaker">' +
      escapeHtml(model) +
      '</div></div>' +
      '<details class="run-provenance"><summary>Run details</summary><div><span><strong>Provider</strong> ' +
      escapeHtml((modelProfiles[model] || {}).route || 'Configured provider') +
      '</span><span><strong>Model</strong> ' +
      escapeHtml(model) +
      '</span><span><strong>Runtime</strong> OpenCode 1.18.3 · OCAdapter 1.0</span><span><strong>Environment</strong> Local · ~/c4os</span><span><strong>Effective</strong> ' +
      escapeHtml(((modelProfiles[model] || {}).capabilities || []).join(', ') || 'Text') +
      '</span></div></details>' +
      '<div class="message__content">' +
      messageBodyMarkup(text, 'response-container') +
      actionsMarkup(
        'message',
        'message-' + messageCounter,
        model,
        markdownPlainText(text).slice(0, 70),
      ) +
      '</div>';
    thread.appendChild(node);
    queueAgentStream(node);
    return node;
  }

  function browserPreview(state) {
    var dashboard = state.url.indexOf('dashboard') > -1;
    return (
      '<span>' +
      (dashboard ? 'WORKSPACE' : 'EXAMPLE DOMAIN') +
      '</span><h3>' +
      (dashboard ? 'Account dashboard' : 'A simple page for examples') +
      '</h3><p>' +
      (dashboard
        ? 'Your recent projects and activity are ready.'
        : 'This domain is ready for browsing and testing.') +
      '</p>'
    );
  }

  function updateBrowserCard(id) {
    var state = artifacts[id];
    var card = document.querySelector('[data-artifact-id="' + id + '"]');
    if (!state || !card) return;
    card.querySelector('[data-artifact-status]').textContent = state.status;
    card.querySelector('[data-browser-address]').value = state.url;
    card.querySelector('[data-browser-preview]').innerHTML =
      browserPreview(state);
    card.querySelector('[data-browser-history]').textContent =
      state.history.length +
      (state.history.length === 1 ? ' navigation' : ' navigations');
    var reply = card.querySelector('[data-reply]');
    reply.setAttribute('data-target-title', 'Browser · ' + state.title);
    reply.setAttribute('data-target-excerpt', state.url);
    var buttons = card.querySelectorAll('[data-browser-action]');
    buttons[0].disabled = state.historyIndex <= 0;
    buttons[1].disabled = state.historyIndex >= state.history.length - 1;
    if (focusedArtifact === id) renderFocusViewer();
  }

  function navigateBrowser(id, url, replaceHistory) {
    var state = artifacts[id];
    if (!state) return;
    if (!/^https?:\/\//i.test(url)) url = 'https://' + url;
    if (!replaceHistory) {
      state.history = state.history.slice(0, state.historyIndex + 1);
      state.history.push(url);
      state.historyIndex = state.history.length - 1;
    }
    state.url = url;
    state.title =
      url.indexOf('dashboard') > -1
        ? 'Account dashboard'
        : url.replace(/^https?:\/\//, '').replace(/\/$/, '') || 'Webpage';
    state.status = replaceHistory ? 'Refreshed' : 'Loaded';
    updateBrowserCard(id);
  }

  function createBrowserArtifact(url) {
    artifactCounter += 1;
    var id = 'browser-' + artifactCounter;
    var normalized = /^https?:\/\//i.test(url) ? url : 'https://' + url;
    artifacts[id] = {
      id: id,
      type: 'browser',
      title: normalized.replace(/^https?:\/\//, '').replace(/\/$/, ''),
      url: normalized,
      history: [normalized],
      historyIndex: 0,
      status: 'Ready',
    };
    var frame = artifactFrameMarkup({
      body:
        '<div class="browser-preview" data-browser-preview>' +
        browserPreview(artifacts[id]) +
        '</div>',
      footer:
        '<footer class="artifact-frame__footer"><span data-artifact-status>Ready</span><span data-browser-history>1 navigation</span></footer>',
      header: browserToolbarMarkup(id, normalized),
      headerClass: 'browser-toolbar',
    });
    var node = createResponseArtifactNode(artifacts[id], {
      activity: 'Created the browser artifact',
      duration: '8 sec',
      frame: frame,
      id: id,
      label: 'Browser artifact ' + artifacts[id].title,
      replyExcerpt: normalized,
      replyTitle: 'Browser · ' + artifacts[id].title,
      summary: 'I prepared this response artifact from the requested webpage.',
      type: 'browser',
    });
    thread.appendChild(node);
    queueAgentStream(node);
  }

  function updateFileCard(id) {
    var state = artifacts[id];
    var card = document.querySelector('[data-artifact-id="' + id + '"]');
    if (!state || !card) return;
    card.querySelector('[data-file-preview]').textContent =
      state.proposedContent || state.content;
    var fileVersion = card.querySelector('[data-file-version]');
    if (fileVersion) fileVersion.textContent = 'Version ' + state.version;
    var artifactMeta = card.querySelector('[data-artifact-meta]');
    if (artifactMeta)
      artifactMeta.textContent = 'Markdown · Version ' + state.version;
    var artifactStatus = card.querySelector('[data-artifact-status]');
    if (artifactStatus) artifactStatus.textContent = state.status;
    card.querySelector('[data-file-approval]').hidden =
      state.editing || !state.proposedContent;
    card.querySelector('[data-file-preview]').hidden = state.editing;
    card.querySelector('[data-file-editor]').hidden = !state.editing;
    card.querySelector('[data-file-edit]').hidden = state.editing;
    card.querySelector('[data-file-cancel]').hidden = !state.editing;
    card.querySelector('[data-file-save]').hidden = !state.editing;
    if (state.editing)
      setEditorText(
        card.querySelector('[data-file-editor-input]'),
        state.draftContent == null
          ? state.proposedContent || state.content
          : state.draftContent,
      );
    if (state.editing && artifactStatus) artifactStatus.textContent = 'Editing';
    if (focusedArtifact === id) renderFocusViewer();
  }

  function beginFileEdit(card) {
    var state = artifacts[card.getAttribute('data-artifact-id')];
    if (!state) return;
    state.editing = true;
    state.draftContent = state.proposedContent || state.content;
    var editor = card.querySelector('[data-file-editor-input]');
    setEditorText(editor, state.draftContent);
    card.querySelector('[data-file-lines]').textContent = lineNumbers(
      editorText(editor),
    );
    updateFileCard(state.id);
    editor.focus();
  }

  function cancelFileEdit(card) {
    var state = artifacts[card.getAttribute('data-artifact-id')];
    if (!state) return;
    state.editing = false;
    state.draftContent = null;
    updateFileCard(state.id);
  }

  function saveFileEdit(card) {
    var state = artifacts[card.getAttribute('data-artifact-id')];
    if (!state) return;
    state.content = editorText(card.querySelector('[data-file-editor-input]'));
    state.draftContent = null;
    state.proposedContent = '';
    state.version += 1;
    state.status = 'Saved';
    state.editing = false;
    updateFileCard(state.id);
  }

  function createFileArtifact(path) {
    artifactCounter += 1;
    var id = 'file-' + artifactCounter;
    var title = path.split('/').pop() || 'untitled.md';
    artifacts[id] = {
      id: id,
      type: 'file',
      title: title,
      path: path.indexOf('/') > -1 ? path : '~/project/docs/' + path,
      content: sampleFileContent(),
      draftContent: null,
      version: 1,
      status: 'Viewing',
      proposedContent: '',
      editing: false,
    };
    var frame = artifactFrameMarkup({
      body:
        '<pre class="file-preview" data-file-preview>' +
        escapeHtml(artifacts[id].content) +
        '</pre>' +
        fileEditorMarkup(title, artifacts[id].content) +
        '<div class="file-approval" data-file-approval hidden><div><strong>2 edits proposed</strong><span>Typography and punctuation</span></div><div><button type="button" data-file-reject>Reject</button><button type="button" data-file-approve>Approve &amp; save</button></div></div>',
      header:
        breadcrumbMarkup(artifacts[id].path, true) +
        fileHeaderActionsMarkup(id, title),
    });
    var node = createResponseArtifactNode(artifacts[id], {
      activity: 'Loaded the file response',
      duration: '11 sec',
      frame: frame,
      id: id,
      label: 'File artifact ' + title,
      replyExcerpt: path,
      replyTitle: 'File · ' + title,
      summary: 'I opened the requested file for review.',
      type: 'file',
    });
    thread.appendChild(node);
    queueAgentStream(node);
  }

  function createFolderArtifact(path) {
    artifactCounter += 1;
    var normalized = path.replace(/\/$/, '') || '~/project';
    var id = 'folder-' + artifactCounter;
    var title = normalized.split('/').filter(Boolean).pop() || 'project';
    artifacts[id] = {
      id: id,
      type: 'folder',
      title: title,
      path: normalized,
      status: 'Read only',
      previewFile: null,
    };
    var node = createResponseArtifactNode(artifacts[id], {
      activity: 'Listed the folder contents',
      duration: '7 sec',
      frame: folderSurfaceMarkup(id, normalized),
      id: id,
      label: 'File Explorer artifact ' + title,
      replyExcerpt: normalized,
      replyTitle: 'File Explorer · ' + title,
      summary:
        'I opened the selected folder as a read-only File Explorer artifact.',
      type: 'folder',
    });
    thread.appendChild(node);
    queueAgentStream(node);
  }

  function terminalOutput(command) {
    if (/^ls\b/.test(command)) return 'index.html  notes.md  src  wireframes';
    if (/^pwd\b/.test(command)) return '/Users/demo/project';
    if (/du|how big|size/i.test(command)) return '42M\t.';
    return 'Command completed successfully.';
  }

  function isLongRunningCommand(command) {
    return /(^|\s)(npm run dev|serve|watch|tail -f)(\s|$)/i.test(command);
  }

  function terminalEntry(id) {
    return (
      terminalSession.entries.filter(function (entry) {
        return entry.id === id;
      })[0] || null
    );
  }

  function syncTerminalComposer() {
    var input = inputForMode('terminal');
    var prefix = document.querySelector('.terminal-prefix');
    if (input) input.placeholder = 'Enter a command';
    if (prefix) prefix.textContent = '$';
    if (currentMode === 'terminal' && !replyTarget) {
      primaryAction.setAttribute('aria-label', 'Run command');
      primaryAction.title = 'Run command';
    }
  }

  function formatTerminalElapsed(seconds) {
    var value = Math.max(0, seconds || 0);
    var minutes = Math.floor(value / 60);
    var remainder = value % 60;
    return (
      String(minutes).padStart(2, '0') +
      ':' +
      String(remainder).padStart(2, '0')
    );
  }

  function terminalResultMarkup(entry) {
    if (entry.status === 'running')
      return (
        '<span class="terminal-live-dot" aria-hidden="true"></span>Running · ' +
        formatTerminalElapsed(entry.elapsed)
      );
    if (entry.status === 'interrupted') return 'Interrupted · Exit 130';
    return 'Exit ' + String(entry.exit == null ? 0 : entry.exit);
  }

  function terminalHeaderActionsMarkup(entry) {
    return (
      '<div class="artifact-frame__actions">' +
      '<button class="terminal-stop" type="button" data-terminal-stop="' +
      escapeHtml(entry.id) +
      '" aria-label="Stop ' +
      escapeHtml(entry.command) +
      '" title="Stop process"' +
      (entry.status === 'running' ? '' : ' hidden') +
      '>' +
      icons.stop +
      '</button>' +
      '<button type="button" data-expand="' +
      escapeHtml(entry.id) +
      '" aria-label="Expand terminal artifact" title="Expand">' +
      icons.expand +
      '</button></div>'
    );
  }

  function terminalCardSurfaceMarkup(entry) {
    return artifactFrameMarkup({
      body:
        '<pre class="terminal-output" data-terminal-output>' +
        escapeHtml(entry.output) +
        '</pre>',
      footer:
        '<footer class="artifact-frame__footer"><span data-terminal-result>' +
        terminalResultMarkup(entry) +
        '</span><span>' +
        terminalSession.id +
        '</span></footer>',
      header:
        '<div class="artifact-frame__identity"><div><strong data-artifact-title>$ ' +
        escapeHtml(entry.command) +
        '</strong><span data-artifact-meta>' +
        escapeHtml(terminalSession.cwd) +
        ' · ' +
        terminalSession.id +
        '</span></div></div>' +
        terminalHeaderActionsMarkup(entry),
    });
  }

  function terminalInlinePromptMarkup() {
    return '<form class="terminal-inline-prompt" data-terminal-session-form><span aria-hidden="true">$</span><label class="sr-only" for="expanded-terminal-input">Terminal command</label><input id="expanded-terminal-input" data-terminal-session-input aria-label="Terminal command" autocomplete="off" spellcheck="false"></form>';
  }

  function terminalSessionMarkup(selectedId) {
    var entry =
      terminalEntry(selectedId) ||
      terminalSession.entries[terminalSession.entries.length - 1];
    var running = entry && entry.status === 'running';
    var output = entry ? '$ ' + entry.command + '\n' + entry.output : '';
    return focusArtifactMarkup(
      'terminal',
      'Expanded terminal session',
      '<header class="terminal-session-header"><div><strong>Terminal</strong><span>' +
        escapeHtml(terminalSession.cwd) +
        ' · ' +
        terminalSession.id +
        '</span></div><div class="artifact-shell__controls"><button class="artifact-shell__control artifact-shell__control--icon terminal-stop" type="button" data-terminal-stop="' +
        escapeHtml(running ? entry.id : '') +
        '" aria-label="Stop current process" title="Stop process"' +
        (running ? '' : ' hidden') +
        '>' +
        icons.stop +
        '</button><button class="artifact-shell__control artifact-shell__control--icon" type="button" data-close-focused-artifact aria-label="Close Terminal artifact" title="Close">' +
        icons.close +
        '</button></div></header>' +
        '<div class="terminal-main-body" data-terminal-main-body data-terminal-entry="' +
        escapeHtml(entry ? entry.id : '') +
        '" data-status="' +
        escapeHtml(entry ? entry.status : 'completed') +
        '" role="log" aria-live="polite"><span class="terminal-main-status" data-terminal-session-result>' +
        (entry ? terminalResultMarkup(entry) : '') +
        '</span><pre class="terminal-main-output" data-terminal-session-output>' +
        escapeHtml(output) +
        '</pre>' +
        (running ? '' : terminalInlinePromptMarkup()) +
        '</div>',
    );
  }

  function updateFocusedTerminal(entry) {
    var sessionBody = focusViewer.querySelector(
      '[data-terminal-main-body][data-terminal-entry="' + entry.id + '"]',
    );
    if (!sessionBody) return;
    sessionBody.setAttribute('data-status', entry.status);
    sessionBody.querySelector('[data-terminal-session-output]').textContent =
      '$ ' + entry.command + '\n' + entry.output;
    sessionBody.querySelector('[data-terminal-session-result]').innerHTML =
      terminalResultMarkup(entry);
    var stop = focusViewer.querySelector(
      '.terminal-session-header [data-terminal-stop]',
    );
    if (stop) {
      stop.hidden = entry.status !== 'running';
      stop.setAttribute(
        'data-terminal-stop',
        entry.status === 'running' ? entry.id : '',
      );
    }
    var prompt = sessionBody.querySelector('[data-terminal-session-form]');
    if (entry.status === 'running' && prompt) prompt.remove();
    if (entry.status !== 'running' && !prompt)
      sessionBody.insertAdjacentHTML('beforeend', terminalInlinePromptMarkup());
    sessionBody.scrollTop = sessionBody.scrollHeight;
  }

  function updateTerminalCard(entry) {
    var card = document.querySelector('[data-artifact-id="' + entry.id + '"]');
    if (card) {
      card.setAttribute('data-terminal-status', entry.status);
      card.setAttribute(
        'aria-label',
        (entry.status === 'running' ? 'Running ' : '') +
          'Terminal artifact ' +
          entry.command +
          ' output',
      );
      card.classList.toggle(
        'artifact-provider--running',
        entry.status === 'running',
      );
      card.querySelector('[data-terminal-output]').textContent = entry.output;
      card.querySelector('[data-terminal-result]').innerHTML =
        terminalResultMarkup(entry);
      var duration = card.querySelector(
        '.artifact-shell__thinking summary span',
      );
      if (duration)
        duration.textContent =
          entry.status === 'running'
            ? 'Running for ' + entry.elapsed + ' sec'
            : entry.status === 'interrupted'
              ? 'Stopped after ' + entry.elapsed + ' sec'
              : 'Worked for 6 sec';
      var stop = card.querySelector('[data-terminal-stop]');
      if (stop) stop.hidden = entry.status !== 'running';
    }
    if (
      focusedArtifact &&
      artifacts[focusedArtifact] &&
      artifacts[focusedArtifact].type === 'terminal'
    )
      updateFocusedTerminal(entry);
  }

  function startTerminalStream(entry) {
    if (terminalStreamTimer) window.clearInterval(terminalStreamTimer);
    terminalStreamTimer = window.setInterval(function () {
      if (entry.status !== 'running') {
        window.clearInterval(terminalStreamTimer);
        terminalStreamTimer = null;
        return;
      }
      terminalStreamTick += 1;
      entry.elapsed += 2;
      var lines = [
        'GET /api/session 200 9ms',
        'hmr update /src/workspace.js',
        'GET /assets/styles.css 200 4ms',
      ];
      entry.output += '\n' + lines[terminalStreamTick % lines.length];
      updateTerminalCard(entry);
      var body = focusViewer.querySelector('[data-terminal-main-body]');
      if (body) body.scrollTop = body.scrollHeight;
    }, 2000);
  }

  function stopTerminalEntry(id) {
    var entry = terminalEntry(id);
    if (!entry || entry.status !== 'running') return;
    entry.output += '\n^C';
    entry.status = 'interrupted';
    entry.exit = 130;
    terminalSession.activeEntryId = null;
    if (terminalStreamTimer) window.clearInterval(terminalStreamTimer);
    terminalStreamTimer = null;
    updateTerminalCard(entry);
    syncTerminalComposer();
  }

  function createTerminalArtifact(command) {
    artifactCounter += 1;
    var id = 'terminal-' + artifactCounter;
    var running = isLongRunningCommand(command);
    var entry = {
      id: id,
      command: command,
      output: running
        ? '> workspace@1.0.0 dev\n> vite --host 127.0.0.1\n\nLocal: http://127.0.0.1:4179/\nready in 412 ms'
        : terminalOutput(command),
      status: running ? 'running' : 'completed',
      exit: running ? null : 0,
      elapsed: 0,
    };
    terminalSession.entries.push(entry);
    if (running) terminalSession.activeEntryId = id;
    artifacts[id] = {
      id: id,
      type: 'terminal',
      title: command,
      sessionId: terminalSession.id,
    };
    var node = createResponseArtifactNode(artifacts[id], {
      activity: running
        ? 'Streaming process output'
        : 'Captured the command output',
      duration: running ? '1 sec' : '6 sec',
      frame: terminalCardSurfaceMarkup(entry),
      id: id,
      label:
        (running ? 'Running ' : '') +
        'Terminal artifact ' +
        command +
        ' output',
      replyExcerpt: entry.output,
      replyTitle: 'Terminal · ' + command,
      summary: running
        ? 'The command is running in the active terminal session.'
        : 'I ran the requested command in the active terminal session.',
      type: 'terminal',
    });
    node.classList.toggle('artifact-provider--running', running);
    node.setAttribute('data-terminal-session', terminalSession.id);
    node.setAttribute('data-terminal-status', entry.status);
    thread.appendChild(node);
    queueAgentStream(node);
    syncTerminalComposer();
    if (running) startTerminalStream(entry);
    if (
      focusedArtifact &&
      artifacts[focusedArtifact] &&
      artifacts[focusedArtifact].type === 'terminal'
    )
      renderFocusViewer();
    return entry;
  }

  function submitReply(text, attachments) {
    var target = replyTarget;
    appendUserMessage(text, replyReferenceText(target), attachments);
    if (target.kind === 'browser' && artifacts[target.id]) {
      navigateBrowser(target.id, 'https://example.com/dashboard', false);
      artifacts[target.id].status = 'Updated from reply';
      updateBrowserCard(target.id);
      appendAssistantMessage(
        'I updated the existing browser artifact with the requested navigation.',
      );
    } else if (target.kind === 'file' && artifacts[target.id]) {
      var file = artifacts[target.id];
      file.proposedContent = file.content
        .replace('keeps chat at the center', 'keeps chat in the center')
        .replace('This file is open', 'This file is now open');
      if (file.proposedContent === file.content)
        file.proposedContent += '\n\nTypos corrected and copy refined.';
      file.status = 'Approval required';
      updateFileCard(target.id);
      appendAssistantMessage(
        'I prepared the file edits. Approve them on the original artifact to save immediately.',
      );
    } else if (target.kind === 'terminal') {
      var referencedEntry = terminalEntry(target.id);
      if (/server|running|still on|active/i.test(text)) {
        appendAssistantMessage(
          referencedEntry && referencedEntry.status === 'running'
            ? 'Yes. The referenced process is still running in the shared terminal session.'
            : 'No. The referenced process is no longer running in the shared terminal session.',
        );
      } else if (/big|size|space/i.test(text)) {
        appendAssistantMessage(
          'Based on the referenced terminal output, the project folder is approximately 42 MB.',
        );
      } else {
        appendAssistantMessage(
          'I used the referenced terminal command and output as context for this response.',
        );
      }
    } else {
      appendAssistantMessage(
        'I used the quoted message as context for this response.',
      );
    }
    replyTarget = null;
    replyStrip.hidden = true;
    replyPanel.hidden = true;
    modeControl.hidden = false;
    setMode(currentMode, false);
  }

  function submitMode() {
    var input = inputForMode(currentMode);
    var value = inputValue(input).trim();
    if (currentMode === 'chat' && showCompatibilityAlert(selectedModelName(), 'send')) return;
    var chatAttachments = currentMode === 'chat' ? takeDraftAttachments() : [];
    if (!value && chatAttachments.length === 0) return;
    if (currentMode === 'chat') {
      appendUserMessage(value, null, chatAttachments);
      appendAssistantMessage(
        chatAttachments.length
          ? '**Attachments added.**\n\nI’ve included the prompt and files in the conversation context.'
          : '**Prompt received.**\n\nI’ve added that prompt to the conversation.',
      );
      clearComposerInput(input);
    } else if (currentMode === 'files') {
      appendUserMessage('Open ' + value);
      if (
        fileSelectionKind === 'folder' ||
        /\/$/.test(value) ||
        value.split('/').pop().indexOf('.') === -1
      )
        createFolderArtifact(value);
      else createFileArtifact(value);
    } else if (currentMode === 'browser') {
      appendUserMessage('Open ' + value);
      createBrowserArtifact(value);
    } else {
      appendUserMessage('$ ' + value);
      createTerminalArtifact(value);
      clearComposerInput(input);
    }
    setMode(currentMode, false);
    scrollThread();
  }

  function scrollThread() {
    window.setTimeout(function () {
      thread.scrollTo({ top: thread.scrollHeight, behavior: 'smooth' });
      window.setTimeout(updateThreadScrollButton, 320);
    }, 20);
  }

  function updateThreadScrollButton() {
    var remaining =
      thread.scrollHeight - thread.scrollTop - thread.clientHeight;
    var scrollable = thread.scrollHeight > thread.clientHeight + 2;
    threadScrollButton.hidden = !scrollable || remaining <= 24;
  }

  /** Shows the shared clipboard result above the composer. */
  function showCopyNotice(message) {
    clearTimeout(copyTimer);
    copyNotifier.hidden = false;
    copyNotifier.textContent = message;
    copyTimer = window.setTimeout(function () {
      copyNotifier.hidden = true;
    }, 1400);
  }

  /** Copies through the legacy selection API when Clipboard API access fails. */
  function fallbackCopyText(text) {
    var area = document.createElement('textarea');
    area.value = text;
    area.style.position = 'fixed';
    area.style.opacity = '0';
    document.body.appendChild(area);
    area.select();
    var copied = false;
    try {
      copied = document.execCommand('copy');
    } catch (error) {
      copied = false;
    }
    area.remove();
    return copied;
  }

  /** Copies text and reports success only after a clipboard write succeeds. */
  function copyText(text) {
    var fallback = function () {
      showCopyNotice(fallbackCopyText(text) ? 'Copied' : 'Unable to copy');
    };
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(function () {
        showCopyNotice('Copied');
      }, fallback);
    } else fallback();
  }

  /** Copies the source representation appropriate to a message or artifact. */
  function contextualCopy(button) {
    var container = button.closest('[data-artifact-card], .message');
    var text = '';
    if (container.matches('.message')) {
      var attachmentGroup = container.querySelector('.message-attachments');
      var messageBody = container.querySelector('.message__body');
      text = [
        attachmentGroup ? attachmentGroup.innerText.trim() : '',
        markdownSource(messageBody).trim(),
      ]
        .filter(Boolean)
        .join('\n');
    } else {
      var id = container.getAttribute('data-artifact-id');
      var type = container.getAttribute('data-artifact-type');
      if (type === 'browser') text = artifacts[id].url;
      if (type === 'file') text = artifacts[id].content;
      if (type === 'folder') text = artifacts[id].path;
      if (type === 'terminal') {
        var entry = terminalSession.entries.filter(function (item) {
          return item.id === id;
        })[0];
        text = '$ ' + entry.command + '\n' + entry.output;
      }
    }
    copyText(text);
  }

  function maximumChatPaneHeight() {
    return Math.max(160, shell.getBoundingClientRect().height * 0.6);
  }

  function clampChatPaneHeight(nextHeight) {
    if (!leftChatPane) return;
    var maximumHeight = maximumChatPaneHeight();
    var minimumHeight = Math.min(minimumChatPaneHeight, maximumHeight);
    if (chatPaneHeight === null)
      chatPaneHeight = shell.getBoundingClientRect().height * 0.4;
    if (typeof nextHeight === 'number') chatPaneHeight = nextHeight;
    chatPaneHeight = Math.round(
      Math.max(minimumHeight, Math.min(maximumHeight, chatPaneHeight)),
    );
    shell.style.setProperty('--left-chat-pane-height', chatPaneHeight + 'px');
    leftChatPaneResize.setAttribute(
      'aria-valuemin',
      String(Math.round(minimumHeight)),
    );
    leftChatPaneResize.setAttribute(
      'aria-valuemax',
      String(Math.round(maximumHeight)),
    );
    leftChatPaneResize.setAttribute('aria-valuenow', String(chatPaneHeight));
  }

  function clearFocusPlaceholder() {
    document
      .querySelectorAll('.artifact-shell--in-workspace')
      .forEach(function (card) {
        card.classList.remove('artifact-shell--in-workspace');
        var placeholder = card.querySelector('[data-focus-placeholder]');
        if (placeholder) placeholder.remove();
      });
  }

  function setFocusPlaceholder(id) {
    var state = artifacts[id];
    var card = document.querySelector('[data-artifact-id="' + id + '"]');
    if (!state || !card) return;
    card.classList.add('artifact-shell--in-workspace');
    var placeholder = document.createElement('div');
    placeholder.className = 'artifact-shell__placeholder';
    placeholder.setAttribute('data-focus-placeholder', '');
    var label =
      state.type === 'browser'
        ? 'Browser'
        : state.type === 'folder'
          ? 'File Explorer'
          : state.type === 'terminal'
            ? 'Terminal session'
            : 'File';
    placeholder.textContent = label + ' open in the workspace · ' + state.title;
    card
      .querySelector('.artifact-shell__identity')
      .insertAdjacentElement('afterend', placeholder);
  }

  /** Applies the response or compact pane context to transcript artifacts. */
  function setThreadArtifactContext(context) {
    thread.setAttribute('data-artifact-context', context);
    thread.querySelectorAll('[data-artifact-card]').forEach(function (card) {
      card.classList.toggle('artifact-shell--pane', context === 'pane');
      card.setAttribute('data-artifact-context', context);
    });
  }

  function renderFocusViewer() {
    var state = artifacts[focusedArtifact];
    if (!state) return;
    if (focusedExplorerFile) {
      focusViewer.innerHTML = explorerSurfaceMarkup(focusedArtifact, {
        file: focusedExplorerFile,
        focused: true,
      });
    } else if (focusedFolderPath || state.type === 'folder') {
      var folderPath = focusedFolderPath || state.path;
      focusViewer.innerHTML = explorerSurfaceMarkup(focusedArtifact, {
        path: folderPath,
        focused: true,
      });
    } else if (state.type === 'browser') {
      focusViewer.innerHTML = focusArtifactMarkup(
        'browser',
        'Focused browser artifact',
        '<div class="browser-focus-toolbar"><button type="button" data-focus-browser="back" aria-label="Back"' +
          (state.historyIndex <= 0 ? ' disabled' : '') +
          '><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m15 18-6-6 6-6"></path></svg></button><button type="button" data-focus-browser="forward" aria-label="Forward"' +
          (state.historyIndex >= state.history.length - 1 ? ' disabled' : '') +
          '><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m9 18 6-6-6-6"></path></svg></button><button type="button" data-focus-browser="refresh" aria-label="Refresh"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M20 11a8 8 0 1 0-2.3 5.7L20 14"></path><path d="M20 6v5h-5"></path></svg></button><label class="sr-only">Web address</label><input data-focus-browser-address value="' +
          escapeHtml(state.url) +
          '" aria-label="Web address"><button class="artifact-shell__control artifact-shell__control--icon" type="button" data-close-focused-artifact aria-label="Close Browser artifact" title="Close">' +
          icons.close +
          '</button></div><div class="browser-focus-page"><small>' +
          (state.url.indexOf('dashboard') > -1
            ? 'WORKSPACE'
            : 'EXAMPLE DOMAIN') +
          '</small><h2>' +
          escapeHtml(state.title) +
          '</h2><p>This expanded Browser artifact uses the entire center workspace while the conversation stays available in the left Chat pane.</p></div>',
      );
    } else if (state.type === 'terminal') {
      focusViewer.innerHTML = terminalSessionMarkup(focusedArtifact);
      window.requestAnimationFrame(function () {
        var body = focusViewer.querySelector('[data-terminal-main-body]');
        if (body) body.scrollTop = body.scrollHeight;
      });
    } else {
      var fileDraft =
        state.draftContent == null
          ? state.proposedContent || state.content
          : state.draftContent;
      focusViewer.innerHTML = focusArtifactMarkup(
        'file',
        'Focused file artifact',
        '<header class="artifact-shell__header"><div class="artifact-shell__breadcrumbs">' +
          breadcrumbMarkup(state.path, true) +
          '</div><div class="artifact-shell__controls"><button class="artifact-shell__control artifact-shell__control--icon" type="button" data-focus-file-edit aria-label="Edit ' +
          escapeHtml(state.title) +
          '" title="Edit"' +
          (state.editing ? ' hidden' : '') +
          '>' +
          icons.edit +
          '</button><button class="artifact-shell__control artifact-shell__control--icon" type="button" data-focus-file-cancel aria-label="Discard changes to ' +
          escapeHtml(state.title) +
          '" title="Discard changes"' +
          (state.editing ? '' : ' hidden') +
          '>' +
          icons.trash +
          '</button><button class="artifact-shell__control artifact-shell__control--icon" type="button" data-focus-file-save aria-label="Save ' +
          escapeHtml(state.title) +
          '" title="Save"' +
          (state.editing ? '' : ' hidden') +
          '>' +
          icons.save +
          '</button><button class="artifact-shell__control artifact-shell__control--icon" type="button" data-close-focused-artifact aria-label="Close File artifact" title="Close">' +
          icons.close +
          '</button></div></header><div class="focus-file-editor" data-focus-file-surface data-editing="' +
          String(state.editing) +
          '"><pre class="focus-file-editor__lines" data-focus-file-lines aria-hidden="true"' +
          (state.editing ? '' : ' hidden') +
          '>' +
          lineNumbers(fileDraft) +
          '</pre><div class="pane-file-editor" data-focus-file-editor role="textbox" aria-label="' +
          (state.editing ? 'Edit ' : 'View ') +
          escapeHtml(state.title) +
          '" aria-multiline="true" aria-readonly="' +
          String(!state.editing) +
          '" contenteditable="' +
          String(state.editing) +
          '" spellcheck="false">' +
          escapeHtml(fileDraft) +
          '</div></div>',
      );
    }
  }

  function focusArtifact(id) {
    var state = artifacts[id];
    if (!state || !artifactProvider(state.type).expandable) return;
    clearFocusPlaceholder();
    if (focusedArtifact === null) {
      modeBeforeArtifact = currentMode;
      setMode('chat', false);
    }
    focusedArtifact = id;
    focusedFolderPath = state.type === 'folder' ? state.path : null;
    focusedExplorerFile = state.type === 'folder' ? state.previewFile : null;
    setFocusPlaceholder(id);
    if (thread.parentElement !== leftChatPaneThread) {
      leftChatPaneThread.appendChild(thread);
      leftChatPaneThread.appendChild(threadScrollButton);
    }
    setThreadArtifactContext('pane');
    threadHost.hidden = true;
    focusViewer.hidden = false;
    leftChatPane.hidden = false;
    shell.setAttribute('data-artifact-focused', 'true');
    renderFocusViewer();
    clampChatPaneHeight();
    window.requestAnimationFrame(updateThreadScrollButton);
  }

  function restoreChat() {
    var closingFileId =
      focusedArtifact &&
      artifacts[focusedArtifact] &&
      artifacts[focusedArtifact].type === 'file'
        ? focusedArtifact
        : null;
    var closingFolderId =
      focusedArtifact &&
      artifacts[focusedArtifact] &&
      artifacts[focusedArtifact].type === 'folder'
        ? focusedArtifact
        : null;
    if (closingFileId && artifacts[closingFileId].editing) {
      artifacts[closingFileId].draftContent = editorText(
        focusViewer.querySelector('[data-focus-file-editor]'),
      );
    }
    if (closingFolderId) {
      if (focusedExplorerFile && focusedExplorerFile.editing) {
        focusedExplorerFile.draftContent = editorText(
          focusViewer.querySelector('[data-explorer-file-editor]'),
        );
      }
      artifacts[closingFolderId].path =
        focusedFolderPath || artifacts[closingFolderId].path;
      artifacts[closingFolderId].previewFile = focusedExplorerFile;
    }
    clearFocusPlaceholder();
    focusedArtifact = null;
    focusedFolderPath = null;
    focusedExplorerFile = null;
    setThreadArtifactContext('response');
    threadHost.appendChild(thread);
    threadHost.appendChild(threadScrollButton);
    threadHost.hidden = false;
    focusViewer.hidden = true;
    focusViewer.innerHTML = '';
    leftChatPane.hidden = true;
    shell.removeAttribute('data-artifact-focused');
    if (modeBeforeArtifact) setMode(modeBeforeArtifact, false);
    modeBeforeArtifact = null;
    if (closingFileId) updateFileCard(closingFileId);
    if (closingFolderId) updateFolderCard(closingFolderId);
    window.requestAnimationFrame(updateThreadScrollButton);
  }

  function handleBrowserAction(card, action) {
    var id = card.getAttribute('data-artifact-id');
    var state = artifacts[id];
    if (action === 'back' && state.historyIndex > 0) {
      state.historyIndex -= 1;
      state.url = state.history[state.historyIndex];
      state.status = 'Back';
      updateBrowserCard(id);
    } else if (
      action === 'forward' &&
      state.historyIndex < state.history.length - 1
    ) {
      state.historyIndex += 1;
      state.url = state.history[state.historyIndex];
      state.status = 'Forward';
      updateBrowserCard(id);
    } else if (action === 'refresh') navigateBrowser(id, state.url, true);
  }

  leftChatPaneResize.addEventListener('pointerdown', function (event) {
    if (activeChatPaneResizeCleanup) activeChatPaneResizeCleanup();
    var bounds = shell.getBoundingClientRect();
    event.preventDefault();
    shell.setAttribute('data-chat-pane-resizing', 'true');
    document.documentElement.style.cursor = 'row-resize';

    function movePane(moveEvent) {
      if (moveEvent.buttons === 0) {
        stopPaneResize();
        return;
      }
      clampChatPaneHeight(bounds.bottom - moveEvent.clientY);
    }

    function stopPaneResize() {
      if (!activeChatPaneResizeCleanup) return;
      shell.removeAttribute('data-chat-pane-resizing');
      document.documentElement.style.cursor = '';
      document.removeEventListener('pointermove', movePane);
      document.removeEventListener('pointerup', stopPaneResize);
      document.removeEventListener('pointercancel', stopPaneResize);
      window.removeEventListener('blur', stopPaneResize);
      activeChatPaneResizeCleanup = null;
    }

    activeChatPaneResizeCleanup = stopPaneResize;
    document.addEventListener('pointermove', movePane);
    document.addEventListener('pointerup', stopPaneResize);
    document.addEventListener('pointercancel', stopPaneResize);
    window.addEventListener('blur', stopPaneResize);
  });

  leftChatPaneResize.addEventListener('keydown', function (event) {
    var nextHeight = chatPaneHeight;
    if (event.key === 'ArrowUp') nextHeight += 16;
    else if (event.key === 'ArrowDown') nextHeight -= 16;
    else if (event.key === 'Home') nextHeight = minimumChatPaneHeight;
    else if (event.key === 'End') nextHeight = maximumChatPaneHeight();
    else return;
    event.preventDefault();
    clampChatPaneHeight(nextHeight);
  });

  shell.addEventListener('click', function (event) {
    var attachButton = event.target.closest('[data-attach-button]');
    if (attachButton) {
      if (currentMode === 'chat' && attachmentInput) attachmentInput.click();
      return;
    }
    var removeAttachment = event.target.closest('[data-remove-attachment]');
    if (removeAttachment) {
      removeDraftAttachment(
        removeAttachment.getAttribute('data-remove-attachment'),
      );
      return;
    }
    var panelToggle = event.target.closest('[data-panel-toggle]');
    if (panelToggle) {
      var side = panelToggle.getAttribute('data-panel-toggle');
      var open = shell.getAttribute('data-' + side + '-open') !== 'false';
      setPanelOpen(side, !open);
      return;
    }
    if (
      narrowQuery.matches &&
      shell.getAttribute('data-left-open') !== 'false' &&
      event.target.closest('.chat-center')
    ) {
      setPanelOpen('left', false);
    }
    var mode = event.target.closest('[data-mode-option]');
    if (mode) {
      setMode(mode.getAttribute('data-mode-option'), true);
      inputForMode(currentMode).focus();
      return;
    }
    var reply = event.target.closest('[data-reply]');
    if (reply) {
      startReply(reply);
      return;
    }
    if (event.target.closest('[data-reply-cancel]')) {
      cancelReply();
      return;
    }
    var codeCopy = event.target.closest('[data-copy-code]');
    if (codeCopy) {
      copyText(
        codeCopy.closest('.markdown-code').querySelector('code').textContent,
      );
      return;
    }
    var copy = event.target.closest('[data-copy]');
    if (copy) {
      contextualCopy(copy);
      return;
    }
    var terminalStop = event.target.closest('[data-terminal-stop]');
    if (terminalStop) {
      stopTerminalEntry(terminalStop.getAttribute('data-terminal-stop'));
      return;
    }
    var expand = event.target.closest('[data-expand]');
    if (expand) {
      focusArtifact(expand.getAttribute('data-expand'));
      return;
    }
    if (event.target.closest('[data-close-focused-artifact]')) {
      restoreChat();
      return;
    }
    if (event.target.closest('[data-restore-chat]')) {
      restoreChat();
      return;
    }
    var browserAction = event.target.closest('[data-browser-action]');
    if (browserAction) {
      handleBrowserAction(
        browserAction.closest('[data-artifact-card]'),
        browserAction.getAttribute('data-browser-action'),
      );
      return;
    }
    var fileEdit = event.target.closest('[data-file-edit]');
    if (fileEdit) {
      beginFileEdit(fileEdit.closest('[data-artifact-card]'));
      return;
    }
    var fileCancel = event.target.closest('[data-file-cancel]');
    if (fileCancel) {
      cancelFileEdit(fileCancel.closest('[data-artifact-card]'));
      return;
    }
    var fileSave = event.target.closest('[data-file-save]');
    if (fileSave) {
      saveFileEdit(fileSave.closest('[data-artifact-card]'));
      return;
    }
    var approve = event.target.closest('[data-file-approve]');
    if (approve) {
      var fileCard = approve.closest('[data-artifact-card]');
      var file = artifacts[fileCard.getAttribute('data-artifact-id')];
      file.content = file.proposedContent;
      file.proposedContent = '';
      file.version += 1;
      file.status = 'Saved';
      updateFileCard(file.id);
      return;
    }
    var reject = event.target.closest('[data-file-reject]');
    if (reject) {
      var rejected =
        artifacts[
          reject
            .closest('[data-artifact-card]')
            .getAttribute('data-artifact-id')
        ];
      rejected.proposedContent = '';
      rejected.status = 'Viewing';
      updateFileCard(rejected.id);
      return;
    }
    var sourceKind = event.target.closest('[data-select-file-kind]');
    if (sourceKind) {
      fileSelectionKind = sourceKind.getAttribute('data-select-file-kind');
      var fileInput = inputForMode('files');
      fileInput.value =
        fileSelectionKind === 'folder' ? '~/project' : 'notes.md';
      fileInput.setAttribute('data-selection-kind', fileSelectionKind);
      primaryAction.setAttribute(
        'aria-label',
        fileSelectionKind === 'folder' ? 'Open folder' : 'Open file',
      );
      primaryAction.title =
        fileSelectionKind === 'folder' ? 'Open folder' : 'Open file';
      fileSourcePopover.hidden = true;
      document
        .querySelector('[data-browse-file]')
        .setAttribute('aria-expanded', 'false');
      syncPrimaryAction();
      fileInput.focus();
      return;
    }
    if (event.target.closest('[data-browse-file]')) {
      var browseButton = document.querySelector('[data-browse-file]');
      var opening = fileSourcePopover.hidden;
      fileSourcePopover.hidden = !opening;
      browseButton.setAttribute('aria-expanded', String(opening));
      return;
    }
    var folderLink = event.target.closest('[data-open-folder]');
    if (folderLink) {
      var folderCard = folderLink.closest('[data-artifact-card]');
      var nextFolderPath = folderLink.getAttribute('data-open-folder');
      if (folderCard) {
        var folderState =
          artifacts[folderCard.getAttribute('data-artifact-id')];
        if (folderState.type === 'file') {
          convertFileCardToFolder(folderCard, folderState, nextFolderPath);
          return;
        }
        folderState.path = nextFolderPath;
        folderState.title = nextFolderPath.split('/').pop();
        folderState.previewFile = null;
        updateFolderCard(folderState.id);
        return;
      }
      focusedExplorerFile = null;
      focusedFolderPath = nextFolderPath;
      renderFocusViewer();
      return;
    }
    var explorerFileLink = event.target.closest('[data-open-explorer-file]');
    if (explorerFileLink) {
      var openedFile = explorerFile(
        explorerFileLink.getAttribute('data-open-explorer-file'),
      );
      var explorerCard = explorerFileLink.closest('[data-artifact-card]');
      if (explorerCard) {
        var explorerState =
          artifacts[explorerCard.getAttribute('data-artifact-id')];
        explorerState.previewFile = openedFile;
        updateFolderCard(explorerState.id);
        return;
      }
      focusedExplorerFile = openedFile;
      renderFocusViewer();
      return;
    }
    if (event.target.closest('[data-focus-file-edit]')) {
      var focusedFileState = artifacts[focusedArtifact];
      var editor = document.querySelector('[data-focus-file-editor]');
      focusedFileState.editing = true;
      focusedFileState.draftContent = editorText(editor);
      editor.setAttribute('contenteditable', 'true');
      editor.setAttribute('aria-readonly', 'false');
      editor.setAttribute(
        'aria-label',
        'Edit ' + artifacts[focusedArtifact].title,
      );
      document
        .querySelector('[data-focus-file-surface]')
        .setAttribute('data-editing', 'true');
      document.querySelector('[data-focus-file-lines]').hidden = false;
      document.querySelector('[data-focus-file-edit]').hidden = true;
      document.querySelector('[data-focus-file-cancel]').hidden = false;
      document.querySelector('[data-focus-file-save]').hidden = false;
      editor.focus();
      return;
    }
    if (event.target.closest('[data-focus-file-cancel]')) {
      var cancelledFile = artifacts[focusedArtifact];
      cancelledFile.editing = false;
      cancelledFile.draftContent = null;
      updateFileCard(focusedArtifact);
      return;
    }
    if (event.target.closest('[data-focus-file-save]')) {
      var activeFile = artifacts[focusedArtifact];
      activeFile.content = editorText(
        document.querySelector('[data-focus-file-editor]'),
      );
      activeFile.draftContent = null;
      activeFile.proposedContent = '';
      activeFile.version += 1;
      activeFile.status = 'Saved';
      activeFile.editing = false;
      updateFileCard(focusedArtifact);
      return;
    }
    var explorerEdit = event.target.closest('[data-explorer-file-edit]');
    if (explorerEdit) {
      var explorerDocument = explorerEdit.closest(
        '.artifact-frame, .artifact-focus',
      );
      var explorerFileState = explorerEdit.closest('[data-artifact-card]')
        ? artifacts[
            explorerEdit
              .closest('[data-artifact-card]')
              .getAttribute('data-artifact-id')
          ].previewFile
        : focusedExplorerFile;
      if (!explorerFileState || !explorerDocument) return;
      explorerFileState.editing = true;
      if (explorerFileState.draftContent == null)
        explorerFileState.draftContent = explorerFileState.content;
      if (explorerEdit.closest('[data-artifact-card]'))
        updateFolderCard(
          explorerEdit
            .closest('[data-artifact-card]')
            .getAttribute('data-artifact-id'),
        );
      else renderFocusViewer();
      var explorerEditor = (
        explorerEdit.closest('[data-artifact-card]') || focusViewer
      ).querySelector('[data-explorer-file-editor]');
      if (explorerEditor) explorerEditor.focus();
      return;
    }
    var explorerCancel = event.target.closest('[data-explorer-file-cancel]');
    if (explorerCancel) {
      var cancelCard = explorerCancel.closest('[data-artifact-card]');
      var cancelFile = cancelCard
        ? artifacts[cancelCard.getAttribute('data-artifact-id')].previewFile
        : focusedExplorerFile;
      if (!cancelFile) return;
      cancelFile.editing = false;
      cancelFile.draftContent = null;
      if (cancelCard)
        updateFolderCard(cancelCard.getAttribute('data-artifact-id'));
      else renderFocusViewer();
      return;
    }
    var explorerSave = event.target.closest('[data-explorer-file-save]');
    if (explorerSave) {
      var saveCard = explorerSave.closest('[data-artifact-card]');
      var saveRoot = saveCard || focusViewer;
      var saveFile = saveCard
        ? artifacts[saveCard.getAttribute('data-artifact-id')].previewFile
        : focusedExplorerFile;
      if (!saveFile) return;
      saveFile.content = editorText(
        saveRoot.querySelector('[data-explorer-file-editor]'),
      );
      saveFile.draftContent = null;
      saveFile.version += 1;
      saveFile.editing = false;
      if (saveCard) updateFolderCard(saveCard.getAttribute('data-artifact-id'));
      else renderFocusViewer();
      return;
    }
    var focusBrowser = event.target.closest('[data-focus-browser]');
    if (focusBrowser && artifacts[focusedArtifact]) {
      var fakeCard = document.querySelector(
        '[data-artifact-id="' + focusedArtifact + '"]',
      );
      handleBrowserAction(
        fakeCard,
        focusBrowser.getAttribute('data-focus-browser'),
      );
    }
  });

  shell.addEventListener('keydown', function (event) {
    if (
      event.key === 'Enter' &&
      event.target.matches('[data-terminal-session-input]')
    ) {
      event.preventDefault();
      event.target.closest('form').requestSubmit();
      return;
    }
    if (
      event.key === 'Enter' &&
      event.target.matches('[data-focus-browser-address]') &&
      artifacts[focusedArtifact]
    ) {
      navigateBrowser(focusedArtifact, event.target.value, false);
    }
    if (
      event.key === 'Enter' &&
      !event.shiftKey &&
      !event.isComposing &&
      event.target.matches('[data-mode-input], [data-reply-input]')
    ) {
      event.preventDefault();
      composer.requestSubmit();
    }
  });

  focusViewer.addEventListener('submit', function (event) {
    var form = event.target.closest('[data-terminal-session-form]');
    if (!form) return;
    event.preventDefault();
    var input = form.querySelector('[data-terminal-session-input]');
    var value = input.value.trim();
    if (!value) return;
    appendUserMessage('$ ' + value);
    var entry = createTerminalArtifact(value);
    clearFocusPlaceholder();
    focusedArtifact = entry.id;
    setFocusPlaceholder(entry.id);
    renderFocusViewer();
    scrollThread();
    var nextInput = focusViewer.querySelector('[data-terminal-session-input]');
    if (nextInput) nextInput.focus();
  });

  shell.addEventListener('input', function (event) {
    if (event.target.matches('[data-file-editor-input]')) {
      var inlineFileCard = event.target.closest('[data-artifact-card]');
      if (
        inlineFileCard &&
        artifacts[inlineFileCard.getAttribute('data-artifact-id')]
      ) {
        artifacts[
          inlineFileCard.getAttribute('data-artifact-id')
        ].draftContent = editorText(event.target);
      }
      event.target
        .closest('[data-file-editor]')
        .querySelector('[data-file-lines]').textContent = lineNumbers(
        editorText(event.target),
      );
    }
    if (event.target.matches('[data-focus-file-editor]')) {
      if (focusedArtifact && artifacts[focusedArtifact])
        artifacts[focusedArtifact].draftContent = editorText(event.target);
      document.querySelector('[data-focus-file-lines]').textContent =
        lineNumbers(editorText(event.target));
    }
    if (event.target.matches('[data-explorer-file-editor]')) {
      var explorerCard = event.target.closest('[data-artifact-card]');
      var activeExplorerFile = explorerCard
        ? artifacts[explorerCard.getAttribute('data-artifact-id')].previewFile
        : focusedExplorerFile;
      if (activeExplorerFile && activeExplorerFile.editing)
        activeExplorerFile.draftContent = editorText(event.target);
      var explorerLines = event.target
        .closest('[data-explorer-file-document]')
        .querySelector('[data-explorer-file-lines]');
      if (explorerLines)
        explorerLines.textContent = lineNumbers(editorText(event.target));
    }
  });

  shell.addEventListener(
    'scroll',
    function (event) {
      if (!event.target.matches('[data-focus-file-editor]')) return;
      var lines = document.querySelector('[data-focus-file-lines]');
      if (lines) lines.scrollTop = event.target.scrollTop;
    },
    true,
  );

  shell.addEventListener(
    'scroll',
    function (event) {
      if (!event.target.matches('[data-explorer-file-editor]')) return;
      var lines = event.target
        .closest('[data-explorer-file-document]')
        .querySelector('[data-explorer-file-lines]');
      if (lines) lines.scrollTop = event.target.scrollTop;
    },
    true,
  );

  composer.addEventListener('input', syncPrimaryAction);
  shell.addEventListener('workspace:new-thread', function () {
    if (focusedArtifact) restoreChat();
    if (replyTarget) cancelReply();
    setMode('chat', true);
    clearComposerInput(inputForMode('chat'));
    discardDraftAttachments();
    syncPrimaryAction();
  });
  shell.addEventListener('workspace:copy', function (event) {
    if (event.detail && typeof event.detail.text === 'string')
      copyText(event.detail.text);
  });
  composer.addEventListener('submit', function (event) {
    event.preventDefault();
    if (replyTarget) {
      var value = inputValue(replyInput).trim();
      if (currentMode === 'chat' && showCompatibilityAlert(selectedModelName(), 'send')) return;
      var replyAttachments =
        currentMode === 'chat' ? takeDraftAttachments() : [];
      if (!value && replyAttachments.length === 0) return;
      submitReply(value, replyAttachments);
      clearComposerInput(replyInput);
      scrollThread();
    } else submitMode();
    syncPrimaryAction();
  });

  thread.addEventListener('scroll', updateThreadScrollButton, {
    passive: true,
  });
  threadScrollButton.addEventListener('click', scrollThread);
  window.addEventListener('resize', updateThreadScrollButton);
  if (window.ResizeObserver)
    new ResizeObserver(updateThreadScrollButton).observe(thread);
  if (window.ResizeObserver && composerDock)
    new ResizeObserver(syncComposerDockHeight).observe(composerDock);
  else window.addEventListener('resize', syncComposerDockHeight);
  new MutationObserver(function () {
    window.requestAnimationFrame(updateThreadScrollButton);
  }).observe(thread, { childList: true });

  document
    .querySelector('[data-mic]')
    .addEventListener('click', function (event) {
      var button = event.currentTarget;
      var active = button.getAttribute('aria-pressed') !== 'true';
      button.setAttribute('aria-pressed', String(active));
      promptStatus.textContent = active ? 'Listening…' : '';
    });

  attachmentInput.addEventListener('change', function (event) {
    addDraftAttachments(event.target.files);
    event.target.value = '';
  });

  shell.addEventListener('dragenter', function (event) {
    if (currentMode !== 'chat' || !hasDraggedFiles(event)) return;
    event.preventDefault();
    fileDragDepth += 1;
    setDropzoneVisible(true);
  });

  shell.addEventListener('dragover', function (event) {
    if (currentMode !== 'chat' || !hasDraggedFiles(event)) return;
    event.preventDefault();
    event.dataTransfer.dropEffect = 'copy';
    setDropzoneVisible(true);
  });

  shell.addEventListener('dragleave', function (event) {
    if (currentMode !== 'chat' || fileDragDepth === 0) return;
    fileDragDepth = Math.max(0, fileDragDepth - 1);
    if (fileDragDepth === 0) setDropzoneVisible(false);
  });

  shell.addEventListener('drop', function (event) {
    if (
      currentMode !== 'chat' ||
      !event.dataTransfer ||
      event.dataTransfer.files.length === 0
    )
      return;
    event.preventDefault();
    fileDragDepth = 0;
    setDropzoneVisible(false);
    addDraftAttachments(event.dataTransfer.files);
  });

  window.addEventListener('dragend', function () {
    fileDragDepth = 0;
    setDropzoneVisible(false);
  });

  window.addEventListener('beforeunload', function () {
    attachmentUrls.forEach(function (url) {
      URL.revokeObjectURL(url);
    });
  });

  var menuTriggers = Array.prototype.slice.call(
    document.querySelectorAll('[data-menu-trigger]'),
  );
  function closeMenus() {
    menuTriggers.forEach(function (trigger) {
      trigger.setAttribute('aria-expanded', 'false');
      var menu = document.querySelector(
        '[data-menu="' + trigger.getAttribute('data-menu-trigger') + '"]',
      );
      if (menu) menu.hidden = true;
    });
  }
  menuTriggers.forEach(function (trigger) {
    trigger.addEventListener('click', function (event) {
      event.stopPropagation();
      var name = trigger.getAttribute('data-menu-trigger');
      var menu = document.querySelector('[data-menu="' + name + '"]');
      var open = trigger.getAttribute('aria-expanded') !== 'true';
      closeMenus();
      trigger.setAttribute('aria-expanded', String(open));
      menu.hidden = !open;
    });
  });
  document.querySelectorAll('[data-menu]').forEach(function (menu) {
    menu.addEventListener('click', function (event) {
      var option = event.target.closest('[data-option]');
      if (!option) return;
      if (menu.getAttribute('data-menu') === 'model') {
        var nextModel = option.getAttribute('data-option');
        if (showCompatibilityAlert(nextModel, 'switch')) {
          closeMenus();
          return;
        }
        applyModelSelection(option);
        closeMenus();
        return;
      }
      menu.querySelectorAll('[data-option]').forEach(function (item) {
        item.setAttribute('aria-checked', String(item === option));
      });
      document.querySelector(
        '[data-option-value="' + menu.getAttribute('data-menu') + '"]',
      ).textContent = option.getAttribute('data-option');
      closeMenus();
    });
  });
  document.querySelector('[data-menu="model"]')?.addEventListener('click', function (event) {
    var filter = event.target.closest('[data-model-filter]');
    if (!filter) return;
    var capability = filter.getAttribute('data-model-filter');
    filter.parentElement.querySelectorAll('[data-model-filter]').forEach(function (item) {
      item.setAttribute('aria-pressed', String(item === filter));
    });
    filter.closest('[data-menu="model"]').querySelectorAll('[data-model-capabilities]').forEach(function (item) {
      item.hidden = capability !== 'all' && item.getAttribute('data-model-capabilities').split(' ').indexOf(capability) === -1;
    });
  });
  document.querySelector('[data-compatibility-alert]')?.addEventListener('click', function (event) {
    var action = event.target.closest('[data-compatibility-action]');
    if (!action) return;
    var kind = action.getAttribute('data-compatibility-action');
    if (kind === 'change-model') {
      var compatible = document.querySelector('[data-menu="model"] [data-option="GPT-5"]');
      if (compatible) applyModelSelection(compatible);
    } else if (kind === 'convert') {
      incompatibleAttachments(pendingModelSwitch || selectedModelName()).forEach(function (attachment) {
        attachment.converted = true;
      });
      if (pendingModelSwitch) {
        var pending = document.querySelector('[data-menu="model"] [data-option="' + pendingModelSwitch + '"]');
        if (pending) applyModelSelection(pending);
      }
      hideCompatibilityAlert();
      renderDraftAttachments();
    } else if (kind === 'remove') {
      var targetModel = pendingModelSwitch || selectedModelName();
      var removeIds = incompatibleAttachments(targetModel).map(function (item) { return item.id; });
      removeIds.forEach(removeDraftAttachment);
      if (pendingModelSwitch) {
        var next = document.querySelector('[data-menu="model"] [data-option="' + pendingModelSwitch + '"]');
        if (next) applyModelSelection(next);
      }
      hideCompatibilityAlert();
    } else {
      hideCompatibilityAlert();
    }
  });
  modeTrigger.addEventListener('click', function (event) {
    event.stopPropagation();
    var shouldOpen = modeTrigger.getAttribute('aria-expanded') !== 'true';
    closeMenus();
    if (shouldOpen) openModePopover();
    else closeModePopover(false);
  });
  modeControl.addEventListener('keydown', function (event) {
    var options = Array.prototype.slice.call(
      modePopover.querySelectorAll('[data-mode-option]'),
    );
    var index = options.indexOf(document.activeElement);
    if (event.key === 'Escape') {
      event.preventDefault();
      closeModePopover(true);
    } else if (
      !modePopover.hidden &&
      (event.key === 'ArrowDown' || event.key === 'ArrowUp')
    ) {
      event.preventDefault();
      var direction = event.key === 'ArrowDown' ? 1 : -1;
      options[(index + direction + options.length) % options.length].focus();
    }
  });
  document.addEventListener('click', function (event) {
    if (!event.target.closest('.option-control')) closeMenus();
    if (!event.target.closest('[data-mode-control]')) closeModePopover(false);
    if (!event.target.closest('.file-source-control')) {
      fileSourcePopover.hidden = true;
      document
        .querySelector('[data-browse-file]')
        .setAttribute('aria-expanded', 'false');
    }
  });

  // Chat and Reply share one editor engine and submit through the same form.
  document
    .querySelectorAll('[data-markdown-editor]')
    .forEach(function (editor) {
      markdown.createEditor(editor, {
        label: editor.getAttribute('aria-label'),
        placeholder: editor.getAttribute('data-placeholder'),
        onChange: syncPrimaryAction,
        onSubmit: function () {
          composer.requestSubmit();
        },
      });
    });

  if (window.location.hash === '#chat-capabilities') {
    attachmentCounter += 1;
    draftAttachments.push({
      id: 'attachment-' + attachmentCounter,
      name: 'concept-board.png',
      size: 284000,
      type: 'image/png',
      previewUrl: '',
    });
    var textOnly = document.querySelector('[data-menu="model"] [data-option="Kimi K2"]');
    if (textOnly) applyModelSelection(textOnly);
    showCompatibilityAlert('Kimi K2', 'send');
  }

  updateBrowserCard('browser-1');
  setPanelOpen('left', shell.getAttribute('data-left-open') !== 'false');
  setMode(currentMode, false);
  syncModelDependentControls();
  normalizeForViewport();
  syncComposerDockHeight();
  window.requestAnimationFrame(updateThreadScrollButton);
  startTerminalStream(terminalEntry('terminal-1'));
})();

//settings navigation, resource management, and dialog interactions
(() => {
  const root = document.querySelector('[data-settings-root]');
  if (!root) return;

  const pages = [...root.querySelectorAll('[data-settings-page]')];
  const navItems = [...root.querySelectorAll('[data-settings-nav]')];
  const validPages = new Set(navItems.map((item) => item.dataset.settingsNav));
  validPages.add('advanced-policies');
  const routeSlug = () => {
    const match = window.location.hash.match(/^#settings\/([^/?]+)/);
    return match ? match[1] : 'providers';
  };
  const notifier = root.querySelector('[data-notifier]');
  let notifyTimer;

  const notify = (message) => {
    if (!notifier) return;
    notifier.textContent = message;
    notifier.hidden = false;
    window.clearTimeout(notifyTimer);
    notifyTimer = window.setTimeout(() => {
      notifier.hidden = true;
    }, 2400);
  };

  const activatePage = (slug, updateHash = true) => {
    const next = validPages.has(slug) ? slug : 'providers';
    pages.forEach((page) => {
      page.hidden = page.dataset.settingsPage !== next;
    });
    navItems.forEach((item) => {
      const activeSlug = next === 'advanced-policies' ? 'configuration' : next;
      const active = item.dataset.settingsNav === activeSlug;
      if (active) item.setAttribute('aria-current', 'page');
      else item.removeAttribute('aria-current');
    });
    if (updateHash && window.location.hash !== `#settings/${next}`)
      history.replaceState(null, '', `#settings/${next}`);
  };

  navItems.forEach((item) =>
    item.addEventListener('click', () =>
      activatePage(item.dataset.settingsNav),
    ),
  );
  window.addEventListener('hashchange', () => activatePage(routeSlug(), false));
  activatePage(routeSlug(), false);

  root.addEventListener('click', (event) => {
    const switchButton = event.target.closest('.switch');
    if (!switchButton) return;
    const next = switchButton.getAttribute('aria-checked') !== 'true';
    switchButton.setAttribute('aria-checked', String(next));
    const providerRow = switchButton.closest('[data-provider-row]');
    if (providerRow) {
      providerRow.dataset.providerActive = String(next);
      syncProviderModels();
      notify(
        `${providerRow.querySelector('strong')?.textContent || 'Provider'} ${next ? 'enabled' : 'disabled'}`,
      );
      return;
    }
    const modelRow = switchButton.closest('[data-model-name]');
    if (modelRow) {
      syncModelAvailability(modelRow);
      updateModelBulkAction();
      notify(`${modelRow.dataset.modelName} ${next ? 'enabled' : 'disabled'}`);
      return;
    }
    notify(next ? 'Setting enabled' : 'Setting disabled');
  });

  const runtimeOptions = [...root.querySelectorAll('[data-runtime-option]')];
  const runtimeSave = root.querySelector('[data-runtime-save]');
  let savedRuntime =
    runtimeOptions.find(
      (option) => option.getAttribute('aria-checked') === 'true',
    )?.dataset.runtimeOption || 'opencode';
  let draftRuntime = savedRuntime;
  const syncRuntimeSelection = () => {
    runtimeOptions.forEach((option) => {
      const selected = option.dataset.runtimeOption === draftRuntime;
      option.setAttribute('aria-checked', String(selected));
      const selectedBadge = option.querySelector('[data-runtime-selected]');
      if (selectedBadge) selectedBadge.hidden = !selected;
    });
    if (runtimeSave) runtimeSave.disabled = draftRuntime === savedRuntime;
  };
  runtimeOptions.forEach((option) =>
    option.addEventListener('click', () => {
      draftRuntime = option.dataset.runtimeOption;
      syncRuntimeSelection();
    }),
  );
  runtimeSave?.addEventListener('click', () => {
    savedRuntime = draftRuntime;
    syncRuntimeSelection();
    const runtimeName =
      runtimeOptions
        .find((option) => option.dataset.runtimeOption === savedRuntime)
        ?.querySelector('strong')?.textContent || 'Runtime';
    notify(`${runtimeName} saved as the workspace runtime`);
  });

  const dialogBackdrop = root.querySelector('[data-dialog-backdrop]');
  const dialogs = [...root.querySelectorAll('[data-dialog]')];
  let activeDialog = null;
  const closeDialog = () => {
    dialogs.forEach((dialog) => {
      dialog.hidden = true;
    });
    if (dialogBackdrop) dialogBackdrop.hidden = true;
    activeDialog = null;
  };
  const openDialog = (id) => {
    const dialog = root.querySelector(`#${id}`);
    if (!dialog || !dialogBackdrop) return;
    dialogs.forEach((item) => {
      item.hidden = item !== dialog;
    });
    dialogBackdrop.hidden = false;
    dialog.hidden = false;
    activeDialog = dialog;
    window.setTimeout(
      () => dialog.querySelector('input, select, button')?.focus(),
      0,
    );
  };
  root
    .querySelectorAll('[data-dialog-open]')
    .forEach((button) =>
      button.addEventListener('click', () =>
        openDialog(button.dataset.dialogOpen),
      ),
    );
  root
    .querySelectorAll('[data-dialog-close]')
    .forEach((button) => button.addEventListener('click', closeDialog));
  dialogBackdrop?.addEventListener('click', (event) => {
    if (event.target === dialogBackdrop) closeDialog();
  });
  document.addEventListener('keydown', (event) => {
    if (event.key === 'Escape' && activeDialog) closeDialog();
  });

  const providerDefaults = {
    openrouter: {
      type: 'OpenRouter',
      label: 'OpenRouter - Personal',
      url: 'https://openrouter.ai/api/v1',
      icon: 'OR',
    },
    huggingface: {
      type: 'Hugging Face',
      label: 'Hugging Face - Personal',
      url: 'https://router.huggingface.co/v1',
      icon: 'HF',
    },
    openai: {
      type: 'OpenAI',
      label: 'OpenAI - Personal',
      url: 'https://api.openai.com/v1',
      icon: 'AI',
    },
    compatible: {
      type: 'OpenAI Compatible',
      label: 'Local AI Server',
      url: 'http://127.0.0.1:4000/v1',
      icon: 'OA',
    },
  };
  const providerForm = root.querySelector('[data-provider-form]');
  const providerStatus = root.querySelector('[data-provider-form-status]');
  const providerTypeSelect = providerForm?.elements.providerType;
  const providerAuthSelect = providerForm?.elements.auth;
  const providerDialogTitle = root.querySelector(
    '[data-provider-dialog-title]',
  );
  const providerSubmit = root.querySelector('[data-provider-submit]');
  const providerBaseURL = providerForm?.querySelector(
    '[data-provider-base-url]',
  );
  const providerAuth = providerForm?.querySelector('[data-provider-auth]');
  const providerHeaderName = providerForm?.querySelector(
    '[data-provider-header-name]',
  );
  const providerAPIKey = providerForm?.querySelector('[data-provider-api-key]');
  const providerHeaders = providerForm?.querySelector(
    '[data-provider-headers]',
  );
  const providerKeyHelp = providerForm?.querySelector(
    '[data-provider-key-help]',
  );
  let editingProviderId = null;

  const updateProviderFields = ({ useDefaults = false } = {}) => {
    if (!providerForm) return;
    const type = providerDefaults[providerTypeSelect.value]
      ? providerTypeSelect.value
      : 'compatible';
    if (useDefaults) {
      const defaults = providerDefaults[type];
      providerForm.elements.label.value = defaults.label;
      providerForm.elements.baseUrl.value = defaults.url;
      providerForm.elements.apiKey.value = '';
      providerForm.elements.auth.value = 'bearer';
      providerForm.elements.headerName.value = 'X-API-Key';
      providerForm.elements.headers.value = '{}';
    }
    const compatible = type === 'compatible';
    const auth = compatible ? providerAuthSelect.value : 'bearer';
    providerBaseURL.hidden = !compatible;
    providerAuth.hidden = !compatible;
    providerHeaders.hidden = !compatible;
    providerHeaderName.hidden = !compatible || auth !== 'header';
    providerAPIKey.hidden = compatible && auth === 'none';
    providerForm.elements.baseUrl.required = compatible;
    providerForm.elements.headerName.required = compatible && auth === 'header';
    providerForm.elements.apiKey.required =
      !editingProviderId && !(compatible && auth === 'none');
    if (providerKeyHelp)
      providerKeyHelp.textContent = editingProviderId
        ? 'Leave blank to keep the saved API key.'
        : 'Required for this provider.';
    if (providerStatus) providerStatus.textContent = '';
  };

  const openProviderAdd = () => {
    if (!providerForm) return;
    editingProviderId = null;
    providerForm.reset();
    providerTypeSelect.value = 'openrouter';
    providerForm.elements.apiKey.type = 'password';
    providerDialogTitle.textContent = 'Add Provider';
    providerSubmit.textContent = 'Save Provider';
    updateProviderFields({ useDefaults: true });
    openDialog('provider-dialog');
  };

  const openProviderEdit = (row) => {
    if (!providerForm || !row) return;
    editingProviderId = row.dataset.providerRow;
    const type = providerDefaults[row.dataset.providerType]
      ? row.dataset.providerType
      : 'compatible';
    providerTypeSelect.value = type;
    providerForm.elements.label.value =
      row.dataset.providerLabel ||
      row.querySelector('strong')?.textContent ||
      '';
    providerForm.elements.baseUrl.value =
      row.dataset.providerUrl || providerDefaults[type].url;
    providerForm.elements.apiKey.value = '';
    providerForm.elements.apiKey.type = 'password';
    providerForm.elements.auth.value = row.dataset.providerAuth || 'bearer';
    providerForm.elements.headerName.value =
      row.dataset.providerHeaderName || 'X-API-Key';
    providerForm.elements.headers.value = row.dataset.providerHeaders || '{}';
    providerDialogTitle.textContent = 'Edit Provider';
    providerSubmit.textContent = 'Save Changes';
    updateProviderFields();
    openDialog('provider-dialog');
  };

  root
    .querySelector('[data-provider-add]')
    ?.addEventListener('click', openProviderAdd);
  providerTypeSelect?.addEventListener('change', () =>
    updateProviderFields({ useDefaults: !editingProviderId }),
  );
  providerAuthSelect?.addEventListener('change', () => updateProviderFields());
  root.addEventListener('click', (event) => {
    const editButton = event.target.closest('[data-provider-configure]');
    if (editButton) openProviderEdit(editButton.closest('[data-provider-row]'));
  });
  root
    .querySelector('[data-password-toggle]')
    ?.addEventListener('click', (event) => {
      const input = providerForm?.elements.apiKey;
      if (!input) return;
      const reveal = input.type === 'password';
      input.type = reveal ? 'text' : 'password';
      event.currentTarget.setAttribute(
        'aria-label',
        reveal ? 'Hide API key' : 'Show API key',
      );
    });
  const providerFormValid = () => {
    if (!providerForm) return false;
    const required = [...providerForm.querySelectorAll('[required]')].filter(
      (input) => !input.closest('label, [data-provider-field]')?.hidden,
    );
    const invalid = required.find((input) => !input.value.trim());
    if (invalid) {
      invalid.focus();
      if (providerStatus)
        providerStatus.textContent = 'Enter all required connection details.';
      return false;
    }
    const label = providerForm.elements.label.value.trim().toLowerCase();
    const duplicate = [...root.querySelectorAll('[data-provider-row]')].find(
      (row) =>
        row.dataset.providerRow !== editingProviderId &&
        (row.dataset.providerLabel || '').trim().toLowerCase() === label,
    );
    if (duplicate) {
      providerForm.elements.label.focus();
      if (providerStatus)
        providerStatus.textContent = 'Provider labels must be unique.';
      return false;
    }
    if (providerTypeSelect.value === 'compatible') {
      try {
        const headers = JSON.parse(providerForm.elements.headers.value || '{}');
        if (!headers || Array.isArray(headers) || typeof headers !== 'object')
          throw new Error();
      } catch {
        providerForm.elements.headers.focus();
        if (providerStatus)
          providerStatus.textContent = 'Headers must be a valid JSON object.';
        return false;
      }
    }
    return true;
  };
  root.querySelector('[data-provider-test]')?.addEventListener('click', () => {
    if (!providerFormValid()) return;
    if (providerStatus)
      providerStatus.textContent =
        'Connected successfully · Models are available.';
  });
  providerForm?.addEventListener('submit', (event) => {
    event.preventDefault();
    if (!providerFormValid()) return;
    const label = providerForm.elements.label.value.trim();
    const type = providerDefaults[providerTypeSelect.value]
      ? providerTypeSelect.value
      : 'compatible';
    const defaults = providerDefaults[type];
    const url =
      type === 'compatible'
        ? providerForm.elements.baseUrl.value.trim()
        : defaults.url;
    const id =
      editingProviderId ||
      label
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, '-')
        .replace(/(^-|-$)/g, '') ||
      'custom';
    let existing = root.querySelector(`[data-provider-row="${id}"]`);
    if (!existing) {
      existing = document.createElement('article');
      existing.className = 'resource-row';
      existing.dataset.providerRow = id;
      existing.dataset.providerActive = 'true';
      existing.dataset.providerModels = '2';
      root.querySelector('[data-provider-list]')?.append(existing);
    }
    existing.dataset.providerType = type;
    existing.dataset.providerLabel = label;
    existing.dataset.providerUrl = url;
    existing.dataset.providerAuth =
      type === 'compatible' ? providerForm.elements.auth.value : 'bearer';
    existing.dataset.providerHeaderName =
      type === 'compatible'
        ? providerForm.elements.headerName.value.trim()
        : '';
    existing.dataset.providerHeaders =
      type === 'compatible' ? providerForm.elements.headers.value.trim() : '{}';
    const active = existing.dataset.providerActive !== 'false';
    existing.innerHTML = `<div class="resource-icon">${escapeHTML(defaults.icon)}</div><div class="resource-copy"><strong>${escapeHTML(label)}</strong><p>${escapeHTML(defaults.type)}</p></div><button class="button" type="button" data-provider-configure data-dialog-open="provider-dialog">Edit</button><button class="switch" type="button" role="switch" aria-checked="${active}" aria-label="Enable ${escapeHTML(label)}"><span></span></button>`;
    syncProviderModels();
    closeDialog();
    notify(`${label} saved`);
  });

  function syncProviderModels() {
    const providers = [...root.querySelectorAll('[data-provider-row]')];
    let active = 0;
    let models = 0;
    providers.forEach((provider) => {
      const enabled = provider.dataset.providerActive === 'true';
      if (enabled) {
        active += 1;
        models += Number(provider.dataset.providerModels || 0);
      }
    });
    const activeCount = root.querySelector('[data-provider-active-count]');
    const modelCount = root.querySelector('[data-provider-model-count]');
    if (activeCount) activeCount.textContent = String(active);
    if (modelCount) modelCount.textContent = String(models);
    filterModels();
  }

  const modelSearch = root.querySelector('[data-model-search]');
  const modelProviderFilter = root.querySelector(
    '[data-model-provider-filter]',
  );
  const modelCapabilityFilter = root.querySelector(
    '[data-model-capability-filter]',
  );
  const modelBulkAction = root.querySelector('[data-model-bulk]');
  const modelEmpty = root.querySelector('[data-model-empty]');
  const escapeModelHTML = (value) =>
    String(value).replace(
      /[&<>'"]/g,
      (character) =>
        ({
          '&': '&amp;',
          '<': '&lt;',
          '>': '&gt;',
          "'": '&#39;',
          '"': '&quot;',
        })[character],
    );
  const modelRows = [...root.querySelectorAll('[data-model-name]')];
  modelRows.forEach((row) => {
    const capabilities = (row.dataset.modelCapabilities || '')
      .split(' ')
      .filter(Boolean);
    const copy = row.querySelector(':scope > div');
    if (copy) {
      const chips = document.createElement('span');
      chips.className = 'model-capability-chips';
      chips.innerHTML = capabilities
        .map(
          (capability) =>
            '<span>' +
            escapeModelHTML(capability[0].toUpperCase() + capability.slice(1)) +
            '</span>',
        )
        .join('');
      chips.insertAdjacentHTML(
        'beforeend',
        '<span>' +
          escapeModelHTML(row.dataset.modelContext || 'Unknown context') +
          '</span>',
      );
      copy.append(chips);
    }
    const heading = copy?.querySelector(':scope > strong');
    if (heading) {
      const modelName = document.createElement('button');
      modelName.className = 'model-name-button';
      modelName.type = 'button';
      modelName.dataset.modelDetails = row.dataset.modelName;
      modelName.textContent = heading.textContent;
      modelName.setAttribute(
        'aria-label',
        'View details for ' + row.dataset.modelName,
      );
      heading.replaceWith(modelName);
    }
  });

  function syncModelAvailability(row) {
    const enabled =
      row.querySelector('.switch')?.getAttribute('aria-checked') === 'true';
    const status = row.querySelector('[data-model-status]');
    if (status) status.textContent = enabled ? 'Enabled' : 'Disabled';
  }

  function visibleModelRows() {
    return modelRows.filter((row) => !row.hidden);
  }

  function updateModelBulkAction() {
    if (!modelBulkAction) return;
    const visible = visibleModelRows();
    const allDisabled =
      visible.length > 0 &&
      visible.every(
        (row) =>
          row.querySelector('.switch')?.getAttribute('aria-checked') ===
          'false',
      );
    modelBulkAction.disabled = visible.length === 0;
    modelBulkAction.textContent = allDisabled
      ? 'Enable'
      : 'Disable';
  }

  function filterModels() {
    const query = modelSearch?.value.trim().toLowerCase() || '';
    const providerValue = modelProviderFilter?.value || 'all';
    const capabilityValue = modelCapabilityFilter?.value || 'all';
    let visible = 0;
    modelRows.forEach((row) => {
      const provider = root.querySelector(
        `[data-provider-row="${row.dataset.modelProvider}"]`,
      );
      const providerActive = provider?.dataset.providerActive === 'true';
      const providerMatch =
        providerValue === 'all' || row.dataset.modelProvider === providerValue;
      const searchMatch =
        !query || row.textContent.toLowerCase().includes(query);
      const capabilityMatch =
        capabilityValue === 'all' ||
        (row.dataset.modelCapabilities || '').split(' ').includes(capabilityValue);
      const match =
        providerActive && providerMatch && capabilityMatch && searchMatch;
      row.hidden = !match;
      if (match) visible += 1;
    });
    if (modelEmpty) modelEmpty.hidden = visible > 0;
    updateModelBulkAction();
  }
  modelSearch?.addEventListener('input', filterModels);
  modelProviderFilter?.addEventListener('change', filterModels);
  modelCapabilityFilter?.addEventListener('change', filterModels);
  root.addEventListener('click', (event) => {
    const button = event.target.closest('[data-model-details]');
    if (!button) return;
    const row = button.closest('[data-model-name]');
    const capabilities = (row?.dataset.modelCapabilities || '')
      .split(' ')
      .filter(Boolean);
    const title = root.querySelector('[data-model-dialog-title]');
    const route = root.querySelector('[data-model-dialog-route]');
    const grid = root.querySelector('[data-model-dialog-capabilities]');
    if (title) title.textContent = row?.dataset.modelName || 'Model capabilities';
    if (route)
      route.textContent =
        (row?.querySelector(':scope > div > span')?.textContent ||
          'Configured provider') +
        ' · ' +
        (row?.dataset.modelContext || 'Unknown context');
    if (grid)
      grid.innerHTML = ['vision', 'tools', 'reasoning', 'audio']
        .map((capability) => {
          const supported = capabilities.includes(capability);
          return (
            '<div><span>' +
            escapeModelHTML(capability[0].toUpperCase() + capability.slice(1)) +
            '</span><strong>' +
            (supported ? 'Supported' : 'Unsupported') +
            '</strong></div>'
          );
        })
        .join('');
    openDialog('model-capability-dialog');
  });
  modelBulkAction?.addEventListener('click', () => {
    const visible = visibleModelRows();
    if (!visible.length) return;
    const enable = visible.every(
      (row) =>
        row.querySelector('.switch')?.getAttribute('aria-checked') === 'false',
    );
    visible.forEach((row) => {
      row
        .querySelector('.switch')
        ?.setAttribute('aria-checked', String(enable));
      syncModelAvailability(row);
    });
    updateModelBulkAction();
    notify(
      `${visible.length} visible model${visible.length === 1 ? '' : 's'} ${enable ? 'enabled' : 'disabled'}`,
    );
  });
  root
    .querySelector('[data-refresh-models]')
    ?.addEventListener('click', (event) => {
      const button = event.currentTarget;
      const previous = button.innerHTML;
      button.textContent = 'Refreshing…';
      button.disabled = true;
      window.setTimeout(() => {
        button.innerHTML = previous;
        button.disabled = false;
        notify('Models refreshed from active providers');
      }, 700);
    });

  const setTabbedPanel = (kind, value) => {
    root
      .querySelectorAll(`[data-${kind}-tab]`)
      .forEach((button) =>
        button.setAttribute(
          'aria-selected',
          String(button.dataset[`${kind}Tab`] === value),
        ),
      );
    root.querySelectorAll(`[data-${kind}-panel]`).forEach((panel) => {
      panel.hidden = panel.dataset[`${kind}Panel`] !== value;
    });
  };
  root
    .querySelectorAll('[data-plugin-tab]')
    .forEach((button) =>
      button.addEventListener('click', () =>
        setTabbedPanel('plugin', button.dataset.pluginTab),
      ),
    );

  const pluginDetails = {
    github: {
      icon: 'GH',
      title: 'GitHub Workflow',
      description:
        'Repository workflows for review, checks, and pull requests.',
      skills: 'PR Review, Fix CI, Address Comments',
      apps: 'GitHub',
      mcp: 'GitHub tools',
    },
    browser: {
      icon: 'WB',
      title: 'Web Research',
      description: 'Current-source research with a browser-backed workflow.',
      skills: 'Browser Research, Source Synthesis',
      apps: 'Browser',
      mcp: 'None',
    },
    analytics: {
      icon: 'AN',
      title: 'Analytics Workspace',
      description: 'Product analytics and spreadsheet reporting workflows.',
      skills: 'Analytics Audit, Funnel Review, Workbook Builder, QA',
      apps: 'Spreadsheets',
      mcp: 'None',
    },
    calendar: {
      icon: 'CA',
      title: 'Calendar Assistant',
      description:
        'Meeting preparation, scheduling, and daily calendar briefs.',
      skills:
        'Daily Brief, Meeting Prep, Group Scheduler, Free Up Time, Calendar',
      apps: 'Google Calendar',
      mcp: 'None',
    },
  };
  const installedPlugins = root.querySelector('[data-installed-plugins]');
  const installPluginFromDialog = root.querySelector(
    '[data-plugin-install-dialog]',
  );
  const uninstallPlugin = root.querySelector('[data-plugin-uninstall]');
  let activePluginId = null;
  const updateInstalledPluginCount = () => {
    const installedCount = root.querySelector(
      '[data-plugin-tab="installed"] span',
    );
    if (installedCount && installedPlugins)
      installedCount.textContent = String(
        installedPlugins.querySelectorAll('.extension-card').length,
      );
  };
  const openPluginDetails = (id) => {
    const data = pluginDetails[id];
    if (!data) return;
    activePluginId = id;
    root.querySelector('[data-plugin-dialog-icon]').textContent = data.icon;
    root.querySelector('[data-plugin-dialog-title]').textContent = data.title;
    root.querySelector('[data-plugin-dialog-body]').innerHTML =
      `<p>${data.description}</p><div class="dialog-capabilities"><div><strong>Skills</strong><span>${data.skills}</span></div><div><strong>Apps</strong><span>${data.apps}</span></div><div><strong>MCP</strong><span>${data.mcp}</span></div></div><nav class="plugin-legal-links" aria-label="${data.title} links"><a href="#" data-plugin-link="Website">Website</a><a href="#" data-plugin-link="Terms">Terms</a><a href="#" data-plugin-link="Privacy Policy">Privacy Policy</a></nav>`;
    const installed = !!installedPlugins?.querySelector(
      `[data-plugin-id="${id}"]`,
    );
    if (installPluginFromDialog) installPluginFromDialog.hidden = installed;
    if (uninstallPlugin) uninstallPlugin.hidden = !installed;
    openDialog('plugin-dialog');
  };
  root
    .querySelector('[data-plugin-dialog-body]')
    ?.addEventListener('click', (event) => {
      const link = event.target.closest('[data-plugin-link]');
      if (!link) return;
      event.preventDefault();
      notify(
        `${link.dataset.pluginLink} opened for ${pluginDetails[activePluginId]?.title || 'plugin'}`,
      );
    });
  root
    .querySelectorAll('[data-plugin-details]')
    .forEach((button) =>
      button.addEventListener('click', () =>
        openPluginDetails(button.dataset.pluginDetails),
      ),
    );
  installPluginFromDialog?.addEventListener('click', () => {
    if (!activePluginId) return;
    const installButton = root.querySelector(
      `[data-plugin-install="${activePluginId}"]`,
    );
    if (!installButton || installButton.disabled) return;
    installButton.click();
    closeDialog();
  });
  uninstallPlugin?.addEventListener('click', () => {
    if (!activePluginId || !installedPlugins) return;
    const name = pluginDetails[activePluginId]?.title || 'Plugin';
    installedPlugins
      .querySelector(`[data-plugin-id="${activePluginId}"]`)
      ?.remove();
    const installButton = root.querySelector(
      `[data-plugin-install="${activePluginId}"]`,
    );
    if (installButton) {
      installButton.textContent = 'Install';
      installButton.disabled = false;
    }
    updateInstalledPluginCount();
    closeDialog();
    notify(`${name} uninstalled`);
  });
  root.querySelectorAll('[data-plugin-install]').forEach((button) =>
    button.addEventListener('click', () => {
      const card = button.closest('.extension-card');
      if (
        card &&
        installedPlugins &&
        !installedPlugins.querySelector(
          `[data-plugin-id="${card.dataset.pluginId}"]`,
        )
      ) {
        const installedCard = card.cloneNode(true);
        installedCard.querySelector('[data-plugin-install]')?.remove();
        const statusBadge = installedCard.querySelector('header .badge');
        if (statusBadge) statusBadge.textContent = 'Installed';
        const footer = installedCard.querySelector('footer');
        const toggle = document.createElement('button');
        toggle.className = 'switch';
        toggle.type = 'button';
        toggle.setAttribute('role', 'switch');
        toggle.setAttribute('aria-checked', 'true');
        toggle.setAttribute(
          'aria-label',
          `Enable ${pluginDetails[button.dataset.pluginInstall]?.title || 'plugin'}`,
        );
        toggle.innerHTML = '<span></span>';
        footer?.append(toggle);
        installedPlugins.append(installedCard);
        updateInstalledPluginCount();
        installedCard
          .querySelector('[data-plugin-details]')
          ?.addEventListener('click', () =>
            openPluginDetails(button.dataset.pluginInstall),
          );
      }
      button.textContent = 'Installed';
      button.disabled = true;
      notify(
        `${pluginDetails[button.dataset.pluginInstall]?.title || 'Plugin'} installed`,
      );
    }),
  );
  root
    .querySelector('[data-plugin-search]')
    ?.addEventListener('input', (event) => {
      const query = event.currentTarget.value.trim().toLowerCase();
      root
        .querySelectorAll('[data-plugin-directory] .extension-card')
        .forEach((card) => {
          card.hidden =
            !!query && !card.textContent.toLowerCase().includes(query);
        });
    });
  const marketplaceToggle = root.querySelector('[data-marketplace-toggle]');
  const marketplaceMenu = root.querySelector('[data-marketplace-menu]');
  const closeMarketplaceMenu = () => {
    if (!marketplaceMenu || !marketplaceToggle) return;
    marketplaceMenu.hidden = true;
    marketplaceToggle.setAttribute('aria-expanded', 'false');
  };
  marketplaceToggle?.addEventListener('click', () => {
    const open = marketplaceMenu?.hidden !== false;
    if (marketplaceMenu) marketplaceMenu.hidden = !open;
    marketplaceToggle.setAttribute('aria-expanded', String(open));
  });
  root
    .querySelector('[data-marketplace-add]')
    ?.addEventListener('click', () => {
      closeMarketplaceMenu();
      openDialog('marketplace-dialog');
    });
  document.addEventListener('click', (event) => {
    if (!marketplaceMenu || marketplaceMenu.hidden) return;
    if (!event.target.closest('.marketplace-picker')) closeMarketplaceMenu();
  });
  const marketplaceForm = root.querySelector('[data-marketplace-form]');
  const marketplaceStatus = root.querySelector('[data-marketplace-status]');
  marketplaceForm?.addEventListener('submit', (event) => {
    event.preventDefault();
    const source = marketplaceForm.elements.source.value.trim();
    if (!source) {
      marketplaceStatus.textContent =
        'Enter a repository, Git URL, or local folder.';
      marketplaceForm.elements.source.focus();
      return;
    }
    marketplaceStatus.textContent = '';
    closeDialog();
    marketplaceForm.reset();
    marketplaceForm.elements.ref.value = 'main';
    notify('Plugin marketplace added');
  });

  const skillData = [
    {
      id: 'frontend-design',
      name: 'Frontend Design',
      description:
        'Build intentional, distinctive application interfaces from an approved structure.',
      overview:
        'Use Frontend Design when an application surface needs a clear visual direction and a thoughtful interface system. It helps shape hierarchy, layout, component treatment, and interaction details while staying grounded in the approved product structure.',
      enabled: true,
    },
    {
      id: 'workspace-review',
      name: 'Workspace Review',
      description:
        'Review workspace changes, validation evidence, and unresolved decisions.',
      overview:
        'Workspace Review provides a focused pass over local changes and their supporting evidence. It identifies mismatches, missing validation, and unresolved decisions before work is presented or handed off.',
      enabled: true,
    },
    {
      id: 'document-editor',
      name: 'Document Editor',
      description:
        'Create and revise structured documents with layout-aware checks.',
      overview:
        'Document Editor supports drafting and revision where document structure and presentation both matter. It keeps headings, tables, pagination, and review-ready layout in view throughout the editing workflow.',
      enabled: false,
    },
    {
      id: 'browser-automation',
      name: 'Browser Automation',
      description:
        'Exercise real browser workflows and capture review evidence.',
      overview:
        'Browser Automation runs realistic browser interactions for review and verification. It can navigate flows, exercise controls, inspect visible states, and capture evidence from the rendered experience.',
      enabled: true,
    },
    {
      id: 'analytics',
      name: 'Analytics',
      description:
        'Set up, improve, or audit product analytics and measurement.',
      overview:
        'Analytics helps define meaningful events, properties, funnels, and quality checks. Use it when measurement needs to be planned, reviewed, or aligned with product decisions.',
      enabled: true,
    },
    {
      id: 'ab-testing',
      name: 'A/B Testing',
      description:
        'Plan experiments, variants, guardrails, and rollout decisions.',
      overview:
        'Use A/B Testing when a product, page, onboarding step, pricing flow, or campaign needs a clear experiment design. It helps define hypotheses, variants, success metrics, guardrails, and rollout decisions.',
      enabled: true,
    },
  ];
  let selectedSkillId = skillData[0]?.id || '';
  const skillList = root.querySelector('[data-skill-list]');
  const skillSearch = root.querySelector('[data-skill-search]');
  const skillEmpty = root.querySelector('[data-skill-empty]');
  const skillDialogTitle = root.querySelector('[data-skill-dialog-title]');
  const skillDialogDescription = root.querySelector(
    '[data-skill-dialog-description]',
  );
  const skillDialogOverview = root.querySelector(
    '[data-skill-dialog-overview]',
  );
  const skillDialogToggle = root.querySelector('[data-skill-dialog-toggle]');
  const skillIcon =
    '<span class="skill-circle-icon" aria-hidden="true"><svg viewBox="0 0 24 24"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"></path><path d="M14 2v6h6"></path></svg></span>';
  const renderSkills = () => {
    if (!skillList) return;
    const query = skillSearch?.value.trim().toLowerCase() || '';
    const list = skillData.filter(
      (skill) =>
        !query ||
        `${skill.name} ${skill.description}`.toLowerCase().includes(query),
    );
    skillList.hidden = list.length === 0;
    if (skillEmpty) skillEmpty.hidden = list.length !== 0;
    skillList.innerHTML = list
      .map(
        (skill) =>
          `<article class="skill-row" data-skill-id="${skill.id}"><button class="skill-row-main" type="button" data-skill-open aria-label="View ${skill.name} details">${skillIcon}<span class="skill-row-copy"><strong>${skill.name}</strong><span>${skill.description}</span></span></button><button class="switch" type="button" role="switch" aria-checked="${skill.enabled}" aria-label="Make ${skill.name} available in this project" data-skill-toggle><span></span></button></article>`,
      )
      .join('');
  };
  const openSkillDialog = (skill) => {
    if (!skill) return;
    selectedSkillId = skill.id;
    skillDialogTitle.textContent = skill.name;
    skillDialogDescription.textContent = skill.description;
    skillDialogOverview.textContent = skill.overview;
    skillDialogToggle.setAttribute('aria-checked', String(skill.enabled));
    skillDialogToggle.setAttribute(
      'aria-label',
      `Make ${skill.name} available in this project`,
    );
    openDialog('skill-dialog');
  };
  skillList?.addEventListener('click', (event) => {
    const row = event.target.closest('[data-skill-id]');
    if (!row) return;
    const skill = skillData.find((item) => item.id === row.dataset.skillId);
    if (!skill) return;
    if (event.target.closest('[data-skill-toggle]')) {
      event.stopPropagation();
      skill.enabled = !skill.enabled;
      renderSkills();
      notify(
        `${skill.name} ${skill.enabled ? 'available' : 'unavailable'} in this project`,
      );
      return;
    }
    if (event.target.closest('[data-skill-open]')) openSkillDialog(skill);
  });
  skillDialogToggle?.addEventListener('click', (event) => {
    event.stopPropagation();
    const skill = skillData.find((item) => item.id === selectedSkillId);
    if (!skill) return;
    skill.enabled = !skill.enabled;
    skillDialogToggle.setAttribute('aria-checked', String(skill.enabled));
    renderSkills();
    notify(
      `${skill.name} ${skill.enabled ? 'available' : 'unavailable'} in this project`,
    );
  });
  root
    .querySelector('[data-skill-uninstall]')
    ?.addEventListener('click', () => {
      const index = skillData.findIndex((item) => item.id === selectedSkillId);
      if (index < 0) return;
      const [skill] = skillData.splice(index, 1);
      selectedSkillId = skillData[0]?.id || '';
      renderSkills();
      closeDialog();
      notify(`${skill.name} uninstalled`);
    });
  root.querySelector('[data-skill-try]')?.addEventListener('click', () => {
    const skill = skillData.find((item) => item.id === selectedSkillId);
    if (!skill) return;
    closeDialog();
    notify(`Opening ${skill.name} in chat`);
  });
  skillSearch?.addEventListener('input', renderSkills);
  renderSkills();

  const mcpServers = [
    {
      id: 'node-repl',
      name: 'node_repl',
      transport: 'stdio',
      command: 'node-repl-mcp',
      arguments: ['--workspace', '.'],
      environment: [],
      passthrough: ['PATH'],
      workingDirectory: '~/code',
      enabled: true,
    },
    {
      id: 'openai-developer-docs',
      name: 'openaiDeveloperDocs',
      transport: 'http',
      url: 'https://developers.openai.com/mcp',
      bearerTokenEnv: 'OPENAI_DOCS_TOKEN',
      headers: [],
      environmentHeaders: [],
      enabled: true,
    },
    {
      id: 'stackpress-blog-mcp',
      name: 'stackpress_blog_mcp',
      transport: 'stdio',
      command: 'stackpress-blog-mcp',
      arguments: ['serve'],
      environment: [{ key: 'NODE_ENV', value: 'development' }],
      passthrough: ['PATH'],
      workingDirectory: '~/code/stackpress-blog',
      enabled: true,
    },
  ];
  const mcpForm = root.querySelector('[data-mcp-form]');
  const mcpTransport = root.querySelector('[data-mcp-transport]');
  const mcpList = root.querySelector('[data-mcp-list]');
  const mcpEmpty = root.querySelector('[data-mcp-empty]');
  const mcpDialogTitle = root.querySelector('[data-mcp-dialog-title]');
  const mcpStatus = root.querySelector('[data-mcp-form-status]');
  let editingMcpId = null;
  const mcpSettingsIcon =
    '<svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="3"></circle><path d="M12 2v3M12 19v3M4.93 4.93l2.12 2.12M16.95 16.95l2.12 2.12M2 12h3M19 12h3M4.93 19.07l2.12-2.12M16.95 7.05l2.12-2.12"></path></svg>';
  const mcpRemoveIcon =
    '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M3 6h18M8 6V4h8v2M19 6l-1 14H6L5 6M10 11v5M14 11v5"></path></svg>';
  const renderMcpServers = () => {
    if (!mcpList) return;
    mcpList.hidden = mcpServers.length === 0;
    if (mcpEmpty) mcpEmpty.hidden = mcpServers.length !== 0;
    mcpList.innerHTML = mcpServers
      .map(
        (server) =>
          `<article class="mcp-row" data-mcp-row="${server.id}"><strong>${escapeHTML(server.name)}</strong><button class="icon-button mcp-configure" type="button" data-mcp-configure aria-label="Configure ${escapeHTML(server.name)}">${mcpSettingsIcon}</button><button class="switch" type="button" role="switch" aria-checked="${server.enabled}" aria-label="Enable ${escapeHTML(server.name)}" data-mcp-toggle><span></span></button></article>`,
      )
      .join('');
  };
  const setMcpTransport = (transport) => {
    const next = transport === 'http' ? 'http' : 'stdio';
    if (mcpTransport) mcpTransport.value = next;
    root
      .querySelectorAll('[data-mcp-transport-option]')
      .forEach((button) =>
        button.setAttribute(
          'aria-pressed',
          String(button.dataset.mcpTransportOption === next),
        ),
      );
    root.querySelectorAll('[data-mcp-panel]').forEach((panel) => {
      panel.hidden = panel.dataset.mcpPanel !== next;
    });
    if (mcpStatus) mcpStatus.textContent = '';
  };
  const appendMcpRepeaterRow = (type, value = {}) => {
    const repeater = root.querySelector(`[data-mcp-repeater="${type}"]`);
    if (!repeater) return;
    const singleValue = typeof value === 'string' ? value : '';
    const pair =
      type === 'environment' ||
      type === 'header' ||
      type === 'environmentHeader';
    const placeholders =
      type === 'environmentHeader'
        ? ['Header', 'Environment variable']
        : ['Key', 'Value'];
    const fields = pair
      ? `<input data-mcp-key value="${escapeHTML(value.key || '')}" placeholder="${placeholders[0]}"><input data-mcp-value value="${escapeHTML(value.value || '')}" placeholder="${placeholders[1]}">`
      : `<input data-mcp-value value="${escapeHTML(singleValue)}" placeholder="${type === 'argument' ? 'Argument' : 'VARIABLE_NAME'}">`;
    repeater.insertAdjacentHTML(
      'beforeend',
      `<div class="mcp-repeat-row${pair ? ' mcp-repeat-row--pair' : ''}" data-mcp-repeat-row>${fields}<button class="mcp-remove-row" type="button" data-mcp-remove-row aria-label="Remove ${type}">${mcpRemoveIcon}</button></div>`,
    );
  };
  const resetMcpRepeaters = (server = {}) => {
    const values = {
      argument: server.arguments || [''],
      environment: server.environment || [{ key: '', value: '' }],
      passthrough: server.passthrough || [''],
      header: server.headers || [{ key: '', value: '' }],
      environmentHeader: server.environmentHeaders || [{ key: '', value: '' }],
    };
    Object.entries(values).forEach(([type, rows]) => {
      const repeater = root.querySelector(`[data-mcp-repeater="${type}"]`);
      if (repeater) repeater.innerHTML = '';
      (rows.length
        ? rows
        : [
            type === 'environment' ||
            type === 'header' ||
            type === 'environmentHeader'
              ? { key: '', value: '' }
              : '',
          ]
      ).forEach((row) => appendMcpRepeaterRow(type, row));
    });
  };
  const openMcpDialog = (server = null) => {
    if (!mcpForm) return;
    editingMcpId = server?.id || null;
    mcpDialogTitle.textContent = server
      ? 'Edit MCP server'
      : 'Connect to a custom MCP';
    mcpForm.elements.name.value = server?.name || '';
    mcpForm.elements.command.value = server?.command || '';
    mcpForm.elements.workingDirectory.value = server?.workingDirectory || '';
    mcpForm.elements.url.value = server?.url || '';
    mcpForm.elements.bearerTokenEnv.value = server?.bearerTokenEnv || '';
    resetMcpRepeaters(server || {});
    setMcpTransport(server?.transport || 'stdio');
    if (mcpStatus) mcpStatus.textContent = '';
    openDialog('mcp-dialog');
  };
  const collectMcpRows = (type) =>
    [
      ...root.querySelectorAll(
        `[data-mcp-repeater="${type}"] [data-mcp-repeat-row]`,
      ),
    ]
      .map((row) => {
        const key = row.querySelector('[data-mcp-key]')?.value.trim();
        const value = row.querySelector('[data-mcp-value]')?.value.trim() || '';
        return key === undefined ? value : { key, value };
      })
      .filter((value) =>
        typeof value === 'string' ? value : value.key || value.value,
      );
  root
    .querySelector('[data-mcp-add-server]')
    ?.addEventListener('click', () => openMcpDialog());
  root.querySelector('[data-mcp-learn]')?.addEventListener('click', (event) => {
    event.preventDefault();
    notify('Opening MCP documentation');
  });
  root
    .querySelectorAll('[data-mcp-transport-option]')
    .forEach((button) =>
      button.addEventListener('click', () =>
        setMcpTransport(button.dataset.mcpTransportOption),
      ),
    );
  root
    .querySelectorAll('[data-mcp-add-row]')
    .forEach((button) =>
      button.addEventListener('click', () =>
        appendMcpRepeaterRow(button.dataset.mcpAddRow),
      ),
    );
  mcpForm?.addEventListener('click', (event) => {
    const remove = event.target.closest('[data-mcp-remove-row]');
    if (remove) remove.closest('[data-mcp-repeat-row]')?.remove();
  });
  mcpList?.addEventListener('click', (event) => {
    const row = event.target.closest('[data-mcp-row]');
    if (!row) return;
    const server = mcpServers.find((item) => item.id === row.dataset.mcpRow);
    if (!server) return;
    if (event.target.closest('[data-mcp-toggle]')) {
      event.stopPropagation();
      server.enabled = !server.enabled;
      renderMcpServers();
      notify(`${server.name} ${server.enabled ? 'enabled' : 'disabled'}`);
      return;
    }
    if (event.target.closest('[data-mcp-configure]')) openMcpDialog(server);
  });
  mcpForm?.addEventListener('submit', (event) => {
    event.preventDefault();
    const name = mcpForm.elements.name.value.trim();
    const transport = mcpTransport.value;
    const requiredValue =
      transport === 'stdio'
        ? mcpForm.elements.command.value.trim()
        : mcpForm.elements.url.value.trim();
    const duplicate = mcpServers.find(
      (server) =>
        server.id !== editingMcpId &&
        server.name.toLowerCase() === name.toLowerCase(),
    );
    if (!name || !requiredValue) {
      if (mcpStatus)
        mcpStatus.textContent = `Enter a name and ${transport === 'stdio' ? 'command' : 'URL'}.`;
      return;
    }
    if (duplicate) {
      if (mcpStatus) mcpStatus.textContent = 'Server names must be unique.';
      return;
    }
    const server = editingMcpId
      ? mcpServers.find((item) => item.id === editingMcpId)
      : { id: `mcp-${Date.now()}`, enabled: true };
    Object.assign(server, {
      name,
      transport,
      command: mcpForm.elements.command.value.trim(),
      arguments: collectMcpRows('argument'),
      environment: collectMcpRows('environment'),
      passthrough: collectMcpRows('passthrough'),
      workingDirectory: mcpForm.elements.workingDirectory.value.trim(),
      url: mcpForm.elements.url.value.trim(),
      bearerTokenEnv: mcpForm.elements.bearerTokenEnv.value.trim(),
      headers: collectMcpRows('header'),
      environmentHeaders: collectMcpRows('environmentHeader'),
    });
    if (!editingMcpId) mcpServers.push(server);
    renderMcpServers();
    closeDialog();
    notify(`${name} ${editingMcpId ? 'updated' : 'added'}`);
  });
  renderMcpServers();

  root
    .querySelector('[data-config-open-external]')
    ?.addEventListener('click', () => notify('Opening config.toml externally'));

  function escapeHTML(value) {
    return String(value).replace(
      /[&<>'"]/g,
      (character) =>
        ({
          '&': '&amp;',
          '<': '&lt;',
          '>': '&gt;',
          "'": '&#39;',
          '"': '&quot;',
        })[character],
    );
  }

  syncProviderModels();
})();

//advanced policy grouping, filtering, and draft-state interactions
(() => {
  const root = document.querySelector('[data-policy-root]');
  if (!root) return;

  const legacyScenarioGroups = [
    {
      id: 'terminal',
      label: 'Terminal',
      items: [
        [
          'terminal.project.read',
          'Runs a non-mutating shell command whose working directory and readable targets stay within the trusted project.',
        ],
        [
          'terminal.project.write',
          'Runs a command that can create, modify, move, or delete content inside the trusted project.',
        ],
        [
          'terminal.external.read',
          'Runs a command that reads files, metadata, processes, or system state outside the trusted project.',
        ],
        [
          'terminal.external.write',
          'Runs a command that can change files, applications, processes, configuration, or system state outside the trusted project.',
        ],
        [
          'terminal.network.read',
          'Runs a command that retrieves remote information without intentionally changing the remote resource.',
        ],
        [
          'terminal.network.write',
          'Runs a command that sends, publishes, uploads, deploys, or otherwise mutates a remote resource.',
        ],
        [
          'terminal.unknown',
          'The command cannot be classified confidently, combines ambiguous shell constructs, or has dynamically resolved targets.',
        ],
      ],
    },
    {
      id: 'git',
      label: 'Git',
      items: [
        [
          'git.project.read',
          'Reads repository state inside a trusted project, such as status, log, diff, show, branches, or blame.',
        ],
        [
          'git.external.read',
          'Reads a local repository outside the trusted project.',
        ],
        [
          'git.remote.read',
          'Reads from a remote Git service without changing remote state.',
        ],
        [
          'git.project.write',
          'Changes repository or worktree state inside the trusted project.',
        ],
        [
          'git.external.write',
          'Changes a local repository or worktree outside the trusted project.',
        ],
        [
          'git.remote.write',
          'Changes remote Git state, including pushes, remote deletions, tags, or repository mutation.',
        ],
        [
          'git.unknown',
          'The Git operation, repository location, or remote effect cannot be determined safely.',
        ],
      ],
    },
    {
      id: 'filesystem',
      label: 'Filesystem',
      items: [
        [
          'file.project.read',
          'Reads or lists files, folders, metadata, links, or search results inside the trusted project.',
        ],
        [
          'file.external.read',
          'Reads or lists filesystem content outside the trusted project.',
        ],
        [
          'file.project.write',
          'Creates or modifies files and folders inside the trusted project.',
        ],
        [
          'file.external.write',
          'Creates or modifies files and folders outside the trusted project.',
        ],
        [
          'file.project.delete',
          'Deletes or trashes content inside the trusted project.',
        ],
        [
          'file.external.delete',
          'Deletes or trashes content outside the trusted project.',
        ],
        [
          'file.project.watch',
          'Watches trusted-project paths for filesystem changes.',
        ],
        ['file.external.watch', 'Watches paths outside the trusted project.'],
        [
          'file.unknown',
          'The resolved path, scope, link destination, or resulting filesystem effect is unknown.',
        ],
      ],
    },
    {
      id: 'browser',
      label: 'Browser',
      items: [
        [
          'browser.load',
          'Loads a public URL or permitted local document using a non-mutating navigation request.',
        ],
        [
          'browser.post',
          'Sends form data or another request capable of changing remote state.',
        ],
        [
          'browser.navigation',
          'Changes browser location or history through navigation, redirects, or link activation.',
        ],
        [
          'browser.use',
          'Interacts with rendered page content, including clicking, typing, selecting, scrolling, or submitting forms.',
        ],
        [
          'browser.read',
          'Reads page text, DOM state, accessibility state, metadata, links, or visible content.',
        ],
        [
          'browser.capture',
          'Captures a screenshot, selected region, page representation, or annotation attachment.',
        ],
        [
          'browser.download',
          'Downloads remote content into the local filesystem.',
        ],
        [
          'browser.upload',
          'Sends a local file or generated artifact to a website.',
        ],
        [
          'browser.local.read',
          'Opens or reads a local file through the Browser surface.',
        ],
        [
          'browser.auth.use',
          'Uses an existing authenticated browser profile or logged-in session.',
        ],
        [
          'browser.unknown',
          'The navigation, interaction, target, authentication impact, or remote mutation cannot be classified safely.',
        ],
      ],
    },
    {
      id: 'network',
      label: 'Network',
      items: [
        [
          'network.read',
          'Retrieves remote data without intending to mutate the remote system.',
        ],
        [
          'network.write',
          'Creates, updates, submits, publishes, or deletes remote data.',
        ],
        ['network.listen', 'Opens a local listening port or server.'],
        [
          'network.connect',
          'Establishes a socket or service connection not adequately described as an ordinary read.',
        ],
        [
          'network.unknown',
          'The destination or remote effect cannot be determined.',
        ],
      ],
    },
    {
      id: 'credentials',
      label: 'Credentials',
      items: [
        [
          'credential.use',
          'Uses a stored secret through a governed operation without exposing its raw value.',
        ],
        [
          'credential.create',
          'Adds or imports a new credential into secure storage.',
        ],
        ['credential.update', 'Replaces or changes a stored credential.'],
        ['credential.delete', 'Removes a credential from secure storage.'],
        [
          'credential.reveal',
          'Exposes raw secret material to a caller or user.',
        ],
        [
          'credential.unknown',
          'The secret, consumer, destination, or manner of use cannot be determined.',
        ],
      ],
    },
    {
      id: 'processes',
      label: 'Processes & apps',
      items: [
        ['process.read', 'Inspects running processes or application state.'],
        [
          'process.start',
          'Starts a process, service, executable, or desktop application.',
        ],
        [
          'process.control',
          'Sends input, signals, focus changes, or other control operations to a running process.',
        ],
        ['process.stop', 'Terminates a process or service.'],
        [
          'process.unknown',
          'The executable, ownership, or resulting effect cannot be determined.',
        ],
      ],
    },
    {
      id: 'desktop',
      label: 'Desktop facilities',
      items: [
        ['clipboard.read', 'Reads current clipboard contents.'],
        ['clipboard.write', 'Replaces or adds clipboard contents.'],
        ['dialog.open', 'Shows a native open-file or folder-selection dialog.'],
        ['dialog.save', 'Shows a native save-location dialog.'],
        ['notification.show', 'Displays a desktop notification.'],
        [
          'system.read',
          'Reads OS, hardware, display, environment, or application information.',
        ],
        ['system.write', 'Changes system or application-level configuration.'],
        [
          'app.open',
          'Opens a file, URL, or resource in another desktop application.',
        ],
        [
          'app.control',
          'Interacts with or controls another desktop application.',
        ],
      ],
    },
    {
      id: 'c4os',
      label: 'C4OS authority',
      items: [
        ['config.read', 'Reads non-sensitive C4OS configuration.'],
        ['config.write', 'Changes C4OS configuration or policy.'],
        [
          'plugin.read',
          'Reads installed plugin metadata, status, or declared capabilities.',
        ],
        [
          'plugin.write',
          'Installs, removes, enables, disables, repairs, or reconfigures a plugin.',
        ],
        ['extension.use', 'Activates extension-provided runtime behavior.'],
        [
          'mcp.read',
          'Lists or reads MCP tools, resources, prompts, or server metadata.',
        ],
        ['mcp.use', 'Invokes a non-mutating MCP operation.'],
        [
          'mcp.write',
          'Invokes an MCP operation that can mutate external state.',
        ],
        ['artifact.read', 'Reads or previews an existing C4OS artifact.'],
        ['artifact.write', 'Creates or modifies an artifact record.'],
        [
          'share.export',
          'Exports, publishes, shares, or sends local content outside C4OS.',
        ],
        ['unknown', 'No reliable authority classification is available.'],
      ],
    },
  ];

  const groups = [
    {
      id: 'workspace-files',
      label: 'Workspace files',
      items: [
        ['workspace.read', 'Read files and folders in the trusted workspace.'],
        ['workspace.modify', 'Create or modify content in the trusted workspace.'],
        ['workspace.delete', 'Delete or trash content in the trusted workspace.'],
        ['workspace.outside', 'Access files or folders outside the trusted workspace.'],
      ],
    },
    {
      id: 'commands-processes',
      label: 'Commands and processes',
      items: [
        ['command.inspect', 'Inspect processes, shell state, and command output.'],
        ['command.workspace', 'Execute commands inside the trusted workspace.'],
        ['command.system', 'Execute commands outside the workspace or against system state.'],
        ['process.control', 'Start, stop, signal, or control a running process.'],
      ],
    },
    {
      id: 'version-control',
      label: 'Version control',
      items: [
        ['git.local.read', 'Read local repository status, history, and differences.'],
        ['git.local.change', 'Change branches, commits, tags, or worktree state locally.'],
        ['git.remote.read', 'Fetch or inspect remote repository information.'],
        ['git.remote.publish', 'Push, publish, delete, or otherwise mutate remote state.'],
      ],
    },
    {
      id: 'network-sharing',
      label: 'Network and sharing',
      items: [
        ['network.retrieve', 'Retrieve remote information without changing it.'],
        ['network.publish', 'Submit, publish, or mutate remote information.'],
        ['network.listen', 'Open a local port or listening service.'],
        ['network.upload', 'Upload or export local data to a named destination.'],
      ],
    },
    {
      id: 'browser-desktop',
      label: 'Browser and desktop',
      items: [
        ['browser.view', 'View, read, or capture browser content.'],
        ['browser.interact', 'Click, type, select, navigate, or submit in a webpage.'],
        ['browser.authenticated', 'Use an existing authenticated browser session.'],
        ['desktop.control', 'Operate another desktop application.'],
      ],
    },
    {
      id: 'credentials',
      label: 'Credentials',
      items: [
        ['credential.use', 'Use a stored credential without revealing its raw value.'],
        ['credential.add', 'Add, change, or remove a credential in secure storage.'],
        ['credential.reveal', 'Reveal or copy a raw credential value.'],
      ],
    },
    {
      id: 'extensions-c4os',
      label: 'Extensions and C4OS',
      items: [
        ['extension.read', 'Read skill, plugin, MCP, and C4OS metadata.'],
        ['extension.use', 'Use an enabled skill, plugin, app, or MCP tool.'],
        ['extension.configure', 'Install, enable, remove, or configure an extension.'],
        ['c4os.policy', 'Change C4OS policy, authority, or managed configuration.'],
        ['artifact.export', 'Export or share an artifact outside C4OS.'],
      ],
    },
  ];

  const options = [
    ['default', 'Use default'],
    ['allow', 'Allow'],
    ['ask', 'Ask'],
    ['deny', 'Deny'],
  ];
  const allItems = groups.flatMap((group) =>
    group.items.map(([key, description]) => ({
      key,
      description,
      groupId: group.id,
      groupLabel: group.label,
    })),
  );
  let activeGroup = groups[0].id;
  let savedPolicies = Object.fromEntries(
    allItems.map((item) => [item.key, 'default']),
  );
  let draftPolicies = { ...savedPolicies };
  let notifyTimer;

  const groupRoot = root.querySelector('[data-policy-groups]');
  const listRoot = root.querySelector('[data-policy-list]');
  const search = root.querySelector('[data-policy-search]');
  const title = root.querySelector('[data-policy-title]');
  const summary = root.querySelector('[data-policy-summary]');
  const count = root.querySelector('[data-policy-count]');
  const empty = root.querySelector('[data-policy-empty]');
  const save = root.querySelector('[data-policy-save]');
  const notifier = root.querySelector('[data-notifier]');
  const categoryView = root.querySelector('[data-policy-category-view]');
  const exceptionView = root.querySelector('[data-policy-exception-view]');
  const searchWrap = root.querySelector('[data-policy-search-wrap]');

  const escapeHTML = (value) =>
    String(value).replace(
      /[&<>'"]/g,
      (character) =>
        ({
          '&': '&amp;',
          '<': '&lt;',
          '>': '&gt;',
          "'": '&#39;',
          '"': '&quot;',
        })[character],
    );
  const isDirty = () =>
    allItems.some(
      (item) => draftPolicies[item.key] !== savedPolicies[item.key],
    );
  const notify = (message) => {
    if (!notifier) return;
    notifier.textContent = message;
    notifier.hidden = false;
    window.clearTimeout(notifyTimer);
    notifyTimer = window.setTimeout(() => {
      notifier.hidden = true;
    }, 2400);
  };

  const renderGroups = () => {
    groupRoot.innerHTML = groups
      .map(
        (group) =>
          `<button class="policy-group" type="button" data-policy-group="${group.id}" aria-current="${group.id === activeGroup}"><span>${escapeHTML(group.label)}</span><span>${group.items.length}</span></button>`,
      )
      .join('');
  };

  const renderPolicies = () => {
    const query = search.value.trim().toLowerCase();
    const group = groups.find((item) => item.id === activeGroup) || groups[0];
    const items = query
      ? allItems.filter((item) =>
          `${item.key} ${item.description} ${item.groupLabel}`
            .toLowerCase()
            .includes(query),
        )
      : group.items.map(([key, description]) => ({
          key,
          description,
          groupId: group.id,
          groupLabel: group.label,
        }));

    title.textContent = query ? 'Search results' : group.label;
    summary.textContent = query
      ? 'Matching identities across all policy groups.'
      : 'Each override is evaluated against the matching tool call.';
    count.textContent = `${items.length} ${items.length === 1 ? 'policy' : 'policies'}`;
    empty.hidden = items.length > 0;
    listRoot.hidden = items.length === 0;
    listRoot.innerHTML = items
      .map((item) => {
        const dirty = draftPolicies[item.key] !== savedPolicies[item.key];
        const optionMarkup = options
          .map(
            ([value, label]) =>
              `<option value="${value}"${draftPolicies[item.key] === value ? ' selected' : ''}>${label}</option>`,
          )
          .join('');
        return `<article class="policy-row" data-policy-row="${escapeHTML(item.key)}" data-dirty="${dirty}"><div class="policy-row-copy"><code>${escapeHTML(item.key)}</code><p>${escapeHTML(item.description)}</p></div><select aria-label="Policy for ${escapeHTML(item.key)}" data-policy-select="${escapeHTML(item.key)}">${optionMarkup}</select></article>`;
      })
      .join('');
    save.disabled = !isDirty();
  };

  groupRoot.addEventListener('click', (event) => {
    const button = event.target.closest('[data-policy-group]');
    if (!button) return;
    activeGroup = button.dataset.policyGroup;
    search.value = '';
    renderGroups();
    renderPolicies();
  });
  listRoot.addEventListener('change', (event) => {
    const select = event.target.closest('[data-policy-select]');
    if (!select) return;
    draftPolicies[select.dataset.policySelect] = select.value;
    renderPolicies();
  });
  search.addEventListener('input', renderPolicies);
  save.addEventListener('click', () => {
    savedPolicies = { ...draftPolicies };
    renderPolicies();
    notify('Advanced policies saved');
  });
  root.querySelectorAll('[data-policy-view]').forEach((button) => {
    button.addEventListener('click', () => {
      const exceptions = button.dataset.policyView === 'exceptions';
      root.querySelectorAll('[data-policy-view]').forEach((item) =>
        item.setAttribute('aria-selected', String(item === button)),
      );
      if (categoryView) categoryView.hidden = exceptions;
      if (exceptionView) exceptionView.hidden = !exceptions;
      if (searchWrap) searchWrap.hidden = exceptions;
    });
  });
  exceptionView?.addEventListener('click', (event) => {
    const revoke = event.target.closest('[data-exception-revoke]');
    if (!revoke) return;
    revoke.closest('article')?.remove();
    notify('Exception revoked');
  });

  renderGroups();
  renderPolicies();
})();

//provider onboarding and workspace launch interactions
(function () {
  'use strict';

  var launch = document.querySelector('[data-launch-spa]');
  var form = document.querySelector('[data-onboarding-provider-form]');
  if (!launch || !form) return;

  var type = form.elements.providerType;
  var auth = form.elements.auth;
  var status = form.querySelector('[data-onboarding-status]');
  var testButton = form.querySelector('[data-onboarding-test]');
  var defaults = {
    openrouter: {
      label: 'OpenRouter - Personal',
      url: 'https://openrouter.ai/api/v1',
    },
    huggingface: {
      label: 'Hugging Face - Personal',
      url: 'https://router.huggingface.co/v1',
    },
    openai: { label: 'OpenAI - Personal', url: 'https://api.openai.com/v1' },
    compatible: { label: 'Local AI Server', url: 'http://127.0.0.1:4000/v1' },
  };
  var regions = {
    baseUrl: form.querySelector('[data-onboarding-base-url]'),
    auth: form.querySelector('[data-onboarding-auth]'),
    headerName: form.querySelector('[data-onboarding-header-name]'),
    apiKey: form.querySelector('[data-onboarding-api-key]'),
    headers: form.querySelector('[data-onboarding-headers]'),
  };

  /** Returns whether a conditional onboarding region is available. */
  function isVisible(element) {
    return element && !element.hidden;
  }

  /** Synchronizes provider-specific fields and their validation rules. */
  function syncFields(useDefaults) {
    var compatible = type.value === 'compatible';
    if (useDefaults) {
      form.elements.label.value = defaults[type.value].label;
      form.elements.baseUrl.value = defaults[type.value].url;
    }
    regions.baseUrl.hidden = !compatible;
    regions.auth.hidden = !compatible;
    regions.headers.hidden = !compatible;
    regions.headerName.hidden = !compatible || auth.value !== 'header';
    regions.apiKey.hidden = compatible && auth.value === 'none';
    form.elements.apiKey.required = isVisible(regions.apiKey);
    form.elements.baseUrl.required = compatible;
    form.elements.headerName.required = isVisible(regions.headerName);
    status.textContent = '';
    status.removeAttribute('data-state');
  }

  /** Validates the visible fields required by the selected provider. */
  function validateForm() {
    var valid = true;
    Array.prototype.forEach.call(
      form.querySelectorAll('input[required]'),
      function (input) {
        var field = input.closest('label, [data-onboarding-field]');
        var invalid = (!field || !field.hidden) && !input.value.trim();
        input.setAttribute('aria-invalid', String(invalid));
        if (invalid) valid = false;
      },
    );
    if (!valid) {
      status.textContent = 'Complete the required connection fields.';
      status.removeAttribute('data-state');
    }
    return valid;
  }

  type.addEventListener('change', function () {
    syncFields(true);
  });
  auth.addEventListener('change', function () {
    syncFields(false);
  });
  form
    .querySelector('[data-onboarding-password-toggle]')
    .addEventListener('click', function (event) {
      var input = form.elements.apiKey;
      var reveal = input.type === 'password';
      input.type = reveal ? 'text' : 'password';
      event.currentTarget.setAttribute(
        'aria-label',
        reveal ? 'Hide API key' : 'Show API key',
      );
    });
  testButton.addEventListener('click', function () {
    if (!validateForm()) return;
    testButton.disabled = true;
    status.textContent = 'Testing connection…';
    window.setTimeout(function () {
      testButton.disabled = false;
      status.textContent = 'Connection successful. Models are available.';
      status.dataset.state = 'success';
    }, 650);
  });
  form.addEventListener('submit', function (event) {
    event.preventDefault();
    if (!validateForm()) return;
    try {
      window.localStorage.setItem('c4os-provider-configured', 'true');
    } catch (error) {
      /* Static preview may restrict storage. */
    }
    window.location.hash = '#start';
  });
  form.addEventListener('input', function (event) {
    if (event.target.matches('input'))
      event.target.removeAttribute('aria-invalid');
  });

  launch.querySelectorAll('[data-workspace-action]').forEach(function (link) {
    link.addEventListener('click', function (event) {
      event.preventDefault();
      var startStatus = launch.querySelector('[data-workspace-start-status]');
      startStatus.textContent = link.dataset.workspaceAction + '…';
      window.setTimeout(function () {
        window.location.hash = '#chat';
      }, 380);
    });
  });

  syncFields(false);
})();

//workspace project, session, and project-menu interactions
(function () {
  'use strict';

  var root = document.querySelector('[data-workspace-projects]');
  if (!root) return;

  var list = root.querySelector('[data-project-list]');
  var search = root.querySelector('[data-session-search]');
  var addButton = root.querySelector('[data-project-add]');
  var folderInput = root.querySelector('[data-project-folder-input]');
  var status = root.querySelector('[data-project-status]');
  var composer = document.querySelector('[data-composer]');
  var thread = document.querySelector('[data-thread]');
  var emptyThread = document.querySelector('[data-new-thread-empty]');
  var emptyThreadHeading = document.querySelector('[data-new-thread-heading]');
  var shell = document.querySelector('[data-panel-shell]');
  var relocateProject = null;
  var draggedProject = null;
  var statusTimer = null;
  var pendingProject = null;
  var activeSession = list.querySelector('.session-item--active');
  var threadSnapshots = {};

  if (activeSession && thread)
    threadSnapshots[activeSession.dataset.sessionId] = thread.innerHTML;

  /** Escapes user-facing project text before building markup. */
  function escapeMarkup(value) {
    return String(value).replace(/[&<>"']/g, function (character) {
      return {
        '&': '&amp;',
        '<': '&lt;',
        '>': '&gt;',
        '"': '&quot;',
        "'": '&#39;',
      }[character];
    });
  }

  /** Returns the current visible name for a project. */
  function projectName(project) {
    var label = project.querySelector('[data-project-toggle] span');
    return label ? label.textContent.trim() : 'Project';
  }

  /** Keeps project action names synchronized after rename or relocation. */
  function syncProjectLabels(project) {
    var name = projectName(project);
    var missing = project.dataset.projectFound === 'false';
    var menuTrigger = project.querySelector('[data-project-menu-trigger]');
    var newChat = project.querySelector('[data-project-new-chat]');
    menuTrigger.setAttribute(
      'aria-label',
      'More actions for ' + (missing ? 'missing ' + name + ' project' : name),
    );
    newChat.setAttribute('aria-label', 'New chat in ' + name);
  }

  /** Shows a temporary project-navigation status message. */
  function notify(message) {
    window.clearTimeout(statusTimer);
    status.textContent = message;
    statusTimer = window.setTimeout(function () {
      status.textContent = '';
    }, 2600);
  }

  /** Closes every project menu except the optional active project. */
  function closeMenus(exceptProject) {
    list.querySelectorAll('[data-project-menu]').forEach(function (menu) {
      var project = menu.closest('[data-project-id]');
      if (project === exceptProject) return;
      menu.hidden = true;
      project.removeAttribute('data-menu-open');
      var trigger = project.querySelector('[data-project-menu-trigger]');
      if (trigger) trigger.setAttribute('aria-expanded', 'false');
    });
  }

  /** Applies one project's expanded or collapsed state. */
  function setProjectExpanded(project, expanded) {
    var sessions = project.querySelector('[data-project-sessions]');
    var toggle = project.querySelector('[data-project-toggle]');
    project.dataset.projectExpanded = String(expanded);
    toggle.setAttribute('aria-expanded', String(expanded));
    sessions.hidden = !expanded;
  }

  /** Saves the active session transcript before navigation. */
  function saveActiveThread() {
    if (activeSession && thread && !pendingProject)
      threadSnapshots[activeSession.dataset.sessionId] = thread.innerHTML;
  }

  /** Restores the transcript associated with a session. */
  function restoreSavedThread(session) {
    if (!thread) return;
    var snapshot = threadSnapshots[session.dataset.sessionId];
    if (snapshot) thread.innerHTML = snapshot;
    else if (Object.keys(threadSnapshots).length)
      thread.innerHTML = threadSnapshots[Object.keys(threadSnapshots)[0]];
    else thread.innerHTML = '';
  }

  /** Activates a session and optionally restores its transcript. */
  function setActiveSession(session, restoreThread) {
    saveActiveThread();
    pendingProject = null;
    if (emptyThread) emptyThread.hidden = true;
    if (thread) thread.hidden = false;
    list.querySelectorAll('.session-item--active').forEach(function (item) {
      item.classList.remove('session-item--active');
    });
    session.classList.add('session-item--active');
    activeSession = session;
    if (restoreThread !== false) restoreSavedThread(session);
    var title = session.dataset.sessionTitle || session.textContent.trim();
    var threadTitle = document.querySelector('.thread-title strong');
    if (threadTitle) threadTitle.textContent = title;
    notify('Opened “' + title + '”');
  }

  /** Shows a stable empty workspace when no saved session remains. */
  function showNoActiveSession() {
    pendingProject = null;
    activeSession = null;
    list.querySelectorAll('.session-item--active').forEach(function (item) {
      item.classList.remove('session-item--active');
    });
    if (thread) thread.hidden = true;
    if (emptyThread) emptyThread.hidden = false;
    if (emptyThreadHeading)
      emptyThreadHeading.textContent = 'Choose a project to start a chat';
    var title = document.querySelector('.thread-title strong');
    if (title) title.textContent = 'No chat selected';
  }

  /** Activates the first remaining session or the empty workspace fallback. */
  function activateFallbackSession() {
    var fallback = list.querySelector('[data-session-id]');
    if (fallback) setActiveSession(fallback);
    else showNoActiveSession();
  }

  /** Opens the empty composer state for a new project chat. */
  function beginNewThread(project) {
    saveActiveThread();
    pendingProject = project;
    list.querySelectorAll('.session-item--active').forEach(function (item) {
      item.classList.remove('session-item--active');
    });
    if (shell) shell.dispatchEvent(new CustomEvent('workspace:new-thread'));
    if (thread) thread.hidden = true;
    if (emptyThread) emptyThread.hidden = false;
    if (emptyThreadHeading)
      emptyThreadHeading.textContent =
        'What do you want to build in ' + projectName(project) + '?';
    var title = document.querySelector('.thread-title strong');
    if (title) title.textContent = 'New chat';
    var input = document.querySelector('[data-mode-input="chat"]');
    if (input) input.focus();
    notify('New chat in “' + projectName(project) + '”');
  }

  /** Creates a compact session title from the initial prompt. */
  function promptTitle(value) {
    var title = value.replace(/\s+/g, ' ').trim();
    if (title.length > 48) title = title.slice(0, 47).trimEnd() + '…';
    return title || 'New chat';
  }

  /** Builds one session row for the project navigation. */
  function sessionMarkup(id, title) {
    id = escapeMarkup(id);
    title = escapeMarkup(title);
    return (
      '<div class="session-item" data-session-id="' +
      id +
      '" data-session-title="' +
      title +
      '"><button type="button" data-session-open>' +
      title +
      '</button><button type="button" data-session-remove aria-label="Remove ' +
      title +
      '" title="Remove"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m18 6-12 12M6 6l12 12"></path></svg></button></div>'
    );
  }

  /** Builds one project component with its first session. */
  function projectMarkup(id, name, path) {
    id = escapeMarkup(id);
    name = escapeMarkup(name);
    path = escapeMarkup(path);
    return (
      '<article class="project-item" data-project-id="' +
      id +
      '" data-project-path="' +
      path +
      '" data-project-found="true" data-project-expanded="true" draggable="true">' +
      '<div class="project-item__row"><button class="project-item__toggle" type="button" data-project-toggle aria-expanded="true"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M3 7h6l2 2h10v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z"></path></svg><span>' +
      name +
      '</span></button>' +
      '<div class="project-item__actions"><button type="button" data-project-menu-trigger aria-haspopup="menu" aria-expanded="false" aria-label="More actions for ' +
      name +
      '" title="More"><svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="5" cy="12" r="1"></circle><circle cx="12" cy="12" r="1"></circle><circle cx="19" cy="12" r="1"></circle></svg></button><button type="button" data-project-new-chat aria-label="New chat in ' +
      name +
      '" title="New chat"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 20h9"></path><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L8 18l-4 1 1-4Z"></path></svg></button></div>' +
      '<div class="project-item__menu" data-project-menu role="menu" hidden><button type="button" role="menuitem" data-project-action="reveal">Reveal</button><button type="button" role="menuitem" data-project-action="copy">Copy path</button><button type="button" role="menuitem" data-project-action="rename">Rename</button><span></span><button class="project-item__menu-danger" type="button" role="menuitem" data-project-action="remove">Remove</button></div></div>' +
      '<div class="project-item__sessions" data-project-sessions>' +
      sessionMarkup('session-' + id + '-welcome', 'New chat') +
      '</div></article>'
    );
  }

  /** Adds or relocates a project from a selected folder. */
  function addProjectFromSelection(files) {
    if (!files || !files.length) return;
    var relative = files[0].webkitRelativePath || files[0].name;
    var name = relative.split('/')[0] || 'New project';
    var path = '~/' + name;

    if (relocateProject && relocateProject.isConnected) {
      var label = relocateProject.querySelector('[data-project-toggle] span');
      var missing = relocateProject.querySelector(
        '[data-project-toggle] small',
      );
      label.textContent = name;
      if (missing) missing.remove();
      relocateProject.dataset.projectPath = path;
      relocateProject.dataset.projectFound = 'true';
      relocateProject.classList.remove('project-item--missing');
      var menu = relocateProject.querySelector('[data-project-menu]');
      var relocate = menu.querySelector('[data-project-action="relocate"]');
      if (relocate) {
        relocate.dataset.projectAction = 'reveal';
        relocate.textContent = 'Reveal';
      }
      syncProjectLabels(relocateProject);
      notify('Relocated “' + name + '”');
      relocateProject = null;
      return;
    }

    var id =
      name
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, '-')
        .replace(/^-|-$/g, '') || 'project';
    id += '-' + Date.now().toString().slice(-5);
    list.insertAdjacentHTML('beforeend', projectMarkup(id, name, path));
    var project = list.lastElementChild;
    setActiveSession(project.querySelector('[data-session-id]'));
    notify('Added “' + name + '”');
  }

  /** Replaces the project name with an inline rename control. */
  function renameProject(project) {
    var toggle = project.querySelector('[data-project-toggle]');
    var row = project.querySelector('.project-item__row');
    var label = toggle.querySelector('span');
    if (!label || row.querySelector('input')) return;
    var original = label.textContent.trim();
    var input = document.createElement('input');
    var finished = false;
    input.className = 'project-item__rename-input';
    input.value = original;
    input.setAttribute('aria-label', 'Rename ' + original);
    toggle.hidden = true;
    row.insertBefore(input, row.querySelector('.project-item__actions'));
    input.focus();
    input.select();

    /** Finishes the inline rename and optionally saves the value. */
    function finish(save) {
      if (finished) return;
      finished = true;
      var next = input.value.trim();
      if (save && next) {
        label.textContent = next;
        syncProjectLabels(project);
      }
      toggle.hidden = false;
      input.remove();
      notify(
        save && next ? 'Renamed project to “' + next + '”' : 'Rename cancelled',
      );
    }

    input.addEventListener('click', function (event) {
      event.stopPropagation();
    });
    input.addEventListener('keydown', function (event) {
      event.stopPropagation();
      if (event.key === 'Enter') {
        event.preventDefault();
        finish(true);
      }
      if (event.key === 'Escape') {
        event.preventDefault();
        finish(false);
      }
    });
    input.addEventListener('blur', function () {
      finish(true);
    });
  }

  /** Copies a project path and reports the result. */
  function copyPath(project) {
    var path = project.dataset.projectPath || '';
    if (shell)
      shell.dispatchEvent(
        new CustomEvent('workspace:copy', { detail: { text: path } }),
      );
  }

  /** Removes a project and reconciles any active or pending workspace state. */
  /** Removes a project after its confirmation step. */
  function removeProject(project) {
    var removedName = projectName(project);
    var removedActive = Boolean(
      activeSession && project.contains(activeSession),
    );
    var removedPending = pendingProject === project;
    project.querySelectorAll('[data-session-id]').forEach(function (session) {
      delete threadSnapshots[session.dataset.sessionId];
    });
    project.remove();
    if (removedActive) activeSession = null;
    if (removedPending) pendingProject = null;
    if (removedPending || (removedActive && !pendingProject))
      activateFallbackSession();
    notify('Removed “' + removedName + '”');
  }

  /** Removes one session and selects a valid fallback when it was active. */
  /** Removes one saved session after its confirmation step. */
  function removeSession(session) {
    var wasActive =
      activeSession === session ||
      session.classList.contains('session-item--active');
    var title = session.dataset.sessionTitle;
    delete threadSnapshots[session.dataset.sessionId];
    session.remove();
    if (wasActive) {
      activeSession = null;
      if (!pendingProject) activateFallbackSession();
    }
    notify('Removed “' + title + '”');
  }

  /** Filters projects and sessions using the current search term. */
  function filterSessions() {
    var query = search.value.trim().toLowerCase();
    list.querySelectorAll('[data-project-id]').forEach(function (project) {
      var matches = 0;
      project.querySelectorAll('[data-session-id]').forEach(function (session) {
        var match =
          !query ||
          (session.dataset.sessionTitle || '').toLowerCase().includes(query);
        session.hidden = !match;
        if (match) matches += 1;
      });
      project.hidden = Boolean(query && !matches);
      var sessions = project.querySelector('[data-project-sessions]');
      sessions.hidden = query
        ? !matches
        : project.dataset.projectExpanded !== 'true';
    });
  }

  addButton.addEventListener('click', function () {
    relocateProject = null;
    folderInput.value = '';
    folderInput.click();
  });
  folderInput.addEventListener('change', function () {
    addProjectFromSelection(folderInput.files);
  });
  search.addEventListener('input', filterSessions);
  if (composer)
    composer.addEventListener(
      'submit',
      function () {
        if (!pendingProject || composer.dataset.mode !== 'chat') return;
        var input = document.querySelector('[data-mode-input="chat"]');
        var value =
          input && window.MarkdownRuntime
            ? window.MarkdownRuntime.fromElement(input).trim()
            : '';
        var hasAttachments = Number(composer.dataset.attachmentCount || 0) > 0;
        if (!value && !hasAttachments) return;

        var project = pendingProject;
        var title = promptTitle(
          value || composer.dataset.firstAttachmentName || 'New chat',
        );
        var sessions = project.querySelector('[data-project-sessions]');
        var id = 'session-new-' + Date.now();
        if (thread) {
          thread.innerHTML = '';
          thread.hidden = false;
        }
        if (emptyThread) emptyThread.hidden = true;
        sessions.insertAdjacentHTML('afterbegin', sessionMarkup(id, title));
        setProjectExpanded(project, true);
        pendingProject = null;
        activeSession = null;
        setActiveSession(sessions.firstElementChild, false);
      },
      true,
    );

  list.addEventListener('click', function (event) {
    var project = event.target.closest('[data-project-id]');
    if (!project) return;
    var menuTrigger = event.target.closest('[data-project-menu-trigger]');
    var action = event.target.closest('[data-project-action]');
    var newChat = event.target.closest('[data-project-new-chat]');
    var toggle = event.target.closest('[data-project-toggle]');
    var removeSessionButton = event.target.closest('[data-session-remove]');
    var openSession = event.target.closest('[data-session-open]');

    if (menuTrigger) {
      var menu = project.querySelector('[data-project-menu]');
      var willOpen = menu.hidden;
      closeMenus(project);
      menu.hidden = !willOpen;
      project.toggleAttribute('data-menu-open', willOpen);
      menuTrigger.setAttribute('aria-expanded', String(willOpen));
      if (willOpen) menu.querySelector('button').focus();
      return;
    }
    if (action) {
      var kind = action.dataset.projectAction;
      closeMenus();
      if (kind === 'reveal')
        notify('Revealed “' + projectName(project) + '” in Finder');
      if (kind === 'copy') copyPath(project);
      if (kind === 'rename') renameProject(project);
      if (kind === 'remove') removeProject(project);
      if (kind === 'relocate') {
        relocateProject = project;
        folderInput.value = '';
        folderInput.click();
      }
      return;
    }
    if (newChat) {
      beginNewThread(project);
      return;
    }
    if (removeSessionButton) {
      var session = removeSessionButton.closest('[data-session-id]');
      removeSession(session);
      return;
    }
    if (openSession) {
      setActiveSession(openSession.closest('[data-session-id]'));
      return;
    }
    if (toggle && !toggle.querySelector('input'))
      setProjectExpanded(project, project.dataset.projectExpanded !== 'true');
  });

  list.addEventListener('dragstart', function (event) {
    var project = event.target.closest('[data-project-id]');
    if (!project) return;
    draggedProject = project;
    project.dataset.dragging = 'true';
    event.dataTransfer.effectAllowed = 'move';
    event.dataTransfer.setData('text/plain', project.dataset.projectId);
  });
  list.addEventListener('dragover', function (event) {
    if (!draggedProject) return;
    event.preventDefault();
    var target = event.target.closest('[data-project-id]');
    list.querySelectorAll('[data-drag-over]').forEach(function (item) {
      item.removeAttribute('data-drag-over');
    });
    if (!target || target === draggedProject) return;
    target.dataset.dragOver = 'true';
    var before =
      event.clientY <
      target.getBoundingClientRect().top + target.offsetHeight / 2;
    list.insertBefore(draggedProject, before ? target : target.nextSibling);
  });
  list.addEventListener('drop', function (event) {
    if (!draggedProject) return;
    event.preventDefault();
    notify('Project order updated');
  });
  list.addEventListener('dragend', function () {
    if (draggedProject) draggedProject.removeAttribute('data-dragging');
    list.querySelectorAll('[data-drag-over]').forEach(function (item) {
      item.removeAttribute('data-drag-over');
    });
    draggedProject = null;
  });

  document.addEventListener('click', function (event) {
    if (!root.contains(event.target)) closeMenus();
    else if (
      !event.target.closest('[data-project-menu]') &&
      !event.target.closest('[data-project-menu-trigger]')
    )
      closeMenus();
  });
  document.addEventListener('keydown', function (event) {
    if (event.key === 'Escape') closeMenus();
  });
})();

//chat metadata disclosure
(function () {
  'use strict';

  var trigger = document.querySelector('[data-chat-meta-trigger]');
  var popover = document.querySelector('[data-chat-meta-popover]');
  if (!trigger || !popover) return;

  function closeChatMeta(restoreFocus) {
    popover.hidden = true;
    trigger.setAttribute('aria-expanded', 'false');
    if (restoreFocus) trigger.focus();
  }

  trigger.addEventListener('click', function (event) {
    event.stopPropagation();
    var shouldOpen = popover.hidden;
    popover.hidden = !shouldOpen;
    trigger.setAttribute('aria-expanded', String(shouldOpen));
  });

  popover.addEventListener('click', function (event) {
    event.stopPropagation();
  });

  document.addEventListener('click', function () {
    closeChatMeta(false);
  });

  document.addEventListener('keydown', function (event) {
    if (event.key === 'Escape' && !popover.hidden) {
      event.preventDefault();
      closeChatMeta(true);
    }
  });
})();

//top-level routing between launch, workspace, and settings surfaces
(function () {
  'use strict';

  var workspace = document.querySelector('[data-panel-shell]');
  var settings = document.querySelector('[data-settings-spa]');
  var launch = document.querySelector('[data-launch-spa]');
  var launchViews = launch
    ? Array.prototype.slice.call(launch.querySelectorAll('[data-launch-view]'))
    : [];
  if (!workspace || !settings || !launch) return;

  /** Returns whether onboarding has stored a provider configuration. */
  function hasConfiguredProvider() {
    try {
      return window.localStorage.getItem('c4os-provider-configured') === 'true';
    } catch (error) {
      return false;
    }
  }

  /** Synchronizes the visible application surface with the URL hash. */
  function syncAppRoute() {
    if (!window.location.hash)
      window.location.hash = hasConfiguredProvider() ? '#start' : '#onboarding';
    var route = window.location.hash.slice(1);
    var settingsActive = /^settings\//.test(route);
    var launchActive = route === 'onboarding' || route === 'start';
    workspace.hidden = settingsActive || launchActive;
    settings.hidden = !settingsActive;
    launch.hidden = !launchActive;
    launchViews.forEach(function (view) {
      view.hidden = view.dataset.launchView !== route;
    });
    document.body.dataset.appView = settingsActive
      ? 'settings'
      : launchActive
        ? route
        : 'workspace';
  }

  window.addEventListener('hashchange', syncAppRoute);
  syncAppRoute();
})();
