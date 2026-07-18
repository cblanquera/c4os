import { useMemo, useState } from "react";

import {
  INITIAL_POLICY_EXCEPTIONS,
  POLICY_GROUPS,
  POLICY_OPTIONS,
  emptyPolicyValues,
  type PolicyException,
  type PolicyValue,
} from "./policy-model";

type PolicyView = "categories" | "exceptions";

export function AdvancedPoliciesScreen() {
  const [view, setView] = useState<PolicyView>("categories");
  const [activeGroup, setActiveGroup] = useState(POLICY_GROUPS[0]!.id);
  const [query, setQuery] = useState("");
  const [saved, setSaved] =
    useState<Record<string, PolicyValue>>(emptyPolicyValues);
  const [draft, setDraft] =
    useState<Record<string, PolicyValue>>(emptyPolicyValues);
  const [exceptions, setExceptions] = useState<readonly PolicyException[]>(
    INITIAL_POLICY_EXCEPTIONS,
  );
  const [notice, setNotice] = useState("");

  const dirty = Object.keys(draft).some((key) => draft[key] !== saved[key]);
  const normalizedQuery = query.trim().toLocaleLowerCase();
  const visibleGroups = useMemo(() => {
    if (!normalizedQuery) {
      return POLICY_GROUPS.filter((group) => group.id === activeGroup);
    }
    return POLICY_GROUPS.map((group) => ({
      ...group,
      items: group.items.filter(
        (item) =>
          item.key.toLocaleLowerCase().includes(normalizedQuery) ||
          item.description.toLocaleLowerCase().includes(normalizedQuery) ||
          group.label.toLocaleLowerCase().includes(normalizedQuery),
      ),
    })).filter((group) => group.items.length > 0);
  }, [activeGroup, normalizedQuery]);

  const visibleCount = visibleGroups.reduce(
    (count, group) => count + group.items.length,
    0,
  );
  const title = normalizedQuery
    ? "Search results"
    : (POLICY_GROUPS.find((group) => group.id === activeGroup)?.label ??
      "Policies");

  function setPolicy(key: string, value: PolicyValue) {
    setDraft((current) => ({ ...current, [key]: value }));
    setNotice("");
  }

  function savePolicies() {
    setSaved({ ...draft });
    setNotice("Policies saved. New actions use the updated effective rules.");
  }

  function revertPolicies() {
    setDraft({ ...saved });
    setNotice("Unsaved policy changes reverted.");
  }

  function revokeException(id: string) {
    setExceptions((current) =>
      current.filter((exception) => exception.id !== id),
    );
    setNotice(
      "Exception revoked. Outstanding matching authorizations expire immediately.",
    );
  }

  return (
    <main className="policy-settings" aria-labelledby="advanced-policies-title">
      <header className="policy-settings__header">
        <div>
          <p className="policy-settings__eyebrow">Permissions</p>
          <h1 id="advanced-policies-title">Advanced Policies</h1>
          <p>Set category rules and review concrete saved exceptions.</p>
        </div>
        <div className="policy-settings__actions">
          <button
            type="button"
            className="button button--quiet"
            onClick={revertPolicies}
            disabled={!dirty}
          >
            Revert
          </button>
          <button
            type="button"
            className="button button--primary"
            onClick={savePolicies}
            disabled={!dirty}
          >
            Save Policies
          </button>
        </div>
      </header>

      <div
        className="policy-settings__tabs"
        role="tablist"
        aria-label="Advanced policy views"
      >
        <button
          type="button"
          role="tab"
          aria-selected={view === "categories"}
          onClick={() => setView("categories")}
        >
          Category rules
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={view === "exceptions"}
          onClick={() => setView("exceptions")}
        >
          Exceptions <span>{exceptions.length}</span>
        </button>
      </div>

      <section
        className="policy-settings__guardrail"
        aria-label="Current policy guardrails"
      >
        <strong>
          Current preset: {dirty ? "Custom (unsaved)" : "Approve for me"}
        </strong>
        <p>
          Unknown, credential-revealing, destructive system, and managed actions
          still ask or deny. The active sandbox, trusted roots, and maximum
          authority cannot be bypassed.
        </p>
      </section>

      {notice ? (
        <p className="policy-settings__notice" role="status">
          {notice}
        </p>
      ) : null}

      {view === "categories" ? (
        <>
          <label className="policy-settings__search">
            <span>Search policies</span>
            <input
              type="search"
              value={query}
              placeholder="Search all policies"
              onChange={(event) => setQuery(event.currentTarget.value)}
            />
          </label>
          <div className="policy-settings__browser">
            <nav aria-label="Policy groups" className="policy-settings__groups">
              {POLICY_GROUPS.map((group) => (
                <button
                  type="button"
                  key={group.id}
                  aria-current={
                    !normalizedQuery && activeGroup === group.id
                      ? "page"
                      : undefined
                  }
                  onClick={() => {
                    setQuery("");
                    setActiveGroup(group.id);
                  }}
                >
                  <span>{group.label}</span>
                  <span>{group.items.length}</span>
                </button>
              ))}
            </nav>
            <section className="policy-settings__results" aria-live="polite">
              <header>
                <div>
                  <h2>{title}</h2>
                  <p>
                    Choose a category default. The most restrictive applicable
                    rule wins.
                  </p>
                </div>
                <span className="policy-settings__count">
                  {visibleCount} rules
                </span>
              </header>
              {visibleCount === 0 ? (
                <div className="policy-settings__empty">
                  <strong>No policies found</strong>
                  <span>Try another action, target, or description.</span>
                </div>
              ) : (
                <div className="policy-settings__list">
                  {visibleGroups.flatMap((group) =>
                    group.items.map((item) => (
                      <label key={item.key} className="policy-settings__row">
                        <span>
                          <code>{item.key}</code>
                          <small>
                            {normalizedQuery ? `${group.label} · ` : ""}
                            {item.description}
                          </small>
                        </span>
                        <select
                          aria-label={`${item.key} policy`}
                          value={draft[item.key]}
                          onChange={(event) =>
                            setPolicy(
                              item.key,
                              event.currentTarget.value as PolicyValue,
                            )
                          }
                        >
                          {POLICY_OPTIONS.map((option) => (
                            <option key={option.value} value={option.value}>
                              {option.label}
                            </option>
                          ))}
                        </select>
                      </label>
                    )),
                  )}
                </div>
              )}
            </section>
          </div>
        </>
      ) : (
        <section
          className="policy-settings__exceptions"
          aria-labelledby="saved-exceptions-title"
        >
          <header>
            <div>
              <h2 id="saved-exceptions-title">Saved exceptions</h2>
              <p>Narrow rules created from approval prompts.</p>
            </div>
            <span className="policy-settings__count">
              {exceptions.length} active
            </span>
          </header>
          {exceptions.length === 0 ? (
            <div className="policy-settings__empty">
              <strong>No saved exceptions</strong>
              <span>Category defaults and safety ceilings remain active.</span>
            </div>
          ) : (
            exceptions.map((exception) => (
              <article key={exception.id}>
                <div>
                  <strong>
                    {exception.decision} <code>{exception.action}</code>
                  </strong>
                  <p>
                    {exception.scope} · {exception.source}
                  </p>
                  <small>{exception.duration}</small>
                </div>
                <button
                  type="button"
                  className="button button--quiet"
                  onClick={() => revokeException(exception.id)}
                >
                  Revoke
                </button>
              </article>
            ))
          )}
        </section>
      )}
    </main>
  );
}
