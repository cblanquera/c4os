# Plugin System And Settings Management Proof Planning

Status: proposed

Proof implementation artifacts, if approved during proof execution, must live under the listed `proofs/<proof-name>/` paths. Do not create proof code during planning replay.

| Proof Path | Proof Question |
| --- | --- |
| proofs/plugin-settings-renderer/ | Prove plugin-declared settings render and persist through config.toml/user config, including field metadata, sensitive fields stored through secure secret storage/keychain with redacted config placeholders, simple `visibleWhen`, unknown-key warnings, and reserved-key validation for `panel`, `enabled`, and `iconOrder`. |
| proofs/codex-marketplace-install-cache/ | Prove marketplace source install, cache path, uninstall, and reinstall semantics. |
| proofs/plugin-svg-sanitization/ | Prove unsafe SVG content is blocked and fallback icon renders. |
| proofs/plugin-lifecycle-pending-restart-and-service-scope/ | Prove UI/settings lifecycle changes apply immediately, backend/native registration changes are pending-restart, unavailable tools are hidden/disabled correctly, typed dependencies in a separate top-level manifest section produce visible blocked/degraded states, and heavy services are lazily started, shared at the narrowest safe scope, observable, and shut down on lifecycle/idle boundaries. |

## Execution Results

Verification command:

```sh
node --test proofs/plugin-settings-renderer/proof.test.mjs proofs/codex-marketplace-install-cache/proof.test.mjs proofs/plugin-svg-sanitization/proof.test.mjs proofs/plugin-lifecycle-pending-restart-and-service-scope/proof.test.mjs
```

| Proof Path | Result | Decision |
| --- | --- | --- |
| proofs/plugin-settings-renderer/ | Passed. Settings schema renders fields, hides `visibleWhen` fields until conditions match, persists sensitive values as secret references, redacts display, warns on unknown keys, and rejects invalid shell-reserved keys. | Promote settings schema, sensitive storage, and reserved-key validation as feasible. |
| proofs/codex-marketplace-install-cache/ | Passed. Marketplace install reads metadata, writes C4OS cache path, removes cache on uninstall, and reinstalls from GitHub/ref source. | Promote installed-cache semantics as feasible. |
| proofs/plugin-svg-sanitization/ | Passed. Static SVG is accepted; scripts, event handlers, and external hrefs are blocked with fallback icon. | Promote static sanitized SVG icon rule as feasible. |
| proofs/plugin-lifecycle-pending-restart-and-service-scope/ | Passed. UI settings apply live, backend tools become pending-restart, unavailable tools hide, dependencies block/degrade visibly, and heavy services share one workspace-scoped instance with idle/disable/uninstall/app-exit shutdown reasons. | Promote restart gating, dependency states, and shared service lifecycle as feasible. |
