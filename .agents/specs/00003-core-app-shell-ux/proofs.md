# Core App Shell UX Proof Planning

Status: proposed

Proof implementation artifacts, if approved during proof execution, must live under the listed `proofs/<proof-name>/` paths. Do not create proof code during planning replay.

| Proof Path | Proof Question |
| --- | --- |
| proofs/shell-panel-resize-and-restore/ | Prove panel toggles, session-scoped visibility, Settings restore, and 640px center minimum. |

## Execution Results

Verification command:

```sh
node --test proofs/shell-panel-resize-and-restore/proof.test.mjs
```

| Proof Path | Result | Decision |
| --- | --- | --- |
| proofs/shell-panel-resize-and-restore/ | Passed. Plugin icon clicks toggle panels, same-side plugins replace the visible panel, Settings closes panels then restores the prior per-session state, session switches restore independent visibility, and resize collision closes the opposite panel before clamping to a 640px center minimum. | Promote the shell layout state-machine direction as feasible for panel toggle, per-session restore, Settings route, and resize invariants. |
