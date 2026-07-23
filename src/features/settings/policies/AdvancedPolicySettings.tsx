import { useMemo, useState } from "react";

import {
  Button,
  Notice,
  StatusRegion,
  Tabs,
} from "../../../components/accessible";
import {
  POLICY_GROUPS,
  POLICY_OPTIONS,
  PRESET_LABELS,
  categoryValues,
  draftValues,
  effectiveResult,
} from "./policy-catalog";
import {
  POLICY_SETTING_KEYS,
  type PolicyGroupDefinition,
  type PolicySettingKey,
  type PolicySettingsActions,
  type PolicySettingsSnapshot,
  type PolicyValue,
  type ReadyPolicySettingsSnapshot,
} from "./types";

import "./advanced-policy-settings.css";

type PolicyView = "category-rules" | "exceptions";
type HeadingLevel = 1 | 2;

export interface AdvancedPolicySettingsProps {
  readonly actions: PolicySettingsActions;
  /** The Settings shell keeps Configuration selected while this child is open. */
  readonly configurationNavigationSelected: boolean;
  readonly headingId?: string;
  readonly headingLevel?: HeadingLevel;
  readonly snapshot: PolicySettingsSnapshot;
}

/**
 * Controlled Advanced Policies UI. Native transport and persistence remain in
 * the injected snapshot/actions boundary; this component owns draft UX only.
 */
export function AdvancedPolicySettings({
  actions,
  configurationNavigationSelected,
  headingId = "advanced-policy-settings-title",
  headingLevel = 2,
  snapshot,
}: AdvancedPolicySettingsProps) {
  const Heading = headingLevel === 1 ? "h1" : "h2";

  return (
    <section
      aria-labelledby={headingId}
      className="advanced-policy-settings"
      data-configuration-navigation-selected={
        configurationNavigationSelected ? "true" : "false"
      }
    >
      <header className="advanced-policy-settings__heading">
        <div>
          <Heading id={headingId}>Advanced Policies</Heading>
          <p>
            Set broad action rules and revoke narrow exceptions remembered from
            approval prompts.
          </p>
        </div>
        {snapshot.status === "ready" ? (
          <span className="advanced-policy-settings__authority">
            Policy version {snapshot.policyVersion}
          </span>
        ) : null}
      </header>

      {snapshot.status === "loading" ? (
        <StatusRegion
          aria-busy="true"
          className="advanced-policy-state advanced-policy-state--loading"
        >
          <span aria-hidden="true" className="advanced-policy-state__spinner" />
          <span>
            <strong>Loading policy settings</strong>
            <span>{snapshot.message}</span>
          </span>
        </StatusRegion>
      ) : null}

      {snapshot.status === "error" ? (
        <Notice
          className="advanced-policy-state"
          title="Advanced Policies are unavailable"
          tone="danger"
        >
          <p>{snapshot.message}</p>
          {snapshot.retryable && actions.onRetry ? (
            <Button
              onPress={() => {
                void actions.onRetry?.().catch(() => undefined);
              }}
            >
              Try again
            </Button>
          ) : null}
        </Notice>
      ) : null}

      {snapshot.status === "ready" ? (
        <PolicyEditor actions={actions} snapshot={snapshot} />
      ) : null}
    </section>
  );
}

