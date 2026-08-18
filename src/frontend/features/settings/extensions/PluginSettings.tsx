import { useMemo, useState } from "react";

import {
  Button,
  ModalDialog,
  Notice,
  StatusRegion,
  Tabs,
} from "../../../components/accessible";
import {
  ExtensionEmpty,
  ExtensionError,
  ExtensionLoading,
} from "./ExtensionState";
import { stateLabel } from "./state-label";
import type {
  AddMarketplaceRequest,
  ExtensionActionResult,
  MarketplaceView,
  PluginSettingsActions,
  PluginSettingsSnapshot,
  PluginView,
} from "./types";

import "./extension-settings.css";

export interface PluginSettingsProps {
  readonly actions: PluginSettingsActions;
  readonly snapshot: PluginSettingsSnapshot;
}

/** Renders the controlled Plugins Settings projection owned by ExtensionService. */
export function PluginSettings({ actions, snapshot }: PluginSettingsProps) {
  if (snapshot.status === "loading") {
    return (
      <ExtensionLoading message={snapshot.message} title="Loading plugins" />
    );
  }

  if (snapshot.status === "error") {
    return (
      <ExtensionError
        message={snapshot.message}
        onRetry={() => void actions.onRetry()}
        retryable={snapshot.retryable}
        title="Plugins are unavailable"
      />
    );
  }

  return <PluginCatalog actions={actions} snapshot={snapshot} />;
}

interface PluginCatalogProps {
  readonly actions: PluginSettingsActions;
  readonly snapshot: Extract<PluginSettingsSnapshot, { status: "ready" }>;
}

function PluginCatalog({ actions, snapshot }: PluginCatalogProps) {
  const [marketplaceId, setMarketplaceId] = useState("all");
  const [query, setQuery] = useState("");
  const normalizedQuery = query.trim().toLocaleLowerCase();
  const installed = snapshot.plugins.filter((plugin) => plugin.isInstalled);
  const directory = useMemo(
    () =>
      snapshot.plugins.filter((plugin) => {
        const matchesMarketplace =
          marketplaceId === "all" || plugin.marketplaceId === marketplaceId;
        const matchesQuery =
          normalizedQuery.length === 0 ||
          `${plugin.name} ${plugin.publisher} ${plugin.summary}`
            .toLocaleLowerCase()
            .includes(normalizedQuery);
        return matchesMarketplace && matchesQuery;
      }),
    [marketplaceId, normalizedQuery, snapshot.plugins],
  );

  return (
    <div className="extension-settings" data-generation={snapshot.generation}>
      <header className="extension-section-heading">
        <div>
          <h2>Plugins</h2>
          <p>Manage installed Plugins and browse configured marketplaces.</p>
        </div>
      </header>

      <Tabs
        label="Plugin catalog sections"
        tabs={[
          {
            id: "installed",
            label: `Installed (${installed.length})`,
            content: (
              <PluginGrid
                actions={actions}
                emptyDetail="Install a verified package from Directory. New installs remain disabled until you enable them."
                emptyTitle="No plugins installed"
                plugins={installed}
              />
            ),
          },
          {
            id: "directory",
            label: "Directory",
            content: (
              <div className="extension-directory">
                <section
                  className="extension-catalog"
                  aria-labelledby="plugin-marketplaces-title"
                >
                  <div className="extension-section-heading">
                    <div>
                      <h3 id="plugin-marketplaces-title">Marketplaces</h3>
                      <p>Catalog sources available to this directory.</p>
                    </div>
                    <div className="extension-heading-actions">
                      <Button
                        isDisabled={snapshot.catalogStatus === "checking"}
                        onPress={() => void actions.onRefreshCatalog()}
                        variant="secondary"
                      >
                        {snapshot.catalogStatus === "checking"
                          ? "Checking…"
                          : "Refresh catalog"}
                      </Button>
                      <AddMarketplaceDialog onAdd={actions.onAddMarketplace} />
                    </div>
                  </div>

                  <StatusRegion
                    aria-busy={snapshot.catalogStatus === "checking"}
                    className="extension-catalog__status"
                    data-state={snapshot.catalogStatus}
                  >
                    <strong>{stateLabel(snapshot.catalogStatus)}</strong>
                    <span>{snapshot.catalogDetail}</span>
                  </StatusRegion>

                  {snapshot.marketplaces.length === 0 ? (
                    <ExtensionEmpty
                      detail="Add a local or Git source to discover Plugin packages."
                      title="No marketplaces added"
                    />
                  ) : (
                    <div
                      className="marketplace-list"
                      role="list"
                      aria-label="Configured marketplaces"
                    >
                      {snapshot.marketplaces.map((marketplace) => (
                        <MarketplaceRow
                          key={marketplace.id}
                          marketplace={marketplace}
                        />
                      ))}
                    </div>
                  )}
                </section>
                <div className="extension-directory-toolbar">
                  <label>
                    <span>Search plugins</span>
                    <input
                      onChange={(event) => setQuery(event.currentTarget.value)}
                      placeholder="Search name, publisher, or capability"
                      type="search"
                      value={query}
                    />
                  </label>
                  <label>
                    <span>Marketplace</span>
                    <select
                      onChange={(event) =>
                        setMarketplaceId(event.currentTarget.value)
                      }
                      value={marketplaceId}
                    >
                      <option value="all">All marketplaces</option>
                      {snapshot.marketplaces.map((marketplace) => (
                        <option key={marketplace.id} value={marketplace.id}>
                          {marketplace.label}
                        </option>
                      ))}
                    </select>
                  </label>
                </div>
                <PluginGrid
                  actions={actions}
                  emptyDetail="Change the search or marketplace filter, or refresh the catalog."
                  emptyTitle="No matching plugins"
                  plugins={directory}
                />
              </div>
            ),
          },
        ]}
      />
    </div>
  );
}

