import { useMemo, useState } from "react";

import {
  Button,
  ModalDialog,
  Notice,
  StatusRegion,
  Switch,
} from "../../../components/accessible";
import {
  ExtensionEmpty,
  ExtensionError,
  ExtensionLoading,
} from "./ExtensionState";
import { stateLabel } from "./state-label";
import type {
  SkillSettingsActions,
  SkillSettingsSnapshot,
  SkillView,
} from "./types";

import "./extension-settings.css";

export interface SkillSettingsProps {
  readonly actions: SkillSettingsActions;
  readonly snapshot: SkillSettingsSnapshot;
}

/** Renders metadata-first Skill discovery without treating the renderer as authority. */
export function SkillSettings({ actions, snapshot }: SkillSettingsProps) {
  if (snapshot.status === "loading") {
    return (
      <ExtensionLoading message={snapshot.message} title="Loading skills" />
    );
  }

  if (snapshot.status === "error") {
    return (
      <ExtensionError
        message={snapshot.message}
        onRetry={() => void actions.onRetry()}
        retryable={snapshot.retryable}
        title="Skills are unavailable"
      />
    );
  }

  return <SkillCatalog actions={actions} snapshot={snapshot} />;
}

interface SkillCatalogProps {
  readonly actions: SkillSettingsActions;
  readonly snapshot: Extract<SkillSettingsSnapshot, { status: "ready" }>;
}

function SkillCatalog({ actions, snapshot }: SkillCatalogProps) {
  const [query, setQuery] = useState("");
  const normalizedQuery = query.trim().toLocaleLowerCase();
  const skills = useMemo(
    () =>
      snapshot.skills.filter((skill) =>
        `${skill.name} ${skill.summary} ${skill.sourceQualifiedId}`
          .toLocaleLowerCase()
          .includes(normalizedQuery),
      ),
    [normalizedQuery, snapshot.skills],
  );

  return (
    <div className="extension-settings" data-generation={snapshot.generation}>
      <section
        className="extension-catalog skill-catalog"
        aria-labelledby="skill-discovery-title"
      >
        <div className="extension-section-heading">
          <div>
            <h2 id="skill-discovery-title">Installed skills</h2>
            <p>
              Discovery reads validated metadata only. Instructions load after
              resolution, eligibility, and an explicit request. Precedence is
              Project, Workspace, User, Plugin, then Bundled.
            </p>
          </div>
        </div>

        <StatusRegion
          aria-busy={snapshot.discoveryStatus === "discovering"}
          className="extension-catalog__status"
          data-state={snapshot.discoveryStatus}
        >
          <strong>{stateLabel(snapshot.discoveryStatus)}</strong>
          <span>{snapshot.discoveryDetail}</span>
        </StatusRegion>

        <label className="skill-search">
          <span>Search installed skills</span>
          <input
            onChange={(event) => setQuery(event.currentTarget.value)}
            placeholder="Search name, summary, or qualified identity"
            type="search"
            value={query}
          />
        </label>

        {snapshot.skills.length === 0 ? (
          <ExtensionEmpty
            detail="Install a Plugin that contributes Skills or add a valid user Skill."
            title="No skills discovered"
          />
        ) : skills.length === 0 ? (
          <ExtensionEmpty
            detail="Change the search to review another source-qualified Skill."
            title="No matching skills"
          />
        ) : (
          <div className="skill-list" role="list" aria-label="Installed skills">
            {skills.map((skill) => (
              <SkillRow actions={actions} key={skill.id} skill={skill} />
            ))}
          </div>
        )}
      </section>
    </div>
  );
}

function SkillRow({
  actions,
  skill,
}: {
  readonly actions: SkillSettingsActions;
  readonly skill: SkillView;
}) {
  return (
    <article
      className="skill-row"
      data-state={skill.effectiveState}
      role="listitem"
    >
      <span className="skill-document-mark" aria-hidden="true">
        S
      </span>
      <div className="skill-row__copy">
        <header>
          <h3>{skill.name}</h3>
          <span
            className="extension-state-pill"
            data-state={skill.effectiveState}
          >
            {stateLabel(skill.effectiveState)}
          </span>
        </header>
        <p>{skill.summary}</p>
        <code>{skill.sourceQualifiedId}</code>
        <div className="skill-row__metadata">
          <span>
            Source: {skill.sourceLabel} · precedence {skill.sourcePrecedence}
          </span>
          <span>{skill.eligibilityDetail}</span>
          {skill.isExplicitSelection ? (
            <strong>Explicit selection</strong>
          ) : null}
        </div>
        {skill.frontmatter.state === "invalid" ? (
          <Notice title="Invalid frontmatter" tone="danger">
            {skill.frontmatter.diagnostic}
          </Notice>
        ) : null}
        {skill.collisions.length > 0 ? (
          <Notice title="Name collision" tone="warning">
            {skill.collisions.length + 1} source-qualified Skills share this
            name. Effective resolution is shown above.
          </Notice>
        ) : null}
      </div>
      <div className="skill-row__actions">
        <Switch
          description={
            skill.effectiveState === "active"
              ? "Effective and available for the next eligible turn."
              : "The service still applies eligibility and precedence."
          }
          isDisabled={skill.frontmatter.state === "invalid"}
          isSelected={skill.enabled}
          label={`${skill.name} availability`}
          onChange={(enabled) => void actions.onSetEnabled(skill.id, enabled)}
        />
        <SkillDetails actions={actions} skill={skill} />
      </div>
    </article>
  );
}

