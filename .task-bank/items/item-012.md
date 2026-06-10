# item-012: Implement Runtime Stop And Crash Interruption Mapping

Status: verified
Type: implementation
Priority: high
Owner: codex

## Goal

Map user stop, runtime abort, process termination, app close, crash, and
force-quit into durable app-owned states.

## Inputs

- `plans/validation/FINDING-001-stop-behavior.md`
- `plans/acceptance/agent-execution.md`
- `plans/acceptance/sessions.md`

## Outputs

- Stop command path.
- Runtime abort mapping.
- App-supervised child-process termination hook.
- Interrupted/crashed restart marking.

## Constraints

- Stop preserves partial assistant output and tool records.
- Pending approvals must not replay after restart.
- Stop must not kill arbitrary external processes outside the app-supervised
  runtime/process tree.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm test`
- `npm run tauri -- build`

## Dependencies

- item-010

## Notes

- Verified on 2026-06-11 with supervised process stop, record preservation,
  restart interruption mapping, Node tests, and native Tauri build.
