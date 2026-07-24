import { useMemo, useState, type ReactNode } from "react";

import { Button, ModalDialog, Notice } from "../../../components/accessible";
import {
  MAX_MCP_OUTPUT_BYTES,
  MAX_MCP_TIMEOUT_MS,
  MIN_MCP_OUTPUT_BYTES,
  MIN_MCP_TIMEOUT_MS,
  type McpServerDefinitionInput,
} from "../../../platform/mcp-service";
import type { McpSettingsActions } from "./types";

interface DraftRow {
  readonly id: number;
  readonly name: string;
  readonly referenceKind: "environment" | "vault";
  readonly value: string;
}

interface McpServerDraft {
  readonly arguments: readonly DraftRow[];
  readonly bearerKind: "environment" | "none" | "vault";
  readonly bearerReference: string;
  readonly command: string;
  readonly displayName: string;
  readonly environment: readonly DraftRow[];
  readonly environmentSecrets: readonly DraftRow[];
  readonly executableSha256: string;
  readonly headerCredentials: readonly DraftRow[];
  readonly headers: readonly DraftRow[];
  readonly headerSecrets: readonly DraftRow[];
  readonly maxOutputBytes: string;
  readonly passthrough: readonly DraftRow[];
  readonly projectId: string;
  readonly scopeKind: "application" | "chat" | "project" | "workspace";
  readonly serverId: string;
  readonly sessionId: string;
  readonly timeoutMs: string;
  readonly transportKind: "stdio" | "streamableHttp";
  readonly trustedRoot: string;
  readonly url: string;
  readonly workspaceId: string;
  readonly workingDirectory: "activeProject" | "c4osHome" | "trustedRoot";
}

export interface McpServerDialogProps {
  readonly actions: McpSettingsActions;
  readonly generation: number;
  readonly initialDefinition?: McpServerDefinitionInput;
}

let nextRowIdentity = 0;

function row(
  name = "",
  value = "",
  referenceKind: DraftRow["referenceKind"] = "environment",
): DraftRow {
  nextRowIdentity += 1;
  return { id: nextRowIdentity, name, referenceKind, value };
}

