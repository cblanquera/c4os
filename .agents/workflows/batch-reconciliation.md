# Batch Reconciliation Workflow

Use this after feedback or QA findings have been audited or validated and the
work contains related low-to-medium-risk fixes that can be verified together.

## Use Batches When

- findings are related by screen, workflow, component, theme, copy, or state
- fixes are low-to-medium risk
- changed surfaces can be verified together
- separate micro-passes would repeat the same setup, review, or verification

Use one item at a time for architecture, security, permissions, data model,
deployment, unclear acceptance, blocked validation, or high rollback risk.

## Process

1. Confirm raw feedback has been validated, rejected, or marked for validation.
2. List each mismatch with source, surface, severity, dependency, and verification.
3. Pick a coherent batch and split unrelated or risky items out.
4. Implement the batch only when active execution is already authorized.
5. Verify the changed surfaces together.
6. Update progress, logs, status, and durable records once at the end.
7. Run document integrity before closeout.

## Stop

Stop when batch membership, verification, progress impact, remaining split-out
items, and the recommended next step are explicit.
