import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { PluginSettings } from "./PluginSettings";
import type {
  PluginSettingsActions,
  PluginSettingsSnapshot,
  PluginView,
} from "./types";

function createActions(): PluginSettingsActions {
  return {
    onActivateUpdate: vi.fn(),
    onAddMarketplace: vi.fn(),
    onDisable: vi.fn(),
    onEnableForNextTurn: vi.fn(),
    onInstallDisabled: vi.fn(),
    onOpenPublisherLink: vi.fn(),
    onRefreshCatalog: vi.fn(),
    onRetry: vi.fn(),
    onReviewHook: vi.fn(),
    onRollback: vi.fn(),
    onStageUpdate: vi.fn(),
    onUninstall: vi.fn(),
  };
}

const plugin: PluginView = {
  apps: [
    {
      id: "workspace-summary",
      settingIds: ["workspace-mode"],
      summary: "Host-rendered declarative Plugin contribution.",
      title: "Workspace summary",
    },
  ],
  capabilities: ["Workspace metadata", "Proposed Git actions"],
  compatibility: "C4OS >=0.1.0 <0.2.0",
  failureCode: null,
  hooks: [
    {
      arguments: ["--reviewed"],
      grants: ["workspace.read"],
      id: "before-commit",
      lastResult: "Completed in 34 ms",
      name: "Before commit",
      review: "Out-of-process hook with bounded input and no network.",
      status: "ready",
    },
  ],
  id: "github-workflow",
  isInstalled: true,
  lastKnownGoodVersion: "1.3.1",
  lifecycle: "installedDisabled",
  marketplaceId: "local",
  mcpServers: [
    {
      id: "sample-mcp",
      name: "Sample MCP metadata",
      settingIds: ["api-credential"],
      transport: "stdio",
    },
  ],
  name: "GitHub Workflow",
  operation: null,
  packageKind: "plugin",
  privacyPolicyUrl: "https://example.test/privacy",
  publisher: "C4OS Labs",
  revocationReason: null,
  rollbackAvailable: true,
  signature: {
    contentKeyId: "content-key-2026",
    contentSignature: "verified",
    originKeyId: "origin-key-2026",
    originSignature: "verified",
    verifiedAt: "2026-07-22T02:00:00Z",
  },
  source: "local/catalog/github-workflow@sha256:abc",
  settings: [
    {
      choices: [],
      description: "Opaque credential reference.",
      id: "api-credential",
      kind: "credentialReference",
      label: "API credential",
      required: false,
    },
  ],
  summary: "Reviewed workflow guidance and declarative Git contributions.",
  termsUrl: "https://example.test/terms",
  trustState: "verified",
  update: {
    availableVersion: "1.4.0",
    currentDigest: "sha256:abc",
    stagedDigest: "sha256:def",
    stagedVersion: "1.4.0",
  },
  version: "1.3.2",
  websiteUrl: "https://example.test",
};

const readySnapshot: PluginSettingsSnapshot = {
  catalogDetail: "1 user-added source verified at generation 4.",
  catalogStatus: "current",
  generation: 4,
  marketplaces: [
    {
      detail: "Immutable local catalog snapshot.",
      gitRef: null,
      id: "local",
      label: "Local Extensions",
      lastCheckedAt: "2026-07-22T02:00:00Z",
      packageCount: 1,
      resolvedCommit: null,
      source: "/fixtures/extensions/catalog",
      sparsePaths: [],
      status: "ready",
      trustedOrigin: "origin:91ab",
    },
  ],
  plugins: [plugin],
  status: "ready",
};

type ExecutingPluginView = Omit<PluginView, "lifecycle"> & {
  readonly lifecycle: "executing";
};