function MarketplaceRow({
  marketplace,
}: {
  readonly marketplace: MarketplaceView;
}) {
  return (
    <article className="marketplace-row" role="listitem">
      <span className="extension-identity" aria-hidden="true">
        M
      </span>
      <div className="marketplace-row__copy">
        <strong>{marketplace.label}</strong>
        <span>{marketplace.source}</span>
        {marketplace.resolvedCommit ? (
          <small>Resolved commit: {marketplace.resolvedCommit}</small>
        ) : null}
        <small>{marketplace.detail}</small>
      </div>
      <dl>
        <div>
          <dt>Status</dt>
          <dd data-state={marketplace.status}>
            {stateLabel(marketplace.status)}
          </dd>
        </div>
        <div>
          <dt>Packages</dt>
          <dd>{marketplace.packageCount}</dd>
        </div>
        <div>
          <dt>Trusted origin</dt>
          <dd>{marketplace.trustedOrigin}</dd>
        </div>
      </dl>
    </article>
  );
}

interface PluginGridProps {
  readonly actions: PluginSettingsActions;
  readonly emptyDetail: string;
  readonly emptyTitle: string;
  readonly plugins: readonly PluginView[];
}

function PluginGrid({
  actions,
  emptyDetail,
  emptyTitle,
  plugins,
}: PluginGridProps) {
  if (plugins.length === 0) {
    return <ExtensionEmpty detail={emptyDetail} title={emptyTitle} />;
  }

  return (
    <div className="plugin-grid" role="list" aria-label="Plugins">
      {plugins.map((plugin) => (
        <PluginCard actions={actions} key={plugin.id} plugin={plugin} />
      ))}
    </div>
  );
}

function PluginCard({
  actions,
  plugin,
}: {
  readonly actions: PluginSettingsActions;
  readonly plugin: PluginView;
}) {
  const notice = pluginNotice(plugin);
  const isExecuting = isPluginExecuting(plugin);

  return (
    <article
      aria-busy={isExecuting}
      className="plugin-card"
      data-state={plugin.lifecycle}
      role="listitem"
    >
      <header>
        <span className="extension-identity" aria-hidden="true">
          {plugin.name.slice(0, 1).toLocaleUpperCase()}
        </span>
        <div>
          <h3>{plugin.name}</h3>
          <p>
            {plugin.publisher} · {plugin.version}
          </p>
        </div>
        <span className="extension-state-pill" data-state={plugin.lifecycle}>
          {plugin.operation
            ? stateLabel(plugin.operation)
            : stateLabel(plugin.lifecycle)}
        </span>
      </header>
      <p className="plugin-card__summary">{plugin.summary}</p>
      <div className="extension-chip-list" aria-label="Capabilities">
        {plugin.capabilities.map((capability) => (
          <span key={capability}>{capability}</span>
        ))}
      </div>
      {notice ? (
        <Notice title={notice.title} tone={notice.tone}>
          {notice.detail}
        </Notice>
      ) : null}
      <footer>
        <PluginDetails actions={actions} plugin={plugin} />
        <PluginPrimaryAction actions={actions} plugin={plugin} />
      </footer>
    </article>
  );
}

