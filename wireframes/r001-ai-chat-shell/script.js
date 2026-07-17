(function () {
  'use strict';

  var shell = document.querySelector('[data-panel-shell]');
  if (!shell) return;

  var narrowQuery = window.matchMedia('(max-width: 859.98px)');
  var minimumWidth = 180;
  var maximumWidth = 420;

  function panelFor(side) {
    return document.querySelector('.side-panel--' + side);
  }

  function toggleFor(side) {
    return document.querySelector('[data-panel-toggle="' + side + '"]');
  }

  function syncPanel(side, isOpen) {
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
      syncPanel('left', false);
      syncPanel('right', false);
    }
  }

  shell.addEventListener('click', function (event) {
    var toggle = event.target.closest('[data-panel-toggle]');
    if (!toggle) return;
    var side = toggle.getAttribute('data-panel-toggle');
    var isOpen = shell.getAttribute('data-' + side + '-open') !== 'false';
    if (narrowQuery.matches && !isOpen) {
      syncPanel(side === 'left' ? 'right' : 'left', false);
    }
    syncPanel(side, !isOpen);
  });

  shell.addEventListener('pointerdown', function (event) {
    var handle = event.target.closest('[data-panel-resize]');
    if (!handle || narrowQuery.matches) return;
    var side = handle.getAttribute('data-panel-resize');
    var bounds = shell.getBoundingClientRect();
    event.preventDefault();
    shell.setAttribute('data-resizing', side);
    document.documentElement.style.cursor = 'col-resize';

    function movePanel(moveEvent) {
      var width = side === 'left'
        ? moveEvent.clientX - bounds.left
        : bounds.right - moveEvent.clientX;
      setPanelWidth(side, width);
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
      syncPanel('left', false);
      syncPanel('right', false);
    }
  });

  narrowQuery.addEventListener('change', normalizeForViewport);
  normalizeForViewport();

  var menuTriggers = Array.prototype.slice.call(document.querySelectorAll('[data-menu-trigger]'));

  function closeMenus(focusTrigger) {
    menuTriggers.forEach(function (trigger) {
      var name = trigger.getAttribute('data-menu-trigger');
      var menu = document.querySelector('[data-menu="' + name + '"]');
      var wasOpen = trigger.getAttribute('aria-expanded') === 'true';
      trigger.setAttribute('aria-expanded', 'false');
      if (menu) menu.hidden = true;
      if (focusTrigger && wasOpen) trigger.focus();
    });
  }

  menuTriggers.forEach(function (trigger) {
    trigger.addEventListener('click', function () {
      var name = trigger.getAttribute('data-menu-trigger');
      var menu = document.querySelector('[data-menu="' + name + '"]');
      var willOpen = trigger.getAttribute('aria-expanded') !== 'true';
      closeMenus(false);
      trigger.setAttribute('aria-expanded', String(willOpen));
      menu.hidden = !willOpen;
      if (willOpen) menu.querySelector('[aria-checked="true"]').focus();
    });
  });

  document.querySelectorAll('[data-menu]').forEach(function (menu) {
    menu.addEventListener('click', function (event) {
      var option = event.target.closest('[data-option]');
      if (!option) return;
      var name = menu.getAttribute('data-menu');
      menu.querySelectorAll('[data-option]').forEach(function (item) {
        item.setAttribute('aria-checked', String(item === option));
      });
      document.querySelector('[data-option-value="' + name + '"]').textContent = option.getAttribute('data-option');
      closeMenus(false);
      document.querySelector('[data-menu-trigger="' + name + '"]').focus();
    });
  });

  document.addEventListener('click', function (event) {
    if (!event.target.closest('.option-control')) closeMenus(false);
  });

  document.addEventListener('keydown', function (event) {
    if (event.key === 'Escape') closeMenus(true);
  });

  var attachButton = document.querySelector('[data-attach-button]');
  var fileInput = document.querySelector('[data-file-input]');
  var attachmentRow = document.querySelector('[data-attachment-row]');
  var attachmentName = document.querySelector('[data-attachment-name]');
  var removeAttachment = document.querySelector('[data-attachment-remove]');

  attachButton.addEventListener('click', function () {
    fileInput.click();
  });

  fileInput.addEventListener('change', function () {
    var file = fileInput.files && fileInput.files[0];
    if (!file) return;
    attachmentName.textContent = file.name;
    attachmentRow.hidden = false;
  });

  removeAttachment.addEventListener('click', function () {
    fileInput.value = '';
    attachmentName.textContent = '';
    attachmentRow.hidden = true;
    attachButton.focus();
  });

  var micButton = document.querySelector('[data-mic]');
  var promptStatus = document.querySelector('[data-prompt-status]');

  micButton.addEventListener('click', function () {
    var active = micButton.getAttribute('aria-pressed') !== 'true';
    micButton.setAttribute('aria-pressed', String(active));
    micButton.setAttribute('aria-label', active ? 'Stop microphone' : 'Use microphone');
    promptStatus.textContent = active ? 'Listening…' : '';
  });

  var form = document.querySelector('[data-prompt-form]');
  var input = document.querySelector('[data-prompt-input]');
  var send = document.querySelector('[data-send]');
  var thread = document.querySelector('[data-thread]');

  function syncPrompt() {
    send.disabled = input.value.trim().length === 0;
    input.style.height = 'auto';
    input.style.height = Math.min(input.scrollHeight, 160) + 'px';
  }

  function createUserMessage(text) {
    var message = document.createElement('div');
    message.className = 'message message--user';
    var body = document.createElement('div');
    body.className = 'message__body';
    var paragraph = document.createElement('p');
    paragraph.textContent = text;
    body.appendChild(paragraph);
    message.appendChild(body);
    return message;
  }

  function createAssistantMessage() {
    var article = document.createElement('article');
    article.className = 'message message--assistant';
    article.innerHTML = '<div class="message__avatar" aria-hidden="true"><svg viewBox="0 0 24 24"><path d="M12 3l1.4 4.6L18 9l-4.6 1.4L12 15l-1.4-4.6L6 9l4.6-1.4L12 3Z"></path><path d="M18.5 15l.7 2.3 2.3.7-2.3.7-.7 2.3-.7-2.3-2.3-.7 2.3-.7.7-2.3Z"></path></svg></div><div class="message__content"><div class="message__speaker"></div><details class="thinking-process"><summary><span>Worked for 14 sec</span><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m9 18 6-6-6-6"></path></svg></summary><div class="thinking-process__content"><p>I’m matching the new prompt to the active workspace context and preparing a concise response.</p><div class="thinking-process__activity"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M14.7 6.3a4 4 0 0 0-5-5L7.5 3.5l3 3L6 11l-3-3L.8 10.2a4 4 0 0 0 5 5L15.5 5.5l-.8.8Z"></path><path d="m12 12 8.5 8.5M19 22l3-3"></path></svg><span>Read the prompt and updated the conversation</span></div><p>The new response will follow the same model label, activity stream, and bubble treatment.</p><div class="thinking-process__activity"><svg viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="4" width="18" height="14" rx="2"></rect><path d="M8 22h8M12 18v4M8 9l2 2 4-4"></path></svg><span>Verified the new message state</span></div></div></details><div class="message__body"><p>I’ve added that prompt to the thread. We can keep refining the workspace from here.</p></div></div>';
    article.querySelector('.message__speaker').textContent = document.querySelector('[data-option-value="model"]').textContent;
    return article;
  }

  form.addEventListener('submit', function (event) {
    event.preventDefault();
    var value = input.value.trim();
    if (!value) return;
    thread.appendChild(createUserMessage(value));
    input.value = '';
    fileInput.value = '';
    attachmentName.textContent = '';
    attachmentRow.hidden = true;
    syncPrompt();
    thread.scrollTop = thread.scrollHeight;

    window.setTimeout(function () {
      thread.appendChild(createAssistantMessage());
      thread.scrollTop = thread.scrollHeight;
    }, 450);
  });

  input.addEventListener('input', syncPrompt);
  input.addEventListener('keydown', function (event) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      form.requestSubmit();
    }
  });

  syncPrompt();
})();