function SkillDetails({
  actions,
  skill,
}: {
  readonly actions: SkillSettingsActions;
  readonly skill: SkillView;
}) {
  const canTry =
    skill.effectiveState === "active" && skill.instructions.state === "loaded";

  return (
    <ModalDialog
      renderActions={(close) => (
        <>
          <Button onPress={close}>Done</Button>
          {skill.sourceKind !== "user" ? (
            <Button
              isDisabled={skill.frontmatter.state === "invalid"}
              onPress={() => {
                void actions.onCustomize(skill.id);
                close();
              }}
              variant="secondary"
            >
              Customize as user copy
            </Button>
          ) : null}
          <Button
            isDisabled={!canTry}
            onPress={() => {
              void actions.onTryInChat(skill.id);
              close();
            }}
            variant="primary"
          >
            Try in Chat
          </Button>
          {skill.isInstalled ? (
            <Button
              onPress={() => {
                void actions.onUninstall(skill.id);
                close();
              }}
              variant="danger"
            >
              Uninstall
            </Button>
          ) : null}
        </>
      )}
      title={skill.name}
      triggerLabel="Details"
    >
      <div className="extension-detail skill-detail">
        <p className="extension-detail__summary">{skill.summary}</p>
        <dl className="extension-facts">
          <Fact label="Qualified identity" value={skill.sourceQualifiedId} />
          <Fact label="Version" value={skill.version} />
          <Fact
            label="Source"
            value={`${skill.sourceLabel} (${skill.sourceKind})`}
          />
          <Fact label="Precedence" value={String(skill.sourcePrecedence)} />
          <Fact
            label="Frontmatter"
            value={stateLabel(skill.frontmatter.state)}
          />
          <Fact label="Eligibility" value={skill.eligibilityDetail} />
          <Fact
            label="Effective state"
            value={stateLabel(skill.effectiveState)}
          />
          <Fact
            label="Selection"
            value={
              skill.isExplicitSelection ? "Explicit" : "Precedence resolved"
            }
          />
        </dl>

        <Switch
          description="Changing this requests a Rust-owned lifecycle transition."
          isDisabled={skill.frontmatter.state === "invalid"}
          isSelected={skill.enabled}
          label="Available in Chat"
          onChange={(enabled) => void actions.onSetEnabled(skill.id, enabled)}
        />

        {skill.collisions.length > 0 ? (
          <section aria-labelledby={`skill-${skill.id}-collisions`}>
            <h3 id={`skill-${skill.id}-collisions`}>Collision resolution</h3>
            <p>
              Automatic precedence remains source-qualified. Choose this Skill
              explicitly only when it should override that resolution.
            </p>
            <ul className="skill-collision-list">
              <li>
                <strong>{skill.sourceQualifiedId}</strong>
                <span>Precedence {skill.sourcePrecedence}</span>
              </li>
              {skill.collisions.map((collision) => (
                <li key={collision.sourceQualifiedId}>
                  <strong>{collision.sourceQualifiedId}</strong>
                  <span>
                    {collision.sourceLabel} · precedence{" "}
                    {collision.sourcePrecedence}
                  </span>
                </li>
              ))}
            </ul>
            <Button
              isDisabled={
                skill.isExplicitSelection ||
                skill.frontmatter.state === "invalid"
              }
              onPress={() => void actions.onSelectExplicitly(skill.id)}
            >
              {skill.isExplicitSelection
                ? "Selected explicitly"
                : "Select this Skill explicitly"}
            </Button>
          </section>
        ) : null}

        <SkillInstructions actions={actions} skill={skill} />
      </div>
    </ModalDialog>
  );
}

function SkillInstructions({
  actions,
  skill,
}: {
  readonly actions: SkillSettingsActions;
  readonly skill: SkillView;
}) {
  return (
    <section aria-labelledby={`skill-${skill.id}-instructions`}>
      <h3 id={`skill-${skill.id}-instructions`}>Instructions</h3>
      {skill.instructions.state === "notLoaded" ? (
        <>
          <p>
            Instructions and resources have not been loaded. Discovery used
            metadata only.
          </p>
          <Button
            isDisabled={
              skill.effectiveState !== "active" ||
              skill.frontmatter.state === "invalid"
            }
            onPress={() => void actions.onLoadInstructions(skill.id)}
            variant="primary"
          >
            Load instructions
          </Button>
        </>
      ) : null}
      {skill.instructions.state === "loading" ? (
        <StatusRegion>Loading validated instructions…</StatusRegion>
      ) : null}
      {skill.instructions.state === "error" ? (
        <Notice title="Instructions could not be loaded" tone="danger">
          {skill.instructions.error}
        </Notice>
      ) : null}
      {skill.instructions.state === "loaded" ? (
        <pre className="skill-instructions">
          {skill.instructions.instructions}
        </pre>
      ) : null}
    </section>
  );
}

function Fact({
  label,
  value,
}: {
  readonly label: string;
  readonly value: string;
}) {
  return (
    <div>
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  );
}
