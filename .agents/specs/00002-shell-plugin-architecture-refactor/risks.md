# Shell Plugin Architecture Refactor Risks

Status: proposed

| ID | Risk | Mitigation |
| --- | --- | --- |
| RISK-001 | Treating app plugins as Tauri plugins could put product authority in the wrong layer. | Keep gateway, manifest, and native-module rules explicit. |
| RISK-002 | Hidden enabled plugin event delivery could cause surprising side effects. | Proof event fanout and require tool-view compatibility declarations. |
| RISK-003 | Full Codex compatibility can expand scope. | Use marketplace lifecycle proofs before implementation tasks. |

## POC Risk Notes

| Risk | POC Result |
| --- | --- |
| RISK-002 | `proofs/tool-event-fanout/` reduces the risk by proving hidden compatible views can receive state updates without panel open, focus, prompt, or duplicate backend execution. Production implementation still needs policy/audit enforcement. |
| RISK-003 | `proofs/bundled-plugin-lifecycle/` reduces lifecycle uncertainty for bundled/default sources. Marketplace compatibility beyond deterministic cache/reinstall semantics remains future proof scope. |
