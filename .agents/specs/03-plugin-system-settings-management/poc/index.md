# Plugin System And Settings Management Proof Planning

Status: proposed

Proof implementation artifacts, if approved during proof execution, must live under the listed `proofs/<proof-name>/` paths. Do not create proof code during planning replay.

| Proof Path | Proof Question |
| --- | --- |
| proofs/plugin-settings-renderer/ | Prove plugin-declared settings render and persist through config.toml/user config, including field metadata, sensitive fields stored through secure secret storage/keychain with redacted config placeholders, simple `visibleWhen`, unknown-key warnings, and reserved-key validation for `panel`, `enabled`, and `iconOrder`. |
| proofs/codex-marketplace-install-cache/ | Prove marketplace source install, cache path, uninstall, and reinstall semantics. |
| proofs/plugin-svg-sanitization/ | Prove unsafe SVG content is blocked and fallback icon renders. |
| proofs/plugin-lifecycle-pending-restart-and-service-scope/ | Prove UI/settings lifecycle changes apply immediately, backend/native registration changes are pending-restart, unavailable tools are hidden/disabled correctly, typed dependencies in a separate top-level manifest section produce visible blocked/degraded states, and heavy services are lazily started, shared at the narrowest safe scope, observable, and shut down on lifecycle/idle boundaries. |
