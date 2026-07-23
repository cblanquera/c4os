import { useEffect, useId, useRef, useState, type ReactNode } from "react";

import { Button, Notice, StatusRegion } from "../../../components/accessible";
import type {
  ConfigurableProviderKind,
  ProviderAuthentication,
  ProviderModel,
} from "../../../platform/provider-service";
import {
  PROVIDER_TYPE_OPTIONS,
  type ProviderFormField,
} from "./provider-profile";
import type { ProviderProfileController } from "./use-provider-profile";

import "./provider-settings.css";

export type ProviderProfileFormMode = "onboarding" | "settings";

export type ProviderProfileFormProps = {
  readonly controller: ProviderProfileController;
  readonly mode: ProviderProfileFormMode;
  readonly onDismiss?: () => void;
};

/** Renders the provider fields shared by onboarding and Settings dialogs. */
export function ProviderProfileForm({
  controller,
  mode,
  onDismiss,
}: ProviderProfileFormProps) {
  const dismissRef = useRef(onDismiss);
  const [revealsSecret, setRevealsSecret] = useState(false);
  const compatibleTitleId = useId();
  const defaultsTitleId = useId();
  const modelGroupName = useId();
  const showsCompatibleFields = controller.draft.kind === "custom";
  const requiresSecret =
    !showsCompatibleFields || controller.draft.authenticationType !== "none";

  useEffect(() => {
    dismissRef.current = onDismiss;
  }, [onDismiss]);

  useEffect(
    () => () => {
      dismissRef.current?.();
    },
    [],
  );

  return (
    <form
      aria-label={
        mode === "onboarding" ? "Provider onboarding" : "Provider profile"
      }
      className="provider-form"
      onSubmit={(event) => event.preventDefault()}
    >
      <div className="provider-form__grid">
        <ProviderField label="Provider type">
          {(fieldProps) => (
            <select
              {...fieldProps}
              disabled={controller.isBusy}
              onChange={(event) =>
                controller.changeKind(
                  event.currentTarget.value as ConfigurableProviderKind,
                )
              }
              value={controller.draft.kind}
            >
              {PROVIDER_TYPE_OPTIONS.map((option) => (
                <option key={option.kind} value={option.kind}>
                  {option.label}
                </option>
              ))}
            </select>
          )}
        </ProviderField>

        <ProviderField
          error={controller.errors.displayName}
          field="displayName"
          label="Profile label"
          showError={controller.showValidation}
        >
          {(errorProps) => (
            <input
              {...errorProps}
              autoComplete="off"
              disabled={controller.isBusy}
              onChange={(event) =>
                controller.changeValue("displayName", event.currentTarget.value)
              }
              required
              value={controller.draft.displayName}
            />
          )}
        </ProviderField>
      </div>

      {showsCompatibleFields ? (
        <section
          aria-labelledby={compatibleTitleId}
          className="provider-form__section"
        >
          <h3 id={compatibleTitleId}>OpenAI Compatible endpoint</h3>
          <div className="provider-form__grid">
            <ProviderField
              error={controller.errors.baseUrl}
              field="baseUrl"
              label="API base URL"
              showError={controller.showValidation}
            >
              {(errorProps) => (
                <input
                  {...errorProps}
                  disabled={controller.isBusy}
                  onChange={(event) =>
                    controller.changeValue("baseUrl", event.currentTarget.value)
                  }
                  placeholder="https://provider.example/v1"
                  required
                  type="url"
                  value={controller.draft.baseUrl}
                />
              )}
            </ProviderField>

            <ProviderField label="Authentication">
              {(fieldProps) => (
                <select
                  {...fieldProps}
                  disabled={controller.isBusy}
                  onChange={(event) =>
                    controller.changeAuthentication(
                      event.currentTarget
                        .value as ProviderAuthentication["type"],
                    )
                  }
                  value={controller.draft.authenticationType}
                >
                  <option value="bearer">Bearer token</option>
                  <option value="apiKeyHeader">API key header</option>
                  <option value="none">None</option>
                </select>
              )}
            </ProviderField>

            {controller.draft.authenticationType === "apiKeyHeader" ? (
              <ProviderField
                error={controller.errors.headerName}
                field="headerName"
                label="API key header name"
                showError={controller.showValidation}
              >
                {(errorProps) => (
                  <input
                    {...errorProps}
                    disabled={controller.isBusy}
                    onChange={(event) =>
                      controller.changeValue(
                        "headerName",
                        event.currentTarget.value,
                      )
                    }
                    required
                    value={controller.draft.headerName}
                  />
                )}
              </ProviderField>
            ) : null}
          </div>

          <ProviderField
            error={controller.errors.headersJson}
            field="headersJson"
            label="Additional literal headers (JSON)"
            showError={controller.showValidation}
          >
            {(errorProps) => (
              <textarea
                {...errorProps}
                disabled={controller.isBusy}
                onChange={(event) =>
                  controller.changeValue(
                    "headersJson",
                    event.currentTarget.value,
                  )
                }
                rows={5}
                spellCheck={false}
                value={controller.draft.headersJson}
              />
            )}
          </ProviderField>
        </section>
      ) : null}

      {requiresSecret ? (
        <ProviderField
          error={controller.errors.secret}
          field="secret"
          label="API key"
          showError={controller.showValidation}
        >
          {(errorProps) => (
            <div className="provider-form__secret">
              <input
                {...errorProps}
                autoComplete="off"
                disabled={controller.isBusy}
                onChange={(event) =>
                  controller.changeSecret(event.currentTarget.value)
                }
                placeholder={
                  controller.draft.hasCredential
                    ? "Leave blank to keep the existing key"
                    : "Enter API key"
                }
                required={!controller.draft.hasCredential}
                type={revealsSecret ? "text" : "password"}
                value={controller.draft.secret}
              />
              <Button
                aria-label={revealsSecret ? "Hide API key" : "Reveal API key"}
                isDisabled={controller.isBusy}
                onPress={() => setRevealsSecret((current) => !current)}
                variant="quiet"
              >
                {revealsSecret ? "Hide" : "Reveal"}
              </Button>
            </div>
          )}
        </ProviderField>
      ) : null}

      {controller.showValidation &&
      Object.keys(controller.errors).length > 0 ? (
        <Notice title="Complete the visible provider fields" tone="warning">
          Only the fields shown for this provider type are required.
        </Notice>
      ) : null}

      <StatusRegion
        aria-busy={controller.operation === "testing"}
        className="provider-connection-status"
        data-state={controller.connection.state}
      >
        <strong>{controller.connection.title}</strong>
        <span>{controller.connection.detail}</span>
      </StatusRegion>

      {controller.models.length > 0 ? (
        <ProviderModelChoices
          controller={controller}
          groupName={modelGroupName}
        />
      ) : null}

      {mode === "onboarding" && controller.connection.state === "success" ? (
        <section
          aria-labelledby={defaultsTitleId}
          className="provider-defaults"
        >
          <h3 id={defaultsTitleId}>Initial defaults</h3>
          <p>
            Confirm these revisable defaults. The first valid Chat submission
            binds them to that Chat.
          </p>
          <dl>
            <div>
              <dt>Model</dt>
              <dd>
                {controller.models.find(
                  ({ modelId }) => modelId === controller.selectedModelId,
                )?.displayName ?? "Choose a production-ready model"}
              </dd>
            </div>
            <div>
              <dt>Runtime</dt>
              <dd>OpenCode</dd>
            </div>
            <div>
              <dt>Environment</dt>
              <dd>Local</dd>
            </div>
          </dl>
        </section>
      ) : null}
    </form>
  );
}

