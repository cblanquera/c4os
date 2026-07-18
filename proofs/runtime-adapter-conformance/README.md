# Runtime Adapter Conformance Proof

Status: passed for the adapter/runtime boundary under test
Updated: 2026-07-18

## Question

Can separate peer `OCAdapter` and `PIAdapter` implementations satisfy one C4OS-owned baseline while declaring their different native capabilities and keeping policy enforcement outside either runtime?

## Pinned packages

- OpenCode SDK and CLI: `1.18.3`
- Pi coding agent and agent core: `0.80.10`

The exact npm versions and integrity hashes are stored in `package-lock.json`.

## Run

```sh
cd proofs/runtime-adapter-conformance
npm test
npm run smoke:opencode
npm run proof:opencode-permission
```

The OpenCode smoke command binds an authenticated loopback server and may require permission to open a local port. It uses a fresh temporary XDG state root and makes no provider-backed prompt.

The permission proof makes one minimal provider-backed request using an existing OpenCode auth record copied into a temporary isolated state root. It requests a file write, observes the live permission request, denies it through the SDK, proves the file is absent before and after denial, redacts credentials from output, and deletes the temporary auth copy on exit. Set `C4OS_OPENCODE_AUTH_SOURCE` to override the normal OpenCode auth path.

Do not pass a per-request `tools` override when relying on OpenCode's configured permission policy. In the pinned runtime, `tools: { write: true }` enabled that tool for the request and bypassed the loaded wildcard `ask` rule. OCAdapter must instead configure the runtime-owned tool set at startup and treat per-request tool overrides as authority-bearing input.

## Transport decision under proof

PIAdapter uses a C4OS-owned Node SDK sidecar. Pi RPC remains a supported alternative, but the SDK surface provides direct `beforeToolCall` interception, explicit custom/built-in tool control, resource loading, and stronger adapter-owned correlation. This selects an integration transport only; it does not make Pi primary or rank Pi against OpenCode.

## Non-goals

No remote OpenCode server, provider matrix, production runtime binary, OS sandbox, or cross-platform certification is included. The live OpenCode permission run uses one minimal provider-backed request and a temporary copy of an existing auth record.

## Result

**Passed for the adapter/runtime boundary under test** on 2026-07-18.

- 11/11 deterministic/current-package tests passed.
- Pi `0.80.10` used its real faux provider and `beforeToolCall`; the denied tool body did not execute.
- OpenCode `1.18.3` started with isolated temporary state and a random password; unauthenticated health returned 401, authenticated health reported the pinned version, a session was created in the isolated directory, and abort succeeded.
- OpenCode `1.18.3` emitted a real permission request for the model-proposed write. C4OS rejected it through the SDK before the file existed, and the file remained absent afterward.
- Both controlled adapters passed baseline, action-intent, denial, allow-once, cancellation, stale-event, recovery, capability, and redaction assertions.

Production packaging and platform cleanup are separate supervisor/release proofs rather than adapter-conformance failures.

See `runtime-adapter-conformance-evidence-2026-07-18.md`.
