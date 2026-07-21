import { expect, test, type Page } from "@playwright/test";

interface BrowserFailures {
  readonly console: string[];
  readonly page: string[];
}

function captureBrowserFailures(page: Page): BrowserFailures {
  const failures: BrowserFailures = { console: [], page: [] };
  page.on("console", (message) => {
    if (message.type() === "error" || message.type() === "warning") {
      failures.console.push(`${message.type()}: ${message.text()}`);
    }
  });
  page.on("pageerror", (error) => failures.page.push(error.message));
  return failures;
}

async function expectNoDocumentOverflow(page: Page) {
  await expect
    .poll(() =>
      page.evaluate(
        () =>
          document.documentElement.scrollWidth <=
          document.documentElement.clientWidth,
      ),
    )
    .toBe(true);
}

async function installNativeTerminalFixture(page: Page) {
  await page.addInitScript(
    ({ initialConversation }) => {
      type JsonRecord = Record<string, unknown>;
      type PendingOperation = {
        readonly artifactId: string;
        readonly input?: string;
        readonly type: "run" | "stdin" | "stop";
      };

      let workspaceGeneration = 90;
      let artifactClientGeneration = 0;
      let conversation = structuredClone(initialConversation) as JsonRecord;
      let focusedArtifactId: string | null = null;
      let completeOnNextArtifactSnapshot: string | null = null;
      let nextCallbackId = 1;
      const callbacks = new Map<
        number,
        {
          readonly callback: (payload: unknown) => void;
          readonly once: boolean;
        }
      >();
      const commands: string[] = [];
      const pendingOperations = new Map<string, PendingOperation>();
      const envelope = (
        request: JsonRecord,
        generation: number,
        payload: unknown,
      ) => ({
        protocolVersion: 1,
        requestId: request.requestId,
        correlationId: request.correlationId,
        generation,
        payload,
      });
      const encode = (value: string) => btoa(value);
      const artifactDigest = (sequence: number) =>
        `sha256:${sequence.toString(16).padStart(64, "0")}`;
      const terminalRecord = ({
        artifactId,
        command,
        commandSequence,
        exitCode,
        foregroundProcessGroupId,
        output,
        pendingApprovalId,
        phase,
        promptReady,
        recordRevision,
        shellProcessId,
        statusMessage,
        stdinReady,
        stopAvailable,
      }: {
        artifactId: string;
        command: string;
        commandSequence: number;
        exitCode: number | null;
        foregroundProcessGroupId: number | null;
        output: string;
        pendingApprovalId: string | null;
        phase: string;
        promptReady: boolean;
        recordRevision: number;
        shellProcessId: number | null;
        statusMessage: string | null;
        stdinReady: boolean;
        stopAvailable: boolean;
      }): JsonRecord => ({
        artifactId,
        projectId: "project:terminal-qa",
        sessionId: "session:terminal-qa",
        providerType: "terminal",
        providerVersion: 1,
        stateSchemaVersion: 1,
        recordRevision,
        title: command,
        focusSupported: true,
        pendingApprovalId,
        status: {
          kind: "ready",
          ...(pendingApprovalId === null
            ? {}
            : {
                message:
                  "Approval is required before this Terminal operation can continue.",
              }),
        },
        sourceLabel: "Direct Terminal operation",
        resourceVersion: {
          sequence: Math.max(1, recordRevision),
          sha256: artifactDigest(Math.max(1, recordRevision)),
          observedAtMs: 20_000 + recordRevision,
        },
        history: [],
        providerState: {
          type: "terminal",
          value: {
            terminalSessionId: "terminal-session:qa",
            commandId: `terminal-command:${commandSequence}`,
            commandSequence,
            command,
            workingDirectoryDisplay: "/project",
            shellPath: "/bin/zsh",
            environmentId: "desktop",
            environmentGeneration: 1,
            processGeneration: 1,
            shellProcessId,
            foregroundProcessGroupId,
            columns: 80,
            rows: 24,
            outputBase64: encode(output),
            outputText: output,
            outputSequence: Math.max(1, recordRevision),
            retainedBytes: output.length,
            droppedBytes: 0,
            phase,
            exitCode,
            statusMessage,
            stdinReady,
            stopAvailable,
            promptReady,
            shellReplaced: false,
          },
        },
      });
      let artifacts: JsonRecord[] = [
        terminalRecord({
          artifactId: "artifact:terminal-1",
          command: "pwd",
          commandSequence: 1,
          exitCode: 0,
          foregroundProcessGroupId: null,
          output: "/project\n",
          pendingApprovalId: null,
          phase: "completed",
          promptReady: true,
          recordRevision: 3,
          shellProcessId: 900,
          statusMessage: null,
          stdinReady: false,
          stopAvailable: false,
        }),
      ];
      const workspace = () => ({
        protocolVersion: 1,
        generation: workspaceGeneration,
        authority: "rust-core",
        workspaceId: "workspace:terminal-qa",
        activeProjectId: "project:terminal-qa",
        activeSessionId: "session:terminal-qa",
        focusedArtifactId,
        artifacts,
      });
      const artifactById = (artifactId: unknown) => {
        const artifact = artifacts.find(
          (candidate) => candidate.artifactId === artifactId,
        );
        if (artifact === undefined)
          throw new Error("Unknown Terminal artifact");
        return artifact;
      };
      const replaceArtifact = (next: JsonRecord) => {
        artifacts = artifacts.map((artifact) =>
          artifact.artifactId === next.artifactId ? next : artifact,
        );
        workspaceGeneration += 1;
      };
      const respondArtifact = (command: string, args: JsonRecord) => {
        const request = args.request as JsonRecord;
        if (request.expectedGeneration !== artifactClientGeneration) {
          throw new Error(
            `Stale Terminal fixture request: expected ${artifactClientGeneration}, received ${String(request.expectedGeneration)}`,
          );
        }
        if (command === "artifact_snapshot") {
          if (completeOnNextArtifactSnapshot !== null) {
            const artifact = artifactById(completeOnNextArtifactSnapshot);
            const provider = artifact.providerState as JsonRecord;
            const value = provider.value as JsonRecord;
            replaceArtifact({
              ...artifact,
              recordRevision: (artifact.recordRevision as number) + 1,
              providerState: {
                ...provider,
                value: {
                  ...value,
                  phase: "completed",
                  foregroundProcessGroupId: null,
                  exitCode: 0,
                  stdinReady: false,
                  stopAvailable: false,
                  promptReady: true,
                },
              },
            });
            completeOnNextArtifactSnapshot = null;
          }
          artifactClientGeneration = workspaceGeneration;
          return envelope(request, workspaceGeneration, workspace());
        }
        const input = (args.input ?? {}) as JsonRecord;
        if (command === "artifact_focus") {
          const artifact = artifactById(input.artifactId);
          if (input.baseRecordRevision !== artifact.recordRevision) {
            throw new Error("Stale Terminal focus revision");
          }
          focusedArtifactId = artifact.artifactId as string;
          workspaceGeneration += 1;
        } else if (command === "artifact_close_focus") {
          focusedArtifactId = null;
          workspaceGeneration += 1;
        } else if (command === "artifact_run_terminal") {
          const commandSequence = artifacts.length + 1;
          const artifactId = `artifact:terminal-${commandSequence}`;
          const promptId = `approval:terminal-run-${commandSequence}`;
          const next = terminalRecord({
            artifactId,
            command: input.command as string,
            commandSequence,
            exitCode: null,
            foregroundProcessGroupId: null,
            output: "",
            pendingApprovalId: promptId,
            phase: "approvalWaiting",
            promptReady: false,
            recordRevision: 1,
            shellProcessId: null,
            statusMessage: null,
            stdinReady: false,
            stopAvailable: false,
          });
          artifacts = [...artifacts, next];
          pendingOperations.set(promptId, {
            artifactId,
            type: "run",
          });
          workspaceGeneration += 1;
        } else if (command === "artifact_terminal_resize") {
          const artifact = artifactById(input.artifactId);
          const provider = artifact.providerState as JsonRecord;
          const value = provider.value as JsonRecord;
          replaceArtifact({
            ...artifact,
            recordRevision: (artifact.recordRevision as number) + 1,
            providerState: {
              ...provider,
              value: {
                ...value,
                columns: input.columns,
                rows: input.rows,
              },
            },
          });
        } else if (command === "artifact_terminal_ack_output") {
          artifactById(input.artifactId);
        } else if (command === "artifact_terminal_stdin") {
          const artifact = artifactById(input.artifactId);
          const provider = artifact.providerState as JsonRecord;
          const value = provider.value as JsonRecord;
          const promptId = `approval:terminal-stdin-${String(artifact.artifactId)}`;
          pendingOperations.set(promptId, {
            artifactId: artifact.artifactId as string,
            input: input.text as string,
            type: "stdin",
          });
          replaceArtifact({
            ...artifact,
            recordRevision: (artifact.recordRevision as number) + 1,
            pendingApprovalId: promptId,
            status: {
              kind: "ready",
              message:
                "Approval is required before this Terminal operation can continue.",
            },
            providerState: {
              ...provider,
              value: { ...value, stopAvailable: false },
            },
          });
        } else if (command === "artifact_terminal_stop") {
          const artifact = artifactById(input.artifactId);
          const provider = artifact.providerState as JsonRecord;
          const value = provider.value as JsonRecord;
          const promptId = `approval:terminal-stop-${String(artifact.artifactId)}`;
          pendingOperations.set(promptId, {
            artifactId: artifact.artifactId as string,
            type: "stop",
          });
          replaceArtifact({
            ...artifact,
            recordRevision: (artifact.recordRevision as number) + 1,
            pendingApprovalId: promptId,
            status: {
              kind: "ready",
              message:
                "Approval is required before this Terminal operation can continue.",
            },
            providerState: {
              ...provider,
              value: { ...value, stopAvailable: false },
            },
          });
        } else if (command === "artifact_answer_approval") {
          const operation = pendingOperations.get(input.promptId as string);
          if (operation === undefined)
            throw new Error("Unknown Terminal approval");
          pendingOperations.delete(input.promptId as string);
          const artifact = artifactById(operation.artifactId);
          const provider = artifact.providerState as JsonRecord;
          const value = provider.value as JsonRecord;
          const recordRevision = (artifact.recordRevision as number) + 1;
          if (input.answer === "deny") {
            replaceArtifact({
              ...artifact,
              recordRevision,
              pendingApprovalId: null,
              status: { kind: "error", message: "Policy denied the command." },
              providerState: {
                ...provider,
                value: {
                  ...value,
                  phase: "failed",
                  shellProcessId: null,
                  foregroundProcessGroupId: null,
                  statusMessage: "Policy denied the command.",
                  stdinReady: false,
                  stopAvailable: false,
                },
              },
            });
          } else if (operation.type === "run") {
            const commandText = value.command as string;
            const waitingForInput = commandText.startsWith("read ");
            const delayedCompletion = commandText === "race completion";
            const running =
              commandText.startsWith("sleep ") || delayedCompletion;
            if (delayedCompletion) {
              completeOnNextArtifactSnapshot = artifact.artifactId as string;
            }
            replaceArtifact({
              ...artifact,
              recordRevision,
              pendingApprovalId: null,
              status: { kind: "ready" },
              providerState: {
                ...provider,
                value: {
                  ...value,
                  phase: waitingForInput
                    ? "stdinReady"
                    : running
                      ? "running"
                      : "completed",
                  shellProcessId: 900,
                  foregroundProcessGroupId:
                    waitingForInput || running ? 901 : null,
                  exitCode: waitingForInput || running ? null : 0,
                  stdinReady: waitingForInput,
                  stopAvailable: waitingForInput || running,
                  promptReady: !waitingForInput && !running,
                },
              },
            });
          } else if (operation.type === "stdin") {
            const output = `${String(value.outputText)}received:${operation.input}\n`;
            replaceArtifact({
              ...artifact,
              recordRevision,
              pendingApprovalId: null,
              status: { kind: "ready" },
              providerState: {
                ...provider,
                value: {
                  ...value,
                  outputBase64: encode(output),
                  outputText: output,
                  outputSequence: (value.outputSequence as number) + 1,
                  retainedBytes: output.length,
                  phase: "completed",
                  foregroundProcessGroupId: null,
                  exitCode: 0,
                  statusMessage: null,
                  stdinReady: false,
                  stopAvailable: false,
                  promptReady: true,
                },
              },
            });
          } else {
            const output = `${String(value.outputText)}^C\n`;
            replaceArtifact({
              ...artifact,
              recordRevision,
              pendingApprovalId: null,
              status: { kind: "ready" },
              providerState: {
                ...provider,
                value: {
                  ...value,
                  outputBase64: encode(output),
                  outputText: output,
                  outputSequence: (value.outputSequence as number) + 1,
                  retainedBytes: output.length,
                  phase: "interrupted",
                  foregroundProcessGroupId: null,
                  exitCode: 130,
                  statusMessage: "Interrupted by Stop.",
                  stdinReady: false,
                  stopAvailable: false,
                  promptReady: true,
                },
              },
            });
          }
        } else if (command === "artifact_reply") {
          const artifact = artifactById(input.artifactId);
          const draft = conversation.draft as JsonRecord;
          conversation = {
            ...conversation,
            generation: (conversation.generation as number) + 1,
            draft: {
              ...draft,
              mode: "chat",
              replyTargetId: artifact.artifactId,
            },
          };
          workspaceGeneration += 1;
        } else {
          throw new Error(`Terminal fixture does not provide ${command}`);
        }
        artifactClientGeneration = workspaceGeneration;
        return envelope(request, workspaceGeneration, workspace());
      };
      const internals = {
        callbacks,
        transformCallback(callback: (payload: unknown) => void, once = false) {
          const id = nextCallbackId++;
          callbacks.set(id, { callback, once });
          return id;
        },
        unregisterCallback(id: number) {
          callbacks.delete(id);
        },
        runCallback(id: number, payload: unknown) {
          const entry = callbacks.get(id);
          entry?.callback(payload);
          if (entry?.once) callbacks.delete(id);
        },
        convertFileSrc(path: string) {
          return path;
        },
        async invoke(command: string, args: JsonRecord = {}) {
          commands.push(command);
          if (command === "plugin:event|listen") return nextCallbackId++;
          if (command === "plugin:event|unlisten") return null;
          const request = args.request as JsonRecord;
          if (command === "platform_snapshot") {
            return envelope(request, 0, {
              contractVersion: 1,
              platform: "macos",
              architecture: "aarch64",
              initialTheme: { scheme: "dark", source: "macosAppearance" },
              liveThemeSource: "webviewPrefersColorScheme",
              window: {
                decorations: "standard",
                titlebarTransparent: false,
                titlebarOverlay: false,
                initiallyVisible: false,
                revealFallbackTimeoutMs: 1500,
              },
              vocabulary: {
                revealAction: "Reveal in Finder",
                primaryModifierSymbol: "⌘",
                alternateModifierSymbol: "⌥",
                shiftModifierSymbol: "⇧",
              },
              capabilities: {
                nativeApplicationMenu: true,
                nativeSettingsShortcut: true,
                nativeFilePicker: true,
                nativeFolderPicker: true,
                nativeWorkspacePicker: true,
                standardWindowDecorations: true,
              },
              settingsMenu: {
                menuItemId: "c4os.menu.settings",
                commandId: "c4os.command.openSettings",
                route: "/settings/providers",
                accelerator: "CmdOrCtrl+,",
                keyboardLabel: "⌘,",
              },
            });
          }
          if (command === "platform_reveal_main") {
            return envelope(request, 0, { revealed: true, fallback: false });
          }
          if (command === "conversation_snapshot") {
            conversation = {
              ...conversation,
              generation: workspaceGeneration,
            };
            return envelope(request, workspaceGeneration, conversation);
          }
          if (command === "conversation_update_draft") {
            if (request.expectedGeneration !== workspaceGeneration) {
              throw new Error(
                `Stale Conversation fixture request: expected ${workspaceGeneration}, received ${String(request.expectedGeneration)}`,
              );
            }
            workspaceGeneration += 1;
            conversation = {
              ...conversation,
              generation: workspaceGeneration,
              draft: {
                ...(conversation.draft as JsonRecord),
                ...(args.input as JsonRecord),
              },
            };
            return envelope(request, workspaceGeneration, conversation);
          }
          if (command.startsWith("artifact_")) {
            return respondArtifact(command, args);
          }
          throw new Error(`Terminal fixture does not provide ${command}`);
        },
      };
      (
        window as unknown as {
          __C4OS_TERMINAL_E2E__: { readonly commands: string[] };
          __TAURI_INTERNALS__: typeof internals;
        }
      ).__TAURI_INTERNALS__ = internals;
      (
        window as unknown as {
          __TAURI_EVENT_PLUGIN_INTERNALS__: {
            unregisterListener(event: string, eventId: number): void;
          };
        }
      ).__TAURI_EVENT_PLUGIN_INTERNALS__ = {
        unregisterListener() {},
      };
      (
        window as unknown as {
          __C4OS_TERMINAL_E2E__: { readonly commands: string[] };
        }
      ).__C4OS_TERMINAL_E2E__ = { commands };
    },
    { initialConversation: conversationSnapshot() },
  );
}

