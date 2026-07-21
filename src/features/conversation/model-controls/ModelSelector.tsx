import type { Key, Selection } from "react-aria-components";
import { useLayoutEffect, useMemo, useRef, useState } from "react";
import {
  Button,
  Dialog,
  DialogTrigger,
  Heading,
  ListBox,
  ListBoxItem,
  Popover,
} from "react-aria-components";

import { Icon } from "../../../components/accessible";
import type {
  ModelCapability,
  ModelCapabilityFilter,
  ModelControlModel,
  ModelControlProvider,
} from "./types";

export interface ModelSelectorProps {
  readonly isDisabled?: boolean;
  readonly models: readonly ModelControlModel[];
  readonly onModelSelect: (model: ModelControlModel) => void;
  readonly providers: readonly ModelControlProvider[];
}

const FILTERS: readonly ModelCapabilityFilter[] = [
  "all",
  "vision",
  "tools",
  "reasoning",
  "audio",
];

const CAPABILITY_LABELS: Record<ModelCapability, string> = {
  vision: "Vision",
  tools: "Tools",
  reasoning: "Reasoning",
  audio: "Audio",
};

/** Renders a controlled model picker with non-committal provider browsing. */
export function ModelSelector({
  isDisabled = false,
  models,
  onModelSelect,
  providers,
}: ModelSelectorProps) {
  const triggerRef = useRef<HTMLButtonElement>(null);
  const shouldRestoreFocus = useRef(false);
  const [isOpen, setIsOpen] = useState(false);
  const [showsProviders, setShowsProviders] = useState(false);
  const [filter, setFilter] = useState<ModelCapabilityFilter>("all");
  const selectedModel = models.find((model) => model.isSelected) ?? null;
  const activeProviderId =
    selectedModel?.providerId ?? providers.at(0)?.id ?? "";
  const [browsedProviderId, setBrowsedProviderId] = useState(activeProviderId);
  const effectiveProviderId = providers.some(
    (provider) => provider.id === browsedProviderId,
  )
    ? browsedProviderId
    : activeProviderId;
  const browsedProvider =
    providers.find((provider) => provider.id === effectiveProviderId) ?? null;
  const visibleModels = useMemo(
    () =>
      models.filter(
        (model) =>
          model.providerId === effectiveProviderId &&
          (filter === "all" || model.capabilities.includes(filter)),
      ),
    [effectiveProviderId, filter, models],
  );

  /** Resets every opening to the active model's provider and restores focus on close. */
  const handleOpenChange = (nextIsOpen: boolean) => {
    if (nextIsOpen) {
      setBrowsedProviderId(activeProviderId);
      setFilter("all");
      setShowsProviders(false);
    } else {
      shouldRestoreFocus.current = true;
    }

    setIsOpen(nextIsOpen);
  };

  /** Browses one provider without reporting a model selection. */
  const handleProviderAction = (key: Key) => {
    setBrowsedProviderId(String(key));
    setShowsProviders(false);
  };

  useLayoutEffect(() => {
    if (!isOpen && shouldRestoreFocus.current) {
      shouldRestoreFocus.current = false;
      triggerRef.current?.focus();
    }
  }, [isOpen]);

  return (
    <DialogTrigger isOpen={isOpen} onOpenChange={handleOpenChange}>
      <Button
        aria-label="Model"
        className="conversation-model-control__trigger"
        isDisabled={isDisabled || models.length === 0}
        ref={triggerRef}
      >
        <span>{selectedModel?.name ?? "Choose model"}</span>
        <Icon aria-hidden="true" name="chevron-down" size={14} />
      </Button>
      <Popover
        className="conversation-model-control__popover"
        offset={6}
        placement="top start"
      >
        <Dialog className="conversation-model-control__dialog">
          {({ close }) => (
            <>
              <Heading
                className="conversation-model-control__title"
                slot="title"
              >
                Choose model
              </Heading>
              {showsProviders ? (
                <ProviderBrowser
                  activeProviderId={activeProviderId}
                  browsedProviderId={effectiveProviderId}
                  onAction={handleProviderAction}
                  providers={providers}
                />
              ) : (
                <>
                  <Button
                    className="conversation-model-control__provider"
                    onPress={() => setShowsProviders(true)}
                  >
                    <Icon
                      aria-hidden="true"
                      className="conversation-model-control__back"
                      name="chevron-right"
                      size={14}
                    />
                    <span>{browsedProvider?.name ?? "Provider"}</span>
                  </Button>
                  <div
                    aria-label="Filter models"
                    className="conversation-model-control__filters"
                    role="group"
                  >
                    {FILTERS.map((option) => (
                      <Button
                        aria-pressed={filter === option}
                        className="conversation-model-control__filter"
                        key={option}
                        onPress={() => setFilter(option)}
                      >
                        {filterLabel(option)}
                      </Button>
                    ))}
                  </div>
                  {visibleModels.length > 0 ? (
                    <ListBox
                      aria-label={`Models from ${browsedProvider?.name ?? "provider"}`}
                      className="conversation-model-control__models"
                      onSelectionChange={(selection) => {
                        const key = selectedKey(selection);
                        if (key === null) {
                          return;
                        }
                        const model = visibleModels.find(
                          (candidate) => modelKey(candidate) === String(key),
                        );
                        if (model) {
                          onModelSelect(model);
                          close();
                        }
                      }}
                      selectedKeys={
                        selectedModel ? [modelKey(selectedModel)] : []
                      }
                      selectionMode="single"
                    >
                      {visibleModels.map((model) => (
                        <ListBoxItem
                          className="conversation-model-control__model"
                          id={modelKey(model)}
                          isDisabled={!model.isAvailable}
                          key={modelKey(model)}
                          aria-label={`${model.name}, ${modelSummary(model)}`}
                          textValue={model.name}
                        >
                          <span className="conversation-model-control__model-copy">
                            <strong>{model.name}</strong>
                            <small>{modelSummary(model)}</small>
                          </span>
                          {model.isSelected ? (
                            <Icon aria-hidden="true" name="check" size={14} />
                          ) : null}
                        </ListBoxItem>
                      ))}
                    </ListBox>
                  ) : (
                    <p
                      className="conversation-model-control__empty"
                      role="status"
                    >
                      No {filter === "all" ? "available" : filterLabel(filter)}{" "}
                      models from {browsedProvider?.name ?? "this provider"}.
                    </p>
                  )}
                </>
              )}
            </>
          )}
        </Dialog>
      </Popover>
    </DialogTrigger>
  );
}