describe("PluginSettings", () => {
  it("keeps loading, failure, and retry states distinct", () => {
    const actions = createActions();
    const { rerender } = render(
      <PluginSettings
        actions={actions}
        snapshot={{ message: "Reading catalog metadata.", status: "loading" }}
      />,
    );
    expect(screen.getByRole("status")).toHaveTextContent("Loading plugins");

    rerender(
      <PluginSettings
        actions={actions}
        snapshot={{
          message: "The verified catalog snapshot could not be read.",
          retryable: true,
          status: "error",
        }}
      />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Plugins are unavailable",
    );
    fireEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(actions.onRetry).toHaveBeenCalledTimes(1);
  });

  it("projects verified package detail and requests explicit next-turn activation", async () => {
    const actions = createActions();
    render(<PluginSettings actions={actions} snapshot={readySnapshot} />);

    expect(
      screen.getByRole("tab", { name: "Installed (1)" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Installed Disabled")).toBeInTheDocument();
    fireEvent.click(
      screen.getByRole("button", { name: "Enable for next turn" }),
    );
    expect(actions.onEnableForNextTurn).toHaveBeenCalledWith("github-workflow");

    fireEvent.click(screen.getByRole("button", { name: "Details" }));
    const dialog = await screen.findByRole("dialog", {
      name: "GitHub Workflow",
    });
    expect(
      within(dialog).getByText("Declarative contributions"),
    ).toBeInTheDocument();
    expect(within(dialog).getByText("API credential")).toBeInTheDocument();
    expect(within(dialog).getByText("Workspace summary")).toBeInTheDocument();
    expect(within(dialog).getByText("Sample MCP metadata")).toBeInTheDocument();
    expect(within(dialog).getByText("Origin key ID")).toBeInTheDocument();
    expect(within(dialog).getByText("origin-key-2026")).toBeInTheDocument();
    expect(within(dialog).getByText("Content key ID")).toBeInTheDocument();
    expect(within(dialog).getByText("content-key-2026")).toBeInTheDocument();
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Privacy Policy" }),
    );
    expect(actions.onOpenPublisherLink).toHaveBeenCalledWith(
      "github-workflow",
      "privacy",
    );
    expect(
      within(dialog).getByText("https://example.test/privacy"),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByRole("button", {
        name: "Activate staged 1.4.0",
      }),
    ).toBeInTheDocument();

    fireEvent.click(
      within(dialog).getByRole("button", {
        name: "Review hook Before commit",
      }),
    );
    expect(actions.onReviewHook).toHaveBeenCalledWith(
      "github-workflow",
      "before-commit",
    );
  });

  it("requires and trims independent marketplace trust pins", async () => {
    const actions = createActions();
    render(<PluginSettings actions={actions} snapshot={readySnapshot} />);

    fireEvent.click(screen.getByRole("button", { name: "Add Marketplace" }));
    const dialog = await screen.findByRole("dialog", {
      name: "Add Marketplace",
    });
    const addButton = within(dialog).getByRole("button", {
      name: "Add Marketplace",
    });
    expect(addButton).toBeDisabled();
    expect(
      within(dialog).getByText(/Obtain the fingerprint independently/u),
    ).toBeInTheDocument();
    fireEvent.change(within(dialog).getByLabelText("Source"), {
      target: { value: "  https://example.test/extensions.git  " },
    });
    expect(addButton).toBeDisabled();
    fireEvent.change(within(dialog).getByLabelText("Trusted origin"), {
      target: { value: "  example-publisher  " },
    });
    fireEvent.change(within(dialog).getByLabelText("Signing key ID"), {
      target: { value: "  origin-key-2026  " },
    });
    fireEvent.change(
      within(dialog).getByLabelText("Public key SHA-256 fingerprint"),
      {
        target: {
          value:
            "  sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef  ",
        },
      },
    );
    expect(addButton).toBeEnabled();
    fireEvent.change(within(dialog).getByLabelText("Git ref (optional)"), {
      target: { value: "  v1  " },
    });
    fireEvent.change(
      within(dialog).getByLabelText("Sparse paths (optional, one per line)"),
      { target: { value: "  plugins/github  \n\nskills/review " } },
    );
    fireEvent.click(addButton);

    expect(actions.onAddMarketplace).toHaveBeenCalledWith({
      gitRef: "v1",
      publicKeySha256:
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      signingKeyId: "origin-key-2026",
      source: "https://example.test/extensions.git",
      sparsePaths: ["plugins/github", "skills/review"],
      trustedOrigin: "example-publisher",
    });
  });

  it("disables lifecycle mutation controls while a plugin operation is busy", async () => {
    render(
      <PluginSettings
        actions={createActions()}
        snapshot={{
          ...readySnapshot,
          plugins: [{ ...plugin, operation: "stagingUpdate" }],
        }}
      />,
    );

    expect(
      screen.getByRole("button", { name: "Enable for next turn" }),
    ).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "Details" }));
    const dialog = await screen.findByRole("dialog", {
      name: "GitHub Workflow",
    });
    expect(
      within(dialog).getByRole("button", { name: "Review hook Before commit" }),
    ).toBeDisabled();
    expect(
      within(dialog).getByRole("button", { name: "Stage 1.4.0" }),
    ).toBeDisabled();
    expect(
      within(dialog).getByRole("button", { name: "Activate staged 1.4.0" }),
    ).toBeDisabled();
    expect(
      within(dialog).getByRole("button", {
        name: "Roll back to last-known-good",
      }),
    ).toBeDisabled();
    expect(
      within(dialog).getByRole("button", { name: "Uninstall" }),
    ).toBeDisabled();
  });

  it("keeps immediate disable available while blocking other executing lifecycle actions", async () => {
    const actions = createActions();
    const executingPlugin = {
      ...plugin,
      lifecycle: "executing",
    } satisfies ExecutingPluginView;
    const executingSnapshot = {
      ...readySnapshot,
      plugins: [executingPlugin],
    };

    render(
      <PluginSettings
        actions={actions}
        snapshot={executingSnapshot as unknown as PluginSettingsSnapshot}
      />,
    );

    const pluginCard = screen
      .getByRole("heading", { name: "GitHub Workflow" })
      .closest("article");
    expect(pluginCard).toHaveAttribute("aria-busy", "true");
    expect(screen.getByText("Executing")).toBeInTheDocument();
    expect(screen.getByText("Reviewed hook executing")).toBeInTheDocument();
    expect(
      screen.getByText(
        /Disable remains available immediately; other lifecycle changes and hook review stay unavailable/u,
      ),
    ).toBeInTheDocument();
    const disableButton = screen.getByRole("button", { name: "Disable" });
    expect(disableButton).toBeEnabled();
    fireEvent.click(disableButton);
    expect(actions.onDisable).toHaveBeenCalledWith("github-workflow");
    expect(
      screen.queryByRole("button", { name: "Enable for next turn" }),
    ).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Details" }));
    const dialog = await screen.findByRole("dialog", {
      name: "GitHub Workflow",
    });
    expect(within(dialog).getByText("Lifecycle")).toBeInTheDocument();
    expect(within(dialog).getByText("Executing")).toBeInTheDocument();
    expect(
      within(dialog).getByRole("button", { name: "Review hook Before commit" }),
    ).toBeDisabled();
    expect(
      within(dialog).getByRole("button", { name: "Stage 1.4.0" }),
    ).toBeDisabled();
    expect(
      within(dialog).getByRole("button", { name: "Activate staged 1.4.0" }),
    ).toBeDisabled();
    expect(
      within(dialog).getByRole("button", {
        name: "Roll back to last-known-good",
      }),
    ).toBeDisabled();
    expect(
      within(dialog).getByRole("button", { name: "Uninstall" }),
    ).toBeDisabled();
  });

  it("keeps interrupted hook recovery visible after restart", async () => {
    render(
      <PluginSettings
        actions={createActions()}
        snapshot={{
          ...readySnapshot,
          plugins: [
            {
              ...plugin,
              failureCode: "worker-restart-recovered",
              lifecycle: "enabled",
            },
          ],
        }}
      />,
    );

    expect(screen.getByText("Interrupted hook recovered")).toBeVisible();
    expect(screen.getByText(/no worker authority was restored/u)).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Details" }));
    const dialog = await screen.findByRole("dialog", {
      name: "GitHub Workflow",
    });
    expect(within(dialog).getByText("Recovery detail")).toBeVisible();
    expect(within(dialog).getByText("worker-restart-recovered")).toBeVisible();
  });

  it("shows hook invocation arguments alongside grants and review digest", async () => {
    const hookWithArguments = {
      ...plugin.hooks[0]!,
      arguments: ["--mode", "review only", "--max-files=12"],
      review: "beforeCommit · sha256:abc",
    } satisfies PluginView["hooks"][number] & {
      readonly arguments: readonly string[];
    };
    const pluginWithArguments = {
      ...plugin,
      hooks: [hookWithArguments],
    } satisfies PluginView;

    render(
      <PluginSettings
        actions={createActions()}
        snapshot={{ ...readySnapshot, plugins: [pluginWithArguments] }}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Details" }));
    const dialog = await screen.findByRole("dialog", {
      name: "GitHub Workflow",
    });
    expect(within(dialog).getByText("beforeCommit · sha256:abc")).toBeVisible();
    expect(within(dialog).getByText("Grants: workspace.read")).toBeVisible();
    expect(
      within(dialog).getByLabelText("Invocation arguments for Before commit"),
    ).toHaveTextContent('["--mode","review only","--max-files=12"]');
  });

  it("keeps a verified package quarantined until explicit disabled installation", () => {
    const actions = createActions();
    render(
      <PluginSettings
        actions={actions}
        snapshot={{
          ...readySnapshot,
          plugins: [
            {
              ...plugin,
              id: "web-research",
              isInstalled: false,
              lifecycle: "quarantined",
              name: "Web Research",
              trustState: "quarantined",
            },
          ],
        }}
      />,
    );

    fireEvent.click(screen.getByRole("tab", { name: "Directory" }));
    expect(screen.getByText("Package quarantined")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Install disabled" }));
    expect(actions.onInstallDisabled).toHaveBeenCalledWith("web-research");
  });

  it("renders a valid empty catalog instead of inventing installed state", () => {
    render(
      <PluginSettings
        actions={createActions()}
        snapshot={{
          catalogDetail: "No sources configured.",
          catalogStatus: "current",
          generation: 1,
          marketplaces: [],
          plugins: [],
          status: "ready",
        }}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "No marketplaces added" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "No plugins installed" }),
    ).toBeInTheDocument();
  });
});