export function McpServerDialog({
  actions,
  generation,
  initialDefinition,
}: McpServerDialogProps) {
  const [draft, setDraft] = useState<McpServerDraft>(() =>
    createDraft(initialDefinition),
  );
  const [submitFailed, setSubmitFailed] = useState(false);
  const errors = useMemo(() => validateDraft(draft), [draft]);
  const editing = initialDefinition !== undefined;
  const title = editing
    ? `Configure ${initialDefinition.displayName}`
    : "Add MCP Server";

  const update = <Key extends keyof McpServerDraft>(
    key: Key,
    value: McpServerDraft[Key],
  ) => setDraft((current) => ({ ...current, [key]: value }));

  return (
    <ModalDialog
      renderActions={(close) => (
        <>
          <Button onPress={close}>Cancel</Button>
          <Button
            isDisabled={errors.length > 0}
            onPress={() => {
              setSubmitFailed(false);
              const definition = toDefinition(draft, generation);
              void Promise.resolve(actions.onSave(definition))
                .then(close)
                .catch(() => setSubmitFailed(true));
            }}
            variant="primary"
          >
            Save Server
          </Button>
        </>
      )}
      title={title}
      triggerLabel={editing ? "Configure" : "Add MCP Server"}
      triggerVariant={editing ? "secondary" : "primary"}
    >
      <form className="mcp-form" onSubmit={(event) => event.preventDefault()}>
        <p className="mcp-form__intro">
          Servers are saved disabled. Test the connection before enabling it for
          a future turn. Secret values stay in the vault or host environment;
          this form stores references only.
        </p>

        {submitFailed ? (
          <Notice title="Server was not saved" tone="danger">
            The draft is still open. Review the page error and try again.
          </Notice>
        ) : null}

        <div className="mcp-form__grid">
          <Field label="Display name">
            <input
              autoFocus
              onChange={(event) =>
                update("displayName", event.currentTarget.value)
              }
              required
              value={draft.displayName}
            />
          </Field>
          <Field label="Server ID">
            <input
              disabled={editing}
              onChange={(event) =>
                update("serverId", event.currentTarget.value)
              }
              pattern="[A-Za-z0-9._:-]+"
              required
              value={draft.serverId}
            />
          </Field>
          <Field label="Scope">
            <select
              onChange={(event) =>
                update(
                  "scopeKind",
                  event.currentTarget.value as McpServerDraft["scopeKind"],
                )
              }
              value={draft.scopeKind}
            >
              <option value="application">Application</option>
              <option value="workspace">Workspace</option>
              <option value="project">Project</option>
              <option value="chat">Chat</option>
            </select>
          </Field>
          <Field label="Transport">
            <select
              onChange={(event) =>
                update(
                  "transportKind",
                  event.currentTarget.value as McpServerDraft["transportKind"],
                )
              }
              value={draft.transportKind}
            >
              <option value="stdio">STDIO</option>
              <option value="streamableHttp">Streamable HTTP</option>
            </select>
          </Field>
        </div>

        {draft.scopeKind !== "application" ? (
          <section aria-labelledby="mcp-scope-fields">
            <h3 id="mcp-scope-fields">Scope identity</h3>
            <div className="mcp-form__grid">
              <Field label="Workspace ID">
                <input
                  onChange={(event) =>
                    update("workspaceId", event.currentTarget.value)
                  }
                  required
                  value={draft.workspaceId}
                />
              </Field>
              {draft.scopeKind === "project" || draft.scopeKind === "chat" ? (
                <Field label="Project ID">
                  <input
                    onChange={(event) =>
                      update("projectId", event.currentTarget.value)
                    }
                    required
                    value={draft.projectId}
                  />
                </Field>
              ) : null}
              {draft.scopeKind === "chat" ? (
                <Field label="Chat session ID">
                  <input
                    onChange={(event) =>
                      update("sessionId", event.currentTarget.value)
                    }
                    required
                    value={draft.sessionId}
                  />
                </Field>
              ) : null}
            </div>
          </section>
        ) : null}

        {draft.transportKind === "stdio" ? (
          <StdioFields draft={draft} update={update} />
        ) : (
          <HttpFields draft={draft} update={update} />
        )}

        <section aria-labelledby="mcp-limits-heading">
          <h3 id="mcp-limits-heading">Limits and trust</h3>
          <div className="mcp-form__grid">
            <Field label="Timeout (milliseconds)">
              <input
                max={MAX_MCP_TIMEOUT_MS}
                min={MIN_MCP_TIMEOUT_MS}
                onChange={(event) =>
                  update("timeoutMs", event.currentTarget.value)
                }
                required
                type="number"
                value={draft.timeoutMs}
              />
            </Field>
            <Field label="Maximum output (bytes)">
              <input
                max={MAX_MCP_OUTPUT_BYTES}
                min={MIN_MCP_OUTPUT_BYTES}
                onChange={(event) =>
                  update("maxOutputBytes", event.currentTarget.value)
                }
                required
                type="number"
                value={draft.maxOutputBytes}
              />
            </Field>
          </div>
          <Notice title="Trust is reviewed after saving" tone="neutral">
            Saving keeps this definition disabled and untrusted. Review the
            exact executable or authenticated endpoint from the server row
            before any connection test can run.
          </Notice>
        </section>

        {errors.length > 0 ? (
          <Notice title="Complete the server configuration" tone="warning">
            <ul className="mcp-form__errors">
              {errors.map((error) => (
                <li key={error}>{error}</li>
              ))}
            </ul>
          </Notice>
        ) : null}
      </form>
    </ModalDialog>
  );
}

type UpdateDraft = <Key extends keyof McpServerDraft>(
  key: Key,
  value: McpServerDraft[Key],
) => void;