interface ProviderBrowserProps {
  readonly activeProviderId: string;
  readonly browsedProviderId: string;
  readonly onAction: (key: Key) => void;
  readonly providers: readonly ModelControlProvider[];
}

/** Renders the compact configured-provider chooser inside the model popover. */
function ProviderBrowser({
  activeProviderId,
  browsedProviderId,
  onAction,
  providers,
}: ProviderBrowserProps) {
  return (
    <section aria-label="Configured providers">
      <h3 className="conversation-model-control__providers-title">Providers</h3>
      <ListBox
        aria-label="Configured providers"
        className="conversation-model-control__providers"
        onSelectionChange={(selection) => {
          const key = selectedKey(selection);
          if (key !== null) {
            onAction(key);
          }
        }}
        selectedKeys={[browsedProviderId]}
        selectionMode="single"
      >
        {providers.map((provider) => (
          <ListBoxItem
            className="conversation-model-control__provider-choice"
            id={provider.id}
            key={provider.id}
            aria-label={
              provider.id === activeProviderId
                ? `${provider.name}, active provider`
                : provider.name
            }
            textValue={provider.name}
          >
            <span>{provider.name}</span>
            {provider.id === activeProviderId ? (
              <span className="conversation-model-control__active-provider">
                Active
              </span>
            ) : null}
          </ListBoxItem>
        ))}
      </ListBox>
    </section>
  );
}

/** Returns the first key from a single-selection React Aria change. */
function selectedKey(selection: Selection): Key | null {
  if (selection === "all") {
    return null;
  }

  const nextKey = selection.values().next();
  return nextKey.done ? null : nextKey.value;
}

/** Produces a collision-free renderer key for a provider-scoped model. */
function modelKey(model: ModelControlModel) {
  return JSON.stringify([model.providerId, model.id]);
}

/** Formats one compact capability/context summary for a model row. */
function modelSummary(model: ModelControlModel) {
  const capabilityLabels = model.capabilities.map(
    (capability) => CAPABILITY_LABELS[capability],
  );
  return [
    model.isAvailable ? null : "Unavailable",
    capabilityLabels.length > 0 ? capabilityLabels.join(" · ") : "Text",
    model.contextLabel,
  ]
    .filter(Boolean)
    .join(" · ");
}

/** Formats a capability filter for visible controls and empty-state copy. */
function filterLabel(filter: ModelCapabilityFilter) {
  return filter === "all" ? "All" : CAPABILITY_LABELS[filter];
}
