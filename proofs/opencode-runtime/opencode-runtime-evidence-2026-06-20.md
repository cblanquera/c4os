# OpenCode Runtime POC Evidence

Date: 2026-06-20
Runtime: OpenCode
Packages:
- opencode-ai: 1.17.8
- @opencode-ai/sdk: 1.17.8

## Command

```sh
node proofs/opencode-runtime/opencode_runtime_poc.mjs
```

## Summary

OpenCode is a viable MVP runtime candidate for the C4OS adapter control plane. The proof started a local OpenCode server through the SDK, created and resumed a project-scoped session, streamed server/session events, verified permission configuration, and called abort on the created session.

## Adapter Contract Results

| Adapter method | Result | Evidence |
| --- | --- | --- |
| createSession | pass | POST /session creates a project-scoped session with projectID, directory, title, cost, token, version, and time metadata. |
| resumeSession | pass | GET /session/:id returns the created session by ID. |
| sendUserMessage | partial | session.prompt/session.promptAsync are exposed, but a full assistant turn requires provider credentials and was not run in this offline proof. |
| streamEvents | pass | event.subscribe streamed server.connected and session.created through the SDK stream. |
| requestApproval | partial | OpenCode exposes permission config and a permission response endpoint, but app-owned approval queue integration must wrap OpenCode permission requests. |
| resolveApproval | partial | postSessionIdPermissionsPermissionId exists in the SDK; no live permission request was produced without invoking a model-backed tool call. |
| stopRun | pass | session.abort is available and accepted for the created session. |
| listArtifacts | partial | session.messages, diff, file, and session metadata are exposed; app-owned artifact identity must be layered by C4OS. |

## Observed Events

```json
[
  {
    "id": "evt_ee542c363001K4zqfISzJJlkEr",
    "type": "server.connected"
  },
  {
    "id": "evt_ee542c367001vMfJggQxEu2jG5",
    "type": "session.created",
    "sessionID": "ses_11abd3c99ffeLWHKpZGKXIXrL2"
  }
]
```

## Gaps

- No model-backed prompt was sent because this proof must not require provider credentials or spend tokens.
- Permission request capture still needs a credentialed run that attempts a sensitive tool call.
- OpenCode state/config paths default to user-level opencode directories unless the host app supplies isolated environment/config paths.

## Recommendation

Use OpenCode as a viable MVP runtime candidate behind a C4OS-owned adapter, but keep app-owned approval records around OpenCode permission prompts.
