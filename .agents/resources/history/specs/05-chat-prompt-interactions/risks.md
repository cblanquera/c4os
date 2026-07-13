# Chat Prompt Interactions Risks

Status: proposed

| ID | Risk | Mitigation |
| --- | --- | --- |
| RISK-001 | Prompt tagging can become frontend-only parsing. | Route through runtime/tool gateway where needed. |
| RISK-002 | Attachments can silently disappear for unsupported models. | Require warning and fallback acceptance. |
