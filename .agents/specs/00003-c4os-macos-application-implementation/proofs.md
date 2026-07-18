# Proof Disposition

## State

The one new pre-Freeze Proof required by R-004 and D-023 passed on 2026-07-18. The other selections are evidence-resolved at the planning boundary; no Proof replaces production verification.

## Inherited Evidence

Spec 00001 contains feasibility evidence for adapter contracts, denial before side effects, policy classification, sidecar supervision, plugin trust and hook isolation, browser isolation, and Local/Docker/OpenSSH execution parity.

Spec 00002 contains macOS feasibility evidence for initial/live theme handling, semantic platform inputs, native menu and standard window behavior, and a representative visual matrix.

These results may answer feasibility questions and shape production tests. They do not prove the production application, selected dependency versions, packaging, signing, or target release.

## Completed Proof — macOS Native WebKit Production Boundary

- **Gap:** GAP-005.
- **Path:** `proofs/macos-wkwebview-production-boundary/`.
- **Hypothesis:** a public-API native `WKWebView` child/controller can embed in the locked Tauri macOS application, keep hostile pages outside all Tauri/Wry/C4OS IPC, preserve platform-default permission behavior, and implement the accepted persistent/ephemeral Browser Environment lifecycle.
- **Expected signal:** a real macOS run proves child-view attachment/resizing/focus; hostile remote-like content cannot observe or invoke a host bridge; sanitized navigation/title/loading/error/crash events reach the core; popups and downloads follow controller policy; camera/microphone and geolocation exercise real WebKit/system permission UX without blanket grant/denial; persistent identifier-backed profiles isolate and survive restart; nonpersistent profiles disappear on close; scoped clearing removes only the target profile.
- **Failure signal:** any page-accessible host IPC, cross-scope storage leak, inability to retain platform-default prompts, reliance on private APIs or a custom raw-profile path, unstable native-view embedding, unsanitized page data crossing the controller, or incomplete ephemeral/scoped-clear behavior.
- **Scope:** current locked macOS, Tauri, WebKit, and `objc2-web-kit` versions; one native child Browser surface; hostile loopback pages plus real permission devices where available; persistent and nonpersistent data stores; navigation, popup, download, crash, and cleanup controller rules.
- **Non-goals:** the production Browser UI, Windows/Linux webviews, distribution signing/notarization, browser sub-tabs, arbitrary automation, or proof that every public website works.
- **Result:** Passed twice consecutively on macOS 26.5.1 (25F80), arm64, with Tauri 2.11.2 and `objc2-web-kit` 0.3.2. All eight checks passed; the final raw result contains no fixture exception or error sentinel.
- **Evidence:** [dated evidence record](../../../proofs/macos-wkwebview-production-boundary/macos-wkwebview-production-boundary-evidence-2026-07-18.md) and [raw final result](../../../proofs/macos-wkwebview-production-boundary/macos-wkwebview-production-boundary-result.json).
- **Implementation constraint learned:** scoped clearing must use public per-store data removal, release the cleared `WKWebsiteDataStore` handle, and reopen the same stable identifier. The registered process-termination callback is present, but the Proof exercises its C4OS normalization deterministically rather than forcing a private-API WebContent crash.

The Proof proposal, command, raw evidence, limits, and accepted implementation constraint are preserved under its Proof directory. Production integration and verification remain required.

## No Additional Pre-Freeze Proofs

- **Renderer:** official Tauri SPA guidance and stable library APIs settle the architecture; typed-boundary and startup restoration belong in production tests.
- **Persistence:** SQLite/rusqlite transactional migration and online-backup APIs settle feasibility; crash/disk-full recovery belongs in production tests.
- **Extensions:** the same-target extension-system, marketplace rollback, trust/hook sandbox, MCP, and supervisor Proofs already cover the selected feasibility boundary.
- **Workspace/configuration/layout:** standard zip validation, online backup, atomic replacement, strict parsing, and generation rules are testable implementation requirements without an unresolved platform primitive.
- **Distribution:** signing, notarization, and signed updater Proofs are deferred until the later distribution milestone selected by D-018.
- **Adapters:** exact OpenCode/Pi version deltas are deferred to post-Freeze adapter task planning by GAP-004 without changing their peer contract.

## Freeze Rule

The macOS Native WebKit Production Boundary Proof is closed with a passing result. No other pre-Freeze Proof is required by the completed implementation selections, and the accepted disposition is Frozen with Spec 00003.
