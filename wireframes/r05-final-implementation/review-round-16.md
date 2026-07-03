# r05 Final Implementation Wireframes - Review Round 16

## Status

Ready for prompt normalization review.

## What Changed

- The prompt normalizes markup after every input event.
- Previously selected `$`, `@`, and `/` references stay blue only when their
  token still exactly matches a recognized reference.
- Typing or backspacing after a blue reference no longer leaves following text
  trapped in that reference span.
- Backspacing to a pending trigger re-runs typeahead detection immediately.

## Review Links

- `./index.html#prompt-suggestions`

## Review Questions

1. After selecting multiple `$`, `@`, and `/` references, do all selected
   references remain blue?
2. When you backspace to a trigger, does the correct typeahead open?
3. When you type a trigger at the end, does only the intended token become
   active rather than turning unrelated text blue?

## Approval Boundary

No `.agents` specs, handoff docs, or durable product docs were updated in this
round.
