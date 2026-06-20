# Raw Wry Native Browser Isolation POC Evidence: 2026-06-20

Status: passed-with-wry-ipc-warning
Started: SystemTime { tv_sec: 1781963621, tv_nsec: 648892000 }
Finished: SystemTime { tv_sec: 1781963624, tv_nsec: 300962000 }

## Scope

Disposable raw Wry/native WebView isolation POC for `item-051` / `TASK-019`. This is not production Browser plugin implementation.

## Checks

- PASS: native surface opens local preview (327ms)
- PASS: controller loads remote-like URL (1ms)
- PASS: native surface opens remote-like URL (54ms)
- PASS: native boundary reports loading and loaded states (0ms)
- PASS: browser page has no Tauri IPC internals or host command access (0ms)
- PASS: privileged protocols are blocked (8ms)
- PASS: controller loads error URL (2ms)
- PASS: native boundary reports error state (2034ms)
- PASS: native boundary reports stopped state (0ms)

## Security Boundary Findings

- Browser page reports did not observe `window.__TAURI__`, `window.__TAURI_INTERNALS__`, C4OS bridge sentinel, provider-secret sentinel, shell-state sentinel, opener access, or a resolved host command.
- Browser page reports did observe Wry's `window.ipc` object on macOS. No Wry IPC handler was registered, `postMessage` did not resolve a host command, and the secret sentinel did not leak. Product Browser content must keep this handler unregistered unless a later explicit policy accepts a narrow message boundary.
- `file:`, `javascript:`, `data:`, and `tauri:` navigation attempts were blocked by native controller policy before navigation.
- Native state reporting covered loading, loaded, blocked, error, current URL, and stopped states through Wry page-load callbacks, WebView URL reads, controller policy, timeout, and close.

## Unresolved Risks

- Remote internet navigation was represented by a second loopback origin, not a live external website.
- Windows and Linux WebView behavior remains untested.
- Production embedding inside the round-003 right rail remains a later product slice.
- Downloads, persistent cookies, logged-in sessions, and browser automation remain out of scope.

## Promotion Decision

Do not promote directly to production Browser plugin yet. Raw Wry on macOS proves the missing no-Tauri-IPC surface for item-051, with a guardrail: page content observes Wry's window.ipc object, so product Browser content must not register a Wry IPC handler or app command bridge. The next product prework can move to Browser state/permission modeling and cross-platform confirmation before implementation.

## Raw Result

