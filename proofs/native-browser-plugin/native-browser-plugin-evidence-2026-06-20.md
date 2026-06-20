# Native Browser Plugin POC Evidence: 2026-06-20

Status: passed
Started: 2026-06-20T13:53:46.013Z
Finished: 2026-06-20T13:53:46.871Z

## Scope

Disposable loopback POC for an isolated browser surface, navigation policy,
and constrained message boundary. This is not product Browser implementation.

## Endpoints

- Shell: http://127.0.0.1:63297/
- Local preview: http://127.0.0.1:63295/
- Remote-like preview: http://127.0.0.1:63296/

## Checks

- PASS: shell exposes app bridge only to host page (9ms)
- PASS: local preview loads in isolated surface (157ms)
- PASS: remote preview loads in isolated surface (155ms)
- PASS: page scripts cannot access host app bridge or secret sentinel (1ms)
- PASS: navigation policy blocks file URL (1ms)
- PASS: navigation policy blocks javascript URL (1ms)
- PASS: reload and stop are observable through constrained shell state (105ms)

## Policy Findings

- Allowed protocols: http:, https:
- Blocked protocols: file:, data:, javascript:
- Cookies: defer persistent session support; sandboxed pages must not share app-shell cookies
- Downloads: defer downloads; native plugin must require explicit user destination and approval
- External browser handoff: defer until user-facing browser policy is accepted

## Raw Result

