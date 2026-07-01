# Shell Plugin Architecture Refactor Risks

Status: proposed

| ID | Risk | Mitigation |
| --- | --- | --- |
| RISK-001 | Treating app plugins as Tauri plugins could put product authority in the wrong layer. | Keep gateway, manifest, and native-module rules explicit. |
| RISK-002 | Hidden enabled plugin event delivery could cause surprising side effects. | Proof event fanout and require tool-view compatibility declarations. |
| RISK-003 | Full Codex compatibility can expand scope. | Use marketplace lifecycle proofs before implementation tasks. |
