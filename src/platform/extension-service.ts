import { invokeNative } from "./native-transport";
import type { ExtensionLifecycle as GeneratedExtensionLifecycle } from "../generated/ExtensionLifecycle";
import type { ExtensionPackageKind as GeneratedExtensionPackageKind } from "../generated/ExtensionPackageKind";
import type { ExtensionServiceSnapshot as GeneratedExtensionServiceSnapshot } from "../generated/ExtensionServiceSnapshot";
import type { ExtensionSourceKind as GeneratedExtensionSourceKind } from "../generated/ExtensionSourceKind";
import type { ExtensionTrustState as GeneratedExtensionTrustState } from "../generated/ExtensionTrustState";
import type { MarketplaceSnapshot as GeneratedMarketplaceSnapshot } from "../generated/MarketplaceSnapshot";
import type { MarketplaceSourceInput as GeneratedMarketplaceSourceInput } from "../generated/MarketplaceSourceInput";
import type { PluginHookSnapshot as GeneratedPluginHookSnapshot } from "../generated/PluginHookSnapshot";
import type { PluginSnapshot as GeneratedPluginSnapshot } from "../generated/PluginSnapshot";
import type { SkillInstructionsSnapshot as GeneratedSkillInstructionsSnapshot } from "../generated/SkillInstructionsSnapshot";
import type { SkillQualifiedIdentity as GeneratedSkillQualifiedIdentity } from "../generated/SkillQualifiedIdentity";
import type { SkillSnapshot as GeneratedSkillSnapshot } from "../generated/SkillSnapshot";
import {
  MAX_IDENTIFIER_BYTES,
  PROTOCOL_VERSION,
  type CorrelationId,
  type RequestId,
  type SnapshotRequest,
  type StateGeneration,
} from "./protocol";
import { ProtocolBoundaryError } from "./tauri-adapter";

export type ExtensionCommand =
  | "extension_snapshot"
  | "extension_add_marketplace"
  | "extension_refresh_catalogs"
  | "extension_install_disabled"
  | "extension_enable"
  | "extension_disable"
  | "extension_stage_update"
  | "extension_activate_update"
  | "extension_rollback"
  | "extension_revoke"
  | "extension_revoke_key"
  | "extension_uninstall"
  | "extension_set_skill_enabled"
  | "extension_select_skill"
  | "extension_customize_skill"
  | "extension_review_hook"
  | "extension_load_skill"
  | "extension_publisher_link";

export type ExtensionLifecycle = GeneratedExtensionLifecycle;
export type ExtensionPackageKind = GeneratedExtensionPackageKind;
export type ExtensionTrustState = GeneratedExtensionTrustState;
export type ExtensionSourceKind = GeneratedExtensionSourceKind;
export type SkillQualifiedIdentity = GeneratedSkillQualifiedIdentity;
export type MarketplaceSnapshot = GeneratedMarketplaceSnapshot;
export type PluginHookSnapshot = GeneratedPluginHookSnapshot;
export type PluginSnapshot = GeneratedPluginSnapshot;
export type SkillSnapshot = GeneratedSkillSnapshot;
export type ExtensionServiceSnapshot = GeneratedExtensionServiceSnapshot;
export type SkillInstructionsSnapshot = GeneratedSkillInstructionsSnapshot;
export type MarketplaceSourceInput = GeneratedMarketplaceSourceInput;

export interface ExtensionTransport {
  invoke(
    command: ExtensionCommand,
    args: Readonly<Record<string, unknown>>,
  ): Promise<unknown>;
}

