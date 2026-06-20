# Tauri Native Browser Isolation POC Evidence: 2026-06-20

Status: failed
Started: SystemTime { tv_sec: 1781963641, tv_nsec: 21578000 }
Finished: SystemTime { tv_sec: 1781963643, tv_nsec: 626263000 }

## Scope

Disposable Tauri-native WebView isolation POC for `item-049` / `TASK-013`. This is not production Browser plugin implementation.

## Checks

- PASS: native surface opens local preview (333ms)
- PASS: native boundary reports loading and loaded states (0ms)
- PASS: native surface opens remote-like URL (44ms)
- PASS: browser page cannot access app bridge or secrets (0ms)
- FAIL: browser page has no Tauri IPC internals (0ms)
- PASS: privileged protocols are blocked (0ms)
- PASS: native boundary reports error state (2005ms)
- PASS: native boundary reports stopped state (0ms)
## Security Boundary Findings

- Browser page reports did not observe `window.__TAURI__`, C4OS bridge sentinel, provider-secret sentinel, shell-state sentinel, opener access, or a resolved secret command.
- Browser page reports did observe `window.__TAURI_INTERNALS__`; `poc_secret` invocation was rejected and did not leak the sentinel. This fails the fully unbridged Browser-surface requirement.
- `file:`, `javascript:`, `data:`, and `tauri:` navigation attempts were blocked by native controller policy before navigation.
- Native state reporting covered loading, loaded, blocked, error, and stopped states through `on_navigation`, `on_page_load`, controller policy, timeout, and close.

## Unresolved Risks

- Remote internet navigation was represented by a second loopback origin, not a live external website.
- Windows and Linux WebView behavior remains untested.
- Production embedding inside the round-003 right rail remains a later product slice.
- Downloads, persistent cookies, logged-in sessions, and browser automation remain out of scope.

## Promotion Decision

Do not promote to production Browser plugin. Tauri WebviewWindow navigation/state policy is feasible on macOS, but the surface is not fully unbridged because page content observes window.__TAURI_INTERNALS__. Next investigate a raw Wry/native WebView or Tauri IPC-stripping/capability hardening POC before product implementation.

## Raw Result

