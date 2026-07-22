# Deterministic MCP transport fixtures

These dependency-free Node fixtures implement the MCP `2025-11-25` wire
surface needed by Task 00012 integration tests. They are intentionally hostile
peers: all descriptions, resource bodies, tool results, URIs, and server
instructions must remain untrusted data.

Both fixtures support exact initialize/version negotiation, tool and resource
discovery, list-change notifications, deterministic echo calls, oversized and
hostile results, a cancellable request, malformed output, and a controlled
crash. Optional JSONL audit output records lifecycle events without recording
authorization values.

## STDIO

```text
node stdio_server.cjs [--scenario normal|invalid-version|malformed|sampling]
  [--audit-path /absolute/path]
```

The fixture reads and writes one JSON-RPC message per line. Diagnostics and
audit data never use stdout.

## Authenticated Streamable HTTP

```text
MCP_FIXTURE_BEARER=fixture-secret node streamable_http_server.cjs
  [--port 0] [--scenario normal|invalid-version|malformed|sampling]
  [--audit-path /absolute/path]
```

The first stdout line is `{"ready":true,"port":<port>}`. The endpoint is
`http://127.0.0.1:<port>/mcp`; it requires the bearer value above. Initialize
returns the deterministic session ID `fixture-session-0001`; established
requests require both `MCP-Session-Id` and `MCP-Protocol-Version` headers. GET
opens an SSE notification stream and DELETE closes the session.

## Tool modes

Discovery returns exactly one tool, `fixture_tool`, and one resource. The tool's
`mode` argument selects `echo`, `slow`, `oversized`, `hostile`, `crash`, or
`malformed` behavior. The single resource includes inert hostile-looking text
so its presentation boundary can be verified without expanding discovery.
The `sampling` scenario also issues one deterministic `sampling/createMessage`
request after initialization and audits only whether the client responded.