export interface ExtensionAdapter {
  readSnapshot(): Promise<ExtensionServiceSnapshot>;
  addMarketplace(
    input: MarketplaceSourceInput,
  ): Promise<ExtensionServiceSnapshot>;
  refreshCatalogs(): Promise<ExtensionServiceSnapshot>;
  installDisabled(packageId: string): Promise<ExtensionServiceSnapshot>;
  enable(packageId: string): Promise<ExtensionServiceSnapshot>;
  disable(packageId: string): Promise<ExtensionServiceSnapshot>;
  stageUpdate(packageId: string): Promise<ExtensionServiceSnapshot>;
  activateUpdate(packageId: string): Promise<ExtensionServiceSnapshot>;
  rollback(packageId: string): Promise<ExtensionServiceSnapshot>;
  revoke(packageId: string, reason: string): Promise<ExtensionServiceSnapshot>;
  revokeKey(keyId: string, reason: string): Promise<ExtensionServiceSnapshot>;
  uninstall(packageId: string): Promise<ExtensionServiceSnapshot>;
  setSkillEnabled(
    skillIdentity: string,
    enabled: boolean,
  ): Promise<ExtensionServiceSnapshot>;
  selectSkill(skillIdentity: string): Promise<ExtensionServiceSnapshot>;
  customizeSkill(skillIdentity: string): Promise<ExtensionServiceSnapshot>;
  reviewHook(
    packageId: string,
    hookId: string,
  ): Promise<ExtensionServiceSnapshot>;
  loadSkill(skillIdentity: string): Promise<SkillInstructionsSnapshot>;
  publisherLink(
    packageId: string,
    link: "privacy" | "terms" | "website",
  ): Promise<string>;
}

const nativeAdapter = createExtensionAdapter({
  invoke(command, args) {
    return invokeNative(command, args);
  },
});

export const readExtensionSnapshot = () => nativeAdapter.readSnapshot();
export const addExtensionMarketplace = (input: MarketplaceSourceInput) =>
  nativeAdapter.addMarketplace(input);
export const refreshExtensionCatalogs = () => nativeAdapter.refreshCatalogs();
export const installExtensionDisabled = (packageId: string) =>
  nativeAdapter.installDisabled(packageId);
export const enableExtension = (packageId: string) =>
  nativeAdapter.enable(packageId);
export const disableExtension = (packageId: string) =>
  nativeAdapter.disable(packageId);
export const stageExtensionUpdate = (packageId: string) =>
  nativeAdapter.stageUpdate(packageId);
export const activateExtensionUpdate = (packageId: string) =>
  nativeAdapter.activateUpdate(packageId);
export const rollbackExtension = (packageId: string) =>
  nativeAdapter.rollback(packageId);
export const revokeExtension = (packageId: string, reason: string) =>
  nativeAdapter.revoke(packageId, reason);
export const revokeExtensionKey = (keyId: string, reason: string) =>
  nativeAdapter.revokeKey(keyId, reason);
export const uninstallExtension = (packageId: string) =>
  nativeAdapter.uninstall(packageId);
export const setExtensionSkillEnabled = (
  skillIdentity: string,
  enabled: boolean,
) => nativeAdapter.setSkillEnabled(skillIdentity, enabled);
export const selectExtensionSkill = (skillIdentity: string) =>
  nativeAdapter.selectSkill(skillIdentity);
export const customizeExtensionSkill = (skillIdentity: string) =>
  nativeAdapter.customizeSkill(skillIdentity);
export const reviewExtensionHook = (packageId: string, hookId: string) =>
  nativeAdapter.reviewHook(packageId, hookId);
export const loadExtensionSkill = (skillIdentity: string) =>
  nativeAdapter.loadSkill(skillIdentity);
export const openExtensionPublisherLink = (
  packageId: string,
  link: "privacy" | "terms" | "website",
) => nativeAdapter.publisherLink(packageId, link);

