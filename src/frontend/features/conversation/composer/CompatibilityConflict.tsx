import type {
  ComposerCompatibilityConflict,
  ComposerConflictAction,
} from "./composer-types";

interface CompatibilityConflictProps {
  readonly conflict: ComposerCompatibilityConflict;
  readonly onAction: (action: ComposerConflictAction) => void;
}

const ACTION_LABELS: Record<ComposerConflictAction, string> = {
  "use-compatible-model": "Use compatible model",
  convert: "Convert",
  "remove-file": "Remove file",
  cancel: "Cancel",
};

/** Presents an authoritative compatibility conflict without choosing a resolution. */
export function CompatibilityConflict({
  conflict,
  onAction,
}: CompatibilityConflictProps) {
  return (
    <section
      aria-labelledby="composer-conflict-title"
      className="conversation-composer__conflict"
      data-attachment-id={conflict.attachmentId}
      role="alert"
    >
      <h3 id="composer-conflict-title">{conflict.title}</h3>
      <p>{conflict.description}</p>
      <div
        aria-label="Attachment compatibility actions"
        className="conversation-composer__conflict-actions"
        role="group"
      >
        {conflict.actions.map((action) => (
          <button key={action} onClick={() => onAction(action)} type="button">
            {ACTION_LABELS[action]}
          </button>
        ))}
      </div>
    </section>
  );
}
