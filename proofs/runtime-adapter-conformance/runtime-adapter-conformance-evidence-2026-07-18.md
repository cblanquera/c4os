# Runtime Adapter Conformance Evidence — 2026-07-18

## Pins

- `@opencode-ai/sdk@1.18.3`
- `opencode-ai@1.18.3`
- `@earendil-works/pi-coding-agent@0.80.10`
- `@earendil-works/pi-agent-core@0.80.10`

`package-lock.json` records registry integrity values.

## Commands and observed signal

```sh
cd proofs/runtime-adapter-conformance
npm test
```

- 11 tests passed; 0 failed, skipped, cancelled, or todo.
- Both adapter contract doubles passed the required C4OS baseline.
- The actual Pi 0.80.10 agent intercepted validated arguments before tool execution; denial produced an error result and the tool body executed zero times.
- Current CLI versions and SDK exports matched the expectation manifests.

```sh
npm run smoke:opencode
```

- OpenCode version: `1.18.3`.
- Unauthenticated health: HTTP 401.
- Authenticated health: healthy.
- Session create: succeeded in canonical isolated temporary directory.
- Session abort: accepted.

```sh
npm run proof:opencode-permission
```

- OpenCode loaded wildcard `ask` policy from an isolated configuration root.
- A provider-backed `openai/gpt-4o-mini` turn proposed a real write.
- OpenCode emitted a legacy permission request (`external_directory`, due to the canonical macOS temporary path boundary).
- The proof observed that the target file did not exist, replied `reject`, aborted the session, and observed that the file still did not exist.
- The copied auth record was deleted with the temporary state and no credential value was printed.
- A per-request `tools: { write: true }` diagnostic run bypassed the configured ask rule; OCAdapter must forbid or independently authorize that override.

## Result

**Passed for the tested adapter boundary.** PIAdapter's current-package pre-tool denial and OCAdapter's real model-backed permission rejection both prevented side effects. Both C4OS contract shapes passed.

## Transport conclusion

Use a C4OS-owned Node SDK sidecar for PIAdapter. Keep RPC documented as an alternative; it was not selected because the SDK sidecar gives C4OS more direct tool interception, resource control, and event correlation. This is not a primary-runtime decision.

## Remaining unknowns

- Provider-backed streaming for both runtimes.
- Signed/bundled sidecars and descendant cleanup on macOS, Windows, and Linux.
- Durable recovery under actual runtime process/database corruption.