function StdioFields({
  draft,
  update,
}: {
  readonly draft: McpServerDraft;
  readonly update: UpdateDraft;
}) {
  return (
    <section aria-labelledby="mcp-stdio-heading">
      <h3 id="mcp-stdio-heading">STDIO process</h3>
      <Field label="Command">
        <input
          onChange={(event) => update("command", event.currentTarget.value)}
          placeholder="/absolute/path/to/mcp-server"
          required
          value={draft.command}
        />
      </Field>
      <Field label="Executable SHA-256">
        <input
          onChange={(event) =>
            update("executableSha256", event.currentTarget.value)
          }
          placeholder="sha256:0123456789abcdef..."
          required
          spellCheck={false}
          value={draft.executableSha256}
        />
      </Field>
      <Repeater
        addLabel="Add argument"
        columns={["Argument"]}
        emptyLabel="No arguments"
        onAdd={() => update("arguments", [...draft.arguments, row()])}
        onChange={(rows) => update("arguments", rows)}
        rows={draft.arguments}
      />
      <Repeater
        addLabel="Add environment variable"
        columns={["Variable", "Literal value (non-secret)"]}
        emptyLabel="No literal environment variables"
        onAdd={() => update("environment", [...draft.environment, row()])}
        onChange={(rows) => update("environment", rows)}
        rows={draft.environment}
        showName
      />
      <Repeater
        addLabel="Add environment passthrough"
        columns={["Variable"]}
        emptyLabel="No environment passthrough"
        onAdd={() => update("passthrough", [...draft.passthrough, row()])}
        onChange={(rows) => update("passthrough", rows)}
        rows={draft.passthrough}
      />
      <SecretRepeater
        addLabel="Add credential reference"
        emptyLabel="No credential-backed environment variables"
        nameLabel="Variable"
        onAdd={() =>
          update("environmentSecrets", [...draft.environmentSecrets, row()])
        }
        onChange={(rows) => update("environmentSecrets", rows)}
        rows={draft.environmentSecrets}
      />
      <div className="mcp-form__grid">
        <Field label="Working directory">
          <select
            onChange={(event) =>
              update(
                "workingDirectory",
                event.currentTarget.value as McpServerDraft["workingDirectory"],
              )
            }
            value={draft.workingDirectory}
          >
            <option value="c4osHome">C4OS Home</option>
            <option value="activeProject">Active project</option>
            <option value="trustedRoot">Trusted root</option>
          </select>
        </Field>
        {draft.workingDirectory === "trustedRoot" ? (
          <Field label="Trusted-root path">
            <input
              onChange={(event) =>
                update("trustedRoot", event.currentTarget.value)
              }
              required
              value={draft.trustedRoot}
            />
          </Field>
        ) : null}
      </div>
    </section>
  );
}

function HttpFields({
  draft,
  update,
}: {
  readonly draft: McpServerDraft;
  readonly update: UpdateDraft;
}) {
  return (
    <section aria-labelledby="mcp-http-heading">
      <h3 id="mcp-http-heading">Streamable HTTP endpoint</h3>
      <Field label="URL">
        <input
          onChange={(event) => update("url", event.currentTarget.value)}
          placeholder="https://mcp.example.com/rpc"
          required
          type="url"
          value={draft.url}
        />
      </Field>
      <div className="mcp-form__grid">
        <Field label="Bearer token source">
          <select
            onChange={(event) =>
              update(
                "bearerKind",
                event.currentTarget.value as McpServerDraft["bearerKind"],
              )
            }
            value={draft.bearerKind}
          >
            <option value="none">None</option>
            <option value="environment">Environment variable</option>
            <option value="vault">Credential reference</option>
          </select>
        </Field>
        {draft.bearerKind !== "none" ? (
          <Field
            label={
              draft.bearerKind === "environment"
                ? "Bearer-token environment variable"
                : "Bearer credential reference"
            }
          >
            <input
              onChange={(event) =>
                update("bearerReference", event.currentTarget.value)
              }
              required
              value={draft.bearerReference}
            />
          </Field>
        ) : null}
      </div>
      <Repeater
        addLabel="Add literal header"
        columns={["Header", "Literal value (non-secret)"]}
        emptyLabel="No literal headers"
        onAdd={() => update("headers", [...draft.headers, row()])}
        onChange={(rows) => update("headers", rows)}
        rows={draft.headers}
        showName
      />
      <Repeater
        addLabel="Add environment-backed header"
        columns={["Header", "Environment variable"]}
        emptyLabel="No environment-backed headers"
        onAdd={() => update("headerSecrets", [...draft.headerSecrets, row()])}
        onChange={(rows) => update("headerSecrets", rows)}
        rows={draft.headerSecrets}
        showName
      />
      <SecretRepeater
        addLabel="Add credential-backed header"
        emptyLabel="No credential-backed headers"
        nameLabel="Header"
        onAdd={() =>
          update("headerCredentials", [...draft.headerCredentials, row()])
        }
        onChange={(rows) => update("headerCredentials", rows)}
        rows={draft.headerCredentials}
      />
    </section>
  );
}

function Field({
  children,
  label,
}: {
  readonly children: ReactNode;
  readonly label: string;
}) {
  return (
    <label className="mcp-field">
      <span>{label}</span>
      {children}
    </label>
  );
}