function PolicyEditor({
  actions,
  snapshot,
}: {
  readonly actions: PolicySettingsActions;
  readonly snapshot: ReadyPolicySettingsSnapshot;
}) {
  const [localAuthority, setLocalAuthority] =
    useState<ReadyPolicySettingsSnapshot | null>(null);
  const authority =
    localAuthority !== null &&
    localAuthority.policyVersion > snapshot.policyVersion
      ? localAuthority
      : snapshot;
  const saved = draftValues(authority.categoryValues);
  const [draftOverrides, setDraftOverrides] = useState<
    Partial<Record<PolicySettingKey, PolicyValue>>
  >({});
  const draft = { ...saved, ...draftOverrides };
  const [view, setView] = useState<PolicyView>("category-rules");
  const [activeGroupId, setActiveGroupId] = useState(POLICY_GROUPS[0]!.id);
  const [query, setQuery] = useState("");
  const [pendingAction, setPendingAction] = useState<
    "saving" | `revoke:${string}` | null
  >(null);
  const [actionError, setActionError] = useState("");
  const [notice, setNotice] = useState("");

  const dirty = POLICY_SETTING_KEYS.some((key) => draft[key] !== saved[key]);
  const busy = pendingAction !== null;
  const normalizedQuery = query.trim().toLocaleLowerCase();
  const visibleGroups = useMemo(
    () => visiblePolicyGroups(activeGroupId, normalizedQuery),
    [activeGroupId, normalizedQuery],
  );
  const visibleCount = visibleGroups.reduce(
    (count, group) => count + group.items.length,
    0,
  );
  const activeGroup = POLICY_GROUPS.find((group) => group.id === activeGroupId);
  const currentPreset = dirty
    ? "Custom (unsaved)"
    : PRESET_LABELS[authority.preset];

  function updatePolicy(key: PolicySettingKey, value: PolicyValue) {
    setDraftOverrides((current) => ({ ...current, [key]: value }));
    setActionError("");
    setNotice("");
  }

  async function savePolicies() {
    if (!dirty || busy) return;
    setPendingAction("saving");
    setActionError("");
    setNotice("");
    try {
      const updated = await actions.onSave({
        categoryValues: categoryValues(draft),
        expectedCoordinatorGeneration: authority.coordinatorGeneration,
        expectedPolicyVersion: authority.policyVersion,
      });
      setLocalAuthority(updated);
      setDraftOverrides({});
      setNotice("Policies saved. New actions use the updated effective rules.");
    } catch (error) {
      setActionError(actionFailure(error, "Policy changes were not saved."));
    } finally {
      setPendingAction(null);
    }
  }

  function revertPolicies() {
    if (busy) return;
    setDraftOverrides({});
    setActionError("");
    setNotice("Unsaved policy changes reverted.");
  }

  async function revokeException(exceptionId: string) {
    if (busy) return;
    setPendingAction(`revoke:${exceptionId}`);
    setActionError("");
    setNotice("");
    try {
      const updated = await actions.onRevokeException({
        exceptionId,
        expectedCoordinatorGeneration: authority.coordinatorGeneration,
        expectedPolicyVersion: authority.policyVersion,
      });
      setLocalAuthority(updated);
      setNotice(
        "Exception revoked. Outstanding matching authorizations expire immediately.",
      );
    } catch (error) {
      setActionError(actionFailure(error, "The exception was not revoked."));
    } finally {
      setPendingAction(null);
    }
  }

  return (
    <div
      aria-busy={busy}
      className="advanced-policy-editor"
      data-generation={authority.policyVersion}
    >
      <section
        aria-label="Current policy guardrails"
        className="advanced-policy-guardrail"
      >
        <div>
          <span>Current preset</span>
          <strong>{currentPreset}</strong>
        </div>
        <p>
          The most restrictive matching result wins. Sandbox, trusted-root,
          maximum-authority, and managed-policy boundaries cannot be bypassed.
        </p>
        <dl>
          <div>
            <dt>Maximum authority</dt>
            <dd>{authority.maximumAuthorityRuleCount}</dd>
          </div>
          <div>
            <dt>Managed requirements</dt>
            <dd>{authority.managedRequirementCount}</dd>
          </div>
        </dl>
      </section>

      {authority.operationError ? (
        <Notice title="The last policy operation failed" tone="danger">
          <p>{authority.operationError}</p>
        </Notice>
      ) : null}
      {actionError ? (
        <Notice title="Policy change was not applied" tone="danger">
          <p>{actionError}</p>
        </Notice>
      ) : null}
      {notice ? <StatusRegion>{notice}</StatusRegion> : null}

      <div className="advanced-policy-toolbar">
        <StatusRegion
          className="advanced-policy-toolbar__state"
          data-dirty={dirty}
        >
          {pendingAction === "saving"
            ? "Saving policy changes…"
            : dirty
              ? "Unsaved policy changes"
              : "No unsaved policy changes"}
        </StatusRegion>
        <div className="advanced-policy-toolbar__actions">
          <Button
            isDisabled={!dirty || busy}
            onPress={revertPolicies}
            variant="quiet"
          >
            Revert
          </Button>
          <Button
            isDisabled={!dirty || busy}
            onPress={() => void savePolicies()}
            variant="primary"
          >
            {pendingAction === "saving" ? "Saving…" : "Save policies"}
          </Button>
        </div>
      </div>

      <Tabs
        label="Advanced policy views"
        onSelectionChange={(key) => setView(key as PolicyView)}
        selectedKey={view}
        tabs={[
          {
            id: "category-rules",
            label: (
              <>
                Category rules <span>{POLICY_SETTING_KEYS.length}</span>
              </>
            ),
            content: (
              <CategoryRules
                activeGroupId={activeGroupId}
                draft={draft}
                effectiveCategoryValues={authority.effectiveCategoryValues}
                normalizedQuery={normalizedQuery}
                onChange={updatePolicy}
                onGroupChange={(groupId) => {
                  setQuery("");
                  setActiveGroupId(groupId);
                }}
                onQueryChange={setQuery}
                preset={authority.basePreset}
                query={query}
                title={
                  normalizedQuery.length > 0
                    ? "Search results"
                    : (activeGroup?.label ?? "Policy rules")
                }
                visibleCount={visibleCount}
                visibleGroups={visibleGroups}
              />
            ),
          },
          {
            id: "exceptions",
            label: (
              <>
                Exceptions <span>{authority.exceptions.length}</span>
              </>
            ),
            content: (
              <PolicyExceptions
                busy={busy}
                exceptions={authority.exceptions}
                onRevoke={(exceptionId) => void revokeException(exceptionId)}
                pendingAction={pendingAction}
              />
            ),
          },
        ]}
      />
    </div>
  );
}

