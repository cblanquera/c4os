# Plugin System And Settings Management Risks

Status: proposed

| ID | Risk | Mitigation |
| --- | --- | --- |
| RISK-001 | Full marketplace compatibility can imply native code loading. | Marketplace plugins bind only to preinstalled native modules. |
| RISK-002 | Malformed icons or manifests could compromise the shell. | Require sanitized SVG, schemaVersion, and visible disabled reasons. |

## POC Risk Notes

| Risk | POC Result |
| --- | --- |
| RISK-001 | `proofs/codex-marketplace-install-cache/` and `proofs/plugin-lifecycle-pending-restart-and-service-scope/` reduce install/lifecycle uncertainty without proving arbitrary native code loading. Native authority remains constrained to preinstalled modules. |
| RISK-002 | `proofs/plugin-svg-sanitization/` reduces icon risk for script, event handler, and external href inputs. Production implementation still needs a real parser/sanitizer rather than regex-only proof logic. |
