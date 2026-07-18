# Platform Window And Menu Proof

Status: proved and accepted for the macOS standard-decoration baseline; Windows and Linux are `not run` and deferred to a separate spec

## Question

Can native menus and standard decorations provide a platform-correct Settings entry and preserve expected window behavior without faking native controls in content?

## Run

```sh
node --test proofs/platform-window-menu/proof.test.mjs
cargo run --manifest-path proofs/platform-window-menu/Cargo.toml
```

Review the native application menu and activate Settings with both the menu item and `Cmd+,`. Verify native traffic lights, titlebar drag, resize, minimize/zoom/full-screen, keyboard focus, light/dark adaptation, dialog focus, Escape, and focus restoration.

## Boundaries

This proof deliberately does not test a transparent, overlay, or custom titlebar. Standard decorations are the fallback baseline. macOS does not establish Windows caption/system-menu or Linux compositor behavior; those targets are `not run`.

## Result

The real native menu and `Cmd+,` both opened the owned Settings route while standard macOS decorations remained intact. See [dated evidence](platform-window-menu-evidence-2026-07-18.md).