function CategoryRules({
  activeGroupId,
  draft,
  effectiveCategoryValues,
  normalizedQuery,
  onChange,
  onGroupChange,
  onQueryChange,
  preset,
  query,
  title,
  visibleCount,
  visibleGroups,
}: {
  readonly activeGroupId: string;
  readonly draft: Readonly<Record<PolicySettingKey, PolicyValue>>;
  readonly effectiveCategoryValues: ReadyPolicySettingsSnapshot["effectiveCategoryValues"];
  readonly normalizedQuery: string;
  readonly onChange: (key: PolicySettingKey, value: PolicyValue) => void;
  readonly onGroupChange: (groupId: string) => void;
  readonly onQueryChange: (query: string) => void;
  readonly preset: ReadyPolicySettingsSnapshot["preset"];
  readonly query: string;
  readonly title: string;
  readonly visibleCount: number;
  readonly visibleGroups: readonly PolicyGroupDefinition[];
}) {
  return (
    <div className="advanced-policy-categories">
      <label className="advanced-policy-search">
        <span>Search policies</span>
        <input
          onChange={(event) => onQueryChange(event.currentTarget.value)}
          placeholder="Search actions, targets, and keys"
          type="search"
          value={query}
        />
      </label>

      <div className="advanced-policy-browser">
        <nav aria-label="Policy groups" className="advanced-policy-groups">
          {POLICY_GROUPS.map((group) => (
            <button
              aria-current={
                normalizedQuery.length === 0 && activeGroupId === group.id
                  ? "page"
                  : undefined
              }
              key={group.id}
              onClick={() => onGroupChange(group.id)}
              type="button"
            >
              <span>{group.label}</span>
              <span>{group.items.length}</span>
            </button>
          ))}
        </nav>

        <section
          aria-labelledby="advanced-policy-result-title"
          className="advanced-policy-results"
        >
          <header>
            <div>
              <h3 id="advanced-policy-result-title">{title}</h3>
              <p>
                Category choices contribute to the final result; stricter
                exceptions and boundaries still win.
              </p>
            </div>
            <span>{visibleCount} rules</span>
          </header>

          {visibleCount === 0 ? (
            <div className="advanced-policy-empty">
              <strong>No policies found</strong>
              <span>
                Try a different action, target, group, or setting key.
              </span>
            </div>
          ) : (
            <div className="advanced-policy-list">
              {visibleGroups.flatMap((group) =>
                group.items.map((item) => {
                  const value = draft[item.key];
                  const effectiveValue =
                    effectiveCategoryValues[item.key] ?? "default";
                  const selectId = `advanced-policy-${item.key.replaceAll(".", "-")}`;
                  return (
                    <article className="advanced-policy-row" key={item.key}>
                      <div className="advanced-policy-row__copy">
                        <label htmlFor={selectId}>{item.label}</label>
                        <code>{item.key}</code>
                        <p>
                          {normalizedQuery.length > 0
                            ? `${group.label} · `
                            : ""}
                          {item.description}
                        </p>
                        <p
                          className="advanced-policy-row__result"
                          data-decision={effectiveValue}
                        >
                          <strong>Effective result:</strong>{" "}
                          {effectiveResult(effectiveValue, preset)}
                        </p>
                      </div>
                      <select
                        aria-label={`${item.key} policy`}
                        id={selectId}
                        onChange={(event) =>
                          onChange(
                            item.key,
                            event.currentTarget.value as PolicyValue,
                          )
                        }
                        value={value}
                      >
                        {POLICY_OPTIONS.map((option) => (
                          <option key={option.value} value={option.value}>
                            {option.label}
                          </option>
                        ))}
                      </select>
                    </article>
                  );
                }),
              )}
            </div>
          )}
        </section>
      </div>
    </div>
  );
}