function PluginPrimaryAction({
  actions,
  plugin,
}: {
  readonly actions: PluginSettingsActions;
  readonly plugin: PluginView;
}) {
  const isBusy = isPluginMutationBlocked(plugin);
  const isDisableBlocked = plugin.operation !== null;

  if (!plugin.isInstalled) {
    return (
      <Button
        isDisabled={
          isBusy ||
          plugin.trustState === "invalid" ||
          plugin.trustState === "revoked"
        }
        onPress={() => void actions.onInstallDisabled(plugin.id)}
        variant="primary"
      >
        Install disabled
      </Button>
    );
  }

  if (
    plugin.lifecycle === "enabled" ||
    plugin.lifecycle === "updateStaged" ||
    isPluginExecuting(plugin)
  ) {
    return (
      <Button
        isDisabled={isDisableBlocked}
        onPress={() => void actions.onDisable(plugin.id)}
      >
        Disable
      </Button>
    );
  }

  if (plugin.lifecycle === "revoked") {
    return (
      <Button
        isDisabled={isBusy}
        onPress={() => void actions.onUninstall(plugin.id)}
        variant="danger"
      >
        Uninstall
      </Button>
    );
  }

  if (plugin.lifecycle === "failed" && plugin.rollbackAvailable) {
    return (
      <Button
        isDisabled={isBusy}
        onPress={() => void actions.onRollback(plugin.id)}
        variant="primary"
      >
        Roll back to last-known-good
      </Button>
    );
  }

  if (plugin.lifecycle === "failed") {
    return (
      <Button
        isDisabled={isBusy}
        onPress={() => void actions.onUninstall(plugin.id)}
        variant="danger"
      >
        Uninstall failed package
      </Button>
    );
  }

  return (
    <Button
      isDisabled={isBusy || plugin.trustState !== "verified"}
      onPress={() => void actions.onEnableForNextTurn(plugin.id)}
      variant="primary"
    >
      Enable for next turn
    </Button>
  );
}

function PluginDetails({
  actions,
  plugin,
}: {
  readonly actions: PluginSettingsActions;
  readonly plugin: PluginView;
}) {
  const isMutationBlocked = isPluginMutationBlocked(plugin);

  return (
    <ModalDialog
      renderActions={(close) => (
        <>
          <Button onPress={close}>Done</Button>
          {plugin.isInstalled ? (
            <Button
              isDisabled={isMutationBlocked}
              onPress={() => {
                void actions.onUninstall(plugin.id);
                close();
              }}
              variant="danger"
            >
              Uninstall
            </Button>
          ) : null}
        </>
      )}
      title={plugin.name}
      triggerLabel="Details"
    >
      <div className="extension-detail">
        <p className="extension-detail__summary">{plugin.summary}</p>
        <dl className="extension-facts">
          <Fact label="Publisher" value={plugin.publisher} />
          <Fact label="Version" value={plugin.version} />
          <Fact
            label="Lifecycle"
            value={stateLabel(plugin.operation ?? plugin.lifecycle)}
          />
          <Fact label="Package kind" value={stateLabel(plugin.packageKind)} />
          <Fact label="Source" value={plugin.source} />
          <Fact label="Trust" value={stateLabel(plugin.trustState)} />
          <Fact label="Compatibility" value={plugin.compatibility} />
          <Fact
            label="Recovery detail"
            value={plugin.failureCode ?? "No recovery event"}
          />
          <Fact
            label="Origin signature"
            value={stateLabel(plugin.signature.originSignature)}
          />
          <Fact label="Origin key ID" value={plugin.signature.originKeyId} />
          <Fact
            label="Content signature"
            value={stateLabel(plugin.signature.contentSignature)}
          />
          <Fact label="Content key ID" value={plugin.signature.contentKeyId} />
          <Fact
            label="Signature checked"
            value={plugin.signature.verifiedAt ?? "Not verified"}
          />
          <Fact
            label="Last-known-good"
            value={plugin.lastKnownGoodVersion ?? "Not established"}
          />
        </dl>

        <section aria-labelledby={`plugin-${plugin.id}-capabilities`}>
          <h3 id={`plugin-${plugin.id}-capabilities`}>Capabilities</h3>
          <ul>
            {plugin.capabilities.map((capability) => (
              <li key={capability}>{capability}</li>
            ))}
          </ul>
        </section>

        <PluginContributions plugin={plugin} />

        <section aria-labelledby={`plugin-${plugin.id}-links`}>
          <h3 id={`plugin-${plugin.id}-links`}>Publisher policies</h3>
          <ul className="extension-link-list">
            <PublisherLink
              kind="website"
              label="Website"
              onOpen={() => actions.onOpenPublisherLink(plugin.id, "website")}
              url={plugin.websiteUrl}
            />
            <PublisherLink
              kind="terms"
              label="Terms"
              onOpen={() => actions.onOpenPublisherLink(plugin.id, "terms")}
              url={plugin.termsUrl}
            />
            <PublisherLink
              kind="privacy"
              label="Privacy Policy"
              onOpen={() => actions.onOpenPublisherLink(plugin.id, "privacy")}
              url={plugin.privacyPolicyUrl}
            />
          </ul>
        </section>

        <PluginHooks actions={actions} plugin={plugin} />
        <PluginUpdate actions={actions} plugin={plugin} />
      </div>
    </ModalDialog>
  );
}