```json
{
  "checks": [
    {
      "duration_ms": 333,
      "evidence": {
        "report": {
          "c4os_bridge": false,
          "href": "http://127.0.0.1:63378/",
          "invoke_status": "rejected:poc_secret not allowed. Plugin not found",
          "label": "local-preview",
          "leaked_secret": false,
          "opener_visible": false,
          "provider_secret": false,
          "shell_state": false,
          "tauri_global": false,
          "tauri_internals": true
        }
      },
      "name": "native surface opens local preview",
      "status": "pass"
    },
    {
      "duration_ms": 0,
      "evidence": {
        "loaded": {
          "event": "Finished",
          "kind": "page-load",
          "label": "poc-browser-surface",
          "status": "loaded",
          "url": "http://127.0.0.1:63378/"
        },
        "loading": {
          "event": "Started",
          "kind": "page-load",
          "label": "poc-browser-surface",
          "status": "loading",
          "url": "http://127.0.0.1:63378/"
        }
      },
      "name": "native boundary reports loading and loaded states",
      "status": "pass"
    },
    {
      "duration_ms": 44,
      "evidence": {
        "currentUrl": "http://127.0.0.1:63379/",
        "report": {
          "c4os_bridge": false,
          "href": "http://127.0.0.1:63379/",
          "invoke_status": "rejected:poc_secret not allowed. Plugin not found",
          "label": "remote-like-preview",
          "leaked_secret": false,
          "opener_visible": false,
          "provider_secret": false,
          "shell_state": false,
          "tauri_global": false,
          "tauri_internals": true
        }
      },
      "name": "native surface opens remote-like URL",
      "status": "pass"
    },
    {
      "duration_ms": 0,
      "evidence": {
        "reports": [
          {
            "c4os_bridge": false,
            "href": "http://127.0.0.1:63378/",
            "invoke_status": "rejected:poc_secret not allowed. Plugin not found",
            "label": "local-preview",
            "leaked_secret": false,
            "opener_visible": false,
            "provider_secret": false,
            "shell_state": false,
            "tauri_global": false,
            "tauri_internals": true
          },
          {
            "c4os_bridge": false,
            "href": "http://127.0.0.1:63379/",
            "invoke_status": "rejected:poc_secret not allowed. Plugin not found",
            "label": "remote-like-preview",
            "leaked_secret": false,
            "opener_visible": false,
            "provider_secret": false,
            "shell_state": false,
            "tauri_global": false,
            "tauri_internals": true
          }
        ]
      },
      "name": "browser page cannot access app bridge or secrets",
      "status": "pass"
    },
    {
      "duration_ms": 0,
      "evidence": {
        "error": "local-preview observed window.__TAURI_INTERNALS__"
      },
      "name": "browser page has no Tauri IPC internals",
      "status": "fail"
    },
    {
      "duration_ms": 0,
      "evidence": {
        "blocked": [
          {
            "blockedBy": "native-controller-policy",
            "currentUrlBefore": "http://127.0.0.1:63379/",
            "kind": "blocked",
            "reason": "blocked protocol: file",
            "scheme": "file",
            "status": "blocked",
            "url": "file:///etc/passwd"
          },
          {
            "blockedBy": "native-controller-policy",
            "currentUrlBefore": "http://127.0.0.1:63379/",
            "kind": "blocked",
            "reason": "blocked protocol: javascript",
            "scheme": "javascript",
            "status": "blocked",
            "url": "javascript:alert(1)"
          },
          {
            "blockedBy": "native-controller-policy",
            "currentUrlBefore": "http://127.0.0.1:63379/",
            "kind": "blocked",
            "reason": "blocked protocol: data",
            "scheme": "data",
            "status": "blocked",
            "url": "data:text/html,<h1>blocked</h1>"
          },
          {
            "blockedBy": "native-controller-policy",
            "currentUrlBefore": "http://127.0.0.1:63379/",
            "kind": "blocked",
            "reason": "blocked protocol: tauri",
            "scheme": "tauri",
            "status": "blocked",
            "url": "tauri://localhost/index.html"
          }
        ]
      },
      "name": "privileged protocols are blocked",
      "status": "pass"
    },
    {
      "duration_ms": 2005,
      "evidence": {
        "currentUrl": "about:blank",
        "kind": "load-error",
        "reason": "no fixture report before timeout",
        "status": "error",
        "url": "http://127.0.0.1:9/native-browser-poc-error"
      },
      "name": "native boundary reports error state",
      "status": "pass"
    },
    {
      "duration_ms": 0,
      "evidence": {
        "kind": "closed",
        "status": "stopped",
        "window": "poc-browser-surface"
      },
      "name": "native boundary reports stopped state",
      "status": "pass"
    }
  ],
  "finishedAt": "SystemTime { tv_sec: 1781963643, tv_nsec: 626263000 }",
  "nativeEvents": [
    {
      "kind": "navigation-allowed",
      "reason": null,
      "scheme": "http",
      "status": "loading",
      "url": "http://127.0.0.1:63378/"
    },
    {
      "event": "Started",
      "kind": "page-load",
      "label": "poc-browser-surface",
      "status": "loading",
      "url": "http://127.0.0.1:63378/"
    },
    {
      "event": "Finished",
      "kind": "page-load",
      "label": "poc-browser-surface",
      "status": "loaded",
      "url": "http://127.0.0.1:63378/"
    },
    {
      "kind": "navigation-allowed",
      "reason": null,
      "scheme": "http",
      "status": "loading",
      "url": "http://127.0.0.1:63379/"
    },
    {
      "event": "Started",
      "kind": "page-load",
      "label": "poc-browser-surface",
      "status": "loading",
      "url": "http://127.0.0.1:63379/"
    },
    {
      "event": "Finished",
      "kind": "page-load",
      "label": "poc-browser-surface",
      "status": "loaded",
      "url": "http://127.0.0.1:63379/"
    },
    {
      "blockedBy": "native-controller-policy",
      "currentUrlBefore": "http://127.0.0.1:63379/",
      "kind": "blocked",
      "reason": "blocked protocol: file",
      "scheme": "file",
      "status": "blocked",
      "url": "file:///etc/passwd"
    },
    {
      "blockedBy": "native-controller-policy",
      "currentUrlBefore": "http://127.0.0.1:63379/",
      "kind": "blocked",
      "reason": "blocked protocol: javascript",
      "scheme": "javascript",
      "status": "blocked",
      "url": "javascript:alert(1)"
    },
    {
      "blockedBy": "native-controller-policy",
      "currentUrlBefore": "http://127.0.0.1:63379/",
      "kind": "blocked",
      "reason": "blocked protocol: data",
      "scheme": "data",
      "status": "blocked",
      "url": "data:text/html,<h1>blocked</h1>"
    },
    {
      "blockedBy": "native-controller-policy",
      "currentUrlBefore": "http://127.0.0.1:63379/",
      "kind": "blocked",
      "reason": "blocked protocol: tauri",
      "scheme": "tauri",
      "status": "blocked",
      "url": "tauri://localhost/index.html"
    },
    {
      "kind": "navigation-allowed",
      "reason": null,
      "scheme": "http",
      "status": "loading",
      "url": "http://127.0.0.1:9/native-browser-poc-error"
    },
    {
      "event": "Started",
      "kind": "page-load",
      "label": "poc-browser-surface",
      "status": "loading",
      "url": "about:blank"
    },
    {
      "event": "Finished",
      "kind": "page-load",
      "label": "poc-browser-surface",
      "status": "loaded",
      "url": "about:blank"
    },
    {
      "currentUrl": "about:blank",
      "kind": "load-error",
      "reason": "no fixture report before timeout",
      "status": "error",
      "url": "http://127.0.0.1:9/native-browser-poc-error"
    },
    {
      "kind": "closed",
      "status": "stopped",
      "window": "poc-browser-surface"
    }
  ],
  "promotionDecision": "Do not promote to production Browser plugin. Tauri WebviewWindow navigation/state policy is feasible on macOS, but the surface is not fully unbridged because page content observes window.__TAURI_INTERNALS__. Next investigate a raw Wry/native WebView or Tauri IPC-stripping/capability hardening POC before product implementation.",
  "reports": [
    {
      "c4os_bridge": false,
      "href": "http://127.0.0.1:63378/",
      "invoke_status": "rejected:poc_secret not allowed. Plugin not found",
      "label": "local-preview",
      "leaked_secret": false,
      "opener_visible": false,
      "provider_secret": false,
      "shell_state": false,
      "tauri_global": false,
      "tauri_internals": true
    },
    {
      "c4os_bridge": false,
      "href": "http://127.0.0.1:63379/",
      "invoke_status": "rejected:poc_secret not allowed. Plugin not found",
      "label": "remote-like-preview",
      "leaked_secret": false,
      "opener_visible": false,
      "provider_secret": false,
      "shell_state": false,
      "tauri_global": false,
      "tauri_internals": true
    }
  ],
  "securityFindings": {
    "appBridge": "browser page reports did not observe window.__TAURI__, C4OS bridge sentinel, provider-secret sentinel, shell-state sentinel, opener access, or a resolved secret command; it did observe window.__TAURI_INTERNALS__",
    "protocols": "file:, javascript:, data:, and tauri: navigation attempts were blocked by native controller policy before navigation",
    "scope": "disposable Tauri POC only; no production Browser plugin enabled",
    "stateBoundary": "native on_navigation/on_page_load plus controller timeout/close produced loading, loaded, blocked, error, and stopped states"
  },
  "startedAt": "SystemTime { tv_sec: 1781963641, tv_nsec: 21578000 }",
  "status": "failed",
  "summary": {
    "failed": 1,
    "passed": 7
  },
  "unresolvedRisks": [
    "remote internet navigation was represented by a second loopback origin, not a live external website",
    "Windows and Linux WebView behavior remains untested",
    "production embedding shape inside the round-003 right rail remains a later product slice",
    "downloads, persistent cookies, logged-in sessions, and browser automation remain out of scope"
  ]
}
```
