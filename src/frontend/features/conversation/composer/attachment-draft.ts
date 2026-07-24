// Attachment inputs use the authoritative attachment identity as their stable key.
export type AttachmentDraftInput = {
  readonly id: string;
};

// Referenced attachments carry the immutable one-based number shown to the user.
export type ReferencedAttachment<Attachment extends AttachmentDraftInput> =
  Attachment & {
    readonly referenceNumber: number;
  };

// The next number survives removals so a later addition never reuses a reference.
export type AttachmentDraft<Attachment extends AttachmentDraftInput> = {
  readonly attachments: readonly ReferencedAttachment<Attachment>[];
  readonly nextReferenceNumber: number;
};

/** Creates the empty immutable attachment-reference ledger for a draft or Chat. */
export function createAttachmentDraft<
  Attachment extends AttachmentDraftInput,
>(): AttachmentDraft<Attachment> {
  return {
    attachments: [],
    nextReferenceNumber: 1,
  };
}

/** Restores a persisted ledger while validating every retained reference. */
export function restoreAttachmentDraft<Attachment extends AttachmentDraftInput>(
  attachments: readonly ReferencedAttachment<Attachment>[],
  nextReferenceNumber: number,
): AttachmentDraft<Attachment> {
  const restored = {
    attachments: attachments.map((attachment) => ({ ...attachment })),
    nextReferenceNumber,
  };
  validateDraft(restored);
  return restored;
}

/**
 * Reconciles authoritative attachment data without renumbering survivors or
 * reusing references released by removals.
 */
export function reconcileAttachmentDraft<
  Attachment extends AttachmentDraftInput,
>(
  draft: AttachmentDraft<Attachment>,
  attachments: readonly Attachment[],
): AttachmentDraft<Attachment> {
  validateDraft(draft);

  const priorById = new Map(
    draft.attachments.map((attachment) => [attachment.id, attachment]),
  );
  const incomingIds = new Set<string>();
  let nextReferenceNumber = draft.nextReferenceNumber;

  const referencedAttachments = attachments.map((attachment) => {
    validateAttachmentId(attachment.id);
    if (incomingIds.has(attachment.id)) {
      throw new Error(`Duplicate attachment identity: ${attachment.id}`);
    }
    incomingIds.add(attachment.id);

    const prior = priorById.get(attachment.id);
    if (prior !== undefined) {
      return {
        ...attachment,
        referenceNumber: prior.referenceNumber,
      };
    }

    if (
      !Number.isSafeInteger(nextReferenceNumber) ||
      nextReferenceNumber >= Number.MAX_SAFE_INTEGER
    ) {
      throw new Error("Attachment reference capacity is exhausted.");
    }
    const referenceNumber = nextReferenceNumber;
    nextReferenceNumber += 1;
    return {
      ...attachment,
      referenceNumber,
    };
  });

  return {
    attachments: referencedAttachments,
    nextReferenceNumber,
  };
}

/** Rejects corrupt ledgers before their references can enter a new projection. */
function validateDraft<Attachment extends AttachmentDraftInput>(
  draft: AttachmentDraft<Attachment>,
): void {
  if (
    !Number.isSafeInteger(draft.nextReferenceNumber) ||
    draft.nextReferenceNumber < 1
  ) {
    throw new Error("Attachment draft has an invalid next reference.");
  }

  const ids = new Set<string>();
  const references = new Set<number>();
  for (const attachment of draft.attachments) {
    validateAttachmentId(attachment.id);
    if (
      !Number.isSafeInteger(attachment.referenceNumber) ||
      attachment.referenceNumber < 1 ||
      attachment.referenceNumber >= draft.nextReferenceNumber ||
      ids.has(attachment.id) ||
      references.has(attachment.referenceNumber)
    ) {
      throw new Error("Attachment draft contains invalid references.");
    }
    ids.add(attachment.id);
    references.add(attachment.referenceNumber);
  }
}

/** Keeps empty or whitespace-only identities out of the reference ledger. */
function validateAttachmentId(attachmentId: string): void {
  if (attachmentId.trim().length === 0) {
    throw new Error("Attachment identity cannot be empty.");
  }
}