export function createExtensionAdapter(
  transport: ExtensionTransport,
  options: {
    readonly requestIdFactory?: () => RequestId;
    readonly correlationIdFactory?: () => CorrelationId;
  } = {},
): ExtensionAdapter {
  const requestIdFactory =
    options.requestIdFactory ?? (() => secureUuid("request") as RequestId);
  const correlationIdFactory =
    options.correlationIdFactory ??
    (() => secureUuid("correlation") as CorrelationId);
  let generation = 0;

  const request = (): SnapshotRequest => ({
    protocolVersion: PROTOCOL_VERSION,
    requestId: requestIdFactory(),
    correlationId: correlationIdFactory(),
    expectedGeneration: generation as StateGeneration,
  });

  const invoke = async <Payload>(
    command: ExtensionCommand,
    snapshotRequest: SnapshotRequest,
    args: Readonly<Record<string, unknown>>,
    parsePayload: (raw: unknown) => Payload,
  ): Promise<{ readonly generation: number; readonly payload: Payload }> => {
    let raw: unknown;
    try {
      raw = await transport.invoke(command, args);
    } catch {
      throw new ProtocolBoundaryError(
        "unavailable",
        "The native Extension service is unavailable.",
        true,
      );
    }
    const envelope = parseEnvelope(raw, parsePayload);
    if (
      envelope.requestId !== snapshotRequest.requestId ||
      envelope.correlationId !== snapshotRequest.correlationId
    ) {
      throw new ProtocolBoundaryError(
        "correlationMismatch",
        "The Extension response identity did not match its request.",
      );
    }
    if (envelope.generation < generation) {
      throw invalidPayload("The Extension response generation regressed.");
    }
    generation = envelope.generation;
    return { generation: envelope.generation, payload: envelope.payload };
  };

  const invokeSnapshot = async (
    command: ExtensionCommand,
    input?: Readonly<Record<string, unknown>>,
  ): Promise<ExtensionServiceSnapshot> => {
    const snapshotRequest = request();
    const result = await invoke(
      command,
      snapshotRequest,
      input === undefined
        ? { request: snapshotRequest }
        : { request: snapshotRequest, input },
      parseExtensionSnapshot,
    );
    if (result.payload.generation !== result.generation) {
      throw invalidPayload("The Extension payload generation is inconsistent.");
    }
    return result.payload;
  };

  const packageMutation = (command: ExtensionCommand, packageId: string) =>
    invokeSnapshot(command, {
      expectedGeneration: generation,
      packageId: asIdentifier(packageId, "package ID"),
    });
  const skillMutation = (command: ExtensionCommand, skillIdentity: string) =>
    invokeSnapshot(command, {
      expectedGeneration: generation,
      skillIdentity: asIdentifier(skillIdentity, "Skill identity", 512),
    });

  return {
    readSnapshot: () => invokeSnapshot("extension_snapshot"),
    addMarketplace(input) {
      return invokeSnapshot("extension_add_marketplace", {
        source: asText(input.source, "marketplace source", 4_096),
        gitRef: asOptionalText(input.gitRef, "Git ref", 512),
        sparsePaths: input.sparsePaths.map((path) =>
          asText(path, "sparse path", 1_024),
        ),
        trustedOrigin: asText(input.trustedOrigin, "trusted origin", 4_096),
        signingKeyId: asIdentifier(input.signingKeyId, "signing key ID"),
        publicKeySha256: asDigest(input.publicKeySha256),
      });
    },
    refreshCatalogs: () => invokeSnapshot("extension_refresh_catalogs"),
    installDisabled: (packageId) =>
      packageMutation("extension_install_disabled", packageId),
    enable: (packageId) => packageMutation("extension_enable", packageId),
    disable: (packageId) => packageMutation("extension_disable", packageId),
    stageUpdate: (packageId) =>
      packageMutation("extension_stage_update", packageId),
    activateUpdate: (packageId) =>
      packageMutation("extension_activate_update", packageId),
    rollback: (packageId) => packageMutation("extension_rollback", packageId),
    revoke(packageId, reason) {
      return invokeSnapshot("extension_revoke", {
        expectedGeneration: generation,
        packageId: asIdentifier(packageId, "package ID"),
        reason: asText(reason, "revocation reason", 1_024),
      });
    },
    revokeKey(keyId, reason) {
      return invokeSnapshot("extension_revoke_key", {
        expectedGeneration: generation,
        keyId: asIdentifier(keyId, "signing key ID"),
        reason: asText(reason, "revocation reason", 1_024),
      });
    },
    uninstall: (packageId) => packageMutation("extension_uninstall", packageId),
    setSkillEnabled(skillIdentity, enabled) {
      return invokeSnapshot("extension_set_skill_enabled", {
        expectedGeneration: generation,
        skillIdentity: asIdentifier(skillIdentity, "Skill identity", 512),
        enabled,
      });
    },
    selectSkill: (skillIdentity) =>
      skillMutation("extension_select_skill", skillIdentity),
    customizeSkill: (skillIdentity) =>
      skillMutation("extension_customize_skill", skillIdentity),
    reviewHook(packageId, hookId) {
      return invokeSnapshot("extension_review_hook", {
        expectedGeneration: generation,
        packageId: asIdentifier(packageId, "package ID"),
        hookId: asIdentifier(hookId, "hook ID"),
      });
    },
    async loadSkill(skillIdentity) {
      const snapshotRequest = request();
      return (
        await invoke(
          "extension_load_skill",
          snapshotRequest,
          {
            request: snapshotRequest,
            input: {
              expectedGeneration: generation,
              skillIdentity: asIdentifier(skillIdentity, "Skill identity", 512),
            },
          },
          parseSkillInstructions,
        )
      ).payload;
    },
    async publisherLink(packageId, link) {
      const snapshotRequest = request();
      return (
        await invoke(
          "extension_publisher_link",
          snapshotRequest,
          {
            request: snapshotRequest,
            input: { packageId: asIdentifier(packageId, "package ID"), link },
          },
          (raw) => asText(raw, "publisher link", 4_096),
        )
      ).payload;
    },
  };
}