function Repeater({
  addLabel,
  columns,
  emptyLabel,
  onAdd,
  onChange,
  rows,
  showName = false,
}: {
  readonly addLabel: string;
  readonly columns: readonly string[];
  readonly emptyLabel: string;
  readonly onAdd: () => void;
  readonly onChange: (rows: readonly DraftRow[]) => void;
  readonly rows: readonly DraftRow[];
  readonly showName?: boolean;
}) {
  return (
    <div className="mcp-repeater">
      <div className="mcp-repeater__heading">
        <span>{emptyLabel}</span>
        <Button onPress={onAdd} variant="quiet">
          {addLabel}
        </Button>
      </div>
      {rows.map((item, index) => (
        <div className="mcp-repeater__row" key={item.id}>
          {showName ? (
            <Field label={`${columns[0]} ${index + 1}`}>
              <input
                onChange={(event) =>
                  onChange(
                    rows.map((candidate) =>
                      candidate.id === item.id
                        ? { ...candidate, name: event.currentTarget.value }
                        : candidate,
                    ),
                  )
                }
                value={item.name}
              />
            </Field>
          ) : null}
          <Field label={`${columns[showName ? 1 : 0]} ${index + 1}`}>
            <input
              onChange={(event) =>
                onChange(
                  rows.map((candidate) =>
                    candidate.id === item.id
                      ? { ...candidate, value: event.currentTarget.value }
                      : candidate,
                  ),
                )
              }
              value={item.value}
            />
          </Field>
          <Button
            aria-label={`Remove ${addLabel.toLocaleLowerCase()} ${index + 1}`}
            onPress={() =>
              onChange(rows.filter((candidate) => candidate.id !== item.id))
            }
            variant="quiet"
          >
            Remove
          </Button>
        </div>
      ))}
    </div>
  );
}

function SecretRepeater({
  addLabel,
  emptyLabel,
  nameLabel,
  onAdd,
  onChange,
  rows,
}: {
  readonly addLabel: string;
  readonly emptyLabel: string;
  readonly nameLabel: string;
  readonly onAdd: () => void;
  readonly onChange: (rows: readonly DraftRow[]) => void;
  readonly rows: readonly DraftRow[];
}) {
  return (
    <div className="mcp-repeater">
      <div className="mcp-repeater__heading">
        <span>{emptyLabel}</span>
        <Button onPress={onAdd} variant="quiet">
          {addLabel}
        </Button>
      </div>
      {rows.map((item, index) => (
        <div
          className="mcp-repeater__row mcp-repeater__row--secret"
          key={item.id}
        >
          <Field label={`${nameLabel} ${index + 1}`}>
            <input
              onChange={(event) =>
                onChange(
                  rows.map((candidate) =>
                    candidate.id === item.id
                      ? { ...candidate, name: event.currentTarget.value }
                      : candidate,
                  ),
                )
              }
              value={item.name}
            />
          </Field>
          <Field label={`Reference source ${index + 1}`}>
            <select
              onChange={(event) =>
                onChange(
                  rows.map((candidate) =>
                    candidate.id === item.id
                      ? {
                          ...candidate,
                          referenceKind: event.currentTarget
                            .value as DraftRow["referenceKind"],
                        }
                      : candidate,
                  ),
                )
              }
              value={item.referenceKind}
            >
              <option value="environment">Environment variable</option>
              <option value="vault">Credential reference</option>
            </select>
          </Field>
          <Field label={`Opaque reference ${index + 1}`}>
            <input
              onChange={(event) =>
                onChange(
                  rows.map((candidate) =>
                    candidate.id === item.id
                      ? { ...candidate, value: event.currentTarget.value }
                      : candidate,
                  ),
                )
              }
              value={item.value}
            />
          </Field>
          <Button
            aria-label={`Remove ${addLabel.toLocaleLowerCase()} ${index + 1}`}
            onPress={() =>
              onChange(rows.filter((candidate) => candidate.id !== item.id))
            }
            variant="quiet"
          >
            Remove
          </Button>
        </div>
      ))}
    </div>
  );
}

