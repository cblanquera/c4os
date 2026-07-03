# r05 Final Implementation Wireframes - Review Round 08

## Status

Ready for Batch 3 prompt-and-approval-flow review.

## What Changed

- Added focused prompt interaction routes for `$` skills, `@` file/plugin
  resource suggestions, and `/` runtime command suggestions.
- Added approval request and remembered-rule states covering allow, ask, deny,
  remember, session-only duration, user-global duration, typed decision
  summaries, and the route to Settings > Configuration.
- Added disabled/dependency-blocked suggestion repair, branch choose/create,
  file attachment, Browser screenshot attachment, Browser annotation bundle,
  unsupported attachment warning, and safe fallback states.
- Updated the coverage matrix for spec 05 and direct overlaps in specs 04, 07,
  10, and 11 while keeping Batch 2 Settings coverage visible.

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

1. Is the approval request clear enough about tool, action, target, risk,
   allow/ask/deny/remember, and session-only versus user-global remembered
   scope?
2. Do the `$`, `@`, and `/` prompt tags read as distinct suggestion systems
   without implying frontend-only execution authority?
3. Do the attachment chips and attachment detail cards show the right shape for
   files, Browser screenshots, and Browser annotation bundles?
4. Are the disabled/dependency-blocked repair state and unsupported-attachment
   fallback warnings explicit enough for a user to recover safely?

## Approval Boundary

No `.agents` specs, handoff docs, or durable product docs were updated in this
round. If this review round is approved, the next step is to update spec 05 and
only the directly affected overlap docs in specs 04, 07, 10, and 11 with links
to the approved routes/states as evidence. If not, revise the named prompt,
approval, attachment, branch, or fallback states inside this same r05 revision.