```json
{
  "checks": [
    {
      "duration_ms": 327,
      "evidence": {
        "report": {
          "c4os_bridge": false,
          "command_status": "wry-ipc-error:undefined is not an object (evaluating 'window.webkit.messageHandlers')",
          "href": "http://127.0.0.1:63268/",
          "label": "local-preview",
          "leaked_secret": false,
          "opener_visible": false,
          "provider_secret": false,
          "shell_state": false,
          "tauri_global": false,
          "tauri_internals": false,
          "wry_ipc": true
        }
      },
      "name": "native surface opens local preview",
      "status": "pass"
    },
    {
      "duration_ms": 1,
      "evidence": {
        "url": "http://127.0.0.1:63269/"
      },
      "name": "controller loads remote-like URL",
      "status": "pass"
    },
    {
      "duration_ms": 54,
      "evidence": {
        "currentUrl": "http://127.0.0.1:63269/",
        "report": {
          "c4os_bridge": false,
          "command_status": "wry-ipc-error:undefined is not an object (evaluating 'window.webkit.messageHandlers')",
          "href": "http://127.0.0.1:63269/",
          "label": "remote-like-preview",
          "leaked_secret": false,
          "opener_visible": false,
          "provider_secret": false,
          "shell_state": false,
          "tauri_global": false,
          "tauri_internals": false,
          "wry_ipc": true
        }
      },
      "name": "native surface opens remote-like URL",
      "status": "pass"
    },
    {
      "duration_ms": 0,
      "evidence": {
        "loaded": {
          "event": "Finished",
          "kind": "page-load",
          "status": "loaded",
          "url": "http://127.0.0.1:63269/"
        },
        "loading": {
          "event": "Started",
          "kind": "page-load",
          "status": "loading",
          "url": "http://127.0.0.1:63268/"
        }
      },
      "name": "native boundary reports loading and loaded states",
      "status": "pass"
    },
    {
      "duration_ms": 0,
      "evidence": {
        "reports": [
          {
            "c4os_bridge": false,
            "command_status": "wry-ipc-error:undefined is not an object (evaluating 'window.webkit.messageHandlers')",
            "href": "http://127.0.0.1:63268/",
            "label": "local-preview",
            "leaked_secret": false,
            "opener_visible": false,
            "provider_secret": false,
            "shell_state": false,
            "tauri_global": false,
            "tauri_internals": false,
            "wry_ipc": true
          },
          {
            "c4os_bridge": false,
            "command_status": "wry-ipc-error:undefined is not an object (evaluating 'window.webkit.messageHandlers')",
            "href": "http://127.0.0.1:63269/",
            "label": "remote-like-preview",
            "leaked_secret": false,
            "opener_visible": false,
            "provider_secret": false,
            "shell_state": false,
            "tauri_global": false,
            "tauri_internals": false,
            "wry_ipc": true
          }
        ]
      },
      "name": "browser page has no Tauri IPC internals or host command access",
      "status": "pass"
    },
    {
      "duration_ms": 8,
      "evidence": {
        "blocked": [
          {
            "blockedBy": "native-controller-policy",
            "currentUrlBefore": "http://127.0.0.1:63269/",
            "kind": "blocked",
            "reason": "blocked protocol: file",
            "scheme": "file",
            "status": "blocked",
            "url": "file:///etc/passwd"
          },
          {
            "blockedBy": "native-controller-policy",
            "currentUrlBefore": "http://127.0.0.1:63269/",
            "kind": "blocked",
            "reason": "blocked protocol: javascript",
            "scheme": "javascript",
            "status": "blocked",
            "url": "javascript:alert(1)"
          },
          {
            "blockedBy": "native-controller-policy",
            "currentUrlBefore": "http://127.0.0.1:63269/",
            "kind": "blocked",
            "reason": "blocked protocol: data",
            "scheme": "data",
            "status": "blocked",
            "url": "data:text/html,<h1>blocked</h1>"
          },
          {
            "blockedBy": "native-controller-policy",
            "currentUrlBefore": "http://127.0.0.1:63269/",
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
      "duration_ms": 2,
      "evidence": {
        "url": "http://127.0.0.1:9/native-browser-wry-poc-error"
      },
      "name": "controller loads error URL",
      "status": "pass"
    },
    {
      "duration_ms": 2034,
      "evidence": {
        "currentUrl": "about:blank",
        "kind": "load-error",
        "reason": "no fixture report before timeout",
        "status": "error",
        "url": "http://127.0.0.1:9/native-browser-wry-poc-error"
      },
      "name": "native boundary reports error state",
      "status": "pass"
    },
    {
      "duration_ms": 0,
      "evidence": {
        "kind": "closed",
        "status": "stopped",
        "window": "wry-browser-surface"
      },
      "name": "native boundary reports stopped state",
      "status": "pass"
    }
  ],
  "finishedAt": "SystemTime { tv_sec: 1781963624, tv_nsec: 300962000 }",
  "nativeEvents": [
    {
      "blockedBy": null,
      "kind": "navigation-allowed",
      "reason": null,
      "scheme": "http",
      "status": "loading",
      "url": "http://127.0.0.1:63268/"
    },
    {
      "event": "Started",
      "kind": "page-load",
      "status": "loading",
      "url": "http://127.0.0.1:63268/"
    },
    {
      "event": "Finished",
      "kind": "page-load",
      "status": "loaded",
      "url": "http://127.0.0.1:63269/"
    },
    {
      "blockedBy": null,
      "kind": "navigation-allowed",
      "reason": null,
      "scheme": "http",
      "status": "loading",
      "url": "http://127.0.0.1:63269/"
    },
    {
      "event": "Started",
      "kind": "page-load",
      "status": "loading",
      "url": "http://127.0.0.1:63269/"
    },
    {
      "event": "Finished",
      "kind": "page-load",
      "status": "loaded",
      "url": "http://127.0.0.1:63269/"
    },
    {
      "blockedBy": "native-controller-policy",
      "currentUrlBefore": "http://127.0.0.1:63269/",
      "kind": "blocked",
      "reason": "blocked protocol: file",
      "scheme": "file",
      "status": "blocked",
      "url": "file:///etc/passwd"
    },
    {
      "blockedBy": "native-controller-policy",
      "currentUrlBefore": "http://127.0.0.1:63269/",
      "kind": "blocked",
      "reason": "blocked protocol: javascript",
      "scheme": "javascript",
      "status": "blocked",
      "url": "javascript:alert(1)"
    },
    {
      "blockedBy": "native-controller-policy",
      "currentUrlBefore": "http://127.0.0.1:63269/",
      "kind": "blocked",
      "reason": "blocked protocol: data",
      "scheme": "data",
      "status": "blocked",
      "url": "data:text/html,<h1>blocked</h1>"
    },
    {
      "blockedBy": "native-controller-policy",
      "currentUrlBefore": "http://127.0.0.1:63269/",
      "kind": "blocked",
      "reason": "blocked protocol: tauri",
      "scheme": "tauri",
      "status": "blocked",
      "url": "tauri://localhost/index.html"
    },
    {
      "blockedBy": null,
      "kind": "navigation-allowed",
      "reason": null,
      "scheme": "http",
      "status": "loading",
      "url": "http://127.0.0.1:9/native-browser-wry-poc-error"
    },
    {
      "event": "Started",
      "kind": "page-load",
      "status": "loading",
      "url": "about:blank"
    },
    {
      "event": "Finished",
      "kind": "page-load",
      "status": "loaded",
      "url": "about:blank"
    },
    {
      "currentUrl": "about:blank",
      "kind": "load-error",
      "reason": "no fixture report before timeout",
      "status": "error",
      "url": "http://127.0.0.1:9/native-browser-wry-poc-error"
    },
    {
      "kind": "closed",
      "status": "stopped",
      "window": "wry-browser-surface"
    }
  ],
  "promotionDecision": "Do not promote directly to production Browser plugin yet. Raw Wry on macOS proves the missing no-Tauri-IPC surface for item-051, with a guardrail: page content observes Wry's window.ipc object, so product Browser content must not register a Wry IPC handler or app command bridge. The next product prework can move to Browser state/permission modeling and cross-platform confirmation before implementation.",
  "reports": [
    {
      "c4os_bridge": false,
      "command_status": "wry-ipc-error:undefined is not an object (evaluating 'window.webkit.messageHandlers')",
      "href": "http://127.0.0.1:63268/",
      "label": "local-preview",
      "leaked_secret": false,
      "opener_visible": false,
      "provider_secret": false,
      "shell_state": false,
      "tauri_global": false,
      "tauri_internals": false,
      "wry_ipc": true
    },
    {
      "c4os_bridge": false,
      "command_status": "wry-ipc-error:undefined is not an object (evaluating 'window.webkit.messageHandlers')",
      "href": "http://127.0.0.1:63269/",
      "label": "remote-like-preview",
      "leaked_secret": false,
      "opener_visible": false,
      "provider_secret": false,
      "shell_state": false,
      "tauri_global": false,
      "tauri_internals": false,
      "wry_ipc": true
    }
  ],
  "securityFindings": {
    "appBridge": "browser page reports did not observe Tauri globals, C4OS bridge sentinel, provider-secret sentinel, shell-state sentinel, opener, or resolved host command; Wry window.ipc was visible but unbound",
    "protocols": "file:, javascript:, data:, and tauri: navigation attempts were blocked by native controller policy before navigation",
    "scope": "disposable raw Wry POC only; no production Browser plugin enabled",
    "stateBoundary": "Wry page-load callbacks, URL reads, controller policy, timeout, and close produced loading, loaded, current URL, blocked, error, and stopped states"
  },
  "startedAt": "SystemTime { tv_sec: 1781963621, tv_nsec: 648892000 }",
  "status": "passed-with-wry-ipc-warning",
  "summary": {
    "failed": 0,
    "passed": 9
  },
  "unresolvedRisks": [
    "remote internet navigation was represented by a second loopback origin, not a live external website",
    "Windows and Linux WebView behavior remains untested",
    "production embedding inside the round-003 right rail remains a later product slice",
    "downloads, persistent cookies, logged-in sessions, and browser automation remain out of scope"
  ],
  "warnings": [
    "Wry exposes window.ipc on macOS even without an explicit IPC handler; product Browser content must not register a privileged Wry IPC handler without a separate policy."
  ]
}
```
