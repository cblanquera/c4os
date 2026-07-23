import {
  POLICY_SETTING_KEYS,
  type PolicyCategoryValues,
  type PolicyGroupDefinition,
  type PolicyPreset,
  type PolicyValue,
} from "./types";

export const POLICY_OPTIONS: ReadonlyArray<{
  readonly label: string;
  readonly value: PolicyValue;
}> = [
  { label: "Use default", value: "default" },
  { label: "Allow", value: "allow" },
  { label: "Ask", value: "ask" },
  { label: "Deny", value: "deny" },
];

export const POLICY_GROUPS: readonly PolicyGroupDefinition[] = [
  {
    id: "workspace-files",
    label: "Workspace files",
    items: [
      {
        key: "workspace.read",
        label: "Read workspace files",
        description: "Read files and folders in the trusted workspace.",
      },
      {
        key: "workspace.modify",
        label: "Change workspace files",
        description: "Create or modify content in the trusted workspace.",
      },
      {
        key: "workspace.delete",
        label: "Delete workspace files",
        description: "Delete or trash content in the trusted workspace.",
      },
      {
        key: "workspace.outside",
        label: "Access outside the workspace",
        description: "Access files or folders outside the trusted workspace.",
      },
    ],
  },
  {
    id: "commands-processes",
    label: "Commands and processes",
    items: [
      {
        key: "command.inspect",
        label: "Inspect command state",
        description: "Inspect processes, shell state, and command output.",
      },
      {
        key: "command.workspace",
        label: "Run workspace commands",
        description: "Execute commands inside the trusted workspace.",
      },
      {
        key: "command.system",
        label: "Run system commands",
        description:
          "Execute commands outside the workspace or against system state.",
      },
      {
        key: "process.control",
        label: "Control processes",
        description: "Start, stop, signal, or control a running process.",
      },
    ],
  },
  {
    id: "version-control",
    label: "Version control",
    items: [
      {
        key: "git.local.read",
        label: "Read local repository state",
        description: "Read local repository status, history, and differences.",
      },
      {
        key: "git.local.change",
        label: "Change local repository state",
        description:
          "Change branches, commits, tags, or worktree state locally.",
      },
      {
        key: "git.remote.read",
        label: "Read remote repository state",
        description: "Fetch or inspect remote repository information.",
      },
      {
        key: "git.remote.publish",
        label: "Publish repository changes",
        description: "Push, publish, delete, or otherwise mutate remote state.",
      },
    ],
  },
  {
    id: "network-sharing",
    label: "Network and sharing",
    items: [
      {
        key: "network.retrieve",
        label: "Retrieve remote information",
        description: "Retrieve remote information without changing it.",
      },
      {
        key: "network.publish",
        label: "Publish remote information",
        description: "Submit, publish, or mutate remote information.",
      },
      {
        key: "network.listen",
        label: "Listen for connections",
        description: "Open a local port or listening service.",
      },
      {
        key: "network.upload",
        label: "Upload local data",
        description: "Upload or export local data to a named destination.",
      },
    ],
  },
  {
    id: "browser-desktop",
    label: "Browser and desktop",
    items: [
      {
        key: "browser.view",
        label: "View browser content",
        description: "View, read, or capture browser content.",
      },
      {
        key: "browser.interact",
        label: "Interact with webpages",
        description: "Click, type, select, navigate, or submit in a webpage.",
      },
      {
        key: "browser.authenticated",
        label: "Use authenticated browser sessions",
        description: "Use an existing authenticated browser session.",
      },
      {
        key: "desktop.control",
        label: "Control desktop applications",
        description: "Operate another desktop application.",
      },
    ],
  },
  {
    id: "credentials",
    label: "Credentials",
    items: [
      {
        key: "credential.use",
        label: "Use stored credentials",
        description: "Use a stored credential without revealing its raw value.",
      },
      {
        key: "credential.add",
        label: "Manage stored credentials",
        description: "Add, change, or remove a credential in secure storage.",
      },
      {
        key: "credential.reveal",
        label: "Reveal credentials",
        description: "Reveal or copy a raw credential value.",
      },
    ],
  },
  {
    id: "extensions-c4os",
    label: "Extensions and C4OS",
    items: [
      {
        key: "extension.read",
        label: "Read extension metadata",
        description: "Read skill, plugin, MCP, and C4OS metadata.",
      },
      {
        key: "extension.use",
        label: "Use extensions",
        description: "Use an enabled skill, plugin, app, or MCP tool.",
      },
      {
        key: "extension.configure",
        label: "Configure extensions",
        description: "Install, enable, remove, or configure an extension.",
      },
      {
        key: "c4os.policy",
        label: "Change C4OS policy",
        description: "Change C4OS policy, authority, or managed configuration.",
      },
      {
        key: "artifact.export",
        label: "Export artifacts",
        description: "Export or share an artifact outside C4OS.",
      },
    ],
  },
];

export const PRESET_LABELS: Readonly<Record<PolicyPreset, string>> = {
  "ask-for-approval": "Ask for approval",
  "approve-safe-actions": "Approve safe actions",
  "approve-for-me": "Approve for me",
  custom: "Custom",
};

export function draftValues(
  categoryValues: PolicyCategoryValues,
): Record<(typeof POLICY_SETTING_KEYS)[number], PolicyValue> {
  return Object.fromEntries(
    POLICY_SETTING_KEYS.map((key) => [key, categoryValues[key] ?? "default"]),
  ) as Record<(typeof POLICY_SETTING_KEYS)[number], PolicyValue>;
}

export function categoryValues(
  draft: Readonly<Record<(typeof POLICY_SETTING_KEYS)[number], PolicyValue>>,
): PolicyCategoryValues {
  return Object.fromEntries(
    POLICY_SETTING_KEYS.map((key) => [
      key,
      draft[key] === "default" ? null : draft[key],
    ]),
  ) as unknown as PolicyCategoryValues;
}

export function effectiveResult(
  value: PolicyValue,
  preset: PolicyPreset,
): string {
  switch (value) {
    case "allow":
      return "Allows matching actions unless a stricter exception, safety boundary, or managed rule asks or denies.";
    case "ask":
      return "Asks before matching actions unless a stricter rule denies them.";
    case "deny":
      return "Denies matching actions.";
    case "default":
      return `Uses ${PRESET_LABELS[preset]}; action facts and stricter boundaries determine the final result.`;
  }
}