function parseEnvelope<Payload>(
  raw: unknown,
  parsePayload: (raw: unknown) => Payload,
) {
  const value = exactRecord(
    raw,
    ["protocolVersion", "requestId", "correlationId", "generation", "payload"],
    "Extension envelope",
  );
  if (value.protocolVersion !== PROTOCOL_VERSION) {
    throw new ProtocolBoundaryError(
      "unknownProtocolVersion",
      "Unsupported protocol version.",
    );
  }
  return {
    requestId: asIdentifier(value.requestId, "request ID") as RequestId,
    correlationId: asIdentifier(
      value.correlationId,
      "correlation ID",
    ) as CorrelationId,
    generation: asGeneration(value.generation),
    payload: parsePayload(value.payload),
  };
}

function parseExtensionSnapshot(raw: unknown): ExtensionServiceSnapshot {
  const value = exactRecord(
    raw,
    [
      "schemaVersion",
      "generation",
      "marketplaces",
      "plugins",
      "skills",
      "selectedSkill",
      "activeWorkers",
      "lastEventId",
    ],
    "Extension snapshot",
  );
  if (value.schemaVersion !== 1)
    throw invalidPayload("Unsupported Extension schema.");
  return {
    schemaVersion: 1,
    generation: asGeneration(value.generation),
    marketplaces: asArray(value.marketplaces, "marketplaces").map(
      parseMarketplace,
    ),
    plugins: asArray(value.plugins, "plugins").map(parsePlugin),
    skills: asArray(value.skills, "skills").map(parseSkill),
    selectedSkill:
      value.selectedSkill === null
        ? null
        : parseSkillIdentity(value.selectedSkill),
    activeWorkers: asBoundedInteger(
      value.activeWorkers,
      "active workers",
      65_535,
      true,
    ),
    lastEventId: asGeneration(value.lastEventId),
  };
}

function parseMarketplace(raw: unknown): MarketplaceSnapshot {
  const value = exactRecord(
    raw,
    [
      "marketplaceId",
      "label",
      "source",
      "gitRef",
      "resolvedCommit",
      "sparsePaths",
      "catalogDigest",
      "status",
      "trustedOrigin",
      "originKeyId",
      "packageCount",
      "lastCheckedAtMs",
      "detail",
    ],
    "marketplace",
  );
  return {
    marketplaceId: asIdentifier(value.marketplaceId, "marketplace ID"),
    label: asText(value.label, "marketplace label", 16_384),
    source: asText(value.source, "marketplace source", 16_384),
    gitRef: asOptionalText(value.gitRef, "Git ref", 512),
    resolvedCommit: asOptionalGitCommit(value.resolvedCommit),
    sparsePaths: asStringArray(value.sparsePaths, "sparse paths", 1_024),
    catalogDigest: asDigest(value.catalogDigest),
    status: asText(value.status, "marketplace status", 160),
    trustedOrigin: asText(value.trustedOrigin, "trusted origin", 4_096),
    originKeyId: asIdentifier(value.originKeyId, "origin key ID"),
    packageCount: asBoundedInteger(
      value.packageCount,
      "package count",
      1_024,
      true,
    ),
    lastCheckedAtMs: asOptionalGeneration(value.lastCheckedAtMs),
    detail: asText(value.detail, "marketplace detail", 16_384),
  };
}