function PluginContributions({ plugin }: { readonly plugin: PluginView }) {
  return (
    <section aria-labelledby={`plugin-${plugin.id}-contributions`}>
      <h3 id={`plugin-${plugin.id}-contributions`}>
        Declarative contributions
      </h3>
      {plugin.settings.length === 0 &&
      plugin.apps.length === 0 &&
      plugin.mcpServers.length === 0 ? (
        <p>This package declares no host-rendered contributions.</p>
      ) : (
        <>
          {plugin.settings.length > 0 ? (
            <div>
              <h4>Settings schema</h4>
              <ul>
                {plugin.settings.map((setting) => (
                  <li key={setting.id}>
                    <strong>{setting.label}</strong> ·{" "}
                    {stateLabel(setting.kind)}
                    {setting.required ? " · required" : ""}
                    <br />
                    <span>{setting.description}</span>
                    {setting.choices.length > 0 ? (
                      <small> Choices: {setting.choices.join(", ")}</small>
                    ) : null}
                  </li>
                ))}
              </ul>
              <p>
                Credential references are opaque C4OS vault identities. Package
                metadata never contains raw secret values.
              </p>
            </div>
          ) : null}
          {plugin.apps.length > 0 ? (
            <div>
              <h4>Apps</h4>
              <ul>
                {plugin.apps.map((app) => (
                  <li key={app.id}>
                    <strong>{app.title}</strong> — {app.summary}
                  </li>
                ))}
              </ul>
            </div>
          ) : null}
          {plugin.mcpServers.length > 0 ? (
            <div>
              <h4>MCP declarations</h4>
              <ul>
                {plugin.mcpServers.map((server) => (
                  <li key={server.id}>
                    <strong>{server.name}</strong> ·{" "}
                    {stateLabel(server.transport)}
                  </li>
                ))}
              </ul>
            </div>
          ) : null}
        </>
      )}
    </section>
  );
}

function PluginHooks({
  actions,
  plugin,
}: {
  readonly actions: PluginSettingsActions;
  readonly plugin: PluginView;
}) {
  return (
    <section aria-labelledby={`plugin-${plugin.id}-hooks`}>
      <h3 id={`plugin-${plugin.id}-hooks`}>Reviewed hooks</h3>
      {plugin.hooks.length === 0 ? (
        <p>This Plugin declares no executable hooks.</p>
      ) : (
        <div className="extension-hook-list">
          {plugin.hooks.map((hook) => {
            const invocationArguments = hookInvocationArguments(hook);

            return (
              <article key={hook.id}>
                <header>
                  <strong>{hook.name}</strong>
                  <span data-state={hook.status}>
                    {stateLabel(hook.status)}
                  </span>
                </header>
                <p>{hook.review}</p>
                <small>
                  Grants:{" "}
                  {hook.grants.length > 0 ? hook.grants.join(", ") : "none"}
                </small>
                {invocationArguments ? (
                  <small aria-label={`Invocation arguments for ${hook.name}`}>
                    Invocation arguments:{" "}
                    <code>{JSON.stringify(invocationArguments)}</code>
                  </small>
                ) : null}
                {hook.lastResult ? (
                  <small>Last result: {hook.lastResult}</small>
                ) : null}
                <Button
                  isDisabled={isPluginMutationBlocked(plugin)}
                  onPress={() => void actions.onReviewHook(plugin.id, hook.id)}
                  variant="secondary"
                >
                  Review hook {hook.name}
                </Button>
              </article>
            );
          })}
        </div>
      )}
    </section>
  );
}

