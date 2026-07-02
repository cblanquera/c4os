# Shell Plugin Architecture Refactor Proof Planning

Status: proposed

Proof implementation artifacts, if approved during proof execution, must live under the listed `proofs/<proof-name>/` paths. Do not create proof code during planning replay.

| Proof Path | Proof Question |
| --- | --- |
| proofs/tool-event-fanout/ | Prove one tool event reaches all enabled compatible visible and hidden plugin views without duplicate backend calls, and that hidden compatible plugin instances only update per-chat state/unread indicators without opening panels, stealing focus, or prompting the user. |
| proofs/bundled-plugin-lifecycle/ | Prove bundled plugins install disabled by default, uninstall, reinstall from bundled/default marketplace, and surface data-delete prompts. |
| proofs/plugin-migration-failure-handling/ | Prove migration failures route to auto-recovery, reset, or visible disabled repair states. |

## Execution Results

Verification command:

```sh
node --test proofs/tool-event-fanout/proof.test.mjs proofs/bundled-plugin-lifecycle/proof.test.mjs proofs/plugin-migration-failure-handling/proof.test.mjs
```

| Proof Path | Result | Decision |
| --- | --- | --- |
| proofs/tool-event-fanout/ | Passed. One `browser.open` backend call fans out to visible compatible views and hidden compatible views; hidden views update state only and do not open, focus, prompt, or duplicate execution. | Promote event fanout as a valid implementation direction for REQ-005. |
| proofs/bundled-plugin-lifecycle/ | Passed. Bundled/default plugin lifecycle covers installed-disabled default, enable, uninstall data-delete prompt, uninstall, reinstall from bundled/default source, and disabled reinstall state. | Promote bundled/default marketplace lifecycle as feasible for REQ-002 and REQ-003. |
| proofs/plugin-migration-failure-handling/ | Passed. Cache failure auto-recovers, corrupt config resets with notice, unsupported schema disables with reason, missing dependency blocks, and security violation disables. | Promote migration classification table as feasible for REQ-008. |
