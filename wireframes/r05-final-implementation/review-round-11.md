# r05 Final Implementation Wireframes - Review Round 11

## Status

Ready for prompt typeahead behavior review.

## What Changed

- `#prompt-suggestions` now starts from the chat-session shape: a sent user
  message with a resolved inline skill reference, plus an editable composer
  draft with `use $grill to ask me questions`.
- Typed trigger text is plain inline prompt text. It no longer uses a blue
  chip, bubble, or background.
- Resolved references render as blue inline text only, matching the intended
  lightweight editing behavior.
- The typeahead popup appears above the fixed composer and only appears for the
  pending `$grill` active query in this route.
- Removed the visible `Typeahead menu` title, explanatory helper text,
  cross-trigger examples, and serialized prompt annotation from the popup.

## Review Links

- `./index.html#prompt-suggestions`
- `./index.html#coverage`

## Review Questions

1. Does the pending `$grill` state now feel like a real editable composer state?
2. Is blue inline text, without a bubble, the right resolved-reference signal?
3. Should separate review routes be added for pending `@` and `/` queries, or
   is the `$` route enough to approve the shared interaction pattern?
4. Does the above-composer popup position match the fixed prompt dock behavior?

## Approval Boundary

No `.agents` specs, handoff docs, or durable product docs were updated in this
round. If this review round is approved, the next step remains updating spec 05
and only the directly affected overlap docs in specs 04, 07, 10, and 11 with
links to the approved routes/states as evidence.
