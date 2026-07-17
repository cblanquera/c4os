# C4OS Proofs

This directory contains isolated experiments used to test C4OS product and
architecture decisions without treating the experiments as production code.
Each proof documents the question it tests, how to run or review it, what it
demonstrates, and what remains outside its scope.

## Start Here

Run every dependency-free Node test harness from the repository root:

```sh
node --test proofs/*/proof.test.mjs
```

Other proofs use Rust, Python, browser review, or historical third-party
dependencies. Open the proof directory's `README.md` before running one of
those artifacts.

## Proof Groups

The proof directories are grouped by the C4OS capability they explore:

- **Runtime and policy:** approvals, attachments, prompt tags, runtime tools,
  event fanout, file access, and Pi/OpenCode adapter boundaries.
- **Plugins and extensions:** lifecycle, settings, migration failures, SVG
  safety, marketplace cache behavior, skills, and extension inventory.
- **Workspace and files:** portable workspace descriptors, shared chat state,
  removal semantics, and editor/backend ownership.
- **Browser:** isolated browser surfaces, state hydration, annotations, and
  document preview ownership.
- **Terminal:** PTY lifecycle, renderer behavior, and the xterm-to-PTY bridge.
- **Shell and debugging:** panel layout/restore behavior and redacted Chat
  Debug history.

## Documentation Contract

The executable source and tests inside each proof directory define what the
proof currently checks. Dated evidence files record a specific historical run;
they are not current project specifications or guarantees about production
behavior.

References to deleted planning files or retired task identifiers should not be
used as proof ownership. When C4OS planning records are introduced again, link
them from the relevant proof README only while those records exist.

## Result Language

- **Passed** means the isolated harness met its own assertions on the recorded
  environment.
- **Partial** means the proof established only part of the intended boundary.
- **Failed** means the tested approach violated at least one proof criterion.
- **Not proven** means the behavior belongs to production integration,
  cross-platform validation, or another explicit follow-up.
