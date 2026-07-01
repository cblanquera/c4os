# Review Workflow

Use this for readiness, risk, consistency, evidence, or traceability review.

## Process

1. Identify the target artifact and review purpose.
2. Check source traceability, MVP boundaries, feature boundaries, risks,
   acceptance criteria, decisions, proof needs, and unresolved questions.
3. Verify that shared truth is in `.agents/context/`, long support is in
   `.agents/references/`, and each spec stands on context, references, and its
   own files rather than sibling specs.
4. Verify accepted grill or decision-question IDs are represented in the right
   spec decisions or explicitly marked superseded, rejected, or deferred.
5. Classify findings as `BLOCKER`, `HIGH`, `MEDIUM`, `LOW`, or `QUESTION`.
6. Reconcile material findings into records, blockers, validation targets, accepted risks, explicit rejections, or batch-reconciliation candidates.

## Stop

Stop when findings are actionable and the next step is validation, revision, freeze, POC, or implementation planning.
