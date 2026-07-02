# Shell Plugin Architecture Refactor Proof Planning

Status: proposed

Proof implementation artifacts, if approved during proof execution, must live under the listed `proofs/<proof-name>/` paths. Do not create proof code during planning replay.

| Proof Path | Proof Question |
| --- | --- |
| proofs/tool-event-fanout/ | Prove one tool event reaches all enabled compatible visible and hidden plugin views without duplicate backend calls. |
| proofs/bundled-plugin-lifecycle/ | Prove bundled plugins install disabled by default, uninstall, reinstall from bundled/default marketplace, and surface data-delete prompts. |
| proofs/plugin-migration-failure-handling/ | Prove migration failures route to auto-recovery, reset, or visible disabled repair states. |
