# Runtime And Tool Policy Risks

Status: proposed

| ID | Risk | Mitigation |
| --- | --- | --- |
| RISK-001 | A central taxonomy could overconstrain plugin-contributed tools. | Let plugins define taxonomy inside gateway contract. |
| RISK-002 | Allow-all user-directed reads could be misapplied to agent-initiated reads. | Record explicit user-directed boundary and approval proofs. |

## POC Risk Notes

| Risk | POC Result |
| --- | --- |
| RISK-001 | `proofs/runtime-tool-discovery-without-plugin-view/` avoids a central app taxonomy by using registered tool identities and app-owned state. Plugin-contributed taxonomy remains future implementation detail. |
| RISK-002 | `proofs/user-directed-file-access-policy/` reduces policy ambiguity by proving user-directed outside reads differ from agent-initiated outside reads. Production implementation still needs request provenance captured in tool envelopes. |
