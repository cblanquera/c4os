export type PolicyValue = "default" | "allow" | "ask" | "deny";

export interface PolicyItem {
  readonly key: string;
  readonly description: string;
}

export interface PolicyGroup {
  readonly id: string;
  readonly label: string;
  readonly items: readonly PolicyItem[];
}

export interface PolicyException {
  readonly id: string;
  readonly decision: "Allow" | "Ask" | "Deny";
  readonly action: string;
  readonly scope: string;
  readonly source: string;
  readonly duration: string;
}

export const POLICY_OPTIONS: ReadonlyArray<{
  readonly value: PolicyValue;
  readonly label: string;
}> = [
  { value: "default", label: "Use default" },
  { value: "allow", label: "Allow" },
  { value: "ask", label: "Ask" },
  { value: "deny", label: "Deny" },
];

export const POLICY_GROUPS: readonly PolicyGroup[] = [
  {
    id: "workspace-files",
    label: "Workspace files",
    items: [
      {
        key: "workspace.read",
        description: "Read files and folders in the trusted workspace.",
      },
      {
        key: "workspace.modify",
        description: "Create or modify content in the trusted workspace.",
      },
      {
        key: "workspace.delete",
        description: "Delete or trash content in the trusted workspace.",
      },
      {
        key: "workspace.outside",
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
        description: "Inspect processes, shell state, and command output.",
      },
      {
        key: "command.workspace",
        description: "Execute commands inside the trusted workspace.",
      },
      {
        key: "command.system",
        description:
          "Execute commands outside the workspace or against system state.",
      },
      {
        key: "process.control",
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
        description: "Read local repository status, history, and differences.",
      },
      {
        key: "git.local.change",
        description:
          "Change branches, commits, tags, or worktree state locally.",
      },
      {
        key: "git.remote.read",
        description: "Fetch or inspect remote repository information.",
      },
      {
        key: "git.remote.publish",
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
        description: "Retrieve remote information without changing it.",
      },
      {
        key: "network.publish",
        description: "Submit, publish, or mutate remote information.",
      },
      {
        key: "network.listen",
        description: "Open a local port or listening service.",
      },
      {
        key: "network.upload",
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
        description: "View, read, or capture browser content.",
      },
      {
        key: "browser.interact",
        description: "Click, type, select, navigate, or submit in a webpage.",
      },
      {
        key: "browser.authenticated",
        description: "Use an existing authenticated browser session.",
      },
      {
        key: "desktop.control",
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
        description: "Use a stored credential without revealing its raw value.",
      },
      {
        key: "credential.add",
        description: "Add, change, or remove a credential in secure storage.",
      },
      {
        key: "credential.reveal",
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
        description: "Read skill, plugin, MCP, and C4OS metadata.",
      },
      {
        key: "extension.use",
        description: "Use an enabled skill, plugin, app, or MCP tool.",
      },
      {
        key: "extension.configure",
        description: "Install, enable, remove, or configure an extension.",
      },
      {
        key: "c4os.policy",
        description: "Change C4OS policy, authority, or managed configuration.",
      },
      {
        key: "artifact.export",
        description: "Export or share an artifact outside C4OS.",
      },
    ],
  },
];

export const INITIAL_POLICY_EXCEPTIONS: readonly PolicyException[] = [
  {
    id: "git-status-workspace",
    decision: "Allow",
    action: "git status",
    scope: "~/c4os · This Workspace",
    source: "OpenCode",
    duration: "Persistent until revoked",
  },
  {
    id: "research-session-read",
    decision: "Allow",
    action: "reads in ~/Documents/Research",
    scope: "Current Chat session",
    source: "Pi",
    duration: "Expires when Chat closes",
  },
];

export function emptyPolicyValues(): Record<string, PolicyValue> {
  return Object.fromEntries(
    POLICY_GROUPS.flatMap((group) =>
      group.items.map((item) => [item.key, "default"]),
    ),
  );
}
