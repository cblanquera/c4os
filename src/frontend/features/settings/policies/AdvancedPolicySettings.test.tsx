import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { AdvancedPolicySettings } from "./AdvancedPolicySettings";
import { POLICY_GROUPS } from "./policy-catalog";
import {
  POLICY_SETTING_KEYS,
  type PolicyCategoryValues,
  type PolicySettingsActions,
  type PolicySettingsSnapshot,
  type ReadyPolicySettingsSnapshot,
} from "./types";

function emptyCategoryValues(): PolicyCategoryValues {
  return Object.fromEntries(
    POLICY_SETTING_KEYS.map((key) => [key, null]),
  ) as unknown as PolicyCategoryValues;
}

function readySnapshot(
  overrides: Partial<ReadyPolicySettingsSnapshot> = {},
): ReadyPolicySettingsSnapshot {
  return {
    authority: "rust-policy-service",
    basePreset: "approve-safe-actions",
    categoryValues: emptyCategoryValues(),
    coordinatorGeneration: 12,
    effectiveCategoryValues: emptyCategoryValues(),
    exceptions: [
      {
        action: "git status",
        decision: "allow",
        duration: "Persistent until revoked",
        exceptionId: "git-status-workspace",
        scope: "~/c4os · workspace-main",
        source: "opencode-primary",
      },
    ],
    managedRequirementCount: 1,
    maximumAuthorityRuleCount: 2,
    policyVersion: 7,
    preset: "approve-safe-actions",
    revocationEpoch: 3,
    status: "ready",
    ...overrides,
  };
}

function settingsActions(snapshot = readySnapshot()): PolicySettingsActions {
  return {
    onRetry: vi.fn(async () => undefined),
    onRevokeException: vi.fn(async ({ exceptionId }) => ({
      ...snapshot,
      coordinatorGeneration: snapshot.coordinatorGeneration + 1,
      exceptions: snapshot.exceptions.filter(
        (exception) => exception.exceptionId !== exceptionId,
      ),
      policyVersion: snapshot.policyVersion + 1,
      revocationEpoch: snapshot.revocationEpoch + 1,
    })),
    onSave: vi.fn(async ({ categoryValues }) => ({
      ...snapshot,
      categoryValues,
      coordinatorGeneration: snapshot.coordinatorGeneration + 1,
      effectiveCategoryValues: categoryValues,
      policyVersion: snapshot.policyVersion + 1,
      preset: "custom" as const,
      revocationEpoch: snapshot.revocationEpoch + 1,
    })),
  };
}

