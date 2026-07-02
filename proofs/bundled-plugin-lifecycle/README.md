# Bundled Plugin Lifecycle Proof

Status: proof artifact
Updated: 2026-07-02

## Run

```sh
node --test proofs/bundled-plugin-lifecycle/proof.test.mjs
```

## Result

The harness proves bundled/default plugins can begin installed but disabled,
be enabled, prompt for plugin-owned data deletion on uninstall, remove the
installed copy, and reinstall from the bundled/default source back to disabled.
