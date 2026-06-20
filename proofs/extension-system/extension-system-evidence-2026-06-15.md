# Extension System POC Evidence: 2026-06-15

Status: passed
Started: 2026-06-15T08:55:50.834Z
Finished: 2026-06-15T08:55:50.877Z

## Scope

Disposable static inventory and harmless MCP connection POC. This does not
enable production Plugins, Skills, MCP Servers, or Hooks.

## Checks

- PASS: static inventory reads manifests without executing extension code (1ms)
- PASS: skill is instruction content, not executable code (1ms)
- PASS: plugin manifest metadata is visible without executing plugin module (1ms)
- PASS: hook proposal is represented without automatic execution (1ms)
- PASS: harmless MCP server connects, lists tools, calls safe tool, and disconnects (36ms)
- PASS: unknown or high-risk extension actions route through policy (0ms)

## Security Findings

- Default state: disabled
- Inventory mode: static manifests before execution
- Unknown tools: approval_required
- High-risk actions: blocked_or_approval_required
- Output taint: extension-output-untrusted

## Limits

- This POC uses local fixture manifests and a harmless stdio MCP fixture.
- It proves a shared inventory/trust schema is feasible for local sources.
- Git/npm install flows, signatures, update policies, and real external MCP
  servers still require follow-up POCs or implementation design.

## Raw Result

