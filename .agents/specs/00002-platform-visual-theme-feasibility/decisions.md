# Decisions And Gaps

## Accepted Constraints

### D-001 — Preserve the system-following product direction

State: Accepted input from the usability KB

C4OS follows the host platform's visual language and current system light/dark preference. No manual C4OS-only theme override is introduced in the reconstructed r012 scope. This spec validates implementation and fallback boundaries; it does not reopen that product choice.

### D-002 — Require target-scoped evidence

State: Accepted research boundary

Documentation may establish API availability, but native appearance and webview behavior require evidence on each claimed target. A pass on one operating system cannot silently establish another. Missing target evidence blocks only that platform-specific claim or release target.

### D-003 — Preserve semantic intent when native values are unavailable

State: Accepted research boundary

Platform-derived colors, fonts, materials, radii, controls, and scrollbars are preferences, not permission to produce inconsistent or inaccessible UI. A neutral semantic-token fallback remains valid when a native value is unavailable or unreliable, provided the fallback is explicit and passes contrast/state acceptance.

## Open Gaps

| ID | Question | Assumption to verify | Owner |
| --- | --- | --- | --- |
| GAP-001 | Which Tauri/native/webview source is authoritative for initial and live theme state on each target? | Use native window state/events first and CSS media queries as a fallback, while keeping window theme unset to follow the system. | Research |
| GAP-002 | Can C4OS resolve platform and scheme before first reveal without a wrong-theme flash or delayed unusable window? | Create the window hidden or pre-style the document, resolve the initial state, then reveal with a bounded fallback. | Research + Proof P-001 |
| GAP-003 | Which system-derived values can C4OS reliably obtain for accent, contrast, reduced motion, fonts, controls, and scrollbars? | Normalize only stable inputs; use CSS system colors/OS APIs/portals where verified and explicit fallback tokens elsewhere. | Research + Proof P-002 |
| GAP-004 | Which native menu, Settings shortcut, titlebar, decoration, inset, caption, and drag behaviors are safe on each platform? | Prefer native decorations and menus; use transparent/custom titlebars only where they preserve expected system behavior. | Research + Proof P-003 |
| GAP-005 | What Linux desktop, portal, compositor, toolkit, and WebKitGTK variations require distinct fallbacks? | Treat Linux as a capability matrix rather than one visual platform; avoid claiming an active desktop font or control language without a stable source. | Research + Proofs P-001–P-003 |
| GAP-006 | Which OS versions and Linux environments define the supported visual/theme release matrix? | Prove only named environments and convert every untested environment into an explicit target gate. | User review after research |

## Conflict Handling

If evidence makes a concrete rule in the platform reference infeasible, do not silently weaken Context. Record the conflict here, propose the narrowest fallback, and obtain user acceptance before revising accepted reusable truth.
