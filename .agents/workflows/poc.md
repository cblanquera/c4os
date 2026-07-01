# POC Workflow

Use this when feasibility could change MVP scope, architecture, runtime strategy, security boundaries, browser isolation, terminal lifecycle, persistence, or extension execution.

## Process

1. Record the proof question and why it matters.
2. Define expected proof, failure signal, and verification method.
3. Keep POC records inside the target spec's `poc/` folder.
4. Keep related assumptions, risks, evidence, decisions, and tasks in the parent spec with `Phase: poc`.
5. Put runnable POC code, harnesses, fixtures, and evidence in `proofs/<proof-name>/`.
6. Do not create branches or prototype code unless the user explicitly asks for implementation.
7. Record whether the result should be promoted, replaced, discarded, or continued.
8. Promote accepted reusable POC learning into the relevant `.agents/context/`
   file before it becomes implementation work.
9. If a POC decision affects multiple specs, copy the relevant decision into
   each affected spec's `decisions.md` and cite the proof record.

## Stop

Stop when the POC result or planned proof can support a product decision.