function createDraft(
  initialDefinition?: McpServerDefinitionInput,
): McpServerDraft {
  const base: McpServerDraft = {
    arguments: [],
    bearerKind: "none",
    bearerReference: "",
    command: "",
    displayName: "",
    environment: [],
    environmentSecrets: [],
    executableSha256: "",
    headerCredentials: [],
    headers: [],
    headerSecrets: [],
    maxOutputBytes: "1048576",
    passthrough: [],
    projectId: "",
    scopeKind: "application",
    serverId: "",
    sessionId: "",
    timeoutMs: "30000",
    transportKind: "stdio",
    trustedRoot: "",
    url: "",
    workspaceId: "",
    workingDirectory: "c4osHome",
  };
  if (!initialDefinition) return base;

  const scope = initialDefinition.scope;
  const scoped = {
    scopeKind: scope.kind,
    workspaceId: scope.kind === "application" ? "" : scope.workspaceId,
    projectId:
      scope.kind === "project" || scope.kind === "chat" ? scope.projectId : "",
    sessionId: scope.kind === "chat" ? scope.sessionId : "",
  };
  const transport = initialDefinition.transport;
  if (transport.kind === "stdio") {
    return {
      ...base,
      ...scoped,
      arguments: transport.arguments.map((value) => row("", value)),
      command: transport.command,
      displayName: initialDefinition.displayName,
      environment: transport.environment
        .filter((binding) => binding.source.kind === "literal")
        .map((binding) =>
          row(
            binding.name,
            binding.source.kind === "literal" ? binding.source.value : "",
          ),
        ),
      environmentSecrets: transport.environment
        .filter((binding) => binding.source.kind === "secret")
        .map((binding) => {
          if (binding.source.kind !== "secret") return row();
          const reference = binding.source.reference;
          return row(
            binding.name,
            reference.kind === "environment"
              ? reference.variable
              : reference.credentialReference,
            reference.kind,
          );
        }),
      executableSha256: transport.executableSha256 ?? "",
      maxOutputBytes: String(initialDefinition.maxOutputBytes),
      passthrough: transport.environment
        .filter((binding) => binding.source.kind === "passthrough")
        .map((binding) => row("", binding.name)),
      serverId: initialDefinition.serverId,
      timeoutMs: String(initialDefinition.timeoutMs),
      transportKind: "stdio",
      trustedRoot:
        transport.workingDirectory.kind === "trustedRoot"
          ? transport.workingDirectory.path
          : "",
      workingDirectory: transport.workingDirectory.kind,
    };
  }

  const bearer = transport.bearer;
  return {
    ...base,
    ...scoped,
    bearerKind: bearer?.kind ?? "none",
    bearerReference:
      bearer === null
        ? ""
        : bearer.kind === "environment"
          ? bearer.variable
          : bearer.credentialReference,
    displayName: initialDefinition.displayName,
    headers: transport.headers
      .filter((header) => header.source.kind === "literal")
      .map((header) =>
        row(
          header.name,
          header.source.kind === "literal" ? header.source.value : "",
        ),
      ),
    headerSecrets: transport.headers
      .filter((header) => header.source.kind === "environment")
      .map((header) => {
        if (header.source.kind === "environment") {
          return row(header.name, header.source.variable, "environment");
        }
        return row();
      }),
    headerCredentials: transport.headers
      .filter((header) => header.source.kind === "secret")
      .map((header) => {
        if (header.source.kind === "secret") {
          const reference = header.source.reference;
          return row(
            header.name,
            reference.kind === "environment"
              ? reference.variable
              : reference.credentialReference,
            reference.kind,
          );
        }
        return row();
      }),
    maxOutputBytes: String(initialDefinition.maxOutputBytes),
    serverId: initialDefinition.serverId,
    timeoutMs: String(initialDefinition.timeoutMs),
    transportKind: "streamableHttp",
    url: transport.url,
  };
}