function conversationSnapshot() {
  return {
    protocolVersion: 1,
    generation: 60,
    authority: "rust-core",
    workspaceId: "workspace:terminal-qa",
    workspaceName: "Terminal QA Workspace",
    activeProjectId: "project:terminal-qa",
    activeSessionId: "session:terminal-qa",
    pending: null,
    draft: {
      prompt: "",
      attachments: [],
      nextAttachmentReference: 1,
      providerId: "provider:test",
      modelId: "model:test",
      reasoningMode: null,
      mode: "chat",
      replyTargetId: null,
    },
    projects: [
      {
        projectId: "project:terminal-qa",
        displayName: "Terminal Project",
        pathState: "found",
        position: 0,
        gitVersioned: false,
      },
    ],
    sessions: [
      {
        sessionId: "session:terminal-qa",
        projectId: "project:terminal-qa",
        title: "Terminal facilities",
        updatedAtMs: 2_000,
      },
    ],
    activeConversation: {
      sessionId: "session:terminal-qa",
      title: "Terminal facilities",
      turns: [
        {
          turnId: "turn:terminal",
          prompt: "Use the Project shell.",
          attachments: [],
          artifactContext: null,
          submittedAtMs: 100,
        },
      ],
      attempts: [
        {
          attemptId: "attempt:terminal",
          turnId: "turn:terminal",
          status: "completed",
          assistantMarkdown: "The Terminal facility is ready.",
          activities: [],
          runtimeId: "runtime:test",
          runtimeKind: "open-code",
          environmentId: "desktop",
          providerId: "provider:test",
          modelId: "model:test",
          adapterId: "adapter:test",
          inputTokens: 10,
          outputTokens: 10,
          durationMs: 100,
        },
      ],
      activeAttemptId: null,
    },
    models: [
      {
        providerId: "provider:test",
        providerName: "Test Provider",
        modelId: "model:test",
        selected: true,
        available: true,
        supportsVision: false,
        supportsTools: true,
        supportsReasoning: false,
        supportsAudio: false,
        contextTokens: 8_192,
      },
    ],
    branchControl: null,
  };
}

