import type { ReactNode } from "react";

export const COMPOSER_MODES = ["chat", "files", "browser", "terminal"] as const;

export type ComposerMode = (typeof COMPOSER_MODES)[number];
export type ComposerPresentationMode = ComposerMode | "reply";

export type AttachmentCompatibility =
  "ready" | "needs-vision" | "needs-audio" | "converted" | "incompatible";

export interface ComposerAttachment {
  readonly compatibility: AttachmentCompatibility;
  readonly fileName: string;
  readonly id: string;
  readonly previewUrl?: string;
  readonly referenceNumber: number;
  readonly sizeLabel: string;
}

export type ComposerConflictAction =
  "use-compatible-model" | "convert" | "remove-file" | "cancel";

export interface ComposerCompatibilityConflict {
  readonly actions: readonly ComposerConflictAction[];
  readonly attachmentId: string;
  readonly description: string;
  readonly title: string;
}

export interface ComposerReplyReference {
  readonly excerpt: string;
  readonly id: string;
  readonly kind:
    | "user-message"
    | "assistant-message"
    | "browser"
    | "file"
    | "folder"
    | "terminal";
  readonly label: string;
}

export interface ComposerControlSlots {
  readonly approval?: ReactNode;
  readonly branch?: ReactNode;
  readonly model?: ReactNode;
  readonly reasoning?: ReactNode;
}

export interface ComposerSubmission {
  readonly attachmentIds: readonly string[];
  readonly mode: ComposerPresentationMode;
  readonly replyReferenceId?: string;
  readonly source: string;
}
