# Extension System Proof

This proof checks whether C4OS can inventory extension metadata without
executing extension code, connect to a harmless local MCP server, and route
unknown or high-risk actions through an approval policy.

## Run

From the repository root:

```sh
node proofs/extension-system/extension_system_poc.mjs
```

The command rewrites `extension-system-evidence-2026-06-15.md` with the run
result.

## Result

The recorded run passed. It demonstrated static manifest inventory, disabled
and untrusted defaults, instruction-only skill handling, a constrained local
MCP call, and approval routing for unknown or high-risk actions.

## Limits

The fixtures are disposable and local. This proof does not establish package
installation, signatures, updates, production plugin execution, or trust in
external MCP servers.

See `extension-system-evidence-2026-06-15.md` for the recorded checks and raw
result.