function PluginUpdate({
  actions,
  plugin,
}: {
  readonly actions: PluginSettingsActions;
  readonly plugin: PluginView;
}) {
  if (!plugin.isInstalled) {
    return null;
  }

  const isMutationBlocked = isPluginMutationBlocked(plugin);

  return (
    <section aria-labelledby={`plugin-${plugin.id}-update`}>
      <h3 id={`plugin-${plugin.id}-update`}>Update and recovery</h3>
      <p>
        Current digest: <code>{plugin.update.currentDigest}</code>
      </p>
      <div className="extension-detail__actions">
        {plugin.update.availableVersion ? (
          <Button
            isDisabled={isMutationBlocked}
            onPress={() => void actions.onStageUpdate(plugin.id)}
          >
            Stage {plugin.update.availableVersion}
          </Button>
        ) : null}
        {plugin.update.stagedVersion ? (
          <Button
            isDisabled={isMutationBlocked}
            onPress={() => void actions.onActivateUpdate(plugin.id)}
            variant="primary"
          >
            Activate staged {plugin.update.stagedVersion}
          </Button>
        ) : null}
        <Button
          isDisabled={!plugin.rollbackAvailable || isMutationBlocked}
          onPress={() => void actions.onRollback(plugin.id)}
        >
          Roll back to last-known-good
        </Button>
      </div>
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

function PublisherLink({
  kind,
  label,
  onOpen,
  url,
}: {
  readonly kind: "privacy" | "terms" | "website";
  readonly label: string;
  readonly onOpen: () => ExtensionActionResult;
  readonly url: string | null;
}) {
  return (
    <li data-link-kind={kind}>
      {url ? (
        <>
          <Button onPress={() => void onOpen()} variant="quiet">
            {label}
          </Button>
          <span>{url}</span>
        </>
      ) : (
        <span>{label} not supplied</span>
      )}
    </li>
  );
}

function pluginNotice(plugin: PluginView):
  | {
      readonly detail: string;
      readonly title: string;
      readonly tone: "danger" | "success" | "warning";
    }
  | undefined {
  if (isPluginExecuting(plugin)) {
    return {
      detail:
        "An explicitly reviewed hook is running. Disable remains available immediately; other lifecycle changes and hook review stay unavailable until execution finishes.",
      title: "Reviewed hook executing",
      tone: "warning",
    };
  }
  if (plugin.lifecycle === "revoked") {
    return {
      detail:
        plugin.revocationReason ?? "The trust authority revoked this package.",
      title: "Revoked — new use and hooks are blocked",
      tone: "danger",
    };
  }
  if (plugin.lifecycle === "failed") {
    return {
      detail: plugin.rollbackAvailable
        ? "Activation failed. The last-known-good version remains available."
        : "Activation failed and needs review.",
      title: "Activation failed",
      tone: "danger",
    };
  }
  if (plugin.failureCode === "worker-restart-recovered") {
    return {
      detail:
        "An interrupted reviewed hook was stopped during restart. The Plugin returned to its prior durable lifecycle and no worker authority was restored.",
      title: "Interrupted hook recovered",
      tone: "warning",
    };
  }
  if (plugin.lifecycle === "updateStaged") {
    return {
      detail:
        "The staged package passed verification. Activation is still explicit.",
      title: "Update staged",
      tone: "warning",
    };
  }
  if (plugin.lifecycle === "rolledBack") {
    return {
      detail: "The last-known-good version is installed disabled.",
      title: "Rollback complete",
      tone: "success",
    };
  }
  if (plugin.lifecycle === "quarantined") {
    return {
      detail:
        "Verified bytes remain isolated until installed. Installation starts disabled.",
      title: "Package quarantined",
      tone: "warning",
    };
  }
  return undefined;
}

type PluginHookWithInvocationArguments = PluginView["hooks"][number] & {
  readonly arguments?: readonly string[];
};

/** Returns signed hook invocation arguments when the service supplies them. */
function hookInvocationArguments(
  hook: PluginView["hooks"][number],
): readonly string[] | undefined {
  return (hook as PluginHookWithInvocationArguments).arguments;
}

/** Identifies the service-owned lifecycle while a reviewed hook is running. */
function isPluginExecuting(plugin: PluginView): boolean {
  return (plugin.lifecycle as string) === "executing";
}

/** Prevents lifecycle mutation while another mutation or hook execution runs. */
function isPluginMutationBlocked(plugin: PluginView): boolean {
  return plugin.operation !== null || isPluginExecuting(plugin);
}

function AddMarketplaceDialog({
  onAdd,
}: {
  readonly onAdd: (request: AddMarketplaceRequest) => ExtensionActionResult;
}) {
  const [gitRef, setGitRef] = useState("");
  const [publicKeySha256, setPublicKeySha256] = useState("");
  const [signingKeyId, setSigningKeyId] = useState("");
  const [source, setSource] = useState("");
  const [sparsePaths, setSparsePaths] = useState("");
  const [trustedOrigin, setTrustedOrigin] = useState("");
  const hasRequiredTrustPins =
    source.trim().length > 0 &&
    trustedOrigin.trim().length > 0 &&
    signingKeyId.trim().length > 0 &&
    publicKeySha256.trim().length > 0;
  const request = (): AddMarketplaceRequest => ({
    gitRef: gitRef.trim() || null,
    publicKeySha256: publicKeySha256.trim(),
    signingKeyId: signingKeyId.trim(),
    source: source.trim(),
    sparsePaths: sparsePaths
      .split("\n")
      .map((path) => path.trim())
      .filter(Boolean),
    trustedOrigin: trustedOrigin.trim(),
  });

  return (
    <ModalDialog
      renderActions={(close) => (
        <>
          <Button onPress={close}>Cancel</Button>
          <Button
            isDisabled={!hasRequiredTrustPins}
            onPress={() => {
              void onAdd(request());
              close();
            }}
            variant="primary"
          >
            Add Marketplace
          </Button>
        </>
      )}
      title="Add Marketplace"
      triggerLabel="Add Marketplace"
      triggerVariant="primary"
    >
      <div className="extension-form">
        <label>
          <span>Source</span>
          <input
            onChange={(event) => setSource(event.currentTarget.value)}
            placeholder="Local folder or Git URL"
            required
            type="text"
            value={source}
          />
        </label>
        <p id="marketplace-trust-guidance">
          Pin the expected origin, signing key ID, and public-key SHA-256
          fingerprint. Obtain the fingerprint independently through a trusted
          channel, never from this marketplace source alone.
        </p>
        <label>
          <span>Trusted origin</span>
          <input
            aria-describedby="marketplace-trust-guidance"
            onChange={(event) => setTrustedOrigin(event.currentTarget.value)}
            placeholder="Publisher or organization identity"
            required
            type="text"
            value={trustedOrigin}
          />
        </label>
        <label>
          <span>Signing key ID</span>
          <input
            aria-describedby="marketplace-trust-guidance"
            onChange={(event) => setSigningKeyId(event.currentTarget.value)}
            placeholder="Expected origin signing key ID"
            required
            type="text"
            value={signingKeyId}
          />
        </label>
        <label>
          <span>Public key SHA-256 fingerprint</span>
          <input
            aria-describedby="marketplace-trust-guidance"
            onChange={(event) => setPublicKeySha256(event.currentTarget.value)}
            placeholder="sha256:…"
            required
            type="text"
            value={publicKeySha256}
          />
        </label>
        <label>
          <span>Git ref (optional)</span>
          <input
            onChange={(event) => setGitRef(event.currentTarget.value)}
            placeholder="main, tag, or immutable commit"
            type="text"
            value={gitRef}
          />
        </label>
        <label>
          <span>Sparse paths (optional, one per line)</span>
          <textarea
            onChange={(event) => setSparsePaths(event.currentTarget.value)}
            rows={3}
            value={sparsePaths}
          />
        </label>
        <p>
          C4OS verifies origin, content, manifest, inventory, and compatibility
          before any package can leave quarantine.
        </p>
      </div>
    </ModalDialog>
  );
}
