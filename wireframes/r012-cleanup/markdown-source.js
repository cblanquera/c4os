//modules
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
import { markdown } from '@codemirror/lang-markdown';
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language';
import {
  EditorSelection,
  EditorState,
  Prec,
  Transaction,
} from '@codemirror/state';
import {
  Decoration,
  EditorView,
  ViewPlugin,
  drawSelection,
  keymap,
  placeholder,
} from '@codemirror/view';
import { tags } from '@lezer/highlight';
import MarkdownIt from 'markdown-it';
import taskLists from 'markdown-it-task-lists';

// Editor instances stay private while the shell reads them through host nodes.
const editors = new WeakMap();

// Response Markdown uses one standards-based parser with raw HTML disabled.
const renderer = new MarkdownIt({
  breaks: true,
  html: false,
  linkify: true,
  typographer: false,
}).use(taskLists, { enabled: false, label: false });

// Links rendered in conversation bubbles open without replacing the wireframe.
const renderLinkOpen = renderer.renderer.rules.link_open;
renderer.renderer.rules.link_open = function (
  tokens,
  index,
  options,
  env,
  self,
) {
  tokens[index].attrSet('rel', 'noreferrer');
  tokens[index].attrSet('target', '_blank');
  return renderLinkOpen
    ? renderLinkOpen(tokens, index, options, env, self)
    : self.renderToken(tokens, index, options);
};

// Code fences preserve the established response-bubble header and Copy action.
renderer.renderer.rules.fence = function (tokens, index) {
  const token = tokens[index];
  const language = token.info.trim().split(/\s+/)[0] || 'text';
  return (
    '<div class="markdown-code"><header><span>' +
    renderer.utils.escapeHtml(language) +
    '</span><button data-copy-code title="Copy code" type="button">Copy</button>' +
    '</header><pre><code>' +
    renderer.utils.escapeHtml(token.content) +
    '</code></pre></div>\n'
  );
};

// Tables gain one responsive wrapper without changing Markdown semantics.
renderer.renderer.rules.table_open = function () {
  return '<div class="markdown-table"><table>\n';
};
renderer.renderer.rules.table_close = function () {
  return '</table></div>\n';
};

// The source remains visible while semantic tokens receive lightweight effects.
const markdownHighlight = HighlightStyle.define([
  { tag: tags.heading, fontWeight: '700' },
  { tag: tags.strong, fontWeight: '700' },
  { tag: tags.emphasis, fontStyle: 'italic' },
  { tag: tags.monospace, fontFamily: 'var(--mono)' },
  { tag: tags.quote, color: 'var(--gray-600)', fontStyle: 'italic' },
  { tag: tags.link, color: 'var(--gray-700)' },
  {
    tag: tags.url,
    color: 'var(--link)',
    textDecoration: 'underline',
    textUnderlineOffset: '2px',
  },
  { tag: tags.meta, color: 'var(--gray-500)' },
  { tag: tags.processingInstruction, color: 'var(--gray-500)' },
]);

/** Returns whether a pasted string is one complete web address. */
function isWebUrl(value) {
  return /^(?:https?:\/\/|www\.)[^\s]+$/i.test(String(value || '').trim());
}

/** Wraps or unwraps every selection with one Markdown marker pair. */
function toggleMarker(view, marker) {
  const transaction = view.state.changeByRange(function (range) {
    const document = view.state.doc;
    const markerLength = marker.length;
    const before = document.sliceString(
      Math.max(0, range.from - markerLength),
      range.from,
    );
    const after = document.sliceString(
      range.to,
      Math.min(document.length, range.to + markerLength),
    );

    // remove a matching pair immediately surrounding the selection
    if (before === marker && after === marker) {
      return {
        changes: [
          { from: range.from - markerLength, to: range.from },
          { from: range.to, to: range.to + markerLength },
        ],
        range: EditorSelection.range(
          range.from - markerLength,
          range.to - markerLength,
        ),
      };
    }

    // collapsed selections receive an empty pair with the caret in the middle
    if (range.empty) {
      return {
        changes: { from: range.from, insert: marker + marker },
        range: EditorSelection.cursor(range.from + markerLength),
      };
    }

    return {
      changes: [
        { from: range.from, insert: marker },
        { from: range.to, insert: marker },
      ],
      range: EditorSelection.range(
        range.from + markerLength,
        range.to + markerLength,
      ),
    };
  });
  view.dispatch(transaction);
  return true;
}

/** Inserts a soft newline and continues Markdown list or quote prefixes. */
function insertComposerLine(view) {
  const selection = view.state.selection.main;
  const line = view.state.doc.lineAt(selection.head);

  // replace ranged selections with a normal soft line break
  if (!selection.empty) {
    view.dispatch({
      changes: { from: selection.from, to: selection.to, insert: '\n' },
      selection: { anchor: selection.from + 1 },
    });
    return true;
  }

  const before = view.state.doc.sliceString(line.from, selection.head);
  const after = view.state.doc.sliceString(selection.head, line.to);
  const unordered = before.match(/^(\s*)([-+*])\s+(.*)$/);
  const ordered = before.match(/^(\s*)(\d+)([.)])\s+(.*)$/);
  const quote = before.match(/^(\s*>\s?)(.*)$/);
  let prefix = '';
  let content = '';

  if (unordered) {
    prefix = unordered[1] + unordered[2] + ' ';
    content = unordered[3];
  } else if (ordered) {
    prefix = ordered[1] + (Number(ordered[2]) + 1) + ordered[3] + ' ';
    content = ordered[4];
  } else if (quote) {
    prefix = quote[1];
    content = quote[2];
  }

  // an empty list or quote marker exits the block without adding another row
  if (prefix && !content.trim() && !after.trim()) {
    view.dispatch({
      changes: { from: line.from, to: line.to, insert: '' },
      selection: { anchor: line.from },
    });
    return true;
  }

  const insertion = '\n' + prefix;
  view.dispatch({
    changes: { from: selection.head, insert: insertion },
    selection: { anchor: selection.head + insertion.length },
  });
  return true;
}

