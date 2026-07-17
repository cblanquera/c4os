(function () {
  'use strict';

  var shell = document.querySelector('[data-panel-shell]');
  if (!shell) return;

  var thread = document.querySelector('[data-thread]');
  var composer = document.querySelector('[data-composer]');
  var modeControl = document.querySelector('[data-mode-control]');
  var modeTrigger = document.querySelector('[data-mode-trigger]');
  var modePopover = document.querySelector('[data-mode-popover]');
  var replyStrip = document.querySelector('[data-reply-strip]');
  var replyPanel = document.querySelector('[data-reply-panel]');
  var replyInput = document.querySelector('[data-reply-input]');
  var primaryAction = document.querySelector('[data-primary-action]');
  var promptStatus = document.querySelector('[data-prompt-status]');
  var copyNotifier = document.querySelector('[data-copy-notifier]');
  var rightTabs = document.querySelector('[data-artifact-tabs]');
  var rightViewer = document.querySelector('[data-artifact-viewer]');
  var narrowQuery = window.matchMedia('(max-width: 859.98px)');
  var minimumWidth = 180;
  var maximumWidth = 480;
  var currentMode = ['chat', 'files', 'browser', 'terminal'].indexOf(location.hash.slice(1)) > -1 ? location.hash.slice(1) : 'chat';
  var replyTarget = null;
  var messageCounter = 10;
  var artifactCounter = 10;
  var copyTimer;
  var openTabs = [];
  var activeTab = null;

  var icons = {
    copy: '<svg viewBox="0 0 24 24" aria-hidden="true"><rect x="9" y="9" width="12" height="12" rx="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>',
    reply: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m9 17-5-5 5-5"></path><path d="M20 18v-2a4 4 0 0 0-4-4H4"></path></svg>',
    expand: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7"></path></svg>',
    close: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m18 6-12 12M6 6l12 12"></path></svg>',
    edit: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 20h9"></path><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L8 18l-4 1 1-4Z"></path></svg>',
    save: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m20 6-11 11-5-5"></path></svg>'
  };

  var artifacts = {
    'browser-1': { id: 'browser-1', type: 'browser', title: 'Example Domain', url: 'https://example.com', history: ['https://example.com'], historyIndex: 0, status: 'Ready' },
    'file-1': { id: 'file-1', type: 'file', title: 'notes.md', path: 'notes.md', content: '# Project notes\n\nThe desktop shell keeps chat at the center.\nArtifact modes extend the same conversation.', version: 1, status: 'Read only', proposedContent: '', editing: false }
  };

  var terminalSession = {
    id: 'shell-1',
    cwd: '~/project',
    entries: [{ id: 'terminal-1', command: 'ls', output: 'index.html  notes.md  src  wireframes', exit: 0 }]
  };

  function escapeHtml(value) {
    return String(value).replace(/[&<>'"]/g, function (character) {
      return { '&': '&amp;', '<': '&lt;', '>': '&gt;', "'": '&#39;', '"': '&quot;' }[character];
    });
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

  function setPanelWidth(side, width) {
    var value = Math.round(Math.max(minimumWidth, Math.min(maximumWidth, width)));
    shell.style.setProperty('--' + side + '-panel-width', value + 'px');
    var handle = document.querySelector('[data-panel-resize="' + side + '"]');
    if (handle) handle.setAttribute('aria-valuenow', String(value));
  }

  function normalizeForViewport() {
    if (narrowQuery.matches) {
      setPanelOpen('left', false);
      setPanelOpen('right', false);
    }
  }

  shell.addEventListener('pointerdown', function (event) {
    var handle = event.target.closest('[data-panel-resize]');
    if (!handle || narrowQuery.matches) return;
    var side = handle.getAttribute('data-panel-resize');
    var bounds = shell.getBoundingClientRect();
    event.preventDefault();
    shell.setAttribute('data-resizing', side);
    document.documentElement.style.cursor = 'col-resize';

    function movePanel(moveEvent) {
      setPanelWidth(side, side === 'left' ? moveEvent.clientX - bounds.left : bounds.right - moveEvent.clientX);
    }

    function stopResize() {
      shell.removeAttribute('data-resizing');
      document.documentElement.style.cursor = '';
      document.removeEventListener('pointermove', movePanel);
      document.removeEventListener('pointerup', stopResize);
      document.removeEventListener('pointercancel', stopResize);
    }

    document.addEventListener('pointermove', movePanel);
    document.addEventListener('pointerup', stopResize);
    document.addEventListener('pointercancel', stopResize);
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
      if (event.key === 'End') current = maximumWidth;
      if (delta || event.key === 'Home' || event.key === 'End') {
        event.preventDefault();
        setPanelWidth(side, current + delta);
      }
    }
    if (event.key === 'Escape' && narrowQuery.matches && !event.target.closest('[data-menu]')) {
      setPanelOpen('left', false);
      setPanelOpen('right', false);
    }
  });

  narrowQuery.addEventListener('change', normalizeForViewport);

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
    var labels = { chat: 'Send message', files: 'Open file', browser: 'Open webpage', terminal: 'Run command' };
    primaryAction.setAttribute('aria-label', labels[mode]);
    primaryAction.title = labels[mode];
    if (updateHash) history.replaceState(null, '', '#' + mode);
    syncPrimaryAction();
  }

  function syncPrimaryAction() {
    var input = replyTarget ? replyInput : inputForMode(currentMode);
    primaryAction.disabled = !input || input.value.trim().length === 0;
    if (input && input.tagName === 'TEXTAREA') {
      input.style.height = 'auto';
      input.style.height = Math.min(input.scrollHeight, 150) + 'px';
    }
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
    replyInput.value = '';
    syncPrimaryAction();
    replyInput.focus();
  }

  function cancelReply() {
    replyTarget = null;
    replyStrip.hidden = true;
    replyPanel.hidden = true;
    modeControl.hidden = false;
    setMode(currentMode, false);
    inputForMode(currentMode).focus();
  }

  function actionsMarkup(kind, id, title, excerpt) {
    return '<div class="' + (kind === 'message' ? 'message-actions' : 'artifact-actions') + '" aria-label="Actions">' +
      '<button type="button" data-copy data-copy-kind="' + (kind === 'message' ? 'message' : 'artifact') + '" aria-label="Copy" title="Copy">' + icons.copy + '</button>' +
      '<button type="button" data-reply data-target-kind="' + kind + '" data-target-id="' + escapeHtml(id) + '" data-target-title="' + escapeHtml(title) + '" data-target-excerpt="' + escapeHtml(excerpt) + '" aria-label="Reply" title="Reply">' + icons.reply + '</button>' +
      '</div>';
  }

  function expandMarkup(id) {
    return '<div class="artifact-header-actions"><button type="button" data-expand="' + escapeHtml(id) + '" aria-label="Expand artifact" title="Expand">' + icons.expand + '</button></div>';
  }

  function browserToolbarMarkup(id, url) {
    return '<header class="artifact__header browser-toolbar">' +
      '<button type="button" data-browser-action="back" aria-label="Back" disabled><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m15 18-6-6 6-6"></path></svg></button>' +
      '<button type="button" data-browser-action="forward" aria-label="Forward" disabled><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m9 18 6-6-6-6"></path></svg></button>' +
      '<button type="button" data-browser-action="refresh" aria-label="Refresh"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M20 11a8 8 0 1 0-2.3 5.7L20 14"></path><path d="M20 6v5h-5"></path></svg></button>' +
      '<label class="sr-only">Current web address</label><input data-browser-address value="' + escapeHtml(url) + '" readonly>' + expandMarkup(id) + '</header>';
  }

  function fileHeaderActionsMarkup(id, title) {
    return '<div class="artifact-header-actions"><button type="button" data-file-edit aria-label="Edit ' + escapeHtml(title) + '" title="Edit">' + icons.edit + '</button>' +
      '<button type="button" data-file-cancel aria-label="Cancel editing ' + escapeHtml(title) + '" title="Cancel" hidden>' + icons.close + '</button>' +
      '<button type="button" data-file-save aria-label="Save ' + escapeHtml(title) + '" title="Save" hidden>' + icons.save + '</button>' +
      '<button type="button" data-expand="' + escapeHtml(id) + '" aria-label="Expand file artifact" title="Expand">' + icons.expand + '</button></div>';
  }

  function lineNumbers(value) {
    return value.split('\n').map(function (_, index) { return index + 1; }).join('\n');
  }

  function fileEditorMarkup(title, content) {
    return '<div class="file-editor" data-file-editor hidden><pre class="file-editor__lines" data-file-lines aria-hidden="true">' + lineNumbers(content) + '</pre><textarea data-file-editor-input aria-label="Edit ' + escapeHtml(title) + '" spellcheck="false">' + escapeHtml(content) + '</textarea></div>';
  }

  function assistantAvatarMarkup() {
    return '<div class="message__avatar" aria-hidden="true"><svg viewBox="0 0 24 24"><path d="M12 3l1.4 4.6L18 9l-4.6 1.4L12 15l-1.4-4.6L6 9l4.6-1.4L12 3Z"></path><path d="M18.5 15l.7 2.3 2.3.7-2.3.7-.7 2.3-.7-2.3-2.3-.7 2.3-.7.7-2.3Z"></path></svg></div>';
  }

  function artifactAvatarMarkup(type) {
    var shapes = {
      browser: '<circle cx="12" cy="12" r="9"></circle><path d="M3 12h18M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18"></path>',
      file: '<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"></path><path d="M14 2v6h6M8 13h8M8 17h6"></path>',
      terminal: '<rect x="3" y="4" width="18" height="16" rx="2"></rect><path d="m7 9 3 3-3 3M13 15h4"></path>'
    };
    return '<div class="message__avatar" aria-hidden="true"><svg viewBox="0 0 24 24">' + shapes[type] + '</svg></div>';
  }

  function thinkingMarkup(duration, summary, activity, className) {
    return '<details class="thinking-process' + (className ? ' ' + className : '') + '"><summary><span>Worked for ' + escapeHtml(duration) + '</span><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m9 18 6-6-6-6"></path></svg></summary><div class="thinking-process__content"><p>' + escapeHtml(summary) + '</p><div class="thinking-process__activity"><svg viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="4" width="18" height="14" rx="2"></rect><path d="M8 9l2 2 4-4"></path></svg><span>' + escapeHtml(activity) + '</span></div></div></details>';
  }

  function artifactPreambleMarkup(duration, summary, activity, type) {
    var model = document.querySelector('[data-option-value="model"]').textContent;
    return thinkingMarkup(duration, summary, activity, 'artifact-thinking') + '<div class="artifact-model-row">' + artifactAvatarMarkup(type) + '<div class="message__speaker">' + escapeHtml(model) + '</div></div>';
  }

  function appendUserMessage(text, reference) {
    messageCounter += 1;
    var node = document.createElement('div');
    node.className = 'message message--user';
    node.setAttribute('data-message-id', 'message-' + messageCounter);
    node.innerHTML = (reference ? '<div class="message-reference">Replying to ' + escapeHtml(reference) + '</div>' : '') +
      '<div class="message__body"><p>' + escapeHtml(text) + '</p></div>' +
      actionsMarkup('message', 'message-' + messageCounter, 'You', text.slice(0, 70));
    thread.appendChild(node);
    return node;
  }

  function appendAssistantMessage(text) {
    messageCounter += 1;
    var model = document.querySelector('[data-option-value="model"]').textContent;
    var node = document.createElement('article');
    node.className = 'message message--assistant';
    node.setAttribute('data-message-id', 'message-' + messageCounter);
    node.innerHTML = thinkingMarkup('12 sec', 'I reviewed the referenced context and prepared the requested next step.', 'Updated the conversation') + '<div class="message-model-row">' + assistantAvatarMarkup() + '<div class="message__speaker">' + escapeHtml(model) + '</div></div>' +
      '<div class="message__content"><div class="message__body response-container"><p>' + escapeHtml(text) + '</p></div>' + actionsMarkup('message', 'message-' + messageCounter, model, text.slice(0, 70)) + '</div>';
    thread.appendChild(node);
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
    if (activeTab === id) renderViewer();
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
    var node = document.createElement('article');
    node.className = 'artifact artifact--browser';
    node.setAttribute('data-artifact-card', '');
    node.setAttribute('data-artifact-id', id);
    node.setAttribute('data-artifact-type', 'browser');
    node.tabIndex = 0;
    node.innerHTML = artifactPreambleMarkup('8 sec', 'I prepared this response artifact from the requested webpage.', 'Created the browser artifact', 'browser') + '<div class="artifact-surface response-container">' + browserToolbarMarkup(id, normalized) +
      '<div class="browser-preview" data-browser-preview>' + browserPreview(artifacts[id]) + '</div><footer class="artifact__footer"><span data-artifact-status>Ready</span><span data-browser-history>1 navigation</span></footer></div>' + actionsMarkup('browser', id, 'Browser · ' + artifacts[id].title, normalized);
    thread.appendChild(node);
  }

  function updateFileCard(id) {
    var state = artifacts[id];
    var card = document.querySelector('[data-artifact-id="' + id + '"]');
    if (!state || !card) return;
    card.querySelector('[data-file-preview]').textContent = state.proposedContent || state.content;
    card.querySelector('[data-file-version]').textContent = 'Version ' + state.version;
    card.querySelector('[data-artifact-meta]').textContent = 'Markdown · Version ' + state.version;
    card.querySelector('[data-artifact-status]').textContent = state.status;
    card.querySelector('[data-file-approval]').hidden = state.editing || !state.proposedContent;
    card.querySelector('[data-file-preview]').hidden = state.editing;
    card.querySelector('[data-file-editor]').hidden = !state.editing;
    card.querySelector('[data-file-edit]').hidden = state.editing;
    card.querySelector('[data-file-cancel]').hidden = !state.editing;
    card.querySelector('[data-file-save]').hidden = !state.editing;
    if (state.editing) card.querySelector('[data-artifact-status]').textContent = 'Editing';
    if (activeTab === id) renderViewer();
  }

  function beginFileEdit(card) {
    var state = artifacts[card.getAttribute('data-artifact-id')];
    if (!state) return;
    state.editing = true;
    var editor = card.querySelector('[data-file-editor-input]');
    editor.value = state.proposedContent || state.content;
    card.querySelector('[data-file-lines]').textContent = lineNumbers(editor.value);
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
    state.content = card.querySelector('[data-file-editor-input]').value;
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
    artifacts[id] = { id: id, type: 'file', title: title, path: path, content: '# ' + title + '\n\nThis file is open in read-only mode.\nReply to request a change.', version: 1, status: 'Read only', proposedContent: '', editing: false };
    var node = document.createElement('article');
    node.className = 'artifact artifact--file';
    node.setAttribute('data-artifact-card', '');
    node.setAttribute('data-artifact-id', id);
    node.setAttribute('data-artifact-type', 'file');
    node.tabIndex = 0;
    node.innerHTML = artifactPreambleMarkup('11 sec', 'I read the requested file and prepared a read-only artifact.', 'Loaded the file response', 'file') + '<div class="artifact-surface response-container"><header class="artifact__header"><div class="artifact__identity"><div><strong data-artifact-title>' + escapeHtml(title) + '</strong><span data-artifact-meta>Markdown · Version 1</span></div></div>' + fileHeaderActionsMarkup(id, title) + '</header><pre class="file-preview" data-file-preview>' + escapeHtml(artifacts[id].content) + '</pre>' + fileEditorMarkup(title, artifacts[id].content) + '<div class="file-approval" data-file-approval hidden><div><strong>2 edits proposed</strong><span>Typography and punctuation</span></div><div><button type="button" data-file-reject>Reject</button><button type="button" data-file-approve>Approve &amp; save</button></div></div><footer class="artifact__footer"><span data-artifact-status>Read only</span><span data-file-version>Version 1</span></footer></div>' + actionsMarkup('file', id, 'File · ' + title, path);
    thread.appendChild(node);
  }

  function terminalOutput(command) {
    if (/^ls\b/.test(command)) return 'index.html  notes.md  src  wireframes';
    if (/^pwd\b/.test(command)) return '/Users/demo/project';
    if (/du|how big|size/i.test(command)) return '42M\t.';
    return 'Command completed successfully.';
  }

  function createTerminalArtifact(command) {
    artifactCounter += 1;
    var id = 'terminal-' + artifactCounter;
    var entry = { id: id, command: command, output: terminalOutput(command), exit: 0 };
    terminalSession.entries.push(entry);
    var node = document.createElement('article');
    node.className = 'artifact artifact--terminal';
    node.setAttribute('data-artifact-card', '');
    node.setAttribute('data-artifact-id', id);
    node.setAttribute('data-artifact-type', 'terminal');
    node.setAttribute('data-terminal-session', terminalSession.id);
    node.tabIndex = 0;
    node.innerHTML = artifactPreambleMarkup('6 sec', 'I ran the requested command in the active terminal session.', 'Captured the command output', 'terminal') + '<div class="artifact-surface response-container"><header class="artifact__header"><div class="artifact__identity"><div><strong data-artifact-title>$ ' + escapeHtml(command) + '</strong><span data-artifact-meta>' + escapeHtml(terminalSession.cwd) + ' · ' + terminalSession.id + '</span></div></div>' + expandMarkup('terminal-session') + '</header><pre class="terminal-output" data-terminal-output>' + escapeHtml(entry.output) + '</pre><footer class="artifact__footer"><span>Exit 0</span><span>' + terminalSession.id + '</span></footer></div>' + actionsMarkup('terminal', id, 'Terminal · ' + command, entry.output);
    thread.appendChild(node);
    if (activeTab === 'terminal-session') renderViewer();
  }

  function submitReply(text) {
    var target = replyTarget;
    appendUserMessage(text, target.title);
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
      createTerminalArtifact(/big|size/i.test(text) ? 'du -sh .' : text);
      appendAssistantMessage('I ran a follow-up command in the same terminal session and appended a new result.');
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
    var value = input.value.trim();
    if (!value) return;
    if (currentMode === 'chat') {
      appendUserMessage(value);
      appendAssistantMessage('I’ve added that prompt to the conversation.');
      input.value = '';
    } else if (currentMode === 'files') {
      appendUserMessage('Open ' + value);
      createFileArtifact(value);
    } else if (currentMode === 'browser') {
      appendUserMessage('Open ' + value);
      createBrowserArtifact(value);
    } else {
      appendUserMessage('$ ' + value);
      createTerminalArtifact(value);
      input.value = '';
    }
    setMode(currentMode, false);
    scrollThread();
  }

  function scrollThread() {
    window.setTimeout(function () { thread.scrollTop = thread.scrollHeight; }, 20);
  }

  function contextualCopy(button) {
    var container = button.closest('[data-artifact-card], .message');
    var text = '';
    if (container.matches('.message')) text = container.querySelector('.message__body').innerText.trim();
    else {
      var id = container.getAttribute('data-artifact-id');
      var type = container.getAttribute('data-artifact-type');
      if (type === 'browser') text = artifacts[id].url;
      if (type === 'file') text = artifacts[id].content;
      if (type === 'terminal') {
        var entry = terminalSession.entries.filter(function (item) { return item.id === id; })[0];
        text = '$ ' + entry.command + '\n' + entry.output;
      }
    }
    var fallback = function () {
      var area = document.createElement('textarea');
      area.value = text;
      document.body.appendChild(area);
      area.select();
      document.execCommand('copy');
      area.remove();
    };
    if (navigator.clipboard && navigator.clipboard.writeText) navigator.clipboard.writeText(text).catch(fallback);
    else fallback();
    clearTimeout(copyTimer);
    copyNotifier.hidden = false;
    copyNotifier.textContent = 'Copied';
    copyTimer = window.setTimeout(function () { copyNotifier.hidden = true; }, 1400);
  }

  function tabTitle(id) {
    if (id === 'terminal-session') return 'Terminal';
    return artifacts[id] ? artifacts[id].title : 'Artifact';
  }

  function openArtifact(id) {
    if (openTabs.indexOf(id) === -1) openTabs.push(id);
    activeTab = id;
    setPanelOpen('right', true);
    renderTabs();
    renderViewer();
  }

  function closeTab(id) {
    var index = openTabs.indexOf(id);
    if (index === -1) return;
    openTabs.splice(index, 1);
    if (activeTab === id) activeTab = openTabs[Math.max(0, index - 1)] || openTabs[0] || null;
    renderTabs();
    renderViewer();
    if (!openTabs.length) setPanelOpen('right', false);
  }

  function renderTabs() {
    rightTabs.innerHTML = openTabs.map(function (id) {
      return '<div class="artifact-tab" role="tab" data-tab-id="' + escapeHtml(id) + '" aria-selected="' + String(activeTab === id) + '"><button class="artifact-tab__select" type="button" data-select-tab="' + escapeHtml(id) + '">' + escapeHtml(tabTitle(id)) + '</button><button type="button" data-close-tab="' + escapeHtml(id) + '" aria-label="Close ' + escapeHtml(tabTitle(id)) + ' tab">' + icons.close + '</button></div>';
    }).join('');
  }

  function renderViewer() {
    if (!activeTab) {
      rightViewer.innerHTML = '<div class="artifact-pane__empty">Select an artifact from the thread.</div>';
      return;
    }
    if (activeTab === 'terminal-session') {
      rightViewer.innerHTML = '<section class="pane-view" role="tabpanel"><div class="pane-view__title"><div><strong>Terminal</strong><span>' + escapeHtml(terminalSession.cwd) + ' · ' + terminalSession.id + '</span></div></div><div class="pane-terminal-history">' + terminalSession.entries.map(function (entry) { return '<div class="pane-terminal-entry"><strong>$ ' + escapeHtml(entry.command) + '</strong>\n' + escapeHtml(entry.output) + '</div>'; }).join('') + '</div></section>';
      return;
    }
    var state = artifacts[activeTab];
    if (!state) return;
    if (state.type === 'browser') {
      rightViewer.innerHTML = '<section class="pane-view" role="tabpanel"><div class="pane-browser-bar"><button type="button" data-pane-browser="back" aria-label="Back"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m15 18-6-6 6-6"></path></svg></button><button type="button" data-pane-browser="forward" aria-label="Forward"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m9 18 6-6-6-6"></path></svg></button><button type="button" data-pane-browser="refresh" aria-label="Refresh"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M20 11a8 8 0 1 0-2.3 5.7L20 14"></path><path d="M20 6v5h-5"></path></svg></button><label class="sr-only">Web address</label><input data-pane-browser-address value="' + escapeHtml(state.url) + '" aria-label="Web address"></div><div class="pane-browser-page"><small>' + (state.url.indexOf('dashboard') > -1 ? 'WORKSPACE' : 'EXAMPLE DOMAIN') + '</small><h2>' + escapeHtml(state.title) + '</h2><p>This interactive browser view stays connected to the inline artifact.</p></div></section>';
    } else {
      rightViewer.innerHTML = '<section class="pane-view" role="tabpanel"><div class="pane-view__title"><div><strong>' + escapeHtml(state.title) + '</strong><span>' + escapeHtml(state.path) + ' · Version ' + state.version + '</span></div><div class="pane-view__actions"><button class="pane-action" type="button" data-pane-file-edit>Edit</button><button class="pane-action pane-action--primary" type="button" data-pane-file-save hidden>Save</button></div></div><textarea class="pane-file-editor" data-pane-file-editor readonly spellcheck="false">' + escapeHtml(state.content) + '</textarea></section>';
    }
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

  shell.addEventListener('click', function (event) {
    var panelToggle = event.target.closest('[data-panel-toggle]');
    if (panelToggle) {
      var side = panelToggle.getAttribute('data-panel-toggle');
      var open = shell.getAttribute('data-' + side + '-open') !== 'false';
      if (narrowQuery.matches && !open) setPanelOpen(side === 'left' ? 'right' : 'left', false);
      setPanelOpen(side, !open);
      return;
    }
    var mode = event.target.closest('[data-mode-option]');
    if (mode) { setMode(mode.getAttribute('data-mode-option'), true); inputForMode(currentMode).focus(); return; }
    var reply = event.target.closest('[data-reply]');
    if (reply) { startReply(reply); return; }
    if (event.target.closest('[data-reply-cancel]')) { cancelReply(); return; }
    var copy = event.target.closest('[data-copy]');
    if (copy) { contextualCopy(copy); return; }
    var expand = event.target.closest('[data-expand]');
    if (expand) { openArtifact(expand.getAttribute('data-expand')); return; }
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
      rejected.status = 'Read only';
      updateFileCard(rejected.id);
      return;
    }
    if (event.target.closest('[data-browse-file]')) { document.querySelector('[data-file-picker]').click(); return; }
    var selectTab = event.target.closest('[data-select-tab]');
    if (selectTab) { activeTab = selectTab.getAttribute('data-select-tab'); renderTabs(); renderViewer(); return; }
    var close = event.target.closest('[data-close-tab]');
    if (close) { closeTab(close.getAttribute('data-close-tab')); return; }
    if (event.target.closest('[data-close-artifact-pane]')) { setPanelOpen('right', false); return; }
    if (event.target.closest('[data-pane-file-edit]')) {
      var editor = document.querySelector('[data-pane-file-editor]');
      editor.readOnly = false;
      document.querySelector('[data-pane-file-save]').hidden = false;
      editor.focus();
      return;
    }
    if (event.target.closest('[data-pane-file-save]')) {
      var activeFile = artifacts[activeTab];
      activeFile.content = document.querySelector('[data-pane-file-editor]').value;
      activeFile.version += 1;
      activeFile.status = 'Saved';
      updateFileCard(activeTab);
      return;
    }
    var paneBrowser = event.target.closest('[data-pane-browser]');
    if (paneBrowser && artifacts[activeTab]) {
      var fakeCard = document.querySelector('[data-artifact-id="' + activeTab + '"]');
      handleBrowserAction(fakeCard, paneBrowser.getAttribute('data-pane-browser'));
    }
  });

  shell.addEventListener('keydown', function (event) {
    if (event.key === 'Enter' && event.target.matches('[data-pane-browser-address]') && artifacts[activeTab]) {
      navigateBrowser(activeTab, event.target.value, false);
    }
    if (event.key === 'Enter' && !event.shiftKey && (event.target.matches('[data-mode-input], [data-reply-input]'))) {
      event.preventDefault();
      composer.requestSubmit();
    }
  });

  shell.addEventListener('input', function (event) {
    if (event.target.matches('[data-file-editor-input]')) {
      event.target.closest('[data-file-editor]').querySelector('[data-file-lines]').textContent = lineNumbers(event.target.value);
    }
  });

  composer.addEventListener('input', syncPrimaryAction);
  composer.addEventListener('submit', function (event) {
    event.preventDefault();
    if (replyTarget) {
      var value = replyInput.value.trim();
      if (!value) return;
      submitReply(value);
      replyInput.value = '';
      scrollThread();
    } else submitMode();
    syncPrimaryAction();
  });

  document.querySelector('[data-file-picker]').addEventListener('change', function (event) {
    var file = event.target.files && event.target.files[0];
    if (file) inputForMode('files').value = file.name;
    syncPrimaryAction();
  });

  document.querySelector('[data-mic]').addEventListener('click', function (event) {
    var button = event.currentTarget;
    var active = button.getAttribute('aria-pressed') !== 'true';
    button.setAttribute('aria-pressed', String(active));
    promptStatus.textContent = active ? 'Listening…' : '';
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
  });

  updateBrowserCard('browser-1');
  setPanelOpen('left', shell.getAttribute('data-left-open') !== 'false');
  setPanelOpen('right', shell.getAttribute('data-right-open') !== 'false');
  setMode(currentMode, false);
  normalizeForViewport();
})();
