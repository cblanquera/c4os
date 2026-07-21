import type {
  ClipboardEvent,
  FormEvent,
  KeyboardEvent,
  ReactNode,
  UIEvent,
} from "react";
import { useLayoutEffect, useRef, useState } from "react";
import {
  Button,
  Menu,
  MenuItem,
  MenuTrigger,
  Popover,
} from "react-aria-components";

import { CompatibilityConflict } from "./CompatibilityConflict";
import { ComposerAttachments } from "./ComposerAttachments";
import { MarkdownSource } from "./MarkdownSource";
import type {
  ComposerAttachment,
  ComposerCompatibilityConflict,
  ComposerConflictAction,
  ComposerControlSlots,
  ComposerMode,
  ComposerPresentationMode,
  ComposerReplyReference,
  ComposerSubmission,
} from "./composer-types";
import { COMPOSER_MODES } from "./composer-types";
import {
  linkSourceSelection,
  type SourceReplacement,
  toggleSourceMarker,
} from "./markdown-source";
import "./composer.css";

export interface ComposerProps {
  readonly attachments?: readonly ComposerAttachment[];
  readonly conflict?: ComposerCompatibilityConflict;
  readonly controls?: ComposerControlSlots;
  readonly isDisabled?: boolean;
  readonly isModeLocked?: boolean;
  readonly isSubmitDisabled?: boolean;
  readonly mode: ComposerPresentationMode;
  readonly onAttach?: () => void;
  readonly onBrowse?: () => void;
  readonly onConflictAction?: (action: ComposerConflictAction) => void;
  readonly onModeChange: (mode: ComposerMode) => void;
  readonly onRemoveAttachment: (attachmentId: string) => void;
  readonly onRemoveReply: () => void;
  readonly onSubmit: (submission: ComposerSubmission) => void;
  readonly onValueChange: (value: string) => void;
  readonly replyReference?: ComposerReplyReference;
  readonly value: string;
}

interface ComposerModeCopy {
  readonly action: string;
  readonly label: string;
  readonly placeholder: string;
}

const MODE_COPY: Record<ComposerPresentationMode, ComposerModeCopy> = {
  chat: { action: "Send", label: "Message", placeholder: "Ask C4OS" },
  files: {
    action: "Open",
    label: "File or folder path",
    placeholder: "Choose a file or folder",
  },
  browser: {
    action: "Open",
    label: "Web address",
    placeholder: "Enter a web address",
  },
  terminal: {
    action: "Run",
    label: "Terminal command",
    placeholder: "Enter a command",
  },
  reply: { action: "Send", label: "Reply", placeholder: "Write a reply" },
};