/** Renders one labelled field with a programmatically connected error. */
function ProviderField({
  children,
  error,
  field,
  label,
  showError = false,
}: {
  readonly children: (props: {
    readonly "aria-describedby"?: string;
    readonly "aria-invalid"?: true;
    readonly id: string;
  }) => ReactNode;
  readonly error?: string | undefined;
  readonly field?: ProviderFormField;
  readonly label: string;
  readonly showError?: boolean;
}) {
  const controlId = useId();
  const generatedErrorId = useId();
  const errorId = field ? `${generatedErrorId}-${field}` : undefined;
  const errorProps =
    showError && error && errorId
      ? ({
          "aria-describedby": errorId,
          "aria-invalid": true,
          id: controlId,
        } as const)
      : { id: controlId };
  return (
    <div className="provider-field">
      <label htmlFor={controlId}>{label}</label>
      <span className="provider-field__control">{children(errorProps)}</span>
      {showError && error && errorId ? (
        <small id={errorId} role="alert">
          {error}
        </small>
      ) : null}
    </div>
  );
}

/** Renders route-specific model evidence with one explicit selection. */
function ProviderModelChoices({
  controller,
  groupName,
}: {
  readonly controller: ProviderProfileController;
  readonly groupName: string;
}) {
  return (
    <fieldset className="provider-models">
      <legend>Model for new Chats</legend>
      <div className="provider-models__list">
        {controller.models.map((model) => (
          <ProviderModelChoice
            controller={controller}
            groupName={groupName}
            key={model.modelId}
            model={model}
          />
        ))}
      </div>
    </fieldset>
  );
}

/** Renders one model with availability, context, and recommendation evidence. */
function ProviderModelChoice({
  controller,
  groupName,
  model,
}: {
  readonly controller: ProviderProfileController;
  readonly groupName: string;
  readonly model: ProviderModel;
}) {
  const isRecommended = model.modelId === controller.recommendedModelId;
  return (
    <label className="provider-model" data-ready={model.productionReady}>
      <input
        checked={model.modelId === controller.selectedModelId}
        disabled={
          controller.isBusy ||
          controller.connection.state !== "success" ||
          !model.productionReady
        }
        name={groupName}
        onChange={() => void controller.selectModel(model.modelId)}
        type="radio"
      />
      <span className="provider-model__copy">
        <strong>{model.displayName}</strong>
        <small>
          {model.modelId} · {model.availability}
          {model.contextTokens === null
            ? ""
            : ` · ${model.contextTokens.toLocaleString()} context tokens`}
        </small>
      </span>
      {isRecommended ? (
        <span className="provider-model__recommended">Recommended</span>
      ) : null}
    </label>
  );
}
