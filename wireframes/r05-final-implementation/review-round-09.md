# r05 Final Implementation Wireframes - Review Round 09

## Status

Ready for revised Batch 3 review after feedback that Round 08 felt too
instructional.

## What Changed

- Reworked the Batch 3 prompt routes to start from the r04/current frontend
  chat-session structure: left Chats panel, thread list, user message, work
  log, agent message, permission prompt, and composer dock.
- Moved approval, remembered-rule, blocked-resource, branch, attachment, and
  fallback states into realistic chat-session surfaces instead of standalone
  annotation cards.
- Made the composer closer to current frontend truth by using attachment
  preview chips, prompt textbox, approval chip, branch/model context strip, and
  visible composer popovers for suggestions and branch selection.
- Preserved the approved r05 shell and Settings structure, while using r04 only
  as the chat/session reference.

## Review Links

- `./index.html#prompt-suggestions`
- `./index.html#approval-dialog`
- `./index.html#remembered-rule-summary`
- `./index.html#blocked-suggestion-repair`
- `./index.html#branch-popover`
- `./index.html#attachment-states`
- `./index.html#safe-fallback`
- `./index.html#coverage`

## Review Questions

1. Does this now feel like the true chat-session frontend shape rather than an
   instructional annotation sheet?
2. Is the approval prompt in the composer dock the right placement for
   allow/ask/deny/remember and session-only versus user-global remembered
   choices?
3. Do the `$`, `@`, and `/` suggestions feel like composer behavior rather
   than documentation?
4. Are file, Browser screenshot, Browser annotation bundle, and unsupported
   attachment fallback states visible without overwhelming the thread?

## Approval Boundary

No `.agents` specs, handoff docs, or durable product docs were updated in this
round. If this review round is approved, the next step remains updating spec 05
and only the directly affected overlap docs in specs 04, 07, 10, and 11 with
links to the approved routes/states as evidence.
