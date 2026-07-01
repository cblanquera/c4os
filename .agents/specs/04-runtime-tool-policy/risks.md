# Runtime And Tool Policy Risks

Status: proposed

| ID | Risk | Mitigation |
| --- | --- | --- |
| RISK-001 | A central taxonomy could overconstrain plugin-contributed tools. | Let plugins define taxonomy inside gateway contract. |
| RISK-002 | Allow-all user-directed reads could be misapplied to agent-initiated reads. | Record explicit user-directed boundary and approval proofs. |