/** Submits the owning composer while leaving Shift+Enter to the editor. */
function submitComposer(view, onSubmit) {
  if (view.composing) return false;
  onSubmit();
  return true;
}

/** Converts a selected label plus pasted URL into Markdown link source. */
function pasteMarkdownLink(event, view) {
  const text = event.clipboardData && event.clipboardData.getData('text/plain');
  if (!text || !isWebUrl(text)) return false;
  const selection = view.state.selection.main;
  if (selection.empty) return false;
  const selected = view.state.doc.sliceString(selection.from, selection.to);
  const target = /^www\./i.test(text.trim())
    ? 'https://' + text.trim()
    : text.trim();
  const replacement = '[' + selected + '](' + target + ')';
  event.preventDefault();
  view.dispatch({
    changes: { from: selection.from, to: selection.to, insert: replacement },
    selection: { anchor: selection.from + replacement.length },
  });
  return true;
}

/** Adds link styling to raw URLs that are not enclosed in Markdown links. */
function rawUrlDecorations(view) {
  const ranges = [];
  const expression = /(?:https?:\/\/|www\.)[^\s<>()]+/gi;
  const document = view.state.doc;

  for (const visible of view.visibleRanges) {
    const source = document.sliceString(visible.from, visible.to);
    let match = expression.exec(source);
    while (match) {
      ranges.push(
        Decoration.mark({ class: 'cm-markdown-raw-url' }).range(
          visible.from + match.index,
          visible.from + match.index + match[0].length,
        ),
      );
      match = expression.exec(source);
    }
  }
  return Decoration.set(ranges, true);
}

// Raw URL decorations update only when the document or viewport changes.
const rawUrlPlugin = ViewPlugin.fromClass(
  class {
    /** Creates the initial visible-range decorations. */
    constructor(view) {
      this.decorations = rawUrlDecorations(view);
    }

    /** Refreshes link decorations when content or the viewport changes. */
    update(update) {
      if (update.docChanged || update.viewportChanged) {
        this.decorations = rawUrlDecorations(update.view);
      }
    }
  },
  {
    decorations: function (value) {
      return value.decorations;
    },
  },
);

/** Builds one source-native Markdown editor inside a composer host. */
function createEditor(element, options) {
  const settings = options || {};
  if (!element || editors.has(element)) return editors.get(element) || null;
  const onChange = settings.onChange || function () {};
  const onSubmit = settings.onSubmit || function () {};
  const extensions = [
    history(),
    drawSelection(),
    markdown(),
    syntaxHighlighting(markdownHighlight),
    EditorView.lineWrapping,
    rawUrlPlugin,
    placeholder(settings.placeholder || element.dataset.placeholder || ''),
    EditorView.contentAttributes.of({
      'aria-label':
        settings.label || element.getAttribute('aria-label') || 'Message AI',
      'aria-multiline': 'true',
    }),
    EditorView.domEventHandlers({
      paste: pasteMarkdownLink,
    }),
    EditorView.updateListener.of(function (update) {
      if (update.docChanged) onChange(update.state.doc.toString());
    }),
    Prec.highest(
      keymap.of(
        [
          {
            key: 'Mod-b',
            run: function (view) {
              return toggleMarker(view, '**');
            },
          },
          {
            key: 'Mod-i',
            run: function (view) {
              return toggleMarker(view, '*');
            },
          },
          { key: 'Shift-Enter', run: insertComposerLine },
          {
            key: 'Enter',
            run: function (view) {
              return submitComposer(view, onSubmit);
            },
          },
        ].concat(historyKeymap, defaultKeymap),
      ),
    ),
  ];
  const state = EditorState.create({
    doc: settings.value || '',
    extensions: extensions,
  });
  const view = new EditorView({ state: state, parent: element });
  editors.set(element, view);
  element.removeAttribute('contenteditable');
  return view;
}

/** Focuses the CodeMirror view owned by one composer host. */
function focusEditor(element) {
  const view = editors.get(element);
  if (view) view.focus();
}

/** Reads canonical Markdown from an editor host or rendered message body. */
function fromElement(element) {
  if (!element) return '';
  const view = editors.get(element);
  if (view) return view.state.doc.toString();
  if (element.hasAttribute && element.hasAttribute('data-markdown-source')) {
    return element.getAttribute('data-markdown-source') || '';
  }
  return (element.textContent || '')
    .replace(/\u00a0/g, ' ')
    .replace(/\r\n?/g, '\n')
    .trim();
}

/** Renders safe Markdown for user and assistant response bubbles. */
function render(source) {
  return renderer.render(String(source || ''));
}

/** Replaces editor content without polluting its undo history. */
function setEditorValue(element, value) {
  const view = editors.get(element);
  if (!view) return;
  const source = String(value || '');
  view.dispatch({
    changes: { from: 0, to: view.state.doc.length, insert: source },
    selection: { anchor: source.length },
    annotations: Transaction.addToHistory.of(false),
  });
}

window.MarkdownRuntime = {
  createEditor: createEditor,
  focusEditor: focusEditor,
  fromElement: fromElement,
  render: render,
  setEditorValue: setEditorValue,
};
