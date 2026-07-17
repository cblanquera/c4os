# Browser QA Notes

## 2026-07-15

- Entry point: `http://127.0.0.1:4173/workflows.html`
- Review screen: `http://127.0.0.1:4173/index.html`
- Wide viewport capture: `desktop.png`
- Narrow viewport capture: `narrow.png` at 720 by 900.
- Workflow launcher opened the SPA successfully.
- Both desktop side panels rendered blank and open by default.
- Left resize separator increased the panel width from 248px to 264px with the ArrowRight key.
- Left panel toggle synchronized `aria-expanded` and `aria-hidden` while closing and reopening the panel.
- Approval menu opened, exposed three choices, and updated the control from Ask to Review.
- Populated prompt enabled Send; pressing Enter appended one user message and one simulated AI response.
- At the narrow viewport, both panels started closed and the center region occupied the full 720px width.
- Browser console check returned no warnings or errors.

## Round 2 - 2026-07-15

- Default viewport capture: `round-2-desktop.png`.
- Narrow viewport capture: `round-2-narrow.png` at 720 by 900.
- Existing user messages contained zero visible speaker labels.
- Existing assistant messages both showed the `GPT-5` model label.
- Composer controls appeared in the requested Attach, model, approval, branch order.
- The first `Worked for 1 min 5 sec` disclosure opened and revealed a three-item work summary.
- A newly submitted prompt created an unlabeled user bubble and a `GPT-5` assistant bubble with a `Worked for 14 sec` disclosure.
- The narrow layout had no horizontal document overflow.
- Browser console check returned no warnings or errors.

## Round 3 - 2026-07-15

- Expanded-state capture: `round-3-thinking.png`.
- The first `Worked for 1 min 5 sec` disclosure revealed two progress paragraphs and two muted activity rows.
- Computed styles confirmed a transparent background, no border, and no box shadow on the expanded content.
- The visible `Work summary` heading and summary-list treatment were absent.
- A newly submitted response used the same unboxed thinking-activity pattern.
- Browser console check returned no warnings or errors.