```json
{
  "startedAt": "2026-06-15T08:55:50.834Z",
  "fixtureRoot": "/var/folders/w2/g12vs0ts64g_3k6kjf_dd_bc0000gn/T/c4os-extension-poc-R1rcxv",
  "checks": [
    {
      "name": "static inventory reads manifests without executing extension code",
      "status": "pass",
      "durationMs": 1,
      "evidence": {
        "inventory": [
          {
            "type": "skill",
            "id": "review-local-docs",
            "name": "review-local-docs",
            "enabled": false,
            "trust": {
              "state": "untrusted",
              "reason": "local fixture not reviewed"
            },
            "provenance": {
              "source": "local-fixture",
              "installSource": "file:///var/folders/w2/g12vs0ts64g_3k6kjf_dd_bc0000gn/T/c4os-extension-poc-R1rcxv/skills/review-local-docs/SKILL.md"
            },
            "permissions": [],
            "execution": {
              "kind": "instruction-only"
            },
            "contentDigest": "chars:179",
            "taint": "instruction-untrusted"
          },
          {
            "type": "plugin",
            "id": "sample-code-indexer",
            "name": "Sample Code Indexer",
            "enabled": false,
            "trust": {
              "state": "untrusted",
              "reason": "executable package not reviewed"
            },
            "provenance": {
              "source": "local-fixture",
              "installSource": "file://poc-fixture/plugins/sample-code-indexer"
            },
            "permissions": [
              "read:workspace"
            ],
            "execution": {
              "kind": "executable-plugin",
              "entrypoint": "./plugin-code.mjs"
            },
            "manifest": {
              "id": "sample-code-indexer",
              "name": "Sample Code Indexer",
              "version": "0.0.0",
              "entrypoint": "./plugin-code.mjs",
              "permissions": [
                "read:workspace"
              ],
              "provenance": {
                "source": "local-fixture",
                "installSource": "file://poc-fixture/plugins/sample-code-indexer"
              }
            },
            "taint": "plugin-manifest-untrusted"
          },
          {
            "type": "hook",
            "id": "before-run-policy-note",
            "name": "Before Run Policy Note",
            "enabled": false,
            "trust": {
              "state": "untrusted",
              "reason": "event binding requires explicit approval"
            },
            "provenance": {
              "source": "local-fixture",
              "installSource": "file://poc-fixture/hooks/before-run-policy-note.json"
            },
            "permissions": [
              "write:session-context"
            ],
            "execution": {
              "kind": "event-binding-proposal",
              "action": "append-policy-note"
            },
            "event": "beforeRun",
            "requiresApproval": true,
            "taint": "hook-proposal-untrusted"
          },
          {
            "type": "mcp",
            "id": "harmless-echo",
            "name": "Harmless Echo MCP",
            "enabled": false,
            "trust": {
              "state": "untrusted",
              "reason": "server not connected until user approval"
            },
            "provenance": {
              "source": "local-fixture",
              "installSource": "file://poc-fixture/mcp/harmless-server.json"
            },
            "permissions": [],
            "execution": {
              "kind": "stdio-mcp-server",
              "command": "/Users/cblanquera/.nvm/versions/node/v22.14.0/bin/node",
              "args": [
                "/Users/cblanquera/server/projects/cblanquera/c4os2/poc/extension-system/harmless-mcp-server.mjs"
              ]
            },
            "taint": "mcp-metadata-untrusted"
          }
        ]
      }
    },
    {
      "name": "skill is instruction content, not executable code",
      "status": "pass",
      "durationMs": 1,
      "evidence": {
        "type": "skill",
        "id": "review-local-docs",
        "name": "review-local-docs",
        "enabled": false,
        "trust": {
          "state": "untrusted",
          "reason": "local fixture not reviewed"
        },
        "provenance": {
          "source": "local-fixture",
          "installSource": "file:///var/folders/w2/g12vs0ts64g_3k6kjf_dd_bc0000gn/T/c4os-extension-poc-R1rcxv/skills/review-local-docs/SKILL.md"
        },
        "permissions": [],
        "execution": {
          "kind": "instruction-only"
        },
        "contentDigest": "chars:179",
        "taint": "instruction-untrusted"
      }
    },
    {
      "name": "plugin manifest metadata is visible without executing plugin module",
      "status": "pass",
      "durationMs": 1,
      "evidence": {
        "type": "plugin",
        "id": "sample-code-indexer",
        "name": "Sample Code Indexer",
        "enabled": false,
        "trust": {
          "state": "untrusted",
          "reason": "executable package not reviewed"
        },
        "provenance": {
          "source": "local-fixture",
          "installSource": "file://poc-fixture/plugins/sample-code-indexer"
        },
        "permissions": [
          "read:workspace"
        ],
        "execution": {
          "kind": "executable-plugin",
          "entrypoint": "./plugin-code.mjs"
        },
        "manifest": {
          "id": "sample-code-indexer",
          "name": "Sample Code Indexer",
          "version": "0.0.0",
          "entrypoint": "./plugin-code.mjs",
          "permissions": [
            "read:workspace"
          ],
          "provenance": {
            "source": "local-fixture",
            "installSource": "file://poc-fixture/plugins/sample-code-indexer"
          }
        },
        "taint": "plugin-manifest-untrusted"
      }
    },
    {
      "name": "hook proposal is represented without automatic execution",
      "status": "pass",
      "durationMs": 1,
      "evidence": {
        "type": "hook",
        "id": "before-run-policy-note",
        "name": "Before Run Policy Note",
        "enabled": false,
        "trust": {
          "state": "untrusted",
          "reason": "event binding requires explicit approval"
        },
        "provenance": {
          "source": "local-fixture",
          "installSource": "file://poc-fixture/hooks/before-run-policy-note.json"
        },
        "permissions": [
          "write:session-context"
        ],
        "execution": {
          "kind": "event-binding-proposal",
          "action": "append-policy-note"
        },
        "event": "beforeRun",
        "requiresApproval": true,
        "taint": "hook-proposal-untrusted"
      }
    },
    {
      "name": "harmless MCP server connects, lists tools, calls safe tool, and disconnects",
      "status": "pass",
      "durationMs": 36,
      "evidence": {
        "initialized": {
          "protocolVersion": "2025-03-26",
          "serverInfo": {
            "name": "c4os-harmless-extension-poc",
            "version": "0.0.0"
          },
          "capabilities": {
            "tools": {}
          }
        },
        "listed": {
          "tools": [
            {
              "name": "echo_safe",
              "description": "Echoes a supplied message without filesystem, network, or shell access.",
              "inputSchema": {
                "type": "object",
                "properties": {
                  "message": {
                    "type": "string"
                  }
                },
                "required": [
                  "message"
                ],
                "additionalProperties": false
              }
            }
          ]
        },
        "called": {
          "content": [
            {
              "type": "text",
              "text": "echo:hello"
            }
          ],
          "isError": false
        },
        "trust": {
          "state": "untrusted",
          "toolPolicy": "approval_required_before_model_use",
          "filesystemAccess": "none-granted"
        }
      }
    },
    {
      "name": "unknown or high-risk extension actions route through policy",
      "status": "pass",
      "durationMs": 0,
      "evidence": {
        "unknownTool": {
          "decision": "approval_required",
          "reason": "unknown or high-risk extension action"
        },
        "highRiskHook": {
          "decision": "blocked_until_enabled",
          "reason": "hooks require explicit enablement"
        },
        "safeSkillRead": {
          "decision": "allow_inventory_only",
          "reason": "static instruction inventory only"
        }
      }
    }
  ],
  "policy": {
    "defaultState": "disabled",
    "inventoryMode": "static manifests before execution",
    "provenanceRequired": true,
    "unknownTools": "approval_required",
    "highRiskActions": "blocked_or_approval_required",
    "outputTaint": "extension-output-untrusted"
  },
  "finishedAt": "2026-06-15T08:55:50.877Z",
  "summary": {
    "status": "passed",
    "passed": 6,
    "failed": 0
  }
}
```
