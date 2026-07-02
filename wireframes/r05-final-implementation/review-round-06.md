# r05 Final Implementation Wireframes - Review Round 06

## Status

Ready for targeted review after Settings feedback on Configuration,
Marketplace, and Skills.

## What Changed

- Settings > Configuration removed remembered/session rule text from the policy
  rows. Rows now show only the server-tool policy summary and icon-only
  edit/revoke actions.
- Settings > Plugin Marketplace now follows the r04 functional pattern and page
  identity: the route renders as the Plugins page, `Built by C4OS` opens a
  menu, `+ Add Marketplace` opens the add-marketplace dialog, and plugin add
  buttons open the connect dialog.
- Settings > Skills now follows the r04 functional pattern: skill rows are
  buttons and open the skill detail modal instead of showing a static inline
  detail panel on the list route.
- Settings > Plugin Detail keeps the rendered plugin form changes from the
  previous feedback round: input, number, switch, select, sensitive/redacted,
  default-only, and icon-order examples in one rendered form.

## Review Links

- `./index.html#settings-configuration`
- `./index.html#settings-plugin-marketplace`
- `./index.html#settings-skills`
- `./index.html#settings-plugin-detail`

## Review Questions

1. In Settings > Configuration, should policy rows keep edit and revoke icon
   actions visible even when no durable user-global approval rule is shown?
2. In Settings > Plugin Marketplace, does the r04-style clickable source menu
   and modal flow match the expected source behavior now?
3. In Settings > Skills, does the r04-style list-to-detail modal match the
   expected skill management behavior now?
4. In Settings > Plugin Detail, is the default-only field example clear enough
   to show plugin-provided config values that the user cannot change?

## Verification

- `node --check wireframes/r05-final-implementation/script.js`
- `git diff --check -- wireframes/r05-final-implementation`
- Browser plugin checks on `http://127.0.0.1:4187/index.html` for the review
  links above.

## Approval Boundary

No `.agents` specs, handoff docs, or durable product docs were updated in this
round. Those updates remain blocked until this batch is explicitly approved.