test("production Terminal composes approval, focused stdin, output acknowledgement, prompt continuation, and Reply", async ({
  page,
}) => {
  const failures = captureBrowserFailures(page);
  await installNativeTerminalFixture(page);
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/#/chat");

  const inline = page.getByRole("article", {
    name: "Terminal response artifact: pwd",
  });
  await expect(inline).toContainText("/project");
  await expect(inline).toContainText("Completed · exit 0");
  await inline.getByRole("button", { name: "Expand" }).focus();
  await page.keyboard.press("Enter");

  let focused = page.getByRole("region", { name: "Focused pwd" });
  await expect(
    focused.getByRole("region", { name: "Output for pwd" }),
  ).toBeVisible();
  await focused
    .getByRole("textbox", { name: "Next command" })
    .fill("read answer");
  await focused.getByRole("button", { name: "Run" }).click();

  focused = page.getByRole("region", { name: "Focused read answer" });
  await expect(focused).toContainText("Waiting for approval");
  await focused.getByRole("button", { name: "Allow" }).click();
  await expect(focused).toContainText("Running · input available");
  await focused.getByRole("textbox", { name: "Process input" }).fill("alpha");
  await focused.getByRole("button", { name: "Send input" }).click();
  await expect(focused).toContainText(
    "Approval is required before this Terminal operation can continue.",
  );
  await focused.getByRole("button", { name: "Allow" }).click();
  await expect(focused).toContainText("Completed · exit 0");
  await expect(
    focused.getByRole("textbox", { name: "Next command" }),
  ).toBeVisible();
  await page.screenshot({
    path: "output/playwright/task-00009-terminal-stdin.png",
    fullPage: true,
  });

  await focused.getByRole("button", { name: "Reply" }).click();
  await expect(
    page.getByRole("region", { name: "Reply reference" }),
  ).toContainText("Reply to terminal");
  const commands = await page.evaluate(
    () =>
      (
        window as unknown as {
          __C4OS_TERMINAL_E2E__: { readonly commands: string[] };
        }
      ).__C4OS_TERMINAL_E2E__.commands,
  );
  expect(commands).toEqual(
    expect.arrayContaining([
      "artifact_run_terminal",
      "artifact_answer_approval",
      "artifact_terminal_stdin",
      "artifact_terminal_ack_output",
      "artifact_reply",
    ]),
  );
  await expectNoDocumentOverflow(page);
  expect(failures.console).toEqual([]);
  expect(failures.page).toEqual([]);
});