/** Renders the controlled, source-preserving composer for every accepted mode. */
export function Composer({
  attachments = [],
  conflict,
  controls = {},
  isDisabled = false,
  isModeLocked = false,
  isSubmitDisabled = false,
  mode,
  onAttach,
  onBrowse,
  onConflictAction,
  onModeChange,
  onRemoveAttachment,
  onRemoveReply,
  onSubmit,
  onValueChange,
  replyReference,
  value,
}: ComposerProps) {
  const markdownPreviewRef = useRef<HTMLPreElement>(null);
  const copy = MODE_COPY[mode];
  const isMarkdownMode = mode === "chat" || mode === "reply";
  const showsAttachments = mode === "chat";
  const hasSubmission =
    value.trim().length > 0 || (showsAttachments && attachments.length > 0);
  const canSubmit =
    !isDisabled && !isSubmitDisabled && !conflict && hasSubmission;

  /** Emits the current controlled draft without claiming durable ownership. */
  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!canSubmit) {
      return;
    }

    onSubmit({
      attachmentIds: showsAttachments
        ? attachments.map((attachment) => attachment.id)
        : [],
      mode,
      ...(replyReference ? { replyReferenceId: replyReference.id } : {}),
      source: value,
    });
  };

  /** Applies send and source-marker keyboard behavior to Markdown drafts. */
  const handleMarkdownKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    const modifierPressed = event.metaKey || event.ctrlKey;
    const formattingMarker =
      event.key.toLocaleLowerCase() === "b"
        ? "**"
        : event.key.toLocaleLowerCase() === "i"
          ? "*"
          : null;

    if (modifierPressed && formattingMarker) {
      event.preventDefault();
      const textarea = event.currentTarget;
      applySourceReplacement(
        textarea,
        toggleSourceMarker(
          value,
          textarea.selectionStart,
          textarea.selectionEnd,
          formattingMarker,
        ),
        onValueChange,
      );
      return;
    }

    if (
      event.key === "Enter" &&
      !event.shiftKey &&
      !event.nativeEvent.isComposing
    ) {
      event.preventDefault();
      event.currentTarget.form?.requestSubmit();
    }
  };

  /** Converts a URL pasted over selected source while leaving ordinary paste native. */
  const handleMarkdownPaste = (event: ClipboardEvent<HTMLTextAreaElement>) => {
    const textarea = event.currentTarget;
    const replacement = linkSourceSelection(
      value,
      textarea.selectionStart,
      textarea.selectionEnd,
      event.clipboardData.getData("text/plain"),
    );

    if (!replacement) {
      return;
    }

    event.preventDefault();
    applySourceReplacement(textarea, replacement, onValueChange);
  };

  /** Keeps the visible syntax layer aligned with the native textarea scroll. */
  const handleMarkdownScroll = (event: UIEvent<HTMLTextAreaElement>) => {
    const preview = markdownPreviewRef.current;
    if (preview) {
      preview.scrollLeft = event.currentTarget.scrollLeft;
      preview.scrollTop = event.currentTarget.scrollTop;
    }
  };

  return (
    <form
      aria-label="Message composer"
      className={`conversation-composer conversation-composer--${mode}`}
      data-composer-dock="fixed-compatible"
      data-composer-mode={mode}
      onSubmit={handleSubmit}
    >
      {mode === "reply" && replyReference ? (
        <section
          aria-label="Reply reference"
          className="conversation-composer__reply"
        >
          <span
            aria-hidden="true"
            className="conversation-composer__reply-icon"
          >
            ↩
          </span>
          <span className="conversation-composer__reply-copy">
            <strong>{replyReference.label}</strong>
            <span>{replyReference.excerpt}</span>
          </span>
          <button
            aria-label="Remove reply reference"
            className="conversation-composer__remove"
            onClick={onRemoveReply}
            type="button"
          >
            ×
          </button>
        </section>
      ) : null}

      {showsAttachments ? (
        <ComposerAttachments
          attachments={attachments}
          onRemove={onRemoveAttachment}
        />
      ) : null}

      {conflict ? (
        <CompatibilityConflict
          conflict={conflict}
          onAction={(action) => onConflictAction?.(action)}
        />
      ) : null}

      <div className="conversation-composer__input-row">
        <label className="conversation-composer__field">
          <span className="conversation-composer__visually-hidden">
            {copy.label}
          </span>
          {isMarkdownMode ? (
            <span className="conversation-composer__source-editor">
              <pre
                aria-hidden="true"
                className="conversation-composer__source-preview"
                ref={markdownPreviewRef}
              >
                <MarkdownSource source={value} />
                {value.endsWith("\n") ? " " : null}
              </pre>
              <textarea
                aria-keyshortcuts="Meta+B Control+B Meta+I Control+I"
                data-enter-behavior="send"
                disabled={isDisabled}
                onChange={(event) => onValueChange(event.currentTarget.value)}
                onKeyDown={handleMarkdownKeyDown}
                onPaste={handleMarkdownPaste}
                onScroll={handleMarkdownScroll}
                placeholder={copy.placeholder}
                rows={2}
                value={value}
              />
            </span>
          ) : (
            <span className="conversation-composer__direct-input">
              {mode === "browser" ? (
                <span
                  aria-hidden="true"
                  className="conversation-composer__prefix"
                >
                  ◉
                </span>
              ) : null}
              {mode === "terminal" ? (
                <span
                  aria-hidden="true"
                  className="conversation-composer__prefix"
                >
                  $
                </span>
              ) : null}
              <input
                disabled={isDisabled}
                onChange={(event) => onValueChange(event.currentTarget.value)}
                placeholder={copy.placeholder}
                value={value}
              />
            </span>
          )}
        </label>
      </div>

      <div className="conversation-composer__toolbar">
        <div
          aria-label="Composer actions"
          className="conversation-composer__toolbar-start"
          role="group"
        >
          {mode !== "reply" ? (
            <ModeMenu
              isDisabled={isModeLocked}
              mode={mode}
              onModeChange={onModeChange}
            />
          ) : null}
          {mode === "chat" && onAttach ? (
            <button onClick={onAttach} type="button">
              Attach
            </button>
          ) : null}
          {mode === "files" && onBrowse ? (
            <button onClick={onBrowse} type="button">
              Browse
            </button>
          ) : null}
          <ComposerControls controls={controls} mode={mode} />
        </div>
        <button disabled={!canSubmit} type="submit">
          {copy.action}
        </button>
      </div>
    </form>
  );
}

