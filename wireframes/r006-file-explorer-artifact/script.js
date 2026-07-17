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
  var threadHost = document.querySelector('[data-thread-host]');
  var threadScrollButton = document.querySelector('[data-thread-scroll-bottom]');
  var focusViewer = document.querySelector('[data-focus-viewer]');
  var leftChatPane = document.querySelector('[data-left-chat-pane]');
  var leftChatPaneThread = document.querySelector('[data-left-chat-pane-thread]');
  var leftChatPaneResize = document.querySelector('[data-left-chat-pane-resize]');
  var fileSourcePopover = document.querySelector('[data-file-source-popover]');
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

  var icons = {
    copy: '<svg viewBox="0 0 24 24" aria-hidden="true"><rect x="9" y="9" width="12" height="12" rx="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>',
    reply: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m9 17-5-5 5-5"></path><path d="M20 18v-2a4 4 0 0 0-4-4H4"></path></svg>',
    expand: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7"></path></svg>',
    close: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m18 6-12 12M6 6l12 12"></path></svg>',
    edit: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 20h9"></path><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L8 18l-4 1 1-4Z"></path></svg>',
    save: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m20 6-11 11-5-5"></path></svg>',
    trash: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M3 6h18M8 6V4h8v2M19 6l-1 14H6L5 6M10 10v6M14 10v6"></path></svg>',
    folder: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M3 7h6l2 2h10v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z"></path></svg>'
  };

  var artifacts = {
    'browser-1': { id: 'browser-1', type: 'browser', title: 'Example Domain', url: 'https://example.com', history: ['https://example.com'], historyIndex: 0, status: 'Ready' },
    'file-1': { id: 'file-1', type: 'file', title: 'notes.md', path: '~/project/docs/notes.md', content: sampleFileContent(), version: 1, status: 'Viewing', proposedContent: '', editing: false }
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
    var labels = { chat: 'Send message', files: fileSelectionKind === 'folder' ? 'Open folder' : 'Open file', browser: 'Open webpage', terminal: 'Run command' };
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

  function ensureUserMessageActions() {
    thread.querySelectorAll('.message--user').forEach(function (message) {
      if (message.querySelector('.message-actions')) return;
      var id = message.getAttribute('data-message-id') || 'message-' + (++messageCounter);
      var body = message.querySelector('.message__body');
      var text = body ? body.innerText.trim() : '';
      message.setAttribute('data-message-id', id);
      message.insertAdjacentHTML('beforeend', actionsMarkup('message', id, 'You', text.slice(0, 70)));
    });
  }

  ensureUserMessageActions();

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

  function explorerFolderBodyMarkup(path, focused) {
    var count = explorerEntries(path).length;
    return explorerRowsMarkup(path) + (focused ? '' : '<footer class="artifact__footer"><span>Browsing</span><span>' + count + ' items</span></footer>');
  }

  function explorerFileActionsMarkup(id, file, focused) {
    return '<div class="artifact-header-actions pane-view__actions">' +
      '<button type="button" data-explorer-file-edit aria-label="Edit ' + escapeHtml(file.title) + '" title="Edit"' + (file.editing ? ' hidden' : '') + '>' + icons.edit + '</button>' +
      '<button type="button" data-explorer-file-cancel aria-label="Discard changes to ' + escapeHtml(file.title) + '" title="Discard changes"' + (file.editing ? '' : ' hidden') + '>' + icons.trash + '</button>' +
      '<button type="button" data-explorer-file-save aria-label="Save ' + escapeHtml(file.title) + '" title="Save"' + (file.editing ? '' : ' hidden') + '>' + icons.save + '</button>' +
      (focused
        ? '<button class="pane-action--icon" type="button" data-close-focused-artifact aria-label="Close File artifact" title="Close">' + icons.close + '</button>'
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
    var wrapper = focused ? 'section' : 'div';
    var className = focused
      ? 'pane-view focus-artifact ' + (file ? 'focus-artifact--file focus-artifact--explorer-file' : 'focus-artifact--folder')
      : 'artifact-surface response-container';
    var headerClass = focused ? 'focus-view__header' : 'artifact__header file-explorer-header';
    var headerActions = file
      ? explorerFileActionsMarkup(id, file, focused)
      : (focused
        ? '<div class="pane-view__actions"><button class="pane-action pane-action--icon" type="button" data-close-focused-artifact aria-label="Close File Explorer artifact" title="Close">' + icons.close + '</button></div>'
        : expandMarkup(id));
    var body = file ? explorerFileBodyMarkup(file, focused) : explorerFolderBodyMarkup(options.path, focused);
    return '<' + wrapper + ' class="' + className + '" aria-label="' + (file ? 'Open file' : 'File Explorer artifact') + '"><header class="' + headerClass + '"><div class="focus-view__breadcrumbs">' + breadcrumbMarkup(file ? file.path : options.path, Boolean(file)) + '</div>' + headerActions + '</header>' + body + '</' + wrapper + '>';
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
    var surface = card.querySelector('.artifact-surface');
    surface.outerHTML = state.previewFile ? explorerFileSurfaceMarkup(id, state.previewFile) : folderSurfaceMarkup(id, state.path);
  }

  function convertFileCardToFolder(card, state, path) {
    state.type = 'folder';
    state.path = path;
    state.title = path.split('/').filter(Boolean).pop() || 'project';
    state.status = 'Browsing';
    state.previewFile = null;
    state.editing = false;
    card.classList.remove('artifact--file');
    card.classList.add('artifact--folder');
    card.setAttribute('data-artifact-type', 'folder');
    card.setAttribute('aria-label', 'File Explorer artifact ' + state.title);
    var avatar = card.querySelector('.artifact-model-row .message__avatar');
    if (avatar) avatar.outerHTML = artifactAvatarMarkup('folder');
    var surface = card.querySelector('.artifact-surface');
    if (surface) surface.outerHTML = folderSurfaceMarkup(state.id, path);
    var reply = card.querySelector('[data-reply]');
    if (reply) {
      reply.setAttribute('data-target-kind', 'folder');
      reply.setAttribute('data-target-title', 'File Explorer · ' + state.title);
      reply.setAttribute('data-target-excerpt', path);
    }
    var actions = card.querySelector('.artifact-actions');
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
      browser: '<circle cx="12" cy="12" r="9"></circle><path d="M3 12h18M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18"></path>',
      file: '<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"></path><path d="M14 2v6h6M8 13h8M8 17h6"></path>',
      folder: '<path d="M3 7h6l2 2h10v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z"></path>',
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
    var node = document.createElement('article');
    node.className = 'artifact artifact--file';
    node.setAttribute('data-artifact-card', '');
    node.setAttribute('data-artifact-id', id);
    node.setAttribute('data-artifact-type', 'file');
    node.tabIndex = 0;
    node.innerHTML = artifactPreambleMarkup('11 sec', 'I opened the requested file for review.', 'Loaded the file response', 'file') + '<div class="artifact-surface response-container"><header class="artifact__header">' + breadcrumbMarkup(artifacts[id].path, true) + fileHeaderActionsMarkup(id, title) + '</header><pre class="file-preview" data-file-preview>' + escapeHtml(artifacts[id].content) + '</pre>' + fileEditorMarkup(title, artifacts[id].content) + '<div class="file-approval" data-file-approval hidden><div><strong>2 edits proposed</strong><span>Typography and punctuation</span></div><div><button type="button" data-file-reject>Reject</button><button type="button" data-file-approve>Approve &amp; save</button></div></div></div>' + actionsMarkup('file', id, 'File · ' + title, path);
    thread.appendChild(node);
  }

  function createFolderArtifact(path) {
    artifactCounter += 1;
    var normalized = path.replace(/\/$/, '') || '~/project';
    var id = 'folder-' + artifactCounter;
    var title = normalized.split('/').filter(Boolean).pop() || 'project';
    artifacts[id] = { id: id, type: 'folder', title: title, path: normalized, status: 'Read only', previewFile: null };
    var node = document.createElement('article');
    node.className = 'artifact artifact--folder';
    node.setAttribute('data-artifact-card', '');
    node.setAttribute('data-artifact-id', id);
    node.setAttribute('data-artifact-type', 'folder');
    node.tabIndex = 0;
    node.setAttribute('aria-label', 'File Explorer artifact ' + title);
    node.innerHTML = artifactPreambleMarkup('7 sec', 'I opened the selected folder as a read-only File Explorer artifact.', 'Listed the folder contents', 'folder') + folderSurfaceMarkup(id, normalized) + actionsMarkup('folder', id, 'File Explorer · ' + title, normalized);
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
    node.innerHTML = artifactPreambleMarkup('6 sec', 'I ran the requested command in the active terminal session.', 'Captured the command output', 'terminal') + '<div class="artifact-surface response-container"><header class="artifact__header"><div class="artifact__identity"><div><strong data-artifact-title>$ ' + escapeHtml(command) + '</strong><span data-artifact-meta>' + escapeHtml(terminalSession.cwd) + ' · ' + terminalSession.id + '</span></div></div></header><pre class="terminal-output" data-terminal-output>' + escapeHtml(entry.output) + '</pre><footer class="artifact__footer"><span>Exit 0</span><span>' + terminalSession.id + '</span></footer></div>' + actionsMarkup('terminal', id, 'Terminal · ' + command, entry.output);
    thread.appendChild(node);
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
      if (fileSelectionKind === 'folder' || /\/$/.test(value) || value.split('/').pop().indexOf('.') === -1) createFolderArtifact(value);
      else createFileArtifact(value);
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

  function contextualCopy(button) {
    var container = button.closest('[data-artifact-card], .message');
    var text = '';
    if (container.matches('.message')) text = container.querySelector('.message__body').innerText.trim();
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
    document.querySelectorAll('.artifact--in-workspace').forEach(function (card) {
      card.classList.remove('artifact--in-workspace');
      var placeholder = card.querySelector('[data-focus-placeholder]');
      if (placeholder) placeholder.remove();
    });
  }

  function setFocusPlaceholder(id) {
    var state = artifacts[id];
    var card = document.querySelector('[data-artifact-id="' + id + '"]');
    if (!state || !card) return;
    card.classList.add('artifact--in-workspace');
    var placeholder = document.createElement('div');
    placeholder.className = 'artifact-focus-placeholder';
    placeholder.setAttribute('data-focus-placeholder', '');
    var label = state.type === 'browser' ? 'Browser' : (state.type === 'folder' ? 'File Explorer' : 'File');
    placeholder.textContent = label + ' open in the workspace · ' + state.title;
    card.querySelector('.artifact-model-row').insertAdjacentElement('afterend', placeholder);
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
      focusViewer.innerHTML = '<section class="pane-view focus-artifact focus-artifact--browser" aria-label="Focused browser artifact"><div class="pane-browser-bar"><button type="button" data-focus-browser="back" aria-label="Back"' + (state.historyIndex <= 0 ? ' disabled' : '') + '><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m15 18-6-6 6-6"></path></svg></button><button type="button" data-focus-browser="forward" aria-label="Forward"' + (state.historyIndex >= state.history.length - 1 ? ' disabled' : '') + '><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m9 18 6-6-6-6"></path></svg></button><button type="button" data-focus-browser="refresh" aria-label="Refresh"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M20 11a8 8 0 1 0-2.3 5.7L20 14"></path><path d="M20 6v5h-5"></path></svg></button><label class="sr-only">Web address</label><input data-focus-browser-address value="' + escapeHtml(state.url) + '" aria-label="Web address"><button class="pane-action--icon" type="button" data-close-focused-artifact aria-label="Close Browser artifact" title="Close">' + icons.close + '</button></div><div class="pane-browser-page"><small>' + (state.url.indexOf('dashboard') > -1 ? 'WORKSPACE' : 'EXAMPLE DOMAIN') + '</small><h2>' + escapeHtml(state.title) + '</h2><p>This expanded Browser artifact uses the entire center workspace while the conversation stays available in the left Chat pane.</p></div></section>';
    } else {
      focusViewer.innerHTML = '<section class="pane-view focus-artifact focus-artifact--file" aria-label="Focused file artifact"><header class="focus-view__header"><div class="focus-view__breadcrumbs">' + breadcrumbMarkup(state.path, true) + '</div><div class="pane-view__actions"><button class="pane-action pane-action--icon" type="button" data-focus-file-edit aria-label="Edit ' + escapeHtml(state.title) + '" title="Edit">' + icons.edit + '</button><button class="pane-action pane-action--icon" type="button" data-focus-file-cancel aria-label="Discard changes to ' + escapeHtml(state.title) + '" title="Discard changes" hidden>' + icons.trash + '</button><button class="pane-action pane-action--icon" type="button" data-focus-file-save aria-label="Save ' + escapeHtml(state.title) + '" title="Save" hidden>' + icons.save + '</button><button class="pane-action pane-action--icon" type="button" data-close-focused-artifact aria-label="Close File artifact" title="Close">' + icons.close + '</button></div></header><div class="focus-file-editor" data-focus-file-surface data-editing="false"><pre class="focus-file-editor__lines" data-focus-file-lines aria-hidden="true" hidden>' + lineNumbers(state.content) + '</pre><div class="pane-file-editor" data-focus-file-editor role="textbox" aria-label="View ' + escapeHtml(state.title) + '" aria-multiline="true" aria-readonly="true" contenteditable="false" spellcheck="false">' + escapeHtml(state.content) + '</div></div></section>';
    }
  }

  function focusArtifact(id) {
    var state = artifacts[id];
    if (!state || ['browser', 'file', 'folder'].indexOf(state.type) === -1) return;
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
    var copy = event.target.closest('[data-copy]');
    if (copy) { contextualCopy(copy); return; }
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
      var explorerDocument = explorerEdit.closest('.artifact-surface, .focus-artifact');
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
    if (event.key === 'Enter' && event.target.matches('[data-focus-browser-address]') && artifacts[focusedArtifact]) {
      navigateBrowser(focusedArtifact, event.target.value, false);
    }
    if (event.key === 'Enter' && !event.shiftKey && (event.target.matches('[data-mode-input], [data-reply-input]'))) {
      event.preventDefault();
      composer.requestSubmit();
    }
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
      var value = replyInput.value.trim();
      if (!value) return;
      submitReply(value);
      replyInput.value = '';
      scrollThread();
    } else submitMode();
    syncPrimaryAction();
  });

  thread.addEventListener('scroll', updateThreadScrollButton, { passive: true });
  threadScrollButton.addEventListener('click', scrollThread);
  window.addEventListener('resize', updateThreadScrollButton);
  if (window.ResizeObserver) new ResizeObserver(updateThreadScrollButton).observe(thread);
  new MutationObserver(function () { window.requestAnimationFrame(updateThreadScrollButton); }).observe(thread, { childList: true });

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
    if (!event.target.closest('.file-source-control')) {
      fileSourcePopover.hidden = true;
      document.querySelector('[data-browse-file]').setAttribute('aria-expanded', 'false');
    }
  });

  updateBrowserCard('browser-1');
  setPanelOpen('left', shell.getAttribute('data-left-open') !== 'false');
  setMode(currentMode, false);
  normalizeForViewport();
  window.requestAnimationFrame(updateThreadScrollButton);
})();
