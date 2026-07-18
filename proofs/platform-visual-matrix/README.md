# Platform Visual Acceptance Matrix Proof

Status: proved and accepted on the tested macOS matrix; Windows and Linux are `not run` and deferred to a separate spec

## Question

Can the semantic contract render representative C4OS surfaces clearly on the accepted macOS target without light leaks, document overflow, lost focus indication, or decorative-motion dependence?

## Run

```sh
node --test proofs/platform-visual-matrix/proof.test.mjs
cargo run --manifest-path proofs/platform-visual-matrix/Cargo.toml
```

Review shell, transcript, artifact, editor, terminal, Settings, popover, and dialog fixtures in macOS Light and Dark. Run the in-app checks, exercise keyboard focus and Escape dismissal, resize to the minimum window, and confirm local code/editor/terminal containment.

## Boundaries

This is a deterministic representative matrix, not full production UI acceptance. Forced-colors and reduced-motion styles are present, but native macOS offers no Windows forced-colors environment. Windows, Linux, unavailable architectures, KDE, and X11 remain `not run`.

## Result

All eight fixture families rendered in macOS Light and Dark, with 23/23 in-app runtime checks and keyboard dismissal/focus behavior passing. See [dated evidence](platform-visual-matrix-evidence-2026-07-18.md).