describe("AdvancedPolicySettings", () => {
  it("pins the native 28-key contract into seven accepted policy groups", () => {
    expect(POLICY_SETTING_KEYS).toEqual([
      "workspace.read",
      "workspace.modify",
      "workspace.delete",
      "workspace.outside",
      "command.inspect",
      "command.workspace",
      "command.system",
      "process.control",
      "git.local.read",
      "git.local.change",
      "git.remote.read",
      "git.remote.publish",
      "network.retrieve",
      "network.publish",
      "network.listen",
      "network.upload",
      "browser.view",
      "browser.interact",
      "browser.authenticated",
      "desktop.control",
      "credential.use",
      "credential.add",
      "credential.reveal",
      "extension.read",
      "extension.use",
      "extension.configure",
      "c4os.policy",
      "artifact.export",
    ]);
    expect(POLICY_GROUPS).toHaveLength(7);
    expect(
      POLICY_GROUPS.flatMap((group) => group.items.map((item) => item.key)),
    ).toEqual(POLICY_SETTING_KEYS);
  });

  it("renders seven groups and searches all action, target, and key descriptions", () => {
    const snapshot = readySnapshot();
    render(
      <AdvancedPolicySettings
        actions={settingsActions(snapshot)}
        configurationNavigationSelected
        snapshot={snapshot}
      />,
    );

    const groups = screen.getByRole("navigation", { name: "Policy groups" });
    expect(within(groups).getAllByRole("button")).toHaveLength(7);
    expect(
      within(groups).getByRole("button", { name: /workspace files\s*4/i }),
    ).toHaveAttribute("aria-current", "page");
    expect(
      within(screen.getByRole("combobox", { name: "workspace.read policy" }))
        .getAllByRole("option")
        .map((option) => option.textContent),
    ).toEqual(["Use default", "Allow", "Ask", "Deny"]);

    fireEvent.change(
      screen.getByRole("searchbox", { name: "Search policies" }),
      { target: { value: "credential" } },
    );
    expect(screen.getByText("credential.use")).toBeInTheDocument();
    expect(screen.getByText("credential.add")).toBeInTheDocument();
    expect(screen.getByText("credential.reveal")).toBeInTheDocument();
    expect(screen.queryByText("workspace.modify")).not.toBeInTheDocument();
  });

  it("explains effective results and exposes live dirty, revert, and save states", async () => {
    const snapshot = readySnapshot();
    let resolveSave!: (snapshot: ReadyPolicySettingsSnapshot) => void;
    const onSave = vi.fn(
      () =>
        new Promise<ReadyPolicySettingsSnapshot>((resolve) => {
          resolveSave = resolve;
        }),
    );
    const actions: PolicySettingsActions = {
      ...settingsActions(snapshot),
      onSave,
    };
    render(
      <AdvancedPolicySettings
        actions={actions}
        configurationNavigationSelected
        snapshot={snapshot}
      />,
    );

    const select = screen.getByRole("combobox", {
      name: "workspace.modify policy",
    });
    const save = screen.getByRole("button", { name: "Save policies" });
    const revert = screen.getByRole("button", { name: "Revert" });
    expect(select).toHaveValue("default");
    expect(save).toBeDisabled();

    fireEvent.change(select, { target: { value: "ask" } });
    expect(screen.getByText("Custom (unsaved)")).toBeInTheDocument();
    const policyRow = select.closest(".advanced-policy-row");
    expect(policyRow).not.toBeNull();
    expect(
      within(policyRow as HTMLElement).getByText(/Uses Approve safe actions/i),
    ).toBeInTheDocument();
    expect(save).toBeEnabled();

    fireEvent.click(save);
    expect(screen.getByText("Saving policy changes…")).toBeInTheDocument();
    expect(
      screen
        .getByText("Saving policy changes…")
        .closest(".advanced-policy-editor"),
    ).toHaveAttribute("aria-busy", "true");
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        expectedCoordinatorGeneration: 12,
        expectedPolicyVersion: 7,
        categoryValues: expect.objectContaining({
          "workspace.modify": "ask",
        }),
      }),
    );

    resolveSave(
      readySnapshot({
        categoryValues: {
          ...emptyCategoryValues(),
          "workspace.modify": "ask",
        },
        coordinatorGeneration: 13,
        effectiveCategoryValues: {
          ...emptyCategoryValues(),
          "workspace.modify": "ask",
        },
        policyVersion: 8,
        preset: "custom",
        revocationEpoch: 4,
      }),
    );
    expect(await screen.findByText(/policies saved/i)).toBeInTheDocument();
    expect(
      within(policyRow as HTMLElement).getByText(
        /asks before matching actions unless a stricter rule/i,
      ),
    ).toBeInTheDocument();
    expect(save).toBeDisabled();

    fireEvent.change(select, { target: { value: "deny" } });
    expect(save).toBeEnabled();
    fireEvent.click(revert);
    expect(select).toHaveValue("ask");
    expect(save).toBeDisabled();
  });

  it("keeps the editable category choice distinct from a stricter derived result", () => {
    render(
      <AdvancedPolicySettings
        actions={settingsActions()}
        configurationNavigationSelected
        snapshot={readySnapshot({
          categoryValues: {
            ...emptyCategoryValues(),
            "workspace.modify": "allow",
          },
          effectiveCategoryValues: {
            ...emptyCategoryValues(),
            "workspace.modify": "ask",
          },
          preset: "custom",
        })}
      />,
    );

    expect(
      screen.getByRole("combobox", { name: "workspace.modify policy" }),
    ).toHaveValue("allow");
    expect(
      screen.getByText(/asks before matching actions unless a stricter rule/i),
    ).toBeInTheDocument();
  });

  it("explains Use default from the retained base preset when policy is Custom", () => {
    const snapshot = readySnapshot({
      basePreset: "approve-safe-actions",
      preset: "custom",
    });
    render(
      <AdvancedPolicySettings
        actions={settingsActions(snapshot)}
        configurationNavigationSelected
        snapshot={snapshot}
      />,
    );
    expect(
      screen.getAllByText(/Uses Approve safe actions/u).length,
    ).toBeGreaterThan(0);
    expect(screen.queryByText(/Uses Custom/u)).toBeNull();
  });

  it("revokes visible concrete exceptions without discarding category drafts", async () => {
    const snapshot = readySnapshot();
    const actions = settingsActions(snapshot);
    const { rerender } = render(
      <AdvancedPolicySettings
        actions={actions}
        configurationNavigationSelected
        snapshot={snapshot}
      />,
    );

    fireEvent.change(
      screen.getByRole("combobox", { name: "workspace.read policy" }),
      { target: { value: "deny" } },
    );
    fireEvent.click(screen.getByRole("tab", { name: /exceptions 1/i }));

    expect(screen.getByText("git status")).toBeInTheDocument();
    expect(screen.getByText("~/c4os · workspace-main")).toBeInTheDocument();
    expect(screen.getByText("opencode-primary")).toBeInTheDocument();
    fireEvent.click(
      screen.getByRole("button", {
        name: "Revoke exception for git status",
      }),
    );

    await waitFor(() => {
      expect(actions.onRevokeException).toHaveBeenCalledWith({
        exceptionId: "git-status-workspace",
        expectedCoordinatorGeneration: 12,
        expectedPolicyVersion: 7,
      });
      expect(screen.queryByText("git status")).not.toBeInTheDocument();
    });
    expect(
      screen.getByRole("tab", { name: /exceptions 0/i }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        /outstanding matching authorizations expire immediately/i,
      ),
    ).toBeInTheDocument();

    rerender(
      <AdvancedPolicySettings
        actions={actions}
        configurationNavigationSelected
        snapshot={readySnapshot({
          coordinatorGeneration: 13,
          exceptions: [],
          policyVersion: 8,
          preset: "custom",
          revocationEpoch: 4,
        })}
      />,
    );

    fireEvent.click(screen.getByRole("tab", { name: /category rules 28/i }));
    expect(
      screen.getByRole("combobox", { name: "workspace.read policy" }),
    ).toHaveValue("deny");
    expect(screen.getByRole("button", { name: "Save policies" })).toBeEnabled();
  });

  it("renders heading/configuration-selection semantics plus loading and retryable errors", async () => {
    const actions = settingsActions();
    const loading: PolicySettingsSnapshot = {
      message: "Reading the Rust policy authority.",
      status: "loading",
    };
    const { rerender } = render(
      <AdvancedPolicySettings
        actions={actions}
        configurationNavigationSelected
        headingId="policy-heading"
        headingLevel={1}
        snapshot={loading}
      />,
    );

    expect(
      screen.getByRole("heading", { level: 1, name: "Advanced Policies" }),
    ).toHaveAttribute("id", "policy-heading");
    expect(screen.getByText("Loading policy settings")).toBeInTheDocument();
    expect(
      screen.getByText("Loading policy settings").closest("section"),
    ).toHaveAttribute("data-configuration-navigation-selected", "true");

    rerender(
      <AdvancedPolicySettings
        actions={actions}
        configurationNavigationSelected
        headingId="policy-heading"
        headingLevel={1}
        snapshot={{
          message: "Policy state could not be read.",
          retryable: true,
          status: "error",
        }}
      />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Policy state could not be read.",
    );
    fireEvent.click(screen.getByRole("button", { name: "Try again" }));
    await waitFor(() => expect(actions.onRetry).toHaveBeenCalledOnce());
  });

  it("keeps a failed save dirty and presents the callback error", async () => {
    const snapshot = readySnapshot();
    const actions: PolicySettingsActions = {
      ...settingsActions(snapshot),
      onSave: vi.fn(async () => {
        throw new Error("Policy changed before the rules could be saved.");
      }),
    };
    render(
      <AdvancedPolicySettings
        actions={actions}
        configurationNavigationSelected
        snapshot={snapshot}
      />,
    );

    fireEvent.change(
      screen.getByRole("combobox", { name: "workspace.read policy" }),
      { target: { value: "allow" } },
    );
    fireEvent.click(screen.getByRole("button", { name: "Save policies" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Policy changed before the rules could be saved.",
    );
    expect(screen.getByRole("button", { name: "Save policies" })).toBeEnabled();
    expect(
      screen.getByRole("combobox", { name: "workspace.read policy" }),
    ).toHaveValue("allow");
  });
});
