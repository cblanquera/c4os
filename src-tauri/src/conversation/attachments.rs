use std::collections::BTreeSet;

/// Metadata accepted when an authoritative picker/materializer adds a draft item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftAttachmentInput {
    pub attachment_id: String,
    pub stable_reference: String,
    pub display_name: String,
}

/// The original one-based reference never changes after assignment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftAttachment {
    pub attachment_id: String,
    pub stable_reference: String,
    pub display_name: String,
    pub original_reference: u32,
}

/// An immutable attachment draft returns a new value for every add or remove.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentDraft {
    items: Vec<DraftAttachment>,
    next_reference: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttachmentDraftError {
    EmptyIdentifier,
    EmptyDisplayName,
    DuplicateAttachment,
    DuplicateStableReference,
    ReferenceExhausted,
    AttachmentNotFound,
    InvalidDraft,
}

impl Default for AttachmentDraft {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            next_reference: 1,
        }
    }
}

impl AttachmentDraft {
    /// Restores a persisted ledger and rejects reused, missing, or stale references.
    pub fn restore(
        items: Vec<DraftAttachment>,
        next_reference: u32,
    ) -> Result<Self, AttachmentDraftError> {
        let draft = Self {
            items,
            next_reference,
        };
        draft.validate()?;
        Ok(draft)
    }

    /// Adds one item without mutating this draft or reusing removed references.
    pub fn add(&self, input: DraftAttachmentInput) -> Result<Self, AttachmentDraftError> {
        self.validate()?;
        validate_input(&input)?;
        if self
            .items
            .iter()
            .any(|item| item.attachment_id == input.attachment_id)
        {
            return Err(AttachmentDraftError::DuplicateAttachment);
        }
        if self
            .items
            .iter()
            .any(|item| item.stable_reference == input.stable_reference)
        {
            return Err(AttachmentDraftError::DuplicateStableReference);
        }
        let following_reference = self
            .next_reference
            .checked_add(1)
            .ok_or(AttachmentDraftError::ReferenceExhausted)?;
        let mut items = self.items.clone();
        items.push(DraftAttachment {
            attachment_id: input.attachment_id,
            stable_reference: input.stable_reference,
            display_name: input.display_name,
            original_reference: self.next_reference,
        });
        Ok(Self {
            items,
            next_reference: following_reference,
        })
    }

    /// Removes one item while preserving every surviving original reference.
    pub fn remove(&self, attachment_id: &str) -> Result<Self, AttachmentDraftError> {
        self.validate()?;
        let mut items = self.items.clone();
        let Some(index) = items
            .iter()
            .position(|item| item.attachment_id == attachment_id)
        else {
            return Err(AttachmentDraftError::AttachmentNotFound);
        };
        items.remove(index);
        Ok(Self {
            items,
            next_reference: self.next_reference,
        })
    }

    /// Newest-first order is a view projection and does not mutate storage order.
    pub fn display_order(&self) -> Vec<&DraftAttachment> {
        let mut items = self.items.iter().collect::<Vec<_>>();
        items.sort_by_key(|item| std::cmp::Reverse(item.original_reference));
        items
    }

    /// Submission/title order follows the immutable original one-based references.
    pub fn submission_order(&self) -> Vec<&DraftAttachment> {
        let mut items = self.items.iter().collect::<Vec<_>>();
        items.sort_by_key(|item| item.original_reference);
        items
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn items(&self) -> &[DraftAttachment] {
        &self.items
    }

    pub fn next_reference(&self) -> u32 {
        self.next_reference
    }

    pub fn validate(&self) -> Result<(), AttachmentDraftError> {
        let mut attachment_ids = BTreeSet::new();
        let mut stable_references = BTreeSet::new();
        let mut original_references = BTreeSet::new();
        for item in &self.items {
            validate_input(&DraftAttachmentInput {
                attachment_id: item.attachment_id.clone(),
                stable_reference: item.stable_reference.clone(),
                display_name: item.display_name.clone(),
            })?;
            if item.original_reference == 0
                || item.original_reference >= self.next_reference
                || !attachment_ids.insert(item.attachment_id.as_str())
                || !stable_references.insert(item.stable_reference.as_str())
                || !original_references.insert(item.original_reference)
            {
                return Err(AttachmentDraftError::InvalidDraft);
            }
        }
        Ok(())
    }
}

fn validate_input(input: &DraftAttachmentInput) -> Result<(), AttachmentDraftError> {
    if input.attachment_id.trim().is_empty() || input.stable_reference.trim().is_empty() {
        return Err(AttachmentDraftError::EmptyIdentifier);
    }
    if input.display_name.trim().is_empty() {
        return Err(AttachmentDraftError::EmptyDisplayName);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(id: &str) -> DraftAttachmentInput {
        DraftAttachmentInput {
            attachment_id: id.into(),
            stable_reference: format!("grant:{id}"),
            display_name: format!("{id}.txt"),
        }
    }

    #[test]
    fn add_is_immutable_and_display_order_is_newest_first() {
        let empty = AttachmentDraft::default();
        let one = empty.add(input("one")).unwrap();
        let two = one.add(input("two")).unwrap();
        assert!(empty.is_empty());
        assert_eq!(one.len(), 1);
        assert_eq!(
            two.display_order()
                .iter()
                .map(|item| item.attachment_id.as_str())
                .collect::<Vec<_>>(),
            ["two", "one"]
        );
        assert_eq!(
            two.submission_order()
                .iter()
                .map(|item| item.original_reference)
                .collect::<Vec<_>>(),
            [1, 2]
        );
    }

    #[test]
    fn removed_reference_is_never_reused() {
        let draft = AttachmentDraft::default()
            .add(input("one"))
            .unwrap()
            .add(input("two"))
            .unwrap()
            .remove("two")
            .unwrap()
            .add(input("three"))
            .unwrap();
        assert_eq!(
            draft
                .submission_order()
                .iter()
                .map(|item| item.original_reference)
                .collect::<Vec<_>>(),
            [1, 3]
        );
    }

    #[test]
    fn duplicate_ids_and_opaque_references_fail_closed() {
        let draft = AttachmentDraft::default().add(input("one")).unwrap();
        assert_eq!(
            draft.add(input("one")),
            Err(AttachmentDraftError::DuplicateAttachment)
        );
        let duplicate_reference = DraftAttachmentInput {
            attachment_id: "two".into(),
            stable_reference: "grant:one".into(),
            display_name: "two.txt".into(),
        };
        assert_eq!(
            draft.add(duplicate_reference),
            Err(AttachmentDraftError::DuplicateStableReference)
        );
    }
}
