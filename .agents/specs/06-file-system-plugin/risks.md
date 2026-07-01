# File System Plugin Risks

Status: proposed

| ID | Risk | Mitigation |
| --- | --- | --- |
| RISK-001 | Path-as-identity relocation can orphan sessions. | Require migration proof for relocated canonical path. |
| RISK-002 | User-global config path choices can drift by OS. | Use platform-standard per-user config paths. |
