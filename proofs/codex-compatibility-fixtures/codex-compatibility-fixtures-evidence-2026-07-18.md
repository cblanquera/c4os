# Codex Compatibility Fixtures Evidence — 2026-07-18

## Source pin

- Official Codex manual fetched 2026-07-18.
- Local validator: `codex-cli 0.145.0-alpha.18` bundled with the ChatGPT desktop app.
- Relevant official sections: Config basics, Advanced Configuration, Configuration Reference, Build plugins, Build skills, Hooks, and Plugins.

## Command

```sh
node --test proofs/codex-compatibility-fixtures/proof.test.mjs
```

## Observed signal

- 8 tests passed; 0 failed, skipped, cancelled, or todo.
- Codex strict config parsing accepted documented fields and rejected `not_a_codex_key`.
- The C4OS importer applied system, user, profile, trusted project root-to-leaf, and CLI precedence.
- An untrusted project suppressed all project configuration.
- Project-local machine settings were ignored with diagnostics.
- Environment-key secret references survived translation; a raw `sk-` value did not.
- `requirements.toml` was rejected as managed policy, not imported as user preference.
- Skills, app/MCP, C4OS settings extension, and hooks received explicit compatibility dispositions.
- The real Codex marketplace loader listed all four fixture shapes—skills, app/MCP, settings extension, and hooks—as available, uninstalled, and disabled.

## Result

**Passed for `codex-0.145-c4os-import-v1`.** This is a one-way supported subset, not exact Codex emulation or lossless export.

## Compatibility boundary

- Direct: selected portable model/approval/sandbox/web fields and documented plugin components.
- Translated: MCP/provider declarations, hooks into disabled trust review, and C4OS-only settings schemas.
- Rejected: raw secrets and managed requirements.
- Ignored with diagnostics: unknown fields and forbidden project-local machine keys.
