# Codex Compatibility Fixtures Proof

Status: passed for declared subset
Updated: 2026-07-18

## Pinned interpretation

This proof targets the official Codex manual fetched 2026-07-18 and the bundled `codex-cli 0.145.0-alpha.18` binary. C4OS compatibility means a versioned, one-way import adapter—not lossless Codex emulation.

## Run

```sh
node --test proofs/codex-compatibility-fixtures/proof.test.mjs
```

## Compatibility dispositions

- **Accepted:** portable fields C4OS can retain directly.
- **Translated:** supported through an explicit C4OS representation.
- **Rejected:** unsafe or non-importable input, including raw secrets and managed policy.
- **Ignored:** unknown fields or project-local machine settings, with diagnostics.

Plugins install disabled. Hooks remain disabled until separately trusted. A plugin `settings` schema is a C4OS extension and is translated rather than claimed as a portable Codex manifest field.

## Non-goals

This proof does not claim round-trip config export, full Codex behavior parity, hook execution, connector authorization, MCP process execution, or marketplace installation.

## Result

**Passed for the declared versioned subset** on 2026-07-18. Eight tests passed. The pinned Codex strict parser accepted known configuration and rejected an unknown key; the pinned marketplace loader listed the fixture as available without installing or enabling it. The C4OS importer proved precedence, untrusted-project suppression, machine-local key handling, environment secret references, raw-secret rejection, managed-policy rejection, plugin component classification, disabled hook trust, and metadata-only marketplace intake.

See `codex-compatibility-fixtures-evidence-2026-07-18.md`.
