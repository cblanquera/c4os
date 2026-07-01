# Plugin System And Settings Management Risks

Status: proposed

| ID | Risk | Mitigation |
| --- | --- | --- |
| RISK-001 | Full marketplace compatibility can imply native code loading. | Marketplace plugins bind only to preinstalled native modules. |
| RISK-002 | Malformed icons or manifests could compromise the shell. | Require sanitized SVG, schemaVersion, and visible disabled reasons. |