function validateDraft(draft: McpServerDraft): string[] {
  const errors: string[] = [];
  if (draft.displayName.trim().length === 0)
    errors.push("Enter a display name.");
  if (!isIdentifier(draft.serverId.trim())) {
    errors.push(
      "Use letters, numbers, dots, underscores, colons, or hyphens in the server ID.",
    );
  }
  if (
    draft.scopeKind !== "application" &&
    !isIdentifier(draft.workspaceId.trim())
  ) {
    errors.push("Enter a valid Workspace ID.");
  }
  if (
    (draft.scopeKind === "project" || draft.scopeKind === "chat") &&
    !isIdentifier(draft.projectId.trim())
  ) {
    errors.push("Enter a valid Project ID.");
  }
  if (draft.scopeKind === "chat" && !isIdentifier(draft.sessionId.trim())) {
    errors.push("Enter a valid Chat session ID.");
  }
  const timeout = Number(draft.timeoutMs);
  if (
    !Number.isSafeInteger(timeout) ||
    timeout < MIN_MCP_TIMEOUT_MS ||
    timeout > MAX_MCP_TIMEOUT_MS
  ) {
    errors.push(
      `Set a timeout from ${MIN_MCP_TIMEOUT_MS} to ${MAX_MCP_TIMEOUT_MS} milliseconds.`,
    );
  }
  const output = Number(draft.maxOutputBytes);
  if (
    !Number.isSafeInteger(output) ||
    output < MIN_MCP_OUTPUT_BYTES ||
    output > MAX_MCP_OUTPUT_BYTES
  ) {
    errors.push(
      `Set an output limit from ${MIN_MCP_OUTPUT_BYTES} to ${MAX_MCP_OUTPUT_BYTES} bytes.`,
    );
  }

  if (draft.transportKind === "stdio") {
    if (!draft.command.trim().startsWith("/"))
      errors.push("Enter an absolute STDIO command path.");
    if (!isDigest(draft.executableSha256.trim())) {
      errors.push("Enter the executable SHA-256 digest.");
    }
    if (draft.arguments.some((item) => item.value.length === 0)) {
      errors.push("Remove blank arguments or enter a value.");
    }
    const environment = [
      ...draft.environment.map((item) => item.name),
      ...draft.passthrough.map((item) => item.value),
      ...draft.environmentSecrets.map((item) => item.name),
    ];
    if (environment.some((name) => !isEnvironmentName(name))) {
      errors.push("Environment variable names must use shell variable syntax.");
    }
    if (new Set(environment).size !== environment.length) {
      errors.push("Environment variable names must be unique.");
    }
    if (draft.environment.some((item) => item.value.length === 0)) {
      errors.push("Literal environment values cannot be blank.");
    }
    if (
      draft.environmentSecrets.some(
        (item) => !isReference(item.referenceKind, item.value),
      )
    ) {
      errors.push("Enter valid opaque credential references.");
    }
    if (
      draft.workingDirectory === "trustedRoot" &&
      draft.trustedRoot.trim().length === 0
    ) {
      errors.push("Enter the trusted-root path.");
    }
  } else {
    if (!isSafeEndpoint(draft.url.trim())) {
      errors.push(
        "Use HTTPS, or HTTP on a loopback host, for the Streamable HTTP URL.",
      );
    }
    if (draft.bearerKind === "none") {
      errors.push("Select an opaque bearer-token reference.");
    } else if (!isReference(draft.bearerKind, draft.bearerReference)) {
      errors.push("Enter a valid bearer-token reference.");
    }
    const headers = [
      ...draft.headers,
      ...draft.headerSecrets,
      ...draft.headerCredentials,
    ];
    if (headers.some((item) => !isHeaderName(item.name))) {
      errors.push("Enter valid HTTP header names.");
    }
    const normalizedHeaders = headers.map((item) =>
      item.name.toLocaleLowerCase(),
    );
    if (
      normalizedHeaders.some((name) =>
        [
          "authorization",
          "cookie",
          "host",
          "accept",
          "content-type",
          "content-length",
          "mcp-session-id",
          "mcp-protocol-version",
          "last-event-id",
        ].includes(name),
      )
    ) {
      errors.push("Transport-owned HTTP headers are reserved.");
    }
    if (new Set(normalizedHeaders).size !== normalizedHeaders.length) {
      errors.push("HTTP header names must be unique.");
    }
    if (draft.headers.some((item) => item.value.length === 0)) {
      errors.push("Literal header values cannot be blank.");
    }
    if (draft.headerSecrets.some((item) => !isEnvironmentName(item.value))) {
      errors.push(
        "Environment-backed headers require valid environment variable names.",
      );
    }
    if (
      draft.headerCredentials.some(
        (item) => !isReference(item.referenceKind, item.value),
      )
    ) {
      errors.push("Credential-backed headers require valid opaque references.");
    }
  }
  return errors;
}