test("production Terminal rebases the shared Workspace generation before saving the next command draft", async ({
  page,
}) => {
  const failures = captureBrowserFailures(page);
  await installNativeTerminalFixture(page);
  await page.goto("/#/chat");

  await page.getByRole("button", { name: "Composer mode" }).click();
  await page.getByRole("menuitemradio", { name: "Terminal" }).click();
  await page
    .getByRole("textbox", { name: "Terminal command" })
    .fill("race completion");
  await page.getByRole("button", { name: "Run", exact: true }).click();
  const artifact = page.getByRole("article", {
    name: "Terminal response artifact: race completion",
  });
  await artifact.getByRole("button", { name: "Allow" }).click();
  await expect(artifact).toContainText("Completed · exit 0");

  const priorDraftWrites = await page.evaluate(
    () =>
      (
        window as unknown as {
          __C4OS_TERMINAL_E2E__: { readonly commands: string[] };
        }
      ).__C4OS_TERMINAL_E2E__.commands.filter(
        (command) => command === "conversation_update_draft",
      ).length,
  );
  await page.getByRole("textbox", { name: "Terminal command" }).fill("pwd");
  await expect
    .poll(() =>
      page.evaluate(
        () =>
          (
            window as unknown as {
              __C4OS_TERMINAL_E2E__: { readonly commands: string[] };
            }
          ).__C4OS_TERMINAL_E2E__.commands.filter(
            (command) => command === "conversation_update_draft",
          ).length,
      ),
    )
    .toBeGreaterThan(priorDraftWrites);
  await expect(
    page.getByText(
      "Conversation state changed before saving the composer draft",
    ),
  ).toHaveCount(0);
  await expectNoDocumentOverflow(page);
  expect(failures.console).toEqual([]);
  expect(failures.page).toEqual([]);
});

