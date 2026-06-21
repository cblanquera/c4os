# Research Traceability

## Capability To Records

| Capability | Requirements | Acceptance | Risks | Questions |
| --- | --- | --- | --- | --- |
| CAP-001 Folder-first workspace | REQ-001, REQ-002, REQ-010 | AC-001, AC-002, AC-010 | RISK-005 |  |
| CAP-002 Persistent sessions | REQ-003, REQ-009, REQ-011, REQ-012 | AC-003, AC-009, AC-011, AC-012 | RISK-001, RISK-003 | Q-001, Q-004, Q-005 |
| CAP-003 Provider/model management | REQ-004, REQ-011, REQ-014 | AC-004, AC-011, AC-014 | RISK-004 |  |
| CAP-004 Approval gateway | REQ-005, REQ-011, REQ-012, REQ-014 | AC-005, AC-011, AC-012, AC-014 | RISK-001, RISK-002 | Q-001 |
| CAP-005 File explorer/editor | REQ-006, REQ-013 | AC-006, AC-013 | RISK-002 |  |
| CAP-006 Artifacts/preview | REQ-007, REQ-013 | AC-007, AC-013 | RISK-002 | Q-002 |
| CAP-007 Browser surface | REQ-007, REQ-013 | AC-007, AC-013 | RISK-002 | Q-002 |
| CAP-008 Terminal surface | REQ-008, REQ-013 | AC-008, AC-013 | RISK-002 | Q-003 |
| CAP-009 Extension install/connect | REQ-014, REQ-016, CON-004 | AC-014 | RISK-002, RISK-003 | Q-007 |

## Source To Records

| Source | Records |
| --- | --- |
| `plans/product-brief.md` | CAP-001 through CAP-009, REQ-001 through REQ-008, CON-001 through CON-004, DEC-001 through DEC-005, risks, assumptions, questions, acceptance |
| `plans/product-interface.md` | REQ-009, CON-005, AC-009, EVD-002 |
| `plans/pegs/*.png` | EVD-003, `wireframes/screens.md` |
| `.agents/specs/research/poc/brief.md` | EVD-004, TASK-002 |
| `.agents/specs/research/poc/results.md` | DEC-006, DEC-007, DEC-008, EVD-005, EVD-006, EVD-007, EVD-008 |
| `proofs/opencode-runtime/opencode-runtime-evidence-2026-06-20.md` | EVD-005, DEC-006 |
| `proofs/pi-runtime/pi-runtime-evidence-2026-06-20.md` | EVD-006, DEC-006 |
| `proofs/native-browser-plugin/native-browser-plugin-evidence-2026-06-20.md` | EVD-007, DEC-007 |
| `proofs/native-browser-wry/native-browser-wry-evidence-2026-06-20.md` | EVD-007, DEC-007 |
| `proofs/native-browser-tauri/native-browser-tauri-evidence-2026-06-20.md` | EVD-007, DEC-007 |
| `proofs/native-terminal-plugin/native-terminal-plugin-evidence-2026-06-20.md` | EVD-008, DEC-008 |
| `proofs/native-terminal-tauri/native-terminal-tauri-evidence-2026-06-20.md` | EVD-008, DEC-008 |
| `wireframes/ui-handoff-spec.md` | EVD-009, DEC-009, REQ-009 through REQ-014, AC-009 through AC-015, TASK-005 |
| `wireframes/r04-single-page-app/README.md` | EVD-009, DEC-009, TASK-005 |
| `wireframes/r04-single-page-app/qa/notes.md` | EVD-009, TASK-005 |
| `.agents/references/research/grill-session.md` | Q-001 through Q-010, DEC-005, DEC-006, DEC-008, DEC-010, DEC-011, REQ-015, REQ-016, `checkpoints.md` |
| `.agents/specs/research/research-freeze.md` | Frozen research findings, candidate MVP scope, required MVP-spec next step |
| `.agents/specs/research/implementation-checkpoint-plan.md` | Implementation sequencing checkpoints |
| `docs/adr/0001-browser-and-agent-authority.md` | Browser and agent authority decision |
| `docs/adr/0002-extension-enablements-and-prompt-tags.md` | Extension enablement and prompt tag decision |
| `.agents/references/research/context-promotion-update.md` | Context promotion summary |