function parsePlugin(raw: unknown): PluginSnapshot {
  const value = exactRecord(
    raw,
    [
      "packageId",
      "packageKind",
      "name",
      "summary",
      "publisher",
      "version",
      "digest",
      "originKeyId",
      "contentKeyId",
      "trust",
      "lifecycle",
      "capabilities",
      "website",
      "terms",
      "privacyPolicy",
      "activeDigest",
      "lastKnownGoodDigest",
      "failureCode",
      "hasReviewedHooks",
      "marketplaceId",
      "source",
      "compatibility",
      "verifiedAtMs",
      "hooks",
      "settings",
      "apps",
      "mcpServers",
      "availableVersion",
      "availableDigest",
      "stagedVersion",
      "stagedDigest",
      "activeVersion",
      "lastKnownGoodVersion",
      "revocationReason",
    ],
    "Plugin",
  );
  return {
    packageId: asIdentifier(value.packageId, "package ID"),
    packageKind: asEnum(
      value.packageKind,
      ["plugin", "skill"] as const,
      "package kind",
    ),
    name: asText(value.name, "Plugin name", 16_384),
    summary: asText(value.summary, "Plugin summary", 16_384),
    publisher: asText(value.publisher, "publisher", 16_384),
    version: asText(value.version, "version", 160),
    digest: asDigest(value.digest),
    originKeyId: asIdentifier(value.originKeyId, "origin key ID"),
    contentKeyId: asIdentifier(value.contentKeyId, "content key ID"),
    trust: asEnum(
      value.trust,
      ["untrusted", "trusted", "revoked", "invalid"] as const,
      "trust",
    ),
    lifecycle: asEnum(
      value.lifecycle,
      [
        "available",
        "quarantined",
        "installedDisabled",
        "enabling",
        "enabled",
        "updateStaged",
        "executing",
        "revoked",
        "failed",
        "rolledBack",
      ] as const,
      "lifecycle",
    ),
    capabilities: asStringArray(value.capabilities, "capabilities", 160),
    website: asOptionalText(value.website, "website", 4_096),
    terms: asOptionalText(value.terms, "terms", 4_096),
    privacyPolicy: asOptionalText(value.privacyPolicy, "privacy policy", 4_096),
    activeDigest: asOptionalDigest(value.activeDigest),
    lastKnownGoodDigest: asOptionalDigest(value.lastKnownGoodDigest),
    failureCode: asOptionalText(value.failureCode, "failure code", 160),
    hasReviewedHooks: asBoolean(value.hasReviewedHooks, "reviewed hook state"),
    marketplaceId: asIdentifier(value.marketplaceId, "marketplace ID"),
    source: asText(value.source, "Plugin source", 16_384),
    compatibility: asText(value.compatibility, "compatibility", 1_024),
    verifiedAtMs: asOptionalGeneration(value.verifiedAtMs),
    hooks: asArray(value.hooks, "hooks").map(parseHook),
    settings: asArray(value.settings, "Plugin settings").map(
      parsePluginSetting,
    ),
    apps: asArray(value.apps, "Plugin apps").map(parsePluginApp),
    mcpServers: asArray(value.mcpServers, "Plugin MCP servers").map(
      parsePluginMcpServer,
    ),
    availableVersion: asOptionalText(
      value.availableVersion,
      "available version",
      160,
    ),
    availableDigest: asOptionalDigest(value.availableDigest),
    stagedVersion: asOptionalText(value.stagedVersion, "staged version", 160),
    stagedDigest: asOptionalDigest(value.stagedDigest),
    activeVersion: asOptionalText(value.activeVersion, "active version", 160),
    lastKnownGoodVersion: asOptionalText(
      value.lastKnownGoodVersion,
      "last-known-good version",
      160,
    ),
    revocationReason: asOptionalText(
      value.revocationReason,
      "revocation reason",
      1_024,
    ),
  };
}

function parsePluginSetting(
  raw: unknown,
): GeneratedPluginSnapshot["settings"][number] {
  const value = exactRecord(
    raw,
    ["settingId", "label", "description", "kind", "required", "choices"],
    "Plugin setting",
  );
  return {
    settingId: asIdentifier(value.settingId, "setting ID"),
    label: asText(value.label, "setting label", 16_384),
    description: asText(value.description, "setting description", 16_384),
    kind: asEnum(
      value.kind,
      [
        "boolean",
        "integer",
        "string",
        "select",
        "credentialReference",
      ] as const,
      "setting kind",
    ),
    required: asBoolean(value.required, "setting required state"),
    choices: asStringArray(value.choices, "setting choices", 16_384),
  };
}

function parsePluginApp(raw: unknown): GeneratedPluginSnapshot["apps"][number] {
  const value = exactRecord(
    raw,
    ["appId", "title", "summary", "settingIds"],
    "Plugin app",
  );
  return {
    appId: asIdentifier(value.appId, "app ID"),
    title: asText(value.title, "app title", 16_384),
    summary: asText(value.summary, "app summary", 16_384),
    settingIds: asArray(value.settingIds, "app setting IDs").map((settingId) =>
      asIdentifier(settingId, "app setting ID"),
    ),
  };
}

