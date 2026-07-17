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
  var threadScrollButton = document.querySelector('[data-thread-scroll-bottom]');
  var focusViewer = document.querySelector('[data-focus-viewer]');
  var leftChatPane = document.querySelector('[data-left-chat-pane]');
  var leftChatPaneThread = document.querySelector('[data-left-chat-pane-thread]');
  var leftChatPaneResize = document.querySelector('[data-left-chat-pane-resize]');
  var fileSourcePopover = document.querySelector('[data-file-source-popover]');
  var attachmentInput = document.querySelector('[data-attachment-input]');
  var attachmentTray = document.querySelector('[data-attachment-tray]');
  var appDropzone = document.querySelector('[data-app-dropzone]');
  var narrowQuery = window.matchMedia('(max-width: 991.98px)');
  var minimumWidth = 180;
  var minimumCenterWidth = 420;
  var minimumChatPaneHeight = 220;
  var currentMode = ['chat', 'files', 'browser', 'terminal'].indexOf(location.hash.slice(1)) > -1 ? location.hash.slice(1) : 'chat';
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
  var reducedMotionQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
  var draftAttachments = [];
  var attachmentUrls = [];
  var attachmentCounter = 0;
  var fileDragDepth = 0;

  var icons = {
    copy: '<svg viewBox="0 0 24 24" aria-hidden="true"><rect x="9" y="9" width="12" height="12" rx="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>',
    reply: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m9 17-5-5 5-5"></path><path d="M20 18v-2a4 4 0 0 0-4-4H4"></path></svg>',
    expand: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7"></path></svg>',
    close: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m18 6-12 12M6 6l12 12"></path></svg>',
    edit: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 20h9"></path><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L8 18l-4 1 1-4Z"></path></svg>',
    save: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m20 6-11 11-5-5"></path></svg>',
    trash: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M3 6h18M8 6V4h8v2M19 6l-1 14H6L5 6M10 10v6M14 10v6"></path></svg>',
    folder: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M3 7h6l2 2h10v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z"></path></svg>',
    file: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"></path><path d="M14 2v6h6M8 13h8M8 17h6"></path></svg>',
    stop: '<svg viewBox="0 0 24 24" aria-hidden="true"><rect x="7" y="7" width="10" height="10" rx="1"></rect></svg>',
    send: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m5 12 7-7 7 7M12 19V5"></path></svg>'
  };

  // Providers declare shell capabilities without owning shared shell markup.
  var artifactProviders = {
    browser: { expandable: true },
    file: { expandable: true },
    folder: { expandable: true },
    terminal: { expandable: true }
  };

  var artifacts = {
    'browser-1': { id: 'browser-1', type: 'browser', title: 'Example Domain', url: 'https://example.com', history: ['https://example.com'], historyIndex: 0, status: 'Ready' },
    'file-1': { id: 'file-1', type: 'file', title: 'notes.md', path: '~/project/docs/notes.md', content: sampleFileContent(), version: 1, status: 'Viewing', proposedContent: '', editing: false },
    'terminal-1': { id: 'terminal-1', type: 'terminal', title: 'npm run dev', sessionId: 'shell-1' }
  };

  var terminalSession = {
    id: 'shell-1',
    cwd: '~/project',
    activeEntryId: 'terminal-1',
    entries: [
      { id: 'terminal-0', command: 'ls', output: 'index.html  notes.md  src  wireframes', status: 'completed', exit: 0 },
      { id: 'terminal-1', command: 'npm run dev', output: '> workspace@1.0.0 dev\n> vite --host 127.0.0.1\n\nLocal: http://127.0.0.1:4179/\nready in 412 ms\nGET / 200 14ms', status: 'running', exit: null, elapsed: 18 }
    ]
  };

  function escapeHtml(value) {
    return String(value).replace(/[&<>'"]/g, function (character) {
      return { '&': '&amp;', '<': '&lt;', '>': '&gt;', "'": '&#39;', '"': '&quot;' }[character];
    });
  }

  /** Returns the Markdown source retained by a rendered message body. */
  function markdownSource(element) {
    if (!element) return '';
    return element.getAttribute('data-markdown-source') || markdown.fromElement(element);
  }

  /** Reads either a native mode input or the source-native Chat editor. */
  function inputValue(input) {
    if (!input) return '';
    if (input.matches('[data-markdown-editor]')) return markdown.fromElement(input);
    return input.value || '';
  }

  /** Clears a native input or Markdown editor through its owning state model. */
  function clearComposerInput(input) {
    if (!input) return;
    if (input.matches('[data-markdown-editor]')) markdown.setEditorValue(input, '');
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
    return '<div class="' + classes.join(' ') + '" data-markdown-source="' +
      escapeHtml(source) + '">' + markdown.render(source) + '</div>';
  }

  /** Produces a compact plain-text excerpt from Markdown source. */
  function markdownPlainText(source) {
    var container = document.createElement('div');
    container.innerHTML = markdown.render(source);
    return container.textContent.trim();
  }

  /** Upgrades seeded message bubbles to the same Markdown contract as new ones. */
  function hydrateMarkdownMessages() {
    thread.querySelectorAll('.message--user .message__body, .message--assistant .message__body')
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
    if (bytes < 1024 * 1024) return Math.max(1, Math.round(bytes / 1024)) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(bytes < 10 * 1024 * 1024 ? 1 : 0) + ' MB';
  }

  /** Resolves a compact uppercase file extension with a MIME fallback. */
  function getAttachmentExtension(attachment) {
    var nameMatch = String(attachment.name || '').match(/\.([^.]+)$/);
    if (nameMatch) return nameMatch[1].toUpperCase().slice(0, 8);
    var mimeSubtype = String(attachment.type || '').split('/')[1] || '';
    return mimeSubtype ? mimeSubtype.split(/[.+-]/)[0].toUpperCase().slice(0, 8) : 'FILE';
  }

  /** Formats the extension and size shared by draft and submitted files. */
  function formatAttachmentMetadata(attachment) {
    return getAttachmentExtension(attachment) + ' · ' + formatFileSize(attachment.size);
  }

  /** Renders the one-based reference shared by draft and submitted files. */
  function attachmentNumberMarkup(index) {
    var number = index + 1;
    return '<span class="attachment-number" aria-label="Attachment ' + number + '" title="Attachment ' + number + '">' + number + '</span>';
  }

  /** Returns attachment/index pairs ordered from newest to oldest. */
  function newestAttachmentEntries(attachments) {
    return (attachments || []).map(function (attachment, index) {
      return { attachment: attachment, index: index };
    }).reverse();
  }

  /** Returns true when a file can be previewed as an image in the composer. */
  function isImageFile(file) {
    return /^image\//.test(file.type) || /\.(avif|bmp|gif|ico|jpe?g|png|svg|webp)$/i.test(file.name);
  }

  /** Renders a draft attachment with a thumbnail or generic file glyph. */
  function draftAttachmentMarkup(attachment, index) {
    var visual = attachment.previewUrl
      ? '<img src="' + escapeHtml(attachment.previewUrl) + '" alt="">'
      : '<span class="composer-attachment__file-icon" aria-hidden="true">' + icons.file + '</span>';
    return '<div class="composer-attachment" data-draft-attachment="' + attachment.id + '">' +
      attachmentNumberMarkup(index) + visual +
      '<span class="composer-attachment__copy"><strong title="' + escapeHtml(attachment.name) + '">' + escapeHtml(attachment.name) + '</strong><small>' + escapeHtml(formatAttachmentMetadata(attachment)) + '</small></span>' +
      '<button type="button" data-remove-attachment="' + attachment.id + '" aria-label="Remove ' + escapeHtml(attachment.name) + '" title="Remove attachment">' + icons.close + '</button></div>';
  }

  /** Keeps the Chat attachment tray synchronized with draft state and mode. */
  function renderDraftAttachments() {
    if (!attachmentTray) return;
    attachmentTray.innerHTML = newestAttachmentEntries(draftAttachments).map(function (entry) {
      return draftAttachmentMarkup(entry.attachment, entry.index);
    }).join('');
    attachmentTray.hidden = currentMode !== 'chat' || draftAttachments.length === 0;
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
        previewUrl: previewUrl
      });
    });
    renderDraftAttachments();
    syncPrimaryAction();
  }

  /** Removes one file from the draft and releases its unused preview URL. */
  function removeDraftAttachment(id) {
    var removed = draftAttachments.find(function (attachment) { return attachment.id === id; });
    draftAttachments = draftAttachments.filter(function (attachment) { return attachment.id !== id; });
    if (removed && removed.previewUrl) {
      URL.revokeObjectURL(removed.previewUrl);
      attachmentUrls = attachmentUrls.filter(function (url) { return url !== removed.previewUrl; });
    }
    renderDraftAttachments();
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

  /** Renders submitted attachment previews inside a user message. */
  function messageAttachmentsMarkup(attachments) {
    if (!attachments || !attachments.length) return '';
    return '<div aria-label="Attached files" class="message-attachments" role="group">' + newestAttachmentEntries(attachments).map(function (entry) {
      var numberBadge = attachmentNumberMarkup(entry.index);
      if (entry.attachment.previewUrl) {
        return '<figure class="message-attachment message-attachment--image">' + numberBadge + '<img src="' + escapeHtml(entry.attachment.previewUrl) + '" alt="Preview of ' + escapeHtml(entry.attachment.name) + '"><figcaption><strong title="' + escapeHtml(entry.attachment.name) + '">' + escapeHtml(entry.attachment.name) + '</strong><small>' + escapeHtml(formatAttachmentMetadata(entry.attachment)) + '</small></figcaption></figure>';
      }
      return '<div class="message-attachment message-attachment--file">' + numberBadge + '<span class="message-attachment__file-icon" aria-hidden="true">' + icons.file + '</span><span><strong title="' + escapeHtml(entry.attachment.name) + '">' + escapeHtml(entry.attachment.name) + '</strong><small>' + escapeHtml(formatAttachmentMetadata(entry.attachment)) + '</small></span></div>';
    }).join('') + '</div>';
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
    var distance = transcript.scrollHeight - transcript.scrollTop - transcript.clientHeight;
    if (distance < 180) transcript.scrollTop = transcript.scrollHeight;
    updateThreadScrollButton();
  }

  /** Updates a typing target as plain text or progressively rendered Markdown. */
  function renderAgentText(element, content) {
    if (element.matches('[data-markdown-source]')) element.innerHTML = markdown.render(content);
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
    var work = thinking && thinking.querySelector('.thinking-process__content p');
    var activity = thinking && thinking.querySelector('.thinking-process__activity span');
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
    var work = thinking && thinking.querySelector('.thinking-process__content p');
    var activity = thinking && thinking.querySelector('.thinking-process__activity span');
    var finalLabel = summary ? summary.getAttribute('data-stream-text') : '';
    await typeAgentText(summary, 'Thinking…');
    await typeAgentText(work, work && work.getAttribute('data-stream-text'));
    await typeAgentText(summary, 'Working…');
    await typeAgentText(activity, activity && activity.getAttribute('data-stream-text'));
    await waitForStream(reducedMotionQuery.matches ? 0 : 120);
    if (summary) summary.textContent = finalLabel;
    if (thinking) thinking.open = false;

    if (node.matches('.message--assistant')) {
      revealAgentStreamPart(node.querySelector('.message-model-row'));
      revealAgentStreamPart(node.querySelector('.message__content'));
      var response = node.querySelector('.message__body');
      await typeAgentText(response, response && response.getAttribute('data-stream-text'));
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
      'Validate long-file scrolling, breadcrumbs, and folder selection.'
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
    return Math.max(minimumWidth, Math.min(width * 0.55, width - minimumCenterWidth));
  }

  function setPanelWidth(side, width) {
    var maximumWidth = maximumPanelWidth();
    var value = Math.round(Math.max(minimumWidth, Math.min(maximumWidth, width)));
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
    setPanelWidth('left', panelFor('left').getBoundingClientRect().width || minimumWidth);
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
      if (moveEvent.buttons === 0) { stopResize(); return; }
      setPanelWidth(side, side === 'left' ? moveEvent.clientX - bounds.left : bounds.right - moveEvent.clientX);
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
    if (event.key === 'Escape' && narrowQuery.matches && !event.target.closest('[data-menu]')) {
      setPanelOpen('left', false);
    }
  });

  narrowQuery.addEventListener('change', normalizeForViewport);
  window.addEventListener('resize', function () {
    setPanelWidth('left', panelFor('left').getBoundingClientRect().width || minimumWidth);
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
      button.setAttribute('aria-checked', String(button.getAttribute('data-mode-option') === mode));
    });
    var selectedOption = document.querySelector('[data-mode-option="' + mode + '"]');
    document.querySelector('[data-mode-trigger-label]').textContent = selectedOption.querySelector('strong').textContent;
    document.querySelector('[data-mode-trigger-icon]').innerHTML = selectedOption.querySelector('.mode-option__icon').innerHTML;
    closeModePopover(false);
    document.querySelectorAll('[data-mode-panel]').forEach(function (panel) {
      panel.hidden = panel.getAttribute('data-mode-panel') !== mode;
    });
    var labels = { chat: 'Send message', files: fileSelectionKind === 'folder' ? 'Open folder' : 'Open file', browser: 'Open webpage', terminal: activeTerminalEntry() ? 'Send process input' : 'Run command' };
    primaryAction.setAttribute('aria-label', labels[mode]);
    primaryAction.title = labels[mode];
    if (updateHash) history.replaceState(null, '', '#' + mode);
    syncTerminalComposer();
    renderDraftAttachments();
    syncPrimaryAction();
  }

  function syncPrimaryAction() {
    var input = replyTarget ? replyInput : inputForMode(currentMode);
    var hasChatAttachments = currentMode === 'chat' && draftAttachments.length > 0;
    primaryAction.disabled = !input || (inputValue(input).trim().length === 0 && !hasChatAttachments);
    if (input && input.tagName === 'TEXTAREA') {
      input.style.height = 'auto';
      input.style.height = Math.min(input.scrollHeight, 150) + 'px';
    }
    window.requestAnimationFrame(syncComposerDockHeight);
  }

  function syncComposerDockHeight() {
    if (!composerDock) return;
    shell.style.setProperty('--composer-dock-height', Math.ceil(composerDock.getBoundingClientRect().height) + 'px');
    updateThreadScrollButton();
  }

  function startReply(button) {
    replyTarget = {
      kind: button.getAttribute('data-target-kind'),
      id: button.getAttribute('data-target-id'),
      title: button.getAttribute('data-target-title'),
      excerpt: button.getAttribute('data-target-excerpt')
    };
    modeControl.hidden = true;
    closeModePopover(false);
    document.querySelectorAll('[data-mode-panel]').forEach(function (panel) { panel.hidden = true; });
    replyStrip.hidden = false;
    replyPanel.hidden = false;
    document.querySelector('[data-reply-title]').textContent = 'Replying to ' + replyTarget.title;
    document.querySelector('[data-reply-excerpt]').textContent = replyTarget.excerpt;
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
    return '<div class="' + (kind === 'message' ? 'message-actions' : 'artifact-shell__actions') + '" aria-label="Actions">' +
      '<button type="button" data-copy data-copy-kind="' + (kind === 'message' ? 'message' : 'artifact') + '" aria-label="Copy" title="Copy">' + icons.copy + '</button>' +
      '<button type="button" data-reply data-target-kind="' + kind + '" data-target-id="' + escapeHtml(id) + '" data-target-title="' + escapeHtml(title) + '" data-target-excerpt="' + escapeHtml(excerpt) + '" aria-label="Reply" title="Reply">' + icons.reply + '</button>' +
      '</div>';
  }

  function ensureUserMessageActions() {
    thread.querySelectorAll('.message--user').forEach(function (message) {
      if (message.querySelector('.message-actions')) return;
      var id = message.getAttribute('data-message-id') || 'message-' + (++messageCounter);
      var body = message.querySelector('.message__body');
      var text = markdownPlainText(markdownSource(body));
      message.setAttribute('data-message-id', id);
      message.insertAdjacentHTML('beforeend', actionsMarkup('message', id, 'You', text.slice(0, 70)));
    });
  }

  hydrateMarkdownMessages();
  ensureUserMessageActions();

  function expandMarkup(id) {
    return '<div class="artifact-frame__actions"><button type="button" data-expand="' + escapeHtml(id) + '" aria-label="Expand artifact" title="Expand">' + icons.expand + '</button></div>';
  }

  function browserToolbarMarkup(id, url) {
    return '<button type="button" data-browser-action="back" aria-label="Back" disabled><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m15 18-6-6 6-6"></path></svg></button>' +
      '<button type="button" data-browser-action="forward" aria-label="Forward" disabled><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m9 18 6-6-6-6"></path></svg></button>' +
      '<button type="button" data-browser-action="refresh" aria-label="Refresh"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M20 11a8 8 0 1 0-2.3 5.7L20 14"></path><path d="M20 6v5h-5"></path></svg></button>' +
      '<label class="sr-only">Current web address</label><input data-browser-address value="' + escapeHtml(url) + '" readonly>' + expandMarkup(id);
  }

  function fileHeaderActionsMarkup(id, title) {
    return '<div class="artifact-frame__actions"><button type="button" data-file-edit aria-label="Edit ' + escapeHtml(title) + '" title="Edit">' + icons.edit + '</button>' +
      '<button type="button" data-file-cancel aria-label="Discard changes to ' + escapeHtml(title) + '" title="Discard changes" hidden>' + icons.trash + '</button>' +
      '<button type="button" data-file-save aria-label="Save ' + escapeHtml(title) + '" title="Save" hidden>' + icons.save + '</button>' +
      '<button type="button" data-expand="' + escapeHtml(id) + '" aria-label="Expand file artifact" title="Expand">' + icons.expand + '</button></div>';
  }

  function lineNumbers(value) {
    return value.split('\n').map(function (_, index) { return index + 1; }).join('\n');
  }

  function editorText(editor) {
    return editor ? editor.innerText.replace(/\r/g, '') : '';
  }

  function setEditorText(editor, value) {
    if (editor) editor.textContent = value;
  }

  function breadcrumbMarkup(path, leafIsFile) {
    var isHome = path.indexOf('~/') === 0;
    var parts = path.replace(/^~\//, '').replace(/^\/+/, '').split('/').filter(Boolean);
    var built = isHome ? '~' : '';
    return '<nav class="file-breadcrumbs" aria-label="File path">' + parts.map(function (part, index) {
      built += '/' + part;
      var current = index === parts.length - 1;
      var segment = current
        ? '<span aria-current="page">' + escapeHtml(part) + '</span>'
        : '<button type="button" data-open-folder="' + escapeHtml(built) + '">' + escapeHtml(part) + '</button>';
      if (leafIsFile && current) return '<span class="file-breadcrumbs__separator">/</span>' + segment;
      return (index ? '<span class="file-breadcrumbs__separator">/</span>' : '') + segment;
    }).join('') + '</nav>';
  }

  function explorerEntries(path) {
    if (/\/docs$/.test(path)) return [
      { kind: 'folder', name: 'guides', detail: '3 items' },
      { kind: 'folder', name: 'references', detail: '2 items' },
      { kind: 'file', name: 'notes.md', detail: '3 KB' },
      { kind: 'file', name: 'architecture.md', detail: '5 KB' }
    ];
    if (/\/src$/.test(path)) return [
      { kind: 'folder', name: 'components', detail: '8 items' },
      { kind: 'folder', name: 'lib', detail: '5 items' },
      { kind: 'file', name: 'app.js', detail: '6 KB' },
      { kind: 'file', name: 'workspace.js', detail: '4 KB' },
      { kind: 'file', name: 'styles.css', detail: '9 KB' }
    ];
    if (/\/wireframes$/.test(path)) return [
      { kind: 'folder', name: 'r004-artifact-focus-swap', detail: '7 items' },
      { kind: 'folder', name: 'r005-left-chat-pane', detail: '9 items' },
      { kind: 'folder', name: 'r006-file-explorer-artifact', detail: '9 items' },
      { kind: 'file', name: 'README.md', detail: '2 KB' }
    ];
    return [
      { kind: 'folder', name: 'src', detail: '12 items' },
      { kind: 'folder', name: 'wireframes', detail: '6 items' },
      { kind: 'folder', name: 'docs', detail: '4 items' },
      { kind: 'file', name: 'index.html', detail: '8 KB' },
      { kind: 'file', name: 'notes.md', detail: '3 KB' },
      { kind: 'file', name: 'package.json', detail: '1 KB' }
    ];
  }

  function sampleExplorerFileContent(name) {
    if (/\.html$/.test(name)) return [
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
      '</html>'
    ].join('\n');
    if (name === 'notes.md') return sampleFileContent();
    return '# ' + name + '\n\nThis file is open from File Explorer.\n\nUse Edit to make changes or the folder breadcrumbs to return to the directory.';
  }

  function explorerFile(path) {
    var title = path.split('/').pop();
    return { title: title, path: path, content: sampleExplorerFileContent(title), version: 1, editing: false };
  }

  function explorerRowsMarkup(path) {
    var rows = explorerEntries(path);
    return '<div class="file-explorer-list">' + rows.map(function (row) {
      var targetPath = path.replace(/\/$/, '') + '/' + row.name;
      var action = row.kind === 'folder'
        ? ' data-open-folder="' + escapeHtml(targetPath) + '"'
        : ' data-open-explorer-file="' + escapeHtml(targetPath) + '"';
      var icon = row.kind === 'folder' ? icons.folder : artifactAvatarMarkup('file').replace(/^<div[^>]*>|<\/div>$/g, '');
      return '<button class="file-explorer-row" type="button"' + action + '><span class="file-explorer-row__icon">' + icon + '</span><span class="file-explorer-row__name">' + escapeHtml(row.name) + '</span><span class="file-explorer-row__detail">' + escapeHtml(row.detail) + '</span></button>';
    }).join('') + '</div>';
  }

  /** Returns the registered provider or a safe non-expandable fallback. */
  function artifactProvider(type) {
    return artifactProviders[type] || { expandable: false };
  }

  /** Builds the shared class contract for an artifact shell context. */
  function artifactShellClass(context, type) {
    return 'artifact-shell artifact-shell--' + context +
      ' artifact-provider artifact-provider--' + type;
  }

  /** Wraps provider markup in the shared artifact body slot. */
  function artifactBodyMarkup(content) {
    return '<div class="artifact-frame__body">' + content + '</div>';
  }

  /** Builds the shared framed surface used by response artifacts. */
  function artifactFrameMarkup(options) {
    var className = 'artifact-frame response-container';
    if (options.className) className += ' ' + options.className;
    return '<div class="' + className + '">' +
      '<header class="artifact-frame__header' +
        (options.headerClass ? ' ' + options.headerClass : '') + '">' +
        options.header +
      '</header>' +
      artifactBodyMarkup(options.body) +
      (options.footer || '') +
      '</div>';
  }

  /** Builds the response-shell slots around provider-owned frame markup. */
  function responseArtifactMarkup(options) {
    return '<div class="artifact-shell__preamble">' +
        artifactPreambleMarkup(
          options.duration,
          options.summary,
          options.activity,
          options.type
        ) +
      '</div>' +
      options.frame +
      actionsMarkup(
        options.type,
        options.id,
        options.replyTitle,
        options.replyExcerpt
      );
  }

  /** Creates a response artifact node with provider-neutral shell attributes. */
  function createResponseArtifactNode(state, options) {
    var context = thread.getAttribute('data-artifact-context') || 'response';
    var node = document.createElement('article');
    node.className = artifactShellClass('response', state.type) +
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
    return '<section class="' + artifactShellClass('focus', type) +
      ' artifact-focus" data-artifact-context="focus" aria-label="' +
      escapeHtml(label) + '">' + content + '</section>';
  }

  function explorerFileActionsMarkup(id, file, focused) {
    var className = focused
      ? 'artifact-shell__controls'
      : 'artifact-frame__actions';
    var buttonClass = focused
      ? ' class="artifact-shell__control artifact-shell__control--icon"'
      : '';
    return '<div class="' + className + '">' +
      '<button' + buttonClass + ' type="button" data-explorer-file-edit aria-label="Edit ' + escapeHtml(file.title) + '" title="Edit"' + (file.editing ? ' hidden' : '') + '>' + icons.edit + '</button>' +
      '<button' + buttonClass + ' type="button" data-explorer-file-cancel aria-label="Discard changes to ' + escapeHtml(file.title) + '" title="Discard changes"' + (file.editing ? '' : ' hidden') + '>' + icons.trash + '</button>' +
      '<button' + buttonClass + ' type="button" data-explorer-file-save aria-label="Save ' + escapeHtml(file.title) + '" title="Save"' + (file.editing ? '' : ' hidden') + '>' + icons.save + '</button>' +
      (focused
        ? '<button class="artifact-shell__control artifact-shell__control--icon" type="button" data-close-focused-artifact aria-label="Close File artifact" title="Close">' + icons.close + '</button>'
        : '<button type="button" data-expand="' + escapeHtml(id) + '" aria-label="Expand file artifact" title="Expand">' + icons.expand + '</button>') +
      '</div>';
  }

  function explorerFileBodyMarkup(file, focused) {
    return '<div class="explorer-file-document' + (focused ? ' explorer-file-document--focused' : '') + '" data-explorer-file-document data-editing="' + String(file.editing) + '">' +
      '<pre class="explorer-file-document__lines" data-explorer-file-lines aria-hidden="true"' + (file.editing ? '' : ' hidden') + '>' + lineNumbers(file.content) + '</pre>' +
      '<div class="explorer-file-document__editor" data-explorer-file-editor role="textbox" aria-label="' + (file.editing ? 'Edit ' : 'View ') + escapeHtml(file.title) + '" aria-multiline="true" aria-readonly="' + String(!file.editing) + '" contenteditable="' + (file.editing ? 'true' : 'false') + '" spellcheck="false">' + escapeHtml(file.content) + '</div>' +
      '</div>';
  }

  function explorerSurfaceMarkup(id, options) {
    var focused = Boolean(options.focused);
    var file = options.file || null;
    var headerActions = file
      ? explorerFileActionsMarkup(id, file, focused)
      : (focused
        ? '<div class="artifact-shell__controls"><button class="artifact-shell__control artifact-shell__control--icon" type="button" data-close-focused-artifact aria-label="Close File Explorer artifact" title="Close">' + icons.close + '</button></div>'
        : expandMarkup(id));
    var body = file ? explorerFileBodyMarkup(file, focused) : explorerRowsMarkup(options.path);
    var footer = !focused && !file
      ? '<footer class="artifact-frame__footer"><span>Read only</span><span>' + explorerEntries(options.path).length + ' items</span></footer>'
      : '';
    var header = '<div class="artifact-shell__breadcrumbs">' +
      breadcrumbMarkup(file ? file.path : options.path, Boolean(file)) +
      '</div>' + headerActions;
    if (focused) {
      return focusArtifactMarkup(
        file ? 'file' : 'folder',
        file ? 'Open file' : 'File Explorer artifact',
        '<header class="artifact-shell__header">' + header + '</header>' + body
      );
    }
    return artifactFrameMarkup({
      body: body,
      footer: footer,
      header: header,
      headerClass: 'file-explorer-header'
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
    surface.outerHTML = state.previewFile ? explorerFileSurfaceMarkup(id, state.previewFile) : folderSurfaceMarkup(id, state.path);
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
    var avatar = card.querySelector('.artifact-shell__identity .message__avatar');
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
    if (actions) actions.setAttribute('aria-label', 'File Explorer artifact actions');
    var copy = card.querySelector('[data-copy]');
    if (copy) copy.setAttribute('aria-label', 'Copy folder path');
  }

  function fileEditorMarkup(title, content) {
    return '<div class="file-editor" data-file-editor hidden><pre class="file-editor__lines" data-file-lines aria-hidden="true">' + lineNumbers(content) + '</pre><div class="file-editor__content" data-file-editor-input role="textbox" aria-label="Edit ' + escapeHtml(title) + '" aria-multiline="true" contenteditable="true" spellcheck="false">' + escapeHtml(content) + '</div></div>';
  }

  function assistantAvatarMarkup() {
    return '<div class="message__avatar" aria-hidden="true"><svg viewBox="0 0 24 24"><path d="M12 3l1.4 4.6L18 9l-4.6 1.4L12 15l-1.4-4.6L6 9l4.6-1.4L12 3Z"></path><path d="M18.5 15l.7 2.3 2.3.7-2.3.7-.7 2.3-.7-2.3-2.3-.7 2.3-.7.7-2.3Z"></path></svg></div>';
  }

  function artifactAvatarMarkup(type) {
    var shapes = {
      generic: '<rect x="4" y="4" width="16" height="16" rx="4"></rect><path d="M8 9h8M8 12h8M8 15h5"></path>',
      browser: '<circle cx="12" cy="12" r="9"></circle><path d="M3 12h18M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18"></path>',
      file: '<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"></path><path d="M14 2v6h6M8 13h8M8 17h6"></path>',
      folder: '<path d="M3 7h6l2 2h10v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z"></path>',
      terminal: '<rect x="3" y="4" width="18" height="16" rx="2"></rect><path d="m7 9 3 3-3 3M13 15h4"></path>'
    };
    return '<div class="message__avatar" aria-hidden="true"><svg viewBox="0 0 24 24">' + (shapes[type] || shapes.generic) + '</svg></div>';
  }

  function thinkingMarkup(duration, summary, activity, className) {
    return '<details class="thinking-process' + (className ? ' ' + className : '') + '"><summary><span>Worked for ' + escapeHtml(duration) + '</span><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m9 18 6-6-6-6"></path></svg></summary><div class="thinking-process__content"><p>' + escapeHtml(summary) + '</p><div class="thinking-process__activity"><svg viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="4" width="18" height="14" rx="2"></rect><path d="M8 9l2 2 4-4"></path></svg><span>' + escapeHtml(activity) + '</span></div></div></details>';
  }

  function artifactPreambleMarkup(duration, summary, activity, type) {
    return thinkingMarkup(duration, summary, activity, 'artifact-shell__thinking') + '<div class="artifact-shell__identity">' + artifactAvatarMarkup(type) + '<div class="message__speaker">C4OS</div></div>';
  }

  function appendUserMessage(text, reference, attachments) {
    messageCounter += 1;
    var node = document.createElement('div');
    node.className = 'message message--user';
    node.setAttribute('data-message-id', 'message-' + messageCounter);
    var bodyText = text ? messageBodyMarkup(text) : '';
    var attachmentNames = (attachments || []).map(function (attachment) { return attachment.name; }).join(', ');
    var excerpt = markdownPlainText(text) || attachmentNames || 'Attached files';
    node.innerHTML = (reference ? '<div class="message-reference">Replying to: ' + escapeHtml(reference) + '</div>' : '') +
      messageAttachmentsMarkup(attachments) +
      bodyText +
      actionsMarkup('message', 'message-' + messageCounter, 'You', excerpt.slice(0, 70));
    thread.appendChild(node);
    return node;
  }

  function replyReferenceText(target) {
    var source = target.kind === 'message' ? target.excerpt : target.title;
    var words = String(source || '').trim().split(/\s+/).filter(Boolean);
    return words.slice(0, 8).join(' ') + (words.length > 8 ? '…' : '');
  }

  function appendAssistantMessage(text) {
    messageCounter += 1;
    var model = document.querySelector('[data-option-value="model"]').textContent;
    var node = document.createElement('article');
    node.className = 'message message--assistant';
    node.setAttribute('data-message-id', 'message-' + messageCounter);
    node.innerHTML = thinkingMarkup('12 sec', 'I reviewed the referenced context and prepared the requested next step.', 'Updated the conversation') + '<div class="message-model-row">' + assistantAvatarMarkup() + '<div class="message__speaker">' + escapeHtml(model) + '</div></div>' +
      '<div class="message__content">' + messageBodyMarkup(text, 'response-container') +
      actionsMarkup('message', 'message-' + messageCounter, model,
        markdownPlainText(text).slice(0, 70)) + '</div>';
    thread.appendChild(node);
    queueAgentStream(node);
    return node;
  }

  function browserPreview(state) {
    var dashboard = state.url.indexOf('dashboard') > -1;
    return '<span>' + (dashboard ? 'WORKSPACE' : 'EXAMPLE DOMAIN') + '</span><h3>' + (dashboard ? 'Account dashboard' : 'A simple page for examples') + '</h3><p>' + (dashboard ? 'Your recent projects and activity are ready.' : 'This domain is ready for browsing and testing.') + '</p>';
  }

  function updateBrowserCard(id) {
    var state = artifacts[id];
    var card = document.querySelector('[data-artifact-id="' + id + '"]');
    if (!state || !card) return;
    card.querySelector('[data-artifact-status]').textContent = state.status;
    card.querySelector('[data-browser-address]').value = state.url;
    card.querySelector('[data-browser-preview]').innerHTML = browserPreview(state);
    card.querySelector('[data-browser-history]').textContent = state.history.length + (state.history.length === 1 ? ' navigation' : ' navigations');
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
    state.title = url.indexOf('dashboard') > -1 ? 'Account dashboard' : (url.replace(/^https?:\/\//, '').replace(/\/$/, '') || 'Webpage');
    state.status = replaceHistory ? 'Refreshed' : 'Loaded';
    updateBrowserCard(id);
  }

  function createBrowserArtifact(url) {
    artifactCounter += 1;
    var id = 'browser-' + artifactCounter;
    var normalized = /^https?:\/\//i.test(url) ? url : 'https://' + url;
    artifacts[id] = { id: id, type: 'browser', title: normalized.replace(/^https?:\/\//, '').replace(/\/$/, ''), url: normalized, history: [normalized], historyIndex: 0, status: 'Ready' };
    var frame = artifactFrameMarkup({
      body: '<div class="browser-preview" data-browser-preview>' +
        browserPreview(artifacts[id]) + '</div>',
      footer: '<footer class="artifact-frame__footer"><span data-artifact-status>Ready</span><span data-browser-history>1 navigation</span></footer>',
      header: browserToolbarMarkup(id, normalized),
      headerClass: 'browser-toolbar'
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
      type: 'browser'
    });
    thread.appendChild(node);
    queueAgentStream(node);
  }

  function updateFileCard(id) {
    var state = artifacts[id];
    var card = document.querySelector('[data-artifact-id="' + id + '"]');
    if (!state || !card) return;
    card.querySelector('[data-file-preview]').textContent = state.proposedContent || state.content;
    var fileVersion = card.querySelector('[data-file-version]');
    if (fileVersion) fileVersion.textContent = 'Version ' + state.version;
    var artifactMeta = card.querySelector('[data-artifact-meta]');
    if (artifactMeta) artifactMeta.textContent = 'Markdown · Version ' + state.version;
    var artifactStatus = card.querySelector('[data-artifact-status]');
    if (artifactStatus) artifactStatus.textContent = state.status;
    card.querySelector('[data-file-approval]').hidden = state.editing || !state.proposedContent;
    card.querySelector('[data-file-preview]').hidden = state.editing;
    card.querySelector('[data-file-editor]').hidden = !state.editing;
    card.querySelector('[data-file-edit]').hidden = state.editing;
    card.querySelector('[data-file-cancel]').hidden = !state.editing;
    card.querySelector('[data-file-save]').hidden = !state.editing;
    if (state.editing && artifactStatus) artifactStatus.textContent = 'Editing';
    if (focusedArtifact === id) renderFocusViewer();
  }

  function beginFileEdit(card) {
    var state = artifacts[card.getAttribute('data-artifact-id')];
    if (!state) return;
    state.editing = true;
    var editor = card.querySelector('[data-file-editor-input]');
    setEditorText(editor, state.proposedContent || state.content);
    card.querySelector('[data-file-lines]').textContent = lineNumbers(editorText(editor));
    updateFileCard(state.id);
    editor.focus();
  }

  function cancelFileEdit(card) {
    var state = artifacts[card.getAttribute('data-artifact-id')];
    if (!state) return;
    state.editing = false;
    updateFileCard(state.id);
  }

  function saveFileEdit(card) {
    var state = artifacts[card.getAttribute('data-artifact-id')];
    if (!state) return;
    state.content = editorText(card.querySelector('[data-file-editor-input]'));
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
    artifacts[id] = { id: id, type: 'file', title: title, path: path.indexOf('/') > -1 ? path : '~/project/docs/' + path, content: sampleFileContent(), version: 1, status: 'Viewing', proposedContent: '', editing: false };
    var frame = artifactFrameMarkup({
      body: '<pre class="file-preview" data-file-preview>' +
        escapeHtml(artifacts[id].content) + '</pre>' +
        fileEditorMarkup(title, artifacts[id].content) +
        '<div class="file-approval" data-file-approval hidden><div><strong>2 edits proposed</strong><span>Typography and punctuation</span></div><div><button type="button" data-file-reject>Reject</button><button type="button" data-file-approve>Approve &amp; save</button></div></div>',
      header: breadcrumbMarkup(artifacts[id].path, true) +
        fileHeaderActionsMarkup(id, title)
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
      type: 'file'
    });
    thread.appendChild(node);
    queueAgentStream(node);
  }

  function createFolderArtifact(path) {
    artifactCounter += 1;
    var normalized = path.replace(/\/$/, '') || '~/project';
    var id = 'folder-' + artifactCounter;
    var title = normalized.split('/').filter(Boolean).pop() || 'project';
    artifacts[id] = { id: id, type: 'folder', title: title, path: normalized, status: 'Read only', previewFile: null };
    var node = createResponseArtifactNode(artifacts[id], {
      activity: 'Listed the folder contents',
      duration: '7 sec',
      frame: folderSurfaceMarkup(id, normalized),
      id: id,
      label: 'File Explorer artifact ' + title,
      replyExcerpt: normalized,
      replyTitle: 'File Explorer · ' + title,
      summary: 'I opened the selected folder as a read-only File Explorer artifact.',
      type: 'folder'
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
    return terminalSession.entries.filter(function (entry) { return entry.id === id; })[0] || null;
  }

  function activeTerminalEntry() {
    return terminalSession.activeEntryId ? terminalEntry(terminalSession.activeEntryId) : null;
  }

  function syncTerminalComposer() {
    var input = inputForMode('terminal');
    var prefix = document.querySelector('.terminal-prefix');
    var active = activeTerminalEntry();
    if (input) input.placeholder = active ? 'Send input to running process' : 'Enter a command';
    if (prefix) prefix.textContent = active ? '›' : '$';
    if (currentMode === 'terminal' && !replyTarget) {
      primaryAction.setAttribute('aria-label', active ? 'Send process input' : 'Run command');
      primaryAction.title = active ? 'Send process input' : 'Run command';
    }
  }

  function formatTerminalElapsed(seconds) {
    var value = Math.max(0, seconds || 0);
    var minutes = Math.floor(value / 60);
    var remainder = value % 60;
    return String(minutes).padStart(2, '0') + ':' + String(remainder).padStart(2, '0');
  }

  function terminalResultMarkup(entry) {
    if (entry.status === 'running') return '<span class="terminal-live-dot" aria-hidden="true"></span>Running · ' + formatTerminalElapsed(entry.elapsed);
    if (entry.status === 'interrupted') return 'Interrupted · Exit 130';
    return 'Exit ' + String(entry.exit == null ? 0 : entry.exit);
  }

  function terminalHeaderActionsMarkup(entry) {
    return '<div class="artifact-frame__actions">' +
      '<button class="terminal-stop" type="button" data-terminal-stop="' + escapeHtml(entry.id) + '" aria-label="Stop ' + escapeHtml(entry.command) + '" title="Stop process"' + (entry.status === 'running' ? '' : ' hidden') + '>' + icons.stop + '</button>' +
      '<button type="button" data-expand="' + escapeHtml(entry.id) + '" aria-label="Expand terminal artifact" title="Expand">' + icons.expand + '</button></div>';
  }

  function terminalCardSurfaceMarkup(entry) {
    return artifactFrameMarkup({
      body: '<pre class="terminal-output" data-terminal-output>' +
        escapeHtml(entry.output) + '</pre>',
      footer: '<footer class="artifact-frame__footer"><span data-terminal-result>' +
        terminalResultMarkup(entry) + '</span><span>' + terminalSession.id +
        '</span></footer>',
      header: '<div class="artifact-frame__identity"><div><strong data-artifact-title>$ ' +
        escapeHtml(entry.command) + '</strong><span data-artifact-meta>' +
        escapeHtml(terminalSession.cwd) + ' · ' + terminalSession.id +
        '</span></div></div>' + terminalHeaderActionsMarkup(entry)
    });
  }

  function terminalInlinePromptMarkup() {
    return '<form class="terminal-inline-prompt" data-terminal-session-form><span aria-hidden="true">$</span><label class="sr-only" for="expanded-terminal-input">Terminal command</label><input id="expanded-terminal-input" data-terminal-session-input aria-label="Terminal command" autocomplete="off" spellcheck="false"></form>';
  }

  function terminalSessionMarkup(selectedId) {
    var entry = terminalEntry(selectedId) || terminalSession.entries[terminalSession.entries.length - 1];
    var running = entry && entry.status === 'running';
    var output = entry ? '$ ' + entry.command + '\n' + entry.output : '';
    return focusArtifactMarkup(
      'terminal',
      'Expanded terminal session',
      '<header class="terminal-session-header"><div><strong>Terminal</strong><span>' +
        escapeHtml(terminalSession.cwd) + ' · ' + terminalSession.id +
        '</span></div><div class="artifact-shell__controls"><button class="artifact-shell__control artifact-shell__control--icon terminal-stop" type="button" data-terminal-stop="' +
        escapeHtml(running ? entry.id : '') +
        '" aria-label="Stop current process" title="Stop process"' +
        (running ? '' : ' hidden') + '>' + icons.stop +
        '</button><button class="artifact-shell__control artifact-shell__control--icon" type="button" data-close-focused-artifact aria-label="Close Terminal artifact" title="Close">' +
        icons.close + '</button></div></header>' +
      '<div class="terminal-main-body" data-terminal-main-body data-terminal-entry="' +
        escapeHtml(entry ? entry.id : '') + '" data-status="' +
        escapeHtml(entry ? entry.status : 'completed') +
        '" role="log" aria-live="polite"><span class="terminal-main-status" data-terminal-session-result>' +
        (entry ? terminalResultMarkup(entry) : '') +
        '</span><pre class="terminal-main-output" data-terminal-session-output>' +
        escapeHtml(output) + '</pre>' +
        (running ? '' : terminalInlinePromptMarkup()) + '</div>'
    );
  }

  function updateFocusedTerminal(entry) {
    var sessionBody = focusViewer.querySelector('[data-terminal-main-body][data-terminal-entry="' + entry.id + '"]');
    if (!sessionBody) return;
    sessionBody.setAttribute('data-status', entry.status);
    sessionBody.querySelector('[data-terminal-session-output]').textContent = '$ ' + entry.command + '\n' + entry.output;
    sessionBody.querySelector('[data-terminal-session-result]').innerHTML = terminalResultMarkup(entry);
    var stop = focusViewer.querySelector('.terminal-session-header [data-terminal-stop]');
    if (stop) {
      stop.hidden = entry.status !== 'running';
      stop.setAttribute('data-terminal-stop', entry.status === 'running' ? entry.id : '');
    }
    var prompt = sessionBody.querySelector('[data-terminal-session-form]');
    if (entry.status === 'running' && prompt) prompt.remove();
    if (entry.status !== 'running' && !prompt) sessionBody.insertAdjacentHTML('beforeend', terminalInlinePromptMarkup());
    sessionBody.scrollTop = sessionBody.scrollHeight;
  }

  function updateTerminalCard(entry) {
    var card = document.querySelector('[data-artifact-id="' + entry.id + '"]');
    if (card) {
      card.setAttribute('data-terminal-status', entry.status);
      card.setAttribute('aria-label', (entry.status === 'running' ? 'Running ' : '') + 'Terminal artifact ' + entry.command + ' output');
      card.classList.toggle('artifact-provider--running', entry.status === 'running');
      card.querySelector('[data-terminal-output]').textContent = entry.output;
      card.querySelector('[data-terminal-result]').innerHTML = terminalResultMarkup(entry);
      var duration = card.querySelector('.artifact-shell__thinking summary span');
      if (duration) duration.textContent = entry.status === 'running' ? 'Running for ' + entry.elapsed + ' sec' : (entry.status === 'interrupted' ? 'Stopped after ' + entry.elapsed + ' sec' : 'Worked for 6 sec');
      var stop = card.querySelector('[data-terminal-stop]');
      if (stop) stop.hidden = entry.status !== 'running';
    }
    if (focusedArtifact && artifacts[focusedArtifact] && artifacts[focusedArtifact].type === 'terminal') updateFocusedTerminal(entry);
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
      var lines = ['GET /api/session 200 9ms', 'hmr update /src/workspace.js', 'GET /assets/styles.css 200 4ms'];
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
      output: running ? '> workspace@1.0.0 dev\n> vite --host 127.0.0.1\n\nLocal: http://127.0.0.1:4179/\nready in 412 ms' : terminalOutput(command),
      status: running ? 'running' : 'completed',
      exit: running ? null : 0,
      elapsed: 0
    };
    terminalSession.entries.push(entry);
    if (running) terminalSession.activeEntryId = id;
    artifacts[id] = { id: id, type: 'terminal', title: command, sessionId: terminalSession.id };
    var node = createResponseArtifactNode(artifacts[id], {
      activity: running ? 'Streaming process output' : 'Captured the command output',
      duration: running ? '1 sec' : '6 sec',
      frame: terminalCardSurfaceMarkup(entry),
      id: id,
      label: (running ? 'Running ' : '') + 'Terminal artifact ' + command + ' output',
      replyExcerpt: entry.output,
      replyTitle: 'Terminal · ' + command,
      summary: running
        ? 'The command is running in the active terminal session.'
        : 'I ran the requested command in the active terminal session.',
      type: 'terminal'
    });
    node.classList.toggle('artifact-provider--running', running);
    node.setAttribute('data-terminal-session', terminalSession.id);
    node.setAttribute('data-terminal-status', entry.status);
    thread.appendChild(node);
    queueAgentStream(node);
    syncTerminalComposer();
    if (running) startTerminalStream(entry);
    if (focusedArtifact && artifacts[focusedArtifact] && artifacts[focusedArtifact].type === 'terminal') renderFocusViewer();
    return entry;
  }

  function submitReply(text, attachments) {
    var target = replyTarget;
    appendUserMessage(text, replyReferenceText(target), attachments);
    if (target.kind === 'browser' && artifacts[target.id]) {
      navigateBrowser(target.id, 'https://example.com/dashboard', false);
      artifacts[target.id].status = 'Updated from reply';
      updateBrowserCard(target.id);
      appendAssistantMessage('I updated the existing browser artifact with the requested navigation.');
    } else if (target.kind === 'file' && artifacts[target.id]) {
      var file = artifacts[target.id];
      file.proposedContent = file.content.replace('keeps chat at the center', 'keeps chat in the center').replace('This file is open', 'This file is now open');
      if (file.proposedContent === file.content) file.proposedContent += '\n\nTypos corrected and copy refined.';
      file.status = 'Approval required';
      updateFileCard(target.id);
      appendAssistantMessage('I prepared the file edits. Approve them on the original artifact to save immediately.');
    } else if (target.kind === 'terminal') {
      var referencedEntry = terminalEntry(target.id);
      if (/server|running|still on|active/i.test(text)) {
        appendAssistantMessage(referencedEntry && referencedEntry.status === 'running'
          ? 'Yes. The referenced process is still running in the shared terminal session.'
          : 'No. The referenced process is no longer running in the shared terminal session.');
      } else if (/big|size|space/i.test(text)) {
        appendAssistantMessage('Based on the referenced terminal output, the project folder is approximately 42 MB.');
      } else {
        appendAssistantMessage('I used the referenced terminal command and output as context for this response.');
      }
    } else {
      appendAssistantMessage('I used the quoted message as context for this response.');
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
    var chatAttachments = currentMode === 'chat' ? takeDraftAttachments() : [];
    if (!value && chatAttachments.length === 0) return;
    if (currentMode === 'chat') {
      appendUserMessage(value, null, chatAttachments);
      appendAssistantMessage(chatAttachments.length
        ? '**Attachments added.**\n\nI’ve included the prompt and files in the conversation context.'
        : '**Prompt received.**\n\nI’ve added that prompt to the conversation.');
      clearComposerInput(input);
    } else if (currentMode === 'files') {
      appendUserMessage('Open ' + value);
      if (fileSelectionKind === 'folder' || /\/$/.test(value) || value.split('/').pop().indexOf('.') === -1) createFolderArtifact(value);
      else createFileArtifact(value);
    } else if (currentMode === 'browser') {
      appendUserMessage('Open ' + value);
      createBrowserArtifact(value);
    } else {
      var active = activeTerminalEntry();
      if (active) {
        active.output += '\n> ' + value;
        updateTerminalCard(active);
      } else {
        appendUserMessage('$ ' + value);
        createTerminalArtifact(value);
      }
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
    var remaining = thread.scrollHeight - thread.scrollTop - thread.clientHeight;
    var scrollable = thread.scrollHeight > thread.clientHeight + 2;
    threadScrollButton.hidden = !scrollable || remaining <= 24;
  }

  /** Copies text with a legacy fallback and shared status notification. */
  function copyText(text) {
    var fallback = function () {
      var area = document.createElement('textarea');
      area.value = text;
      document.body.appendChild(area);
      area.select();
      document.execCommand('copy');
      area.remove();
    };
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).catch(fallback);
    } else fallback();
    clearTimeout(copyTimer);
    copyNotifier.hidden = false;
    copyNotifier.textContent = 'Copied';
    copyTimer = window.setTimeout(function () { copyNotifier.hidden = true; }, 1400);
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
        markdownSource(messageBody).trim()
      ].filter(Boolean).join('\n');
    }
    else {
      var id = container.getAttribute('data-artifact-id');
      var type = container.getAttribute('data-artifact-type');
      if (type === 'browser') text = artifacts[id].url;
      if (type === 'file') text = artifacts[id].content;
      if (type === 'folder') text = artifacts[id].path;
      if (type === 'terminal') {
        var entry = terminalSession.entries.filter(function (item) { return item.id === id; })[0];
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
    if (chatPaneHeight === null) chatPaneHeight = shell.getBoundingClientRect().height * 0.4;
    if (typeof nextHeight === 'number') chatPaneHeight = nextHeight;
    chatPaneHeight = Math.round(Math.max(minimumHeight, Math.min(maximumHeight, chatPaneHeight)));
    shell.style.setProperty('--left-chat-pane-height', chatPaneHeight + 'px');
    leftChatPaneResize.setAttribute('aria-valuemin', String(Math.round(minimumHeight)));
    leftChatPaneResize.setAttribute('aria-valuemax', String(Math.round(maximumHeight)));
    leftChatPaneResize.setAttribute('aria-valuenow', String(chatPaneHeight));
  }

  function clearFocusPlaceholder() {
    document.querySelectorAll('.artifact-shell--in-workspace').forEach(function (card) {
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
    var label = state.type === 'browser' ? 'Browser' : (state.type === 'folder' ? 'File Explorer' : (state.type === 'terminal' ? 'Terminal session' : 'File'));
    placeholder.textContent = label + ' open in the workspace · ' + state.title;
    card.querySelector('.artifact-shell__identity').insertAdjacentElement('afterend', placeholder);
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
      focusViewer.innerHTML = explorerSurfaceMarkup(focusedArtifact, { file: focusedExplorerFile, focused: true });
    } else if (focusedFolderPath || state.type === 'folder') {
      var folderPath = focusedFolderPath || state.path;
      focusViewer.innerHTML = explorerSurfaceMarkup(focusedArtifact, { path: folderPath, focused: true });
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
          icons.close + '</button></div><div class="browser-focus-page"><small>' +
          (state.url.indexOf('dashboard') > -1 ? 'WORKSPACE' : 'EXAMPLE DOMAIN') +
          '</small><h2>' + escapeHtml(state.title) +
          '</h2><p>This expanded Browser artifact uses the entire center workspace while the conversation stays available in the left Chat pane.</p></div>'
      );
    } else if (state.type === 'terminal') {
      focusViewer.innerHTML = terminalSessionMarkup(focusedArtifact);
      window.requestAnimationFrame(function () {
        var body = focusViewer.querySelector('[data-terminal-main-body]');
        if (body) body.scrollTop = body.scrollHeight;
      });
    } else {
      focusViewer.innerHTML = focusArtifactMarkup(
        'file',
        'Focused file artifact',
        '<header class="artifact-shell__header"><div class="artifact-shell__breadcrumbs">' +
          breadcrumbMarkup(state.path, true) +
          '</div><div class="artifact-shell__controls"><button class="artifact-shell__control artifact-shell__control--icon" type="button" data-focus-file-edit aria-label="Edit ' +
          escapeHtml(state.title) + '" title="Edit">' + icons.edit +
          '</button><button class="artifact-shell__control artifact-shell__control--icon" type="button" data-focus-file-cancel aria-label="Discard changes to ' +
          escapeHtml(state.title) + '" title="Discard changes" hidden>' + icons.trash +
          '</button><button class="artifact-shell__control artifact-shell__control--icon" type="button" data-focus-file-save aria-label="Save ' +
          escapeHtml(state.title) + '" title="Save" hidden>' + icons.save +
          '</button><button class="artifact-shell__control artifact-shell__control--icon" type="button" data-close-focused-artifact aria-label="Close File artifact" title="Close">' +
          icons.close + '</button></div></header><div class="focus-file-editor" data-focus-file-surface data-editing="false"><pre class="focus-file-editor__lines" data-focus-file-lines aria-hidden="true" hidden>' +
          lineNumbers(state.content) +
          '</pre><div class="pane-file-editor" data-focus-file-editor role="textbox" aria-label="View ' +
          escapeHtml(state.title) +
          '" aria-multiline="true" aria-readonly="true" contenteditable="false" spellcheck="false">' +
          escapeHtml(state.content) + '</div></div>'
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
    } else if (action === 'forward' && state.historyIndex < state.history.length - 1) {
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
      if (moveEvent.buttons === 0) { stopPaneResize(); return; }
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
      removeDraftAttachment(removeAttachment.getAttribute('data-remove-attachment'));
      return;
    }
    var panelToggle = event.target.closest('[data-panel-toggle]');
    if (panelToggle) {
      var side = panelToggle.getAttribute('data-panel-toggle');
      var open = shell.getAttribute('data-' + side + '-open') !== 'false';
      setPanelOpen(side, !open);
      return;
    }
    if (narrowQuery.matches && shell.getAttribute('data-left-open') !== 'false' && event.target.closest('.chat-center')) {
      setPanelOpen('left', false);
    }
    var mode = event.target.closest('[data-mode-option]');
    if (mode) { setMode(mode.getAttribute('data-mode-option'), true); inputForMode(currentMode).focus(); return; }
    var reply = event.target.closest('[data-reply]');
    if (reply) { startReply(reply); return; }
    if (event.target.closest('[data-reply-cancel]')) { cancelReply(); return; }
    var codeCopy = event.target.closest('[data-copy-code]');
    if (codeCopy) {
      copyText(codeCopy.closest('.markdown-code').querySelector('code').textContent);
      return;
    }
    var copy = event.target.closest('[data-copy]');
    if (copy) { contextualCopy(copy); return; }
    var terminalStop = event.target.closest('[data-terminal-stop]');
    if (terminalStop) { stopTerminalEntry(terminalStop.getAttribute('data-terminal-stop')); return; }
    var expand = event.target.closest('[data-expand]');
    if (expand) { focusArtifact(expand.getAttribute('data-expand')); return; }
    if (event.target.closest('[data-close-focused-artifact]')) { restoreChat(); return; }
    if (event.target.closest('[data-restore-chat]')) { restoreChat(); return; }
    var browserAction = event.target.closest('[data-browser-action]');
    if (browserAction) { handleBrowserAction(browserAction.closest('[data-artifact-card]'), browserAction.getAttribute('data-browser-action')); return; }
    var fileEdit = event.target.closest('[data-file-edit]');
    if (fileEdit) { beginFileEdit(fileEdit.closest('[data-artifact-card]')); return; }
    var fileCancel = event.target.closest('[data-file-cancel]');
    if (fileCancel) { cancelFileEdit(fileCancel.closest('[data-artifact-card]')); return; }
    var fileSave = event.target.closest('[data-file-save]');
    if (fileSave) { saveFileEdit(fileSave.closest('[data-artifact-card]')); return; }
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
      var rejected = artifacts[reject.closest('[data-artifact-card]').getAttribute('data-artifact-id')];
      rejected.proposedContent = '';
      rejected.status = 'Viewing';
      updateFileCard(rejected.id);
      return;
    }
    var sourceKind = event.target.closest('[data-select-file-kind]');
    if (sourceKind) {
      fileSelectionKind = sourceKind.getAttribute('data-select-file-kind');
      var fileInput = inputForMode('files');
      fileInput.value = fileSelectionKind === 'folder' ? '~/project' : 'notes.md';
      fileInput.setAttribute('data-selection-kind', fileSelectionKind);
      primaryAction.setAttribute('aria-label', fileSelectionKind === 'folder' ? 'Open folder' : 'Open file');
      primaryAction.title = fileSelectionKind === 'folder' ? 'Open folder' : 'Open file';
      fileSourcePopover.hidden = true;
      document.querySelector('[data-browse-file]').setAttribute('aria-expanded', 'false');
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
        var folderState = artifacts[folderCard.getAttribute('data-artifact-id')];
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
      var openedFile = explorerFile(explorerFileLink.getAttribute('data-open-explorer-file'));
      var explorerCard = explorerFileLink.closest('[data-artifact-card]');
      if (explorerCard) {
        var explorerState = artifacts[explorerCard.getAttribute('data-artifact-id')];
        explorerState.previewFile = openedFile;
        updateFolderCard(explorerState.id);
        return;
      }
      focusedExplorerFile = openedFile;
      renderFocusViewer();
      return;
    }
    if (event.target.closest('[data-focus-file-edit]')) {
      var editor = document.querySelector('[data-focus-file-editor]');
      editor.setAttribute('contenteditable', 'true');
      editor.setAttribute('aria-readonly', 'false');
      editor.setAttribute('aria-label', 'Edit ' + artifacts[focusedArtifact].title);
      document.querySelector('[data-focus-file-surface]').setAttribute('data-editing', 'true');
      document.querySelector('[data-focus-file-lines]').hidden = false;
      document.querySelector('[data-focus-file-edit]').hidden = true;
      document.querySelector('[data-focus-file-cancel]').hidden = false;
      document.querySelector('[data-focus-file-save]').hidden = false;
      editor.focus();
      return;
    }
    if (event.target.closest('[data-focus-file-cancel]')) { renderFocusViewer(); return; }
    if (event.target.closest('[data-focus-file-save]')) {
      var activeFile = artifacts[focusedArtifact];
      activeFile.content = editorText(document.querySelector('[data-focus-file-editor]'));
      activeFile.version += 1;
      activeFile.status = 'Saved';
      updateFileCard(focusedArtifact);
      return;
    }
    var explorerEdit = event.target.closest('[data-explorer-file-edit]');
    if (explorerEdit) {
      var explorerDocument = explorerEdit.closest('.artifact-frame, .artifact-focus');
      var explorerFileState = explorerEdit.closest('[data-artifact-card]')
        ? artifacts[explorerEdit.closest('[data-artifact-card]').getAttribute('data-artifact-id')].previewFile
        : focusedExplorerFile;
      if (!explorerFileState || !explorerDocument) return;
      explorerFileState.editing = true;
      if (explorerEdit.closest('[data-artifact-card]')) updateFolderCard(explorerEdit.closest('[data-artifact-card]').getAttribute('data-artifact-id'));
      else renderFocusViewer();
      var explorerEditor = (explorerEdit.closest('[data-artifact-card]') || focusViewer).querySelector('[data-explorer-file-editor]');
      if (explorerEditor) explorerEditor.focus();
      return;
    }
    var explorerCancel = event.target.closest('[data-explorer-file-cancel]');
    if (explorerCancel) {
      var cancelCard = explorerCancel.closest('[data-artifact-card]');
      var cancelFile = cancelCard ? artifacts[cancelCard.getAttribute('data-artifact-id')].previewFile : focusedExplorerFile;
      if (!cancelFile) return;
      cancelFile.editing = false;
      if (cancelCard) updateFolderCard(cancelCard.getAttribute('data-artifact-id'));
      else renderFocusViewer();
      return;
    }
    var explorerSave = event.target.closest('[data-explorer-file-save]');
    if (explorerSave) {
      var saveCard = explorerSave.closest('[data-artifact-card]');
      var saveRoot = saveCard || focusViewer;
      var saveFile = saveCard ? artifacts[saveCard.getAttribute('data-artifact-id')].previewFile : focusedExplorerFile;
      if (!saveFile) return;
      saveFile.content = editorText(saveRoot.querySelector('[data-explorer-file-editor]'));
      saveFile.version += 1;
      saveFile.editing = false;
      if (saveCard) updateFolderCard(saveCard.getAttribute('data-artifact-id'));
      else renderFocusViewer();
      return;
    }
    var focusBrowser = event.target.closest('[data-focus-browser]');
    if (focusBrowser && artifacts[focusedArtifact]) {
      var fakeCard = document.querySelector('[data-artifact-id="' + focusedArtifact + '"]');
      handleBrowserAction(fakeCard, focusBrowser.getAttribute('data-focus-browser'));
    }
  });

  shell.addEventListener('keydown', function (event) {
    if (event.key === 'Enter' && event.target.matches('[data-terminal-session-input]')) {
      event.preventDefault();
      event.target.closest('form').requestSubmit();
      return;
    }
    if (event.key === 'Enter' && event.target.matches('[data-focus-browser-address]') && artifacts[focusedArtifact]) {
      navigateBrowser(focusedArtifact, event.target.value, false);
    }
    if (event.key === 'Enter' && !event.shiftKey && !event.isComposing &&
      event.target.matches('[data-mode-input], [data-reply-input]')) {
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
      event.target.closest('[data-file-editor]').querySelector('[data-file-lines]').textContent = lineNumbers(editorText(event.target));
    }
    if (event.target.matches('[data-focus-file-editor]')) {
      document.querySelector('[data-focus-file-lines]').textContent = lineNumbers(editorText(event.target));
    }
    if (event.target.matches('[data-explorer-file-editor]')) {
      var explorerLines = event.target.closest('[data-explorer-file-document]').querySelector('[data-explorer-file-lines]');
      if (explorerLines) explorerLines.textContent = lineNumbers(editorText(event.target));
    }
  });

  shell.addEventListener('scroll', function (event) {
    if (!event.target.matches('[data-focus-file-editor]')) return;
    var lines = document.querySelector('[data-focus-file-lines]');
    if (lines) lines.scrollTop = event.target.scrollTop;
  }, true);

  shell.addEventListener('scroll', function (event) {
    if (!event.target.matches('[data-explorer-file-editor]')) return;
    var lines = event.target.closest('[data-explorer-file-document]').querySelector('[data-explorer-file-lines]');
    if (lines) lines.scrollTop = event.target.scrollTop;
  }, true);

  composer.addEventListener('input', syncPrimaryAction);
  composer.addEventListener('submit', function (event) {
    event.preventDefault();
    if (replyTarget) {
      var value = inputValue(replyInput).trim();
      var replyAttachments = currentMode === 'chat' ? takeDraftAttachments() : [];
      if (!value && replyAttachments.length === 0) return;
      submitReply(value, replyAttachments);
      clearComposerInput(replyInput);
      scrollThread();
    } else submitMode();
    syncPrimaryAction();
  });

  thread.addEventListener('scroll', updateThreadScrollButton, { passive: true });
  threadScrollButton.addEventListener('click', scrollThread);
  window.addEventListener('resize', updateThreadScrollButton);
  if (window.ResizeObserver) new ResizeObserver(updateThreadScrollButton).observe(thread);
  if (window.ResizeObserver && composerDock) new ResizeObserver(syncComposerDockHeight).observe(composerDock);
  else window.addEventListener('resize', syncComposerDockHeight);
  new MutationObserver(function () { window.requestAnimationFrame(updateThreadScrollButton); }).observe(thread, { childList: true });

  document.querySelector('[data-mic]').addEventListener('click', function (event) {
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
    if (currentMode !== 'chat' || !event.dataTransfer || event.dataTransfer.files.length === 0) return;
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
    attachmentUrls.forEach(function (url) { URL.revokeObjectURL(url); });
  });

  var menuTriggers = Array.prototype.slice.call(document.querySelectorAll('[data-menu-trigger]'));
  function closeMenus() {
    menuTriggers.forEach(function (trigger) {
      trigger.setAttribute('aria-expanded', 'false');
      var menu = document.querySelector('[data-menu="' + trigger.getAttribute('data-menu-trigger') + '"]');
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
      menu.querySelectorAll('[data-option]').forEach(function (item) { item.setAttribute('aria-checked', String(item === option)); });
      document.querySelector('[data-option-value="' + menu.getAttribute('data-menu') + '"]').textContent = option.getAttribute('data-option');
      closeMenus();
    });
  });
  modeTrigger.addEventListener('click', function (event) {
    event.stopPropagation();
    var shouldOpen = modeTrigger.getAttribute('aria-expanded') !== 'true';
    closeMenus();
    if (shouldOpen) openModePopover();
    else closeModePopover(false);
  });
  modeControl.addEventListener('keydown', function (event) {
    var options = Array.prototype.slice.call(modePopover.querySelectorAll('[data-mode-option]'));
    var index = options.indexOf(document.activeElement);
    if (event.key === 'Escape') {
      event.preventDefault();
      closeModePopover(true);
    } else if (!modePopover.hidden && (event.key === 'ArrowDown' || event.key === 'ArrowUp')) {
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
      document.querySelector('[data-browse-file]').setAttribute('aria-expanded', 'false');
    }
  });

  // Chat and Reply share one editor engine and submit through the same form.
  document.querySelectorAll('[data-markdown-editor]').forEach(function (editor) {
    markdown.createEditor(editor, {
      label: editor.getAttribute('aria-label'),
      placeholder: editor.getAttribute('data-placeholder'),
      onChange: syncPrimaryAction,
      onSubmit: function () { composer.requestSubmit(); }
    });
  });

  updateBrowserCard('browser-1');
  setPanelOpen('left', shell.getAttribute('data-left-open') !== 'false');
  setMode(currentMode, false);
  normalizeForViewport();
  syncComposerDockHeight();
  window.requestAnimationFrame(updateThreadScrollButton);
  startTerminalStream(terminalEntry('terminal-1'));
})();
