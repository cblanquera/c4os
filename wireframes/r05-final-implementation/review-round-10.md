# r05 Final Implementation Wireframes - Review Round 10

## Status

Ready for revised prompt-reference terminology review.

## What Changed

- Added the clearer prompt-reference vocabulary to the local context:
  trigger, active query, typeahead menu, resolved inline reference, unresolved
  token, and serialized prompt.
- Updated `#prompt-suggestions` to show trigger-based composer behavior instead
  of a generic suggestion list.
- Resolved inline references now render distinctly in blue in the composer.
- The Skills typeahead shows `$` as the active trigger, `grill` as the active
  query, filtered skill results, and a selected exact match for
  `grill-me-with-docs`.
- Added scoped `@` and `/` examples: `@` lists plugin resources before files
  and folders, while `/` lists C4OS built-in commands before accepted
  file/resource matches.
- Added serialized prompt text showing how visible references expand before the
  runtime receives the prompt.

## Review Links

- `./index.html#prompt-suggestions`
- `./index.html#coverage`

## Review Questions

1. Does "trigger / active query / typeahead menu / resolved inline reference /
   unresolved token / serialized prompt" match the behavior you intended?
2. Is blue the right visual signal for an exact-match or selected resolved
   inline reference?
3. Does the serialized prompt example make it clear that runtime input differs
   from the visible composer text?
4. Should unresolved trigger text send as plain text, or should C4OS block send
   until the user resolves or removes it?

## Approval Boundary

No `.agents` specs, handoff docs, or durable product docs were updated in this
round. If this review round is approved, the next step remains updating spec 05
and only the directly affected overlap docs in specs 04, 07, 10, and 11 with
links to the approved routes/states as evidence.