function PolicyExceptions({
  busy,
  exceptions,
  onRevoke,
  pendingAction,
}: {
  readonly busy: boolean;
  readonly exceptions: ReadyPolicySettingsSnapshot["exceptions"];
  readonly onRevoke: (exceptionId: string) => void;
  readonly pendingAction: "saving" | `revoke:${string}` | null;
}) {
  return (
    <section
      aria-labelledby="advanced-policy-exceptions-title"
      className="advanced-policy-exceptions"
    >
      <header>
        <div>
          <h3 id="advanced-policy-exceptions-title">Concrete exceptions</h3>
          <p>
            Prompt-created exceptions bind a specific action, target, scope, and
            authority. Revoking one leaves category rules unchanged.
          </p>
        </div>
        <span>{exceptions.length} active</span>
      </header>

      {exceptions.length === 0 ? (
        <div className="advanced-policy-empty">
          <strong>No concrete exceptions</strong>
          <span>Category defaults and safety boundaries remain active.</span>
        </div>
      ) : (
        <div className="advanced-policy-exception-list">
          {exceptions.map((exception) => {
            const revoking =
              pendingAction === `revoke:${exception.exceptionId}`;
            return (
              <article key={exception.exceptionId}>
                <div>
                  <span data-decision={exception.decision}>
                    {decisionLabel(exception.decision)}
                  </span>
                  <strong>{exception.action}</strong>
                  <p>{exception.scope}</p>
                  <dl>
                    <div>
                      <dt>Source</dt>
                      <dd>{exception.source}</dd>
                    </div>
                    <div>
                      <dt>Duration</dt>
                      <dd>{exception.duration}</dd>
                    </div>
                  </dl>
                </div>
                <Button
                  aria-label={`Revoke exception for ${exception.action}`}
                  isDisabled={busy}
                  onPress={() => onRevoke(exception.exceptionId)}
                  variant="danger"
                >
                  {revoking ? "Revoking…" : "Revoke"}
                </Button>
              </article>
            );
          })}
        </div>
      )}
    </section>
  );
}

function visiblePolicyGroups(
  activeGroupId: string,
  normalizedQuery: string,
): readonly PolicyGroupDefinition[] {
  if (normalizedQuery.length === 0) {
    return POLICY_GROUPS.filter((group) => group.id === activeGroupId);
  }
  return POLICY_GROUPS.map((group) => ({
    ...group,
    items: group.items.filter((item) =>
      `${group.label} ${item.label} ${item.key} ${item.description}`
        .toLocaleLowerCase()
        .includes(normalizedQuery),
    ),
  })).filter((group) => group.items.length > 0);
}

function actionFailure(error: unknown, fallback: string): string {
  return error instanceof Error && error.message.trim().length > 0
    ? error.message
    : fallback;
}

function decisionLabel(decision: "allow" | "ask" | "deny"): string {
  return `${decision.slice(0, 1).toLocaleUpperCase()}${decision.slice(1)}`;
}
