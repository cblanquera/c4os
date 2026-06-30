import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

describe("TASK-017 structured runtime tool events", () => {
  it("executes provider terminal.run lifecycle events through the gateway and reflects only structured Agent terminal output", async () => {
    const openRouterSource = await readFile(new URL("../../backend/src/openrouter.rs", import.meta.url), "utf8");
    const sessionsSource = await readFile(new URL("../../backend/src/runtime_sessions.rs", import.meta.url), "utf8");
    const frontendSource = await readFile(new URL("../../frontend/data.js", import.meta.url), "utf8");

    assert.match(openRouterSource, /tool_calls/);
    assert.match(openRouterSource, /EVENT_TOOL_CALL_REQUESTED/);
    assert.match(openRouterSource, /ProviderToolCallState/);
    assert.match(openRouterSource, /buffer\.arguments\.push_str/);
    assert.match(openRouterSource, /tool: Some\(name\)/);

    assert.match(sessionsSource, /is_terminal_tool_request/);
    assert.match(sessionsSource, /dispatch_tool_call\(ToolCallRequest/);
    assert.match(sessionsSource, /TOOL_TERMINAL_RUN/);
    assert.match(sessionsSource, /EVENT_TOOL_OUTPUT_DELTA/);
    assert.doesNotMatch(sessionsSource, /prompt\.starts_with\("run "\)/);

    assert.match(frontendSource, /sessionHasRuntimeAgentTerminal/);
    assert.match(frontendSource, /preserveTerminal: !sessionHasRuntimeAgentTerminal\(payload\.session\)/);
  });
});