function parsePluginMcpServer(
  raw: unknown,
): GeneratedPluginSnapshot["mcpServers"][number] {
  const value = exactRecord(
    raw,
    ["serverId", "name", "transport", "settingIds"],
    "Plugin MCP server",
  );
  return {
    serverId: asIdentifier(value.serverId, "MCP server ID"),
    name: asText(value.name, "MCP server name", 16_384),
    transport: asEnum(
      value.transport,
      ["stdio", "streamableHttp"] as const,
      "MCP transport",
    ),
    settingIds: asArray(value.settingIds, "MCP setting IDs").map((settingId) =>
      asIdentifier(settingId, "MCP setting ID"),
    ),
  };
}

function parseHook(raw: unknown): PluginHookSnapshot {
  const value = exactRecord(
    raw,
    [
      "hookId",
      "name",
      "arguments",
      "event",
      "reviewDigest",
      "reviewed",
      "status",
      "grants",
      "lastResult",
    ],
    "hook",
  );
  return {
    hookId: asIdentifier(value.hookId, "hook ID"),
    name: asText(value.name, "hook name", 1_024),
    arguments: asStringArray(value.arguments, "hook arguments", 1_024),
    event: asIdentifier(value.event, "hook event"),
    reviewDigest: asDigest(value.reviewDigest),
    reviewed: asBoolean(value.reviewed, "hook review state"),
    status: asText(value.status, "hook status", 160),
    grants: asStringArray(value.grants, "hook grants", 160),
    lastResult: asOptionalText(value.lastResult, "hook result", 16_384),
  };
}

function parseSkill(raw: unknown): SkillSnapshot {
  const value = exactRecord(
    raw,
    [
      "identity",
      "name",
      "summary",
      "packageId",
      "sourceLabel",
      "enabled",
      "eligible",
      "valid",
      "active",
      "shadowed",
      "instructionsLoaded",
      "collisionSources",
      "diagnostic",
      "version",
      "isInstalled",
      "eligibilityDetail",
    ],
    "Skill",
  );
  return {
    identity: parseSkillIdentity(value.identity),
    name: asText(value.name, "Skill name", 16_384),
    summary: asText(value.summary, "Skill summary", 16_384),
    packageId: asOptionalText(value.packageId, "Skill package ID", 160),
    sourceLabel: asText(value.sourceLabel, "Skill source label", 1_024),
    enabled: asBoolean(value.enabled, "Skill enabled state"),
    eligible: asBoolean(value.eligible, "Skill eligibility"),
    valid: asBoolean(value.valid, "Skill validity"),
    active: asBoolean(value.active, "Skill active state"),
    shadowed: asBoolean(value.shadowed, "Skill shadow state"),
    instructionsLoaded: asBoolean(value.instructionsLoaded, "Skill load state"),
    collisionSources: asArray(value.collisionSources, "Skill collisions").map(
      parseSkillIdentity,
    ),
    diagnostic: asOptionalText(value.diagnostic, "Skill diagnostic", 16_384),
    version: asText(value.version, "Skill version", 160),
    isInstalled: asBoolean(value.isInstalled, "Skill installed state"),
    eligibilityDetail: asText(
      value.eligibilityDetail,
      "Skill eligibility detail",
      16_384,
    ),
  };
}

function parseSkillIdentity(raw: unknown): SkillQualifiedIdentity {
  const value = exactRecord(
    raw,
    ["sourceKind", "sourceId", "skillId"],
    "Skill identity",
  );
  return {
    sourceKind: asEnum(
      value.sourceKind,
      [
        "projectLocal",
        "workspaceLocal",
        "userGlobal",
        "pluginProvided",
        "bundled",
      ] as const,
      "Skill source kind",
    ),
    sourceId: asIdentifier(value.sourceId, "Skill source ID"),
    skillId: asIdentifier(value.skillId, "Skill ID"),
  };
}

function parseSkillInstructions(raw: unknown): SkillInstructionsSnapshot {
  const value = exactRecord(
    raw,
    [
      "identity",
      "packageId",
      "instructions",
      "entrypointDigest",
      "referencedResources",
    ],
    "Skill instructions",
  );
  return {
    identity: parseSkillIdentity(value.identity),
    packageId: asOptionalText(value.packageId, "Skill package ID", 160),
    instructions: asText(value.instructions, "Skill instructions", 256 * 1_024),
    entrypointDigest: asDigest(value.entrypointDigest),
    referencedResources: asStringArray(
      value.referencedResources,
      "Skill references",
      1_024,
    ),
  };
}