function toDefinition(
  draft: McpServerDraft,
  generation: number,
): McpServerDefinitionInput {
  const scope =
    draft.scopeKind === "application"
      ? { kind: "application" as const }
      : draft.scopeKind === "workspace"
        ? { kind: "workspace" as const, workspaceId: draft.workspaceId.trim() }
        : draft.scopeKind === "project"
          ? {
              kind: "project" as const,
              workspaceId: draft.workspaceId.trim(),
              projectId: draft.projectId.trim(),
            }
          : {
              kind: "chat" as const,
              workspaceId: draft.workspaceId.trim(),
              projectId: draft.projectId.trim(),
              sessionId: draft.sessionId.trim(),
            };
  const transport =
    draft.transportKind === "stdio"
      ? {
          kind: "stdio" as const,
          command: draft.command.trim(),
          arguments: draft.arguments.map((item) => item.value),
          environment: [
            ...draft.environment.map((item) => ({
              name: item.name.trim(),
              source: { kind: "literal" as const, value: item.value },
            })),
            ...draft.passthrough.map((item) => ({
              name: item.value.trim(),
              source: { kind: "passthrough" as const },
            })),
            ...draft.environmentSecrets.map((item) => ({
              name: item.name.trim(),
              source: {
                kind: "secret" as const,
                reference:
                  item.referenceKind === "environment"
                    ? {
                        kind: "environment" as const,
                        variable: item.value.trim(),
                      }
                    : {
                        kind: "vault" as const,
                        credentialReference: item.value.trim(),
                      },
              },
            })),
          ],
          workingDirectory:
            draft.workingDirectory === "trustedRoot"
              ? { kind: "trustedRoot" as const, path: draft.trustedRoot.trim() }
              : { kind: draft.workingDirectory },
          executableSha256: draft.executableSha256.trim(),
        }
      : {
          kind: "streamableHttp" as const,
          url: draft.url.trim(),
          bearer:
            draft.bearerKind === "none"
              ? null
              : draft.bearerKind === "environment"
                ? {
                    kind: "environment" as const,
                    variable: draft.bearerReference.trim(),
                  }
                : {
                    kind: "vault" as const,
                    credentialReference: draft.bearerReference.trim(),
                  },
          headers: [
            ...draft.headers.map((item) => ({
              name: item.name.trim(),
              source: { kind: "literal" as const, value: item.value },
            })),
            ...draft.headerSecrets.map((item) => ({
              name: item.name.trim(),
              source: {
                kind: "environment" as const,
                variable: item.value.trim(),
              },
            })),
            ...draft.headerCredentials.map((item) => ({
              name: item.name.trim(),
              source: {
                kind: "secret" as const,
                reference:
                  item.referenceKind === "environment"
                    ? {
                        kind: "environment" as const,
                        variable: item.value.trim(),
                      }
                    : {
                        kind: "vault" as const,
                        credentialReference: item.value.trim(),
                      },
              },
            })),
          ],
        };
  return {
    expectedGeneration: generation,
    serverId: draft.serverId.trim(),
    displayName: draft.displayName.trim(),
    scope,
    transport,
    timeoutMs: Number(draft.timeoutMs),
    maxOutputBytes: Number(draft.maxOutputBytes),
  };
}

function isIdentifier(value: string): boolean {
  return /^[A-Za-z0-9._:-]+$/u.test(value);
}

function isDigest(value: string): boolean {
  return /^sha256:[0-9a-fA-F]{64}$/u.test(value);
}

function isEnvironmentName(value: string): boolean {
  const trimmed = value.trim();
  const normalized = trimmed.toLocaleUpperCase("en-US");
  return (
    /^[A-Za-z_][A-Za-z0-9_]*$/u.test(trimmed) &&
    !["HOME", "PATH", "SHELL", "TMPDIR"].includes(normalized) &&
    !normalized.startsWith("DYLD_") &&
    !normalized.startsWith("LD_")
  );
}

function isHeaderName(value: string): boolean {
  return /^[!#$%&'*+.^_`|~A-Za-z0-9-]+$/u.test(value.trim());
}

function isReference(kind: "environment" | "vault", value: string): boolean {
  return kind === "environment"
    ? isEnvironmentName(value)
    : isIdentifier(value.trim());
}

function isSafeEndpoint(value: string): boolean {
  try {
    const url = new URL(value);
    if (url.username.length > 0 || url.password.length > 0) return false;
    if (url.protocol === "https:") return true;
    return (
      url.protocol === "http:" &&
      ["127.0.0.1", "::1", "[::1]", "localhost"].includes(url.hostname)
    );
  } catch {
    return false;
  }
}
