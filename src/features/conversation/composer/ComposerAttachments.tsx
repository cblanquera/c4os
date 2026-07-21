import type { ComposerAttachment } from "./composer-types";

interface ComposerAttachmentsProps {
  readonly attachments: readonly ComposerAttachment[];
  readonly onRemove: (attachmentId: string) => void;
}

const COMPATIBILITY_LABELS = {
  ready: "Ready",
  "needs-vision": "Needs Vision",
  "needs-audio": "Needs Audio",
  converted: "Converted",
  incompatible: "Incompatible",
} as const;

/** Renders newest-first draft chips while retaining immutable reference numbers. */
export function ComposerAttachments({
  attachments,
  onRemove,
}: ComposerAttachmentsProps) {
  const newestFirst = [...attachments].sort(
    (left, right) => right.referenceNumber - left.referenceNumber,
  );

  if (newestFirst.length === 0) {
    return null;
  }

  return (
    <section
      aria-label="Draft attachments"
      className="conversation-composer__attachments"
    >
      <ul>
        {newestFirst.map((attachment) => (
          <li
            className="conversation-composer__attachment"
            data-compatibility={attachment.compatibility}
            key={attachment.id}
          >
            {attachment.previewUrl ? (
              <img
                alt={`Preview of ${attachment.fileName}`}
                className="conversation-composer__attachment-preview"
                src={attachment.previewUrl}
              />
            ) : (
              <span
                aria-hidden="true"
                className="conversation-composer__attachment-icon"
              >
                ◇
              </span>
            )}
            <span className="conversation-composer__attachment-copy">
              <span className="conversation-composer__attachment-name">
                <span className="conversation-composer__attachment-reference">
                  #{attachment.referenceNumber}
                </span>{" "}
                {attachment.fileName}
              </span>
              <span className="conversation-composer__attachment-metadata">
                {attachmentExtension(attachment.fileName)} ·{" "}
                {attachment.sizeLabel}
              </span>
              <span className="conversation-composer__attachment-status">
                {COMPATIBILITY_LABELS[attachment.compatibility]}
              </span>
            </span>
            <button
              aria-label={`Remove ${attachment.fileName} attachment #${attachment.referenceNumber}`}
              className="conversation-composer__remove"
              onClick={() => onRemove(attachment.id)}
              type="button"
            >
              ×
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}

/** Returns a compact display extension without mutating the attachment record. */
function attachmentExtension(fileName: string): string {
  const extension = fileName.includes(".") ? fileName.split(".").pop() : null;
  return extension?.toLocaleUpperCase() || "FILE";
}
