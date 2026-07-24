import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { McpSettings } from "../../../../../src/frontend/features/settings/mcp/McpSettings";
import type {
  McpServerView,
  McpSettingsActions,
  McpSettingsSnapshot,
} from "../../../../../src/frontend/features/settings/mcp/types";

const digest = `sha256:${"a".repeat(64)}`;

function actions(): McpSettingsActions {
  return {
    onDelete: vi.fn(),
    onDisable: vi.fn(),
    onEnableForNextTurn: vi.fn(),
    onAnswerTrust: vi.fn(),
    onRequestTrust: vi.fn(),
    onRecover: vi.fn(),
    onRetry: vi.fn(),
    onRevoke: vi.fn(),
    onSave: vi.fn(),
    onTest: vi.fn(),
  };
}

function server(overrides: Partial<McpServerView> = {}): McpServerView {
  return {
    activeRequests: 0,
    capabilities: ["Tools", "Resources"],
    canConfigure: true,
    canDelete: true,
    canDisable: false,
    canEnable: true,
    canRevoke: true,
    canRecover: false,
    canRequestTrust: false,
    canTest: true,
    definition: {
      expectedGeneration: 4,
      serverId: "docs.server",
      displayName: "Documentation server",
      scope: { kind: "application" },
      transport: {
        kind: "stdio",
        command: "/usr/bin/docs-mcp",
        arguments: ["--stdio"],
        environment: [
          {
            name: "DOCS_TOKEN",
            source: { kind: "literal", value: "must-not-render" },
          },
        ],
        workingDirectory: { kind: "c4osHome" },
        executableSha256: digest,
      },
      timeoutMs: 30_000,
      maxOutputBytes: 1_048_576,
    },
    failureCode: null,
    failureDetail: null,
    id: "docs.server",
    lastConnectedAt: null,
    lifecycleGeneration: 1,
    name: "Documentation server",
    nextRestartAt: null,
    protocolVersion: null,
    resourceCount: 1,
    restartAttempts: 0,
    scopeLabel: "Application",
    serverIdentity: null,
    sourceKind: "user",
    sourceLabel: "User configuration",
    state: "disabled",
    toolCount: 1,
    transportLabel: "STDIO",
    transportSummary: "/usr/bin/docs-mcp",
    trust: "trusted",
    trustApproval: null,
    ...overrides,
  };
}

function snapshot(
  servers: readonly McpServerView[] = [server()],
): Extract<McpSettingsSnapshot, { status: "ready" }> {
  return {
    activeWorkers: 0,
    generation: 4,
    servers,
    status: "ready",
  };
}