export function stableSkillIdentity(identity: SkillQualifiedIdentity): string {
  const source = {
    projectLocal: "project",
    workspaceLocal: "workspace",
    userGlobal: "user",
    pluginProvided: "plugin",
    bundled: "bundled",
  }[identity.sourceKind];
  return `${source}:${identity.sourceId}:${identity.skillId}`;
}

function exactRecord(
  raw: unknown,
  keys: readonly string[],
  label: string,
): Record<string, unknown> {
  if (typeof raw !== "object" || raw === null || Array.isArray(raw))
    throw invalidPayload(`${label} is invalid.`);
  const value = raw as Record<string, unknown>;
  const actual = Object.keys(value).sort();
  const expected = [...keys].sort();
  if (
    actual.length !== expected.length ||
    actual.some((key, index) => key !== expected[index])
  )
    throw invalidPayload(`${label} fields are invalid.`);
  return value;
}

function asArray(raw: unknown, label: string): readonly unknown[] {
  if (!Array.isArray(raw) || raw.length > 4_096)
    throw invalidPayload(`${label} are invalid.`);
  return raw;
}
function asStringArray(raw: unknown, label: string, maximum: number): string[] {
  return asArray(raw, label).map((value) => asText(value, label, maximum));
}
function asText(raw: unknown, label: string, maximum: number): string {
  if (
    typeof raw !== "string" ||
    raw.length === 0 ||
    raw.length > maximum ||
    raw.includes("\0")
  )
    throw invalidPayload(`${label} is invalid.`);
  return raw;
}
function asOptionalText(
  raw: unknown,
  label: string,
  maximum: number,
): string | null {
  return raw === null ? null : asText(raw, label, maximum);
}
function asIdentifier(
  raw: unknown,
  label: string,
  maximum = MAX_IDENTIFIER_BYTES,
): string {
  const value = asText(raw, label, maximum);
  if (!/^[A-Za-z0-9._:@/-]+$/.test(value))
    throw invalidPayload(`${label} is invalid.`);
  return value;
}
function asBoolean(raw: unknown, label: string): boolean {
  if (typeof raw !== "boolean") throw invalidPayload(`${label} is invalid.`);
  return raw;
}
function asGeneration(raw: unknown): number {
  if (!Number.isSafeInteger(raw) || (raw as number) < 0)
    throw invalidPayload("Generation is invalid.");
  return raw as number;
}
function asOptionalGeneration(raw: unknown): number | null {
  return raw === null ? null : asGeneration(raw);
}
function asBoundedInteger(
  raw: unknown,
  label: string,
  maximum: number,
  allowZero = false,
): number {
  const value = asGeneration(raw);
  if (value > maximum || (!allowZero && value === 0))
    throw invalidPayload(`${label} is invalid.`);
  return value;
}
function asDigest(raw: unknown): string {
  if (typeof raw !== "string" || !/^sha256:[0-9a-f]{64}$/.test(raw))
    throw invalidPayload("Digest is invalid.");
  return raw;
}
function asOptionalDigest(raw: unknown): string | null {
  return raw === null ? null : asDigest(raw);
}
function asOptionalGitCommit(raw: unknown): string | null {
  if (raw === null) return null;
  if (typeof raw !== "string" || !/^(?:[0-9a-f]{40}|[0-9a-f]{64})$/.test(raw))
    throw invalidPayload("Resolved Git commit is invalid.");
  return raw;
}
function asEnum<const Values extends readonly string[]>(
  raw: unknown,
  values: Values,
  label: string,
): Values[number] {
  if (typeof raw !== "string" || !values.includes(raw))
    throw invalidPayload(`${label} is invalid.`);
  return raw as Values[number];
}
function secureUuid(label: string): string {
  const value = globalThis.crypto?.randomUUID?.();
  if (!value)
    throw new ProtocolBoundaryError(
      "unavailable",
      `A secure ${label} is unavailable.`,
    );
  return value;
}
function invalidPayload(message: string): ProtocolBoundaryError {
  return new ProtocolBoundaryError("invalidPayload", message);
}