test("production Terminal Stop remains approval-bound, interrupts at 130, and returns a reusable prompt", async ({
  page,
}) => {
  const failures = captureBrowserFailures(page);
  await installNativeTerminalFixture(page);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/#/chat");
  await page
    .getByRole("article", { name: "Terminal response artifact: pwd" })
    .getByRole("button", { name: "Expand" })
    .click();

  let focused = page.getByRole("region", { name: "Focused pwd" });
  await focused.getByRole("textbox", { name: "Next command" }).fill("sleep 30");
  await focused.getByRole("button", { name: "Run" }).click();
  focused = page.getByRole("region", { name: "Focused sleep 30" });
  await focused.getByRole("button", { name: "Allow" }).click();
  await expect(focused).toContainText("Running");
  await focused.getByRole("button", { name: "Stop sleep 30" }).click();
  await expect(focused.getByRole("button", { name: "Allow" })).toBeVisible();
  await focused.getByRole("button", { name: "Allow" }).click();
  await expect(focused).toContainText("Interrupted · exit 130");
  await expect(
    focused.getByRole("textbox", { name: "Next command" }),
  ).toBeVisible();
  await page.screenshot({
    path: "output/playwright/task-00009-terminal-stop.png",
    fullPage: true,
  });
  await expectNoDocumentOverflow(page);
  expect(failures.console).toEqual([]);
  expect(failures.page).toEqual([]);
});