```json
{
  "startedAt": "2026-06-20T13:53:46.013Z",
  "localPreview": "http://127.0.0.1:63295/",
  "remotePreview": "http://127.0.0.1:63296/",
  "shell": "http://127.0.0.1:63297/",
  "checks": [
    {
      "name": "shell exposes app bridge only to host page",
      "status": "pass",
      "durationMs": 9,
      "evidence": {
        "bridgeVisible": true
      }
    },
    {
      "name": "local preview loads in isolated surface",
      "status": "pass",
      "durationMs": 157,
      "evidence": {
        "navigated": {
          "status": "loaded",
          "currentUrl": "http://127.0.0.1:63295/",
          "lastError": "",
          "events": [
            {
              "kind": "message",
              "label": "local-preview",
              "href": "http://127.0.0.1:63295/"
            },
            {
              "kind": "load",
              "href": "http://127.0.0.1:63295/"
            }
          ]
        },
        "latest": {
          "kind": "browser-page-report",
          "label": "local-preview",
          "href": "http://127.0.0.1:63295/",
          "bridgeVisible": false,
          "secretVisible": false,
          "topAccessDenied": true,
          "cookieWriteSucceeded": false,
          "cookieValue": "cookie-error:SecurityError"
        }
      }
    },
    {
      "name": "remote preview loads in isolated surface",
      "status": "pass",
      "durationMs": 155,
      "evidence": {
        "navigated": {
          "status": "loaded",
          "currentUrl": "http://127.0.0.1:63296/",
          "lastError": "",
          "events": [
            {
              "kind": "message",
              "label": "local-preview",
              "href": "http://127.0.0.1:63295/"
            },
            {
              "kind": "load",
              "href": "http://127.0.0.1:63295/"
            },
            {
              "kind": "message",
              "label": "remote-preview",
              "href": "http://127.0.0.1:63296/"
            },
            {
              "kind": "load",
              "href": "http://127.0.0.1:63296/"
            }
          ]
        },
        "latest": {
          "kind": "browser-page-report",
          "label": "remote-preview",
          "href": "http://127.0.0.1:63296/",
          "bridgeVisible": false,
          "secretVisible": false,
          "topAccessDenied": true,
          "cookieWriteSucceeded": false,
          "cookieValue": "cookie-error:SecurityError"
        }
      }
    },
    {
      "name": "page scripts cannot access host app bridge or secret sentinel",
      "status": "pass",
      "durationMs": 1,
      "evidence": {
        "kind": "browser-page-report",
        "label": "remote-preview",
        "href": "http://127.0.0.1:63296/",
        "bridgeVisible": false,
        "secretVisible": false,
        "topAccessDenied": true,
        "cookieWriteSucceeded": false,
        "cookieValue": "cookie-error:SecurityError"
      }
    },
    {
      "name": "navigation policy blocks file URL",
      "status": "pass",
      "durationMs": 1,
      "evidence": {
        "status": "blocked",
        "currentUrl": "http://127.0.0.1:63296/",
        "lastError": "blocked protocol: file:",
        "events": [
          {
            "kind": "message",
            "label": "local-preview",
            "href": "http://127.0.0.1:63295/"
          },
          {
            "kind": "load",
            "href": "http://127.0.0.1:63295/"
          },
          {
            "kind": "message",
            "label": "remote-preview",
            "href": "http://127.0.0.1:63296/"
          },
          {
            "kind": "load",
            "href": "http://127.0.0.1:63296/"
          }
        ],
        "reason": "blocked protocol: file:"
      }
    },
    {
      "name": "navigation policy blocks javascript URL",
      "status": "pass",
      "durationMs": 1,
      "evidence": {
        "status": "blocked",
        "currentUrl": "http://127.0.0.1:63296/",
        "lastError": "blocked protocol: javascript:",
        "events": [
          {
            "kind": "message",
            "label": "local-preview",
            "href": "http://127.0.0.1:63295/"
          },
          {
            "kind": "load",
            "href": "http://127.0.0.1:63295/"
          },
          {
            "kind": "message",
            "label": "remote-preview",
            "href": "http://127.0.0.1:63296/"
          },
          {
            "kind": "load",
            "href": "http://127.0.0.1:63296/"
          }
        ],
        "reason": "blocked protocol: javascript:"
      }
    },
    {
      "name": "reload and stop are observable through constrained shell state",
      "status": "pass",
      "durationMs": 105,
      "evidence": {
        "reloaded": {
          "status": "reloaded",
          "currentUrl": "http://127.0.0.1:63296/",
          "lastError": "blocked protocol: javascript:",
          "events": [
            {
              "kind": "message",
              "label": "local-preview",
              "href": "http://127.0.0.1:63295/"
            },
            {
              "kind": "load",
              "href": "http://127.0.0.1:63295/"
            },
            {
              "kind": "message",
              "label": "remote-preview",
              "href": "http://127.0.0.1:63296/"
            },
            {
              "kind": "load",
              "href": "http://127.0.0.1:63296/"
            },
            {
              "kind": "message",
              "label": "remote-preview",
              "href": "http://127.0.0.1:63296/"
            },
            {
              "kind": "load",
              "href": "http://127.0.0.1:63296/"
            }
          ]
        },
        "stopped": {
          "status": "stopped",
          "currentUrl": "about:blank",
          "lastError": "blocked protocol: javascript:",
          "events": [
            {
              "kind": "message",
              "label": "local-preview",
              "href": "http://127.0.0.1:63295/"
            },
            {
              "kind": "load",
              "href": "http://127.0.0.1:63295/"
            },
            {
              "kind": "message",
              "label": "remote-preview",
              "href": "http://127.0.0.1:63296/"
            },
            {
              "kind": "load",
              "href": "http://127.0.0.1:63296/"
            },
            {
              "kind": "message",
              "label": "remote-preview",
              "href": "http://127.0.0.1:63296/"
            },
            {
              "kind": "load",
              "href": "http://127.0.0.1:63296/"
            }
          ]
        }
      }
    }
  ],
  "policy": {
    "allowedProtocols": [
      "http:",
      "https:"
    ],
    "blockedProtocols": [
      "file:",
      "data:",
      "javascript:"
    ],
    "cookies": "defer persistent session support; sandboxed pages must not share app-shell cookies",
    "downloads": "defer downloads; native plugin must require explicit user destination and approval",
    "externalBrowserHandoff": "defer until user-facing browser policy is accepted"
  },
  "finishedAt": "2026-06-20T13:53:46.871Z",
  "summary": {
    "status": "passed",
    "passed": 7,
    "failed": 0
  }
}
```