interface ModeMenuProps {
  readonly isDisabled: boolean;
  readonly mode: ComposerMode;
  readonly onModeChange: (mode: ComposerMode) => void;
}

/** Renders the upward single-selection mode menu with trigger restoration. */
function ModeMenu({ isDisabled, mode, onModeChange }: ModeMenuProps) {
  const triggerRef = useRef<HTMLButtonElement>(null);
  const shouldRestoreFocus = useRef(false);
  const [isOpen, setIsOpen] = useState(false);

  /** Tracks overlay visibility and requests focus restoration after dismissal. */
  const handleOpenChange = (nextIsOpen: boolean) => {
    shouldRestoreFocus.current = !nextIsOpen;
    setIsOpen(nextIsOpen);
  };

  useLayoutEffect(() => {
    if (!isOpen && shouldRestoreFocus.current) {
      shouldRestoreFocus.current = false;
      triggerRef.current?.focus();
    }
  }, [isOpen]);

  return (
    <MenuTrigger isOpen={isOpen} onOpenChange={handleOpenChange}>
      <Button
        aria-label="Composer mode"
        className="conversation-composer__mode"
        isDisabled={isDisabled}
        ref={triggerRef}
      >
        {modeLabel(mode)}
      </Button>
      <Popover
        className="conversation-composer__mode-popover"
        offset={6}
        placement="top start"
      >
        <Menu
          aria-label="Composer modes"
          onAction={(key) => onModeChange(key as ComposerMode)}
          selectedKeys={[mode]}
          selectionMode="single"
        >
          {COMPOSER_MODES.map((option) => (
            <MenuItem id={option} key={option} textValue={modeLabel(option)}>
              {modeLabel(option)}
            </MenuItem>
          ))}
        </Menu>
      </Popover>
    </MenuTrigger>
  );
}

interface ComposerControlsProps {
  readonly controls: ComposerControlSlots;
  readonly mode: ComposerPresentationMode;
}

/** Places only the controls accepted for the active composer mode. */
function ComposerControls({ controls, mode }: ComposerControlsProps) {
  if (mode === "chat") {
    return (
      <>
        <ControlSlot label="Model">{controls.model}</ControlSlot>
        <ControlSlot label="Reasoning effort">{controls.reasoning}</ControlSlot>
        <ControlSlot label="Approval preset">{controls.approval}</ControlSlot>
        <ControlSlot label="Git branch">{controls.branch}</ControlSlot>
      </>
    );
  }

  if (mode === "files") {
    return <ControlSlot label="Git branch">{controls.branch}</ControlSlot>;
  }

  return null;
}

interface ControlSlotProps {
  readonly children?: ReactNode;
  readonly label: string;
}

/** Adds an accessible grouping hook around an injected authoritative control. */
function ControlSlot({ children, label }: ControlSlotProps) {
  if (!children) {
    return null;
  }

  return (
    <span
      aria-label={label}
      className="conversation-composer__control"
      role="group"
    >
      {children}
    </span>
  );
}

/** Applies a textarea range edit so native editing history remains in the path. */
function applySourceReplacement(
  textarea: HTMLTextAreaElement,
  replacement: SourceReplacement,
  onValueChange: (value: string) => void,
) {
  textarea.setRangeText(
    replacement.replacement,
    replacement.rangeStart,
    replacement.rangeEnd,
    "preserve",
  );
  textarea.setSelectionRange(
    replacement.selectionStart,
    replacement.selectionEnd,
  );
  onValueChange(textarea.value);
}

/** Formats an accepted composer mode for visible menu copy. */
function modeLabel(mode: ComposerMode): string {
  return `${mode[0]?.toLocaleUpperCase()}${mode.slice(1)}`;
}