describe("MCP Settings", () => {
  it("keeps loading, error, retry, and empty states distinct", () => {
    const handlers = actions();
    const { rerender } = render(
      <McpSettings
        actions={handlers}
        snapshot={{ message: "Reading native state.", status: "loading" }}
      />,
    );
    expect(screen.getByRole("status")).toHaveTextContent("Loading MCP servers");

    rerender(
      <McpSettings
        actions={handlers}
        snapshot={{
          message: "The durable snapshot could not be read.",
          retryable: true,
          status: "error",
        }}
      />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent(
      "MCP Servers are unavailable",
    );
    fireEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(handlers.onRetry).toHaveBeenCalledTimes(1);

    rerender(<McpSettings actions={handlers} snapshot={snapshot([])} />);
    expect(
      screen.getByRole("heading", { name: "No MCP servers configured" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Add MCP Server" }),
    ).toBeEnabled();
  });

  it("saves an untrusted STDIO definition with repeatable process fields", async () => {
    const handlers = actions();
    render(<McpSettings actions={handlers} snapshot={snapshot([])} />);

    fireEvent.click(screen.getByRole("button", { name: "Add MCP Server" }));
    const dialog = await screen.findByRole("dialog", {
      name: "Add MCP Server",
    });
    const save = within(dialog).getByRole("button", { name: "Save Server" });
    expect(save).toBeDisabled();

    fireEvent.change(within(dialog).getByLabelText("Display name"), {
      target: { value: "Documentation server" },
    });
    fireEvent.change(within(dialog).getByLabelText("Server ID"), {
      target: { value: "docs.server" },
    });
    fireEvent.change(within(dialog).getByLabelText("Command"), {
      target: { value: "/usr/bin/docs-mcp" },
    });
    fireEvent.change(within(dialog).getByLabelText("Executable SHA-256"), {
      target: { value: digest },
    });
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Add argument" }),
    );
    fireEvent.change(within(dialog).getByLabelText("Argument 1"), {
      target: { value: "--stdio" },
    });
    fireEvent.click(
      within(dialog).getByRole("button", {
        name: "Add environment passthrough",
      }),
    );
    fireEvent.change(within(dialog).getByLabelText("Variable 1"), {
      target: { value: "LANG" },
    });
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Add credential reference" }),
    );
    fireEvent.change(within(dialog).getAllByLabelText("Variable 1")[1]!, {
      target: { value: "DOCS_TOKEN" },
    });
    fireEvent.change(within(dialog).getByLabelText("Reference source 1"), {
      target: { value: "vault" },
    });
    fireEvent.change(within(dialog).getByLabelText("Opaque reference 1"), {
      target: { value: "credential.docs" },
    });
    expect(save).toBeEnabled();
    fireEvent.click(save);

    expect(handlers.onSave).toHaveBeenCalledWith({
      expectedGeneration: 4,
      serverId: "docs.server",
      displayName: "Documentation server",
      scope: { kind: "application" },
      transport: {
        kind: "stdio",
        command: "/usr/bin/docs-mcp",
        arguments: ["--stdio"],
        environment: [
          { name: "LANG", source: { kind: "passthrough" } },
          {
            name: "DOCS_TOKEN",
            source: {
              kind: "secret",
              reference: {
                kind: "vault",
                credentialReference: "credential.docs",
              },
            },
          },
        ],
        workingDirectory: { kind: "c4osHome" },
        executableSha256: digest,
      },
      timeoutMs: 30_000,
      maxOutputBytes: 1_048_576,
    });
  });

  it("saves exact Streamable HTTP fields without collecting bearer values", async () => {
    const handlers = actions();
    render(<McpSettings actions={handlers} snapshot={snapshot([])} />);
    fireEvent.click(screen.getByRole("button", { name: "Add MCP Server" }));
    const dialog = await screen.findByRole("dialog", {
      name: "Add MCP Server",
    });

    fireEvent.change(within(dialog).getByLabelText("Display name"), {
      target: { value: "Remote docs" },
    });
    fireEvent.change(within(dialog).getByLabelText("Server ID"), {
      target: { value: "remote.docs" },
    });
    fireEvent.change(within(dialog).getByLabelText("Transport"), {
      target: { value: "streamableHttp" },
    });
    fireEvent.change(within(dialog).getByLabelText("URL"), {
      target: { value: "https://mcp.example.test/rpc" },
    });
    fireEvent.change(within(dialog).getByLabelText("Bearer token source"), {
      target: { value: "environment" },
    });
    fireEvent.change(
      within(dialog).getByLabelText("Bearer-token environment variable"),
      { target: { value: "MCP_TOKEN" } },
    );
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Add literal header" }),
    );
    fireEvent.change(within(dialog).getByLabelText("Header 1"), {
      target: { value: "X-Client" },
    });
    fireEvent.change(
      within(dialog).getByLabelText("Literal value (non-secret) 1"),
      {
        target: { value: "c4os" },
      },
    );
    fireEvent.click(
      within(dialog).getByRole("button", {
        name: "Add environment-backed header",
      }),
    );
    fireEvent.change(within(dialog).getAllByLabelText("Header 1")[1]!, {
      target: { value: "X-Workspace" },
    });
    fireEvent.change(within(dialog).getByLabelText("Environment variable 1"), {
      target: { value: "WORKSPACE_SLUG" },
    });
    const save = within(dialog).getByRole("button", { name: "Save Server" });
    expect(save).toBeEnabled();
    fireEvent.click(save);

    expect(handlers.onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        transport: {
          kind: "streamableHttp",
          url: "https://mcp.example.test/rpc",
          bearer: { kind: "environment", variable: "MCP_TOKEN" },
          headers: [
            {
              name: "X-Client",
              source: { kind: "literal", value: "c4os" },
            },
            {
              name: "X-Workspace",
              source: { kind: "environment", variable: "WORKSPACE_SLUG" },
            },
          ],
        },
      }),
    );
    expect(JSON.stringify(vi.mocked(handlers.onSave).mock.calls)).not.toContain(
      "raw-bearer-value",
    );
  });

  it("keeps a valid draft open when the native save rejects", async () => {
    const handlers = actions();
    vi.mocked(handlers.onSave).mockRejectedValueOnce(new Error("conflict"));
    render(<McpSettings actions={handlers} snapshot={snapshot()} />);
    fireEvent.click(screen.getByRole("button", { name: "Configure" }));
    const dialog = await screen.findByRole("dialog", {
      name: "Configure Documentation server",
    });
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Save Server" }),
    );

    expect(
      await within(dialog).findByText("Server was not saved"),
    ).toBeInTheDocument();
    expect(within(dialog).getByLabelText("Display name")).toHaveValue(
      "Documentation server",
    );
    expect(dialog).toBeInTheDocument();
  });

  it("keeps the last-good server list visible beside an operation error", () => {
    render(
      <McpSettings
        actions={actions()}
        snapshot={{
          ...snapshot(),
          operationError: "The native mutation was rejected.",
        }}
      />,
    );
    expect(screen.getByText("Documentation server")).toBeInTheDocument();
    expect(screen.getByText("MCP change was not applied")).toBeInTheDocument();
    expect(
      screen.getByText("The native mutation was rejected."),
    ).toBeInTheDocument();
  });

  it("surfaces lifecycle actions, failure states, and safe details", async () => {
    const handlers = actions();
    render(<McpSettings actions={handlers} snapshot={snapshot()} />);

    expect(screen.getByText("Disabled")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Test" }));
    expect(handlers.onTest).toHaveBeenCalledWith("docs.server");
    fireEvent.click(
      screen.getByRole("button", { name: "Enable for next turn" }),
    );
    expect(handlers.onEnableForNextTurn).toHaveBeenCalledWith("docs.server");

    fireEvent.click(screen.getByRole("button", { name: "Details" }));
    const details = await screen.findByRole("dialog", {
      name: "Documentation server",
    });
    expect(
      within(details).getByText(/literal value hidden/u),
    ).toBeInTheDocument();
    expect(
      within(details).queryByText("must-not-render"),
    ).not.toBeInTheDocument();
    fireEvent.change(within(details).getByLabelText("Revocation reason"), {
      target: { value: "Publisher key revoked" },
    });
    fireEvent.click(
      within(details).getByRole("button", { name: "Revoke authority" }),
    );
    expect(handlers.onRevoke).toHaveBeenCalledWith(
      "docs.server",
      "Publisher key revoked",
    );
  });

  it("requires an explicit answer for the exact pending trust review", () => {
    const handlers = actions();
    const pending = server({
      canEnable: false,
      canRequestTrust: true,
      canTest: false,
      trust: "pending",
    });
    const approval = server({
      canEnable: false,
      canRequestTrust: false,
      canTest: false,
      trust: "pending",
      trustApproval: { promptId: "prompt-mcp", state: "pending" },
    });
    const { rerender } = render(
      <McpSettings actions={handlers} snapshot={snapshot([pending])} />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Review trust" }));
    expect(handlers.onRequestTrust).toHaveBeenCalledWith("docs.server");

    rerender(
      <McpSettings actions={handlers} snapshot={snapshot([approval])} />,
    );
    expect(
      screen.getByText("Exact definition awaiting approval"),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Deny trust" }));
    expect(handlers.onAnswerTrust).toHaveBeenCalledWith(
      "docs.server",
      "prompt-mcp",
      "deny",
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Allow exact definition" }),
    );
    expect(handlers.onAnswerTrust).toHaveBeenCalledWith(
      "docs.server",
      "prompt-mcp",
      "allow",
    );
  });

  it("keeps immediate disable and revocation available while executing", () => {
    const handlers = actions();
    render(
      <McpSettings
        actions={handlers}
        snapshot={snapshot([
          server({
            canConfigure: false,
            canDelete: false,
            canDisable: true,
            canEnable: false,
            canRevoke: true,
            canTest: false,
            state: "executing",
          }),
        ])}
      />,
    );
    const row = screen.getByRole("listitem");
    expect(row).toHaveAttribute("aria-busy", "true");
    expect(screen.getByText("Executing")).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Configure" }),
    ).not.toBeInTheDocument();
    const disable = screen.getByRole("button", { name: "Disable" });
    expect(disable).toBeEnabled();
    fireEvent.click(disable);
    expect(handlers.onDisable).toHaveBeenCalledWith("docs.server");
    fireEvent.click(screen.getByRole("button", { name: "Details" }));
    expect(screen.getByText("Revoke server authority")).toBeInTheDocument();
  });

  it("distinguishes denied, timed-out, revoked, and restarting projections", () => {
    render(
      <McpSettings
        actions={actions()}
        snapshot={snapshot([
          server({ id: "denied", name: "Denied", state: "denied" }),
          server({ id: "timeout", name: "Timeout", state: "timedOut" }),
          server({ id: "revoked", name: "Revoked", state: "revoked" }),
          server({ id: "restart", name: "Restart", state: "restarting" }),
        ])}
      />,
    );
    expect(screen.getAllByText("Denied").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Timed out").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Revoked").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Restarting").length).toBeGreaterThan(0);
  });

  it("uses explicit bounded recovery instead of enabling a failed server", () => {
    const handlers = actions();
    const { rerender } = render(
      <McpSettings
        actions={handlers}
        snapshot={snapshot([
          server({
            canEnable: false,
            canRecover: false,
            failureDetail: "Bounded recovery is available.",
            nextRestartAt: "Jul 22, 2026, 3:00 PM",
            state: "failed",
          }),
        ])}
      />,
    );
    expect(
      screen.queryByRole("button", { name: "Enable for next turn" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Recover server" }),
    ).toBeDisabled();

    rerender(
      <McpSettings
        actions={handlers}
        snapshot={snapshot([
          server({
            canEnable: false,
            canRecover: true,
            failureDetail: "Bounded recovery is available.",
            nextRestartAt: null,
            state: "failed",
          }),
        ])}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "Recover server" }));
    expect(handlers.onRecover).toHaveBeenCalledWith("docs.server");
    expect(handlers.onEnableForNextTurn).not.toHaveBeenCalled();
  });
});
