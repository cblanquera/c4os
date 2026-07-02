# r05 Final Implementation Wireframes - Review Round 07

## Status

Ready for targeted review after Marketplace source and plugin-purpose feedback.

## What Changed

- The Built by C4OS marketplace menu now shows `Built by C4OS`, a separator
  line, then `+ Add Marketplace`.
- The Built by C4OS catalog now uses the pending-spec C4OS plugins:
  File system, File editor, Terminal, Chat Debug, and Browser.
- The plugin connect modal keeps `Advanced settings` as a path to
  `#settings-plugin-detail`, where the rendered plugin configuration form,
  repair states, and tool policy remain reviewable.
- Installed plugin examples, dependency-blocked states, uninstall prompt, and
  repair-state example now align to the same C4OS plugin set instead of old
  third-party examples.

## Review Links

- `./index.html#settings-plugin-marketplace`
- `./index.html#settings-plugin-detail`
- `./index.html#settings-plugin-states`

## Review Questions

1. Does the Built by C4OS menu read correctly with the source label and
   separator above `+ Add Marketplace`?
2. Are the five pending-spec C4OS plugins the right marketplace items for this
   review batch?
3. Does `Advanced settings` clearly route reviewers to the rendered settings
   form, repair states, and tool policy review surface?

## Approval Boundary

No `.agents` specs, handoff docs, or durable product docs were updated in this
round. Those updates remain blocked until this batch is explicitly approved.
