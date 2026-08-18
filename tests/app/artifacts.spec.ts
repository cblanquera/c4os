import { expect, test, type Page } from "@playwright/test";

type ArtifactFixtureState =
  "browser" | "conflict" | "file" | "folder" | "proposed";

const ORIGINAL_FILE = [
  "# Artifact workspace",
  "",
  "This file is read through a native capability.",
].join("\n");
const EDITED_FILE = [
  "# Artifact workspace",
  "",
  "This retained draft follows the focused File.",
].join("\n");
const PROPOSED_FILE = ORIGINAL_FILE.replace(
  "native capability",
  "bounded Reply proposal",
);

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

function expectNoBrowserFailures(failures: BrowserFailures) {
  expect(failures.console).toEqual([]);
  expect(failures.page).toEqual([]);
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

async function installNativeArtifactFixture(
  page: Page,
  state: ArtifactFixtureState,
) {
  await page.addInitScript(
    ({ initialArtifactSnapshot, initialConversationSnapshot }) => {
      type JsonRecord = Record<string, unknown>;

      let artifactSnapshot = structuredClone(
        initialArtifactSnapshot,
      ) as JsonRecord;
      let conversationSnapshot = structuredClone(
        initialConversationSnapshot,
      ) as JsonRecord;
      let artifactClientGeneration = 0;
      let nextCallbackId = 1;
      const commands: string[] = [];
      const callbacks = new Map<
        number,
        {
          readonly callback: (payload: unknown) => void;
          readonly once: boolean;
        }
      >();
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
      const artifactGeneration = () => artifactSnapshot.generation as number;
      const artifactRecords = () => artifactSnapshot.artifacts as JsonRecord[];
      const activeArtifact = () => artifactRecords()[0];
      const artifactInput = (args: JsonRecord) => args.input as JsonRecord;
      const requireArtifactRequest = (args: JsonRecord) => {
        const request = args.request as JsonRecord;
        if (request.expectedGeneration !== artifactClientGeneration) {
          throw new Error(
            `Stale artifact fixture request: expected ${artifactClientGeneration}, received ${String(request.expectedGeneration)}`,
          );
        }
        return request;
      };
      const requireExactArtifact = (args: JsonRecord) => {
        const input = artifactInput(args);
        const artifact = activeArtifact();
        if (
          input.artifactId !== artifact.artifactId ||
          input.baseRecordRevision !== artifact.recordRevision
        ) {
          throw new Error("Artifact fixture received a stale record revision.");
        }
        return { artifact, input };
      };
      const advanceArtifactWorkspace = () => {
        artifactSnapshot = {
          ...artifactSnapshot,
          generation: artifactGeneration() + 1,
        };
      };
      const commitArtifact = (
        kind: string,
        update: (artifact: JsonRecord) => JsonRecord,
      ) => {
        const prior = activeArtifact();
        const nextRevision = (prior.recordRevision as number) + 1;
        const resourceVersion = prior.resourceVersion as JsonRecord;
        const next = {
          ...update(prior),
          recordRevision: nextRevision,
          history: [
            ...((prior.history as JsonRecord[]) ?? []),
            {
              recordRevision: nextRevision,
              kind,
              recordedAtMs: 10_000 + nextRevision,
              resourceVersion,
            },
          ],
        };
        artifactSnapshot = {
          ...artifactSnapshot,
          generation: artifactGeneration() + 1,
          artifacts: [next],
        };
      };
      const setConversationReply = (artifactId: unknown) => {
        const draft = conversationSnapshot.draft as JsonRecord;
        const generation = (conversationSnapshot.generation as number) + 1;
        conversationSnapshot = {
          ...conversationSnapshot,
          generation,
          draft: { ...draft, mode: "chat", replyTargetId: artifactId },
        };
      };
      const respondArtifact = (command: string, args: JsonRecord): unknown => {
        const request = requireArtifactRequest(args);
        if (command === "artifact_snapshot") {
          artifactClientGeneration = artifactGeneration();
          return envelope(request, artifactGeneration(), artifactSnapshot);
        }
        if (command === "artifact_close_focus") {
          advanceArtifactWorkspace();
          artifactSnapshot = { ...artifactSnapshot, focusedArtifactId: null };
        } else {
          const { artifact, input } = requireExactArtifact(args);
          const providerState = artifact.providerState as JsonRecord;
          const providerValue = providerState.value as JsonRecord;
          const fileState = providerValue.state as JsonRecord;
          if (command === "artifact_focus") {
            advanceArtifactWorkspace();
            artifactSnapshot = {
              ...artifactSnapshot,
              focusedArtifactId: artifact.artifactId,
            };
          } else if (
            command === "artifact_mount_browser" ||
            command === "artifact_resize_browser" ||
            command === "artifact_focus_native_browser" ||
            command === "artifact_detach_browser"
          ) {
            // The browser fixture owns only renderer geometry evidence. Native
            // WKWebView attachment remains covered by the Rust/native tier.
          } else if (command === "artifact_begin_file_edit") {
            commitArtifact("fileEditBegan", (current) => ({
              ...current,
              providerState: {
                ...providerState,
                value: {
                  ...providerValue,
                  state: {
                    phase: "edit",
                    content: fileState.content,
                    draft: fileState.content,
                  },
                },
              },
            }));
          } else if (command === "artifact_update_file_draft") {
            const content = input.content as string;
            commitArtifact("fileDraftUpdated", (current) => ({
              ...current,
              providerState: {
                ...providerState,
                value: {
                  ...providerValue,
                  state: {
                    phase: content === fileState.content ? "edit" : "dirty",
                    content: fileState.content,
                    draft: content,
                  },
                },
              },
            }));
          } else if (command === "artifact_save_file") {
            const content =
              fileState.phase === "proposed" || fileState.phase === "approval"
                ? fileState.proposedContent
                : fileState.draft;
            const priorVersion = artifact.resourceVersion as JsonRecord;
            commitArtifact("fileSaved", (current) => ({
              ...current,
              pendingApprovalId: null,
              resourceVersion: {
                sequence: (priorVersion.sequence as number) + 1,
                sha256: `sha256:${"2".repeat(64)}`,
                observedAtMs: (priorVersion.observedAtMs as number) + 1,
              },
              providerState: {
                ...providerState,
                value: {
                  ...providerValue,
                  versionLabel: `Version ${(priorVersion.sequence as number) + 1}`,
                  state: { phase: "read", content },
                },
              },
            }));
          } else if (command === "artifact_discard_file_draft") {
            commitArtifact("fileDraftDiscarded", (current) => ({
              ...current,
              providerState: {
                ...providerState,
                value: {
                  ...providerValue,
                  state: { phase: "read", content: fileState.content },
                },
              },
            }));
          } else if (command === "artifact_reject_file_proposal") {
            commitArtifact("fileProposalRejected", (current) => ({
              ...current,
              providerState: {
                ...providerState,
                value: {
                  ...providerValue,
                  state: { phase: "read", content: fileState.content },
                },
              },
            }));
          } else if (command === "artifact_resolve_file_conflict") {
            const reload = input.resolution === "reloadCurrent";
            commitArtifact("fileConflictResolved", (current) => ({
              ...current,
              providerState: {
                ...providerState,
                value: {
                  ...providerValue,
                  state: reload
                    ? { phase: "read", content: fileState.content }
                    : {
                        phase: "dirty",
                        content: fileState.content,
                        draft: fileState.draft,
                      },
                },
              },
            }));
          } else if (command === "artifact_select_folder_entry") {
            if (input.entryId === "entry:docs") {
              commitArtifact("folderNavigated", (current) => ({
                ...current,
                title: "docs",
                providerState: {
                  type: "folder",
                  value: folderProvider("docs"),
                },
              }));
            } else if (input.entryId === "entry:guide") {
              commitArtifact("folderFileSelected", (current) => ({
                ...current,
                title: "guide.md",
                providerType: "file",
                providerState: {
                  type: "file",
                  value: fileProvider({
                    phase: "read",
                    content: "# Guide\n\nNested File conversion is active.",
                  }),
                },
              }));
            } else {
              throw new Error("Artifact fixture received an unknown entry.");
            }
          } else if (command === "artifact_navigate_folder") {
            const path = input.projectRelativePath;
            commitArtifact("folderBreadcrumbNavigated", (current) => ({
              ...current,
              title: path === "docs" ? "docs" : "project",
              providerType: "folder",
              providerState: {
                type: "folder",
                value: folderProvider(path === "docs" ? "docs" : "root"),
              },
            }));
          } else if (command === "artifact_refresh_folder") {
            commitArtifact("folderRefreshed", (current) => current);
          } else if (command === "artifact_reply") {
            setConversationReply(artifact.artifactId);
            advanceArtifactWorkspace();
          } else {
            throw new Error(`Artifact fixture does not provide ${command}`);
          }
        }
        artifactClientGeneration = artifactGeneration();
        return envelope(request, artifactGeneration(), artifactSnapshot);
      };
      const fileProvider = (state: JsonRecord): JsonRecord => ({
        breadcrumbs: [
          { id: "", label: "project", isCurrent: false },
          { id: "docs", label: "docs", isCurrent: false },
          { id: "docs/README.md", label: "README.md", isCurrent: true },
        ],
        languageLabel: "Markdown",
        versionLabel: "Version 7",
        state,
      });
      const folderProvider = (path: "docs" | "root"): JsonRecord =>
        path === "docs"
          ? {
              breadcrumbs: [
                { id: "", label: "project", isCurrent: false },
                { id: "docs", label: "docs", isCurrent: true },
              ],
              entries: [
                {
                  id: "entry:reference",
                  kind: "folder",
                  metadata: "1 item",
                  name: "reference",
                },
                {
                  id: "entry:guide",
                  kind: "file",
                  metadata: "1 KB · Markdown",
                  name: "guide.md",
                },
              ],
              listing: { phase: "ready", message: null },
              listingLimit: 20,
              selectedEntryId: "entry:guide",
            }
          : {
              breadcrumbs: [{ id: "", label: "project", isCurrent: true }],
              entries: [
                {
                  id: "entry:docs",
                  kind: "folder",
                  metadata: "2 items",
                  name: "docs",
                },
                {
                  id: "entry:src",
                  kind: "folder",
                  metadata: "8 items",
                  name: "src",
                },
                {
                  id: "entry:readme",
                  kind: "file",
                  metadata: "2 KB · Markdown",
                  name: "README.md",
                },
              ],
              listing: { phase: "ready", message: null },
              listingLimit: 3,
              selectedEntryId: null,
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
        async invoke(command: string, args: JsonRecord = {}): Promise<unknown> {
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
            return envelope(
              request,
              conversationSnapshot.generation as number,
              conversationSnapshot,
            );
          }
          if (command === "conversation_update_draft") {
            const input = args.input as JsonRecord;
            const generation = (conversationSnapshot.generation as number) + 1;
            conversationSnapshot = {
              ...conversationSnapshot,
              generation,
              draft: {
                ...(conversationSnapshot.draft as JsonRecord),
                ...input,
              },
            };
            return envelope(request, generation, conversationSnapshot);
          }
          if (command.startsWith("artifact_")) {
            return respondArtifact(command, args);
          }
          throw new Error(`Artifact fixture does not provide ${command}`);
        },
      };
      (
        window as unknown as {
          __C4OS_ARTIFACT_E2E__: { readonly commands: string[] };
          __TAURI_INTERNALS__: typeof internals;
          __C4OS_QA_NATIVE_FIXTURE__: "deterministic-e2e";
        }
      ).__TAURI_INTERNALS__ = internals;
      (
        window as unknown as {
          __C4OS_QA_NATIVE_FIXTURE__: "deterministic-e2e";
        }
      ).__C4OS_QA_NATIVE_FIXTURE__ = "deterministic-e2e";
      (
        window as unknown as {
          __C4OS_ARTIFACT_E2E__: { readonly commands: string[] };
        }
      ).__C4OS_ARTIFACT_E2E__ = { commands };
    },
    {
      initialArtifactSnapshot: artifactWorkspace(state),
      initialConversationSnapshot: conversationSnapshot(),
    },
  );
}

function artifactWorkspace(state: ArtifactFixtureState) {
  const fileState =
    state === "proposed"
      ? {
          phase: "proposed",
          content: ORIGINAL_FILE,
          proposedContent: PROPOSED_FILE,
          proposalDiff:
            "-This file is read through a native capability.\n+This file is read through a bounded Reply proposal.",
          proposalSummary: "Replace one capability description",
        }
      : state === "conflict"
        ? {
            phase: "conflict",
            content: ORIGINAL_FILE.replace("native", "externally updated"),
            draft: EDITED_FILE,
            conflictMessage: "README.md changed after this draft was captured",
            currentVersionLabel: "Version 8",
          }
        : { phase: "read", content: ORIGINAL_FILE };
  const browser = state === "browser";
  const folder = state === "folder";
  return {
    protocolVersion: 1,
    generation: 70,
    authority: "rust-core",
    workspaceId: "workspace:qa",
    activeProjectId: "project:qa",
    activeSessionId: "session:qa",
    focusedArtifactId: null,
    artifacts: [
      {
        artifactId: browser
          ? "artifact:browser-docs"
          : folder
            ? "artifact:folder-project"
            : "artifact:file-readme",
        projectId: "project:qa",
        sessionId: "session:qa",
        providerType: browser ? "browser" : folder ? "folder" : "file",
        providerVersion: 1,
        stateSchemaVersion: 1,
        recordRevision: 7,
        title: browser
          ? "C4OS documentation"
          : folder
            ? "project"
            : "README.md",
        focusSupported: true,
        pendingApprovalId: null,
        status: { kind: "ready" },
        sourceLabel: browser
          ? "Per-Project Browser"
          : folder
            ? "Trusted Project folder"
            : "Trusted Project file · docs/README.md",
        resourceVersion: {
          sequence: 7,
          sha256: `sha256:${"1".repeat(64)}`,
          observedAtMs: 9_000,
        },
        history: [],
        providerState: browser
          ? {
              type: "browser",
              value: {
                currentUrl: "https://docs.example.test/c4os",
                pageTitle: "C4OS documentation",
                phase: "ready",
                refreshing: false,
                canGoBack: true,
                canGoForward: false,
                controllerGeneration: 4,
                mountGeneration: 2,
                environmentScope: "per-project",
                pendingOperation: null,
                pendingTargetUrl: null,
                notices: [
                  {
                    id: "browser-notice:ready",
                    kind: "information",
                    title: "Browser ready",
                    message:
                      "The native page is attached to this exact artifact.",
                  },
                ],
              },
            }
          : folder
            ? {
                type: "folder",
                value: {
                  breadcrumbs: [{ id: "", label: "project", isCurrent: true }],
                  entries: [
                    {
                      id: "entry:docs",
                      kind: "folder",
                      metadata: "2 items",
                      name: "docs",
                    },
                    {
                      id: "entry:src",
                      kind: "folder",
                      metadata: "8 items",
                      name: "src",
                    },
                    {
                      id: "entry:readme",
                      kind: "file",
                      metadata: "2 KB · Markdown",
                      name: "README.md",
                    },
                  ],
                  listing: { phase: "ready", message: null },
                  listingLimit: 3,
                  selectedEntryId: null,
                },
              }
            : {
                type: "file",
                value: {
                  breadcrumbs: [
                    { id: "", label: "project", isCurrent: false },
                    { id: "docs", label: "docs", isCurrent: false },
                    {
                      id: "docs/README.md",
                      label: "README.md",
                      isCurrent: true,
                    },
                  ],
                  languageLabel: "Markdown",
                  versionLabel: "Version 7",
                  state: fileState,
                },
              },
      },
    ],
  };
}

function conversationSnapshot() {
  return {
    protocolVersion: 1,
    generation: 61,
    authority: "rust-core",
    workspaceId: "workspace:qa",
    workspaceName: "C4OS QA Workspace",
    activeProjectId: "project:qa",
    activeSessionId: "session:qa",
    pending: null,
    draft: {
      prompt: "",
      attachments: [],
      nextAttachmentReference: 1,
      providerId: "openai",
      modelId: "openai/gpt-5",
      reasoningMode: "medium",
      mode: "chat",
      replyTargetId: null,
    },
    projects: [
      {
        projectId: "project:qa",
        displayName: "AI Desktop UI",
        pathState: "found",
        position: 0,
        gitVersioned: true,
      },
    ],
    sessions: [
      {
        sessionId: "session:qa",
        projectId: "project:qa",
        title: "Review File facilities",
        updatedAtMs: 2_000,
      },
    ],
    activeConversation: {
      sessionId: "session:qa",
      title: "Review File facilities",
      turns: [
        {
          turnId: "turn:qa-artifact",
          prompt: "Open the trusted artifact and keep Chat available.",
          attachments: [],
          artifactContext: null,
          mcpProvenance: null,
          submittedAtMs: 100,
        },
      ],
      attempts: [
        {
          attemptId: "attempt:qa-artifact",
          turnId: "turn:qa-artifact",
          status: "completed",
          assistantMarkdown: "The native facility is ready for review.",
          activities: [],
          runtimeId: "runtime:qa-opencode",
          runtimeKind: "open-code",
          environmentId: "local",
          providerId: "openai",
          modelId: "openai/gpt-5",
          adapterId: "adapter:opencode",
          inputTokens: 900,
          outputTokens: 180,
          durationMs: 1_200,
        },
      ],
      activeAttemptId: null,
    },
    models: [
      {
        providerId: "openai",
        providerName: "OpenAI",
        modelId: "openai/gpt-5",
        selected: true,
        available: true,
        supportsVision: true,
        supportsTools: true,
        supportsReasoning: true,
        supportsAudio: false,
        contextTokens: 400_000,
      },
    ],
    branchControl: {
      currentBranch: "main",
      branches: [{ name: "main", targetOid: "1".repeat(40), selected: true }],
      pendingApprovalId: null,
      operationStatus: null,
      operationMessage: null,
    },
  };
}

test("production File reads, edits, saves, and preserves Reply through the native artifact adapter", async ({
  page,
}) => {
  const failures = captureBrowserFailures(page);
  await installNativeArtifactFixture(page, "file");
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/#/chat");

  const file = page.getByRole("article", {
    name: "File response artifact: README.md",
  });
  await expect(file).toContainText(
    "This file is read through a native capability.",
  );
  await expect(
    file.getByRole("navigation", { name: "File breadcrumbs" }),
  ).toBeVisible();
  await expect(file.getByRole("button", { name: "README.md" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await expect(file).toContainText("Version 7 · Markdown · Read only");

  const artifactBody = file.locator('[data-artifact-scroll-region="body"]');
  await artifactBody.focus();
  await expect(artifactBody).toBeFocused();
  await file.getByRole("button", { name: "Edit" }).focus();
  await page.keyboard.press("Enter");
  const editor = file.getByRole("textbox", { name: "Edit README.md" });
  await editor.fill(EDITED_FILE);
  await expect(file.getByRole("status")).toContainText("Unsaved changes");
  await file.getByRole("button", { name: "Save" }).click();
  await expect(file).toContainText(
    "This retained draft follows the focused File.",
  );
  await expect(file).toContainText("Version 8 · Markdown · Read only");

  await file.getByRole("button", { name: "Reply" }).click();
  await expect(
    page.getByRole("region", { name: "Reply reference" }),
  ).toContainText("Reply to file");
  const commands = await page.evaluate(
    () =>
      (
        window as unknown as {
          __C4OS_ARTIFACT_E2E__: { readonly commands: string[] };
        }
      ).__C4OS_ARTIFACT_E2E__.commands,
  );
  expect(commands).toEqual(
    expect.arrayContaining([
      "artifact_snapshot",
      "artifact_begin_file_edit",
      "artifact_update_file_draft",
      "artifact_save_file",
      "artifact_reply",
    ]),
  );
  await expectNoDocumentOverflow(page);
  expectNoBrowserFailures(failures);
});

test("production File exposes a bounded proposed diff and both proposal decisions", async ({
  page,
}) => {
  const failures = captureBrowserFailures(page);
  await installNativeArtifactFixture(page, "proposed");
  await page.goto("/#/chat");

  let file = page.getByRole("article", {
    name: "File response artifact: README.md",
  });
  await expect(file.getByRole("status")).toContainText(
    "Proposed change · Replace one capability description",
  );
  await expect(file.getByLabel("Proposed File diff")).toContainText(
    "+This file is read through a bounded Reply proposal.",
  );
  await expect(file.getByRole("button", { name: "Approve" })).toBeEnabled();
  await page.screenshot({
    path: "output/playwright/task-00022-file-proposed.png",
  });
  await file.getByRole("button", { name: "Reject" }).focus();
  await page.keyboard.press("Enter");
  await expect(file).toContainText("native capability");
  await expect(file.getByLabel("Proposed File diff")).toHaveCount(0);

  await page.reload();
  file = page.getByRole("article", {
    name: "File response artifact: README.md",
  });
  await expect(file.getByLabel("Proposed File diff")).toBeVisible();
  await file.getByRole("button", { name: "Approve" }).click();
  await expect(file).toContainText("bounded Reply proposal");
  await expect(file).toContainText("Version 8 · Markdown · Read only");
  expectNoBrowserFailures(failures);
});

test("production File conflict keeps or reloads the exact controlled draft", async ({
  page,
}) => {
  const failures = captureBrowserFailures(page);
  await installNativeArtifactFixture(page, "conflict");
  await page.goto("/#/chat");

  let file = page.getByRole("article", {
    name: "File response artifact: README.md",
  });
  await expect(file.getByRole("alert")).toContainText(
    "README.md changed after this draft was captured · Version 8",
  );
  await page.screenshot({
    path: "output/playwright/task-00022-file-conflict.png",
  });
  await file.getByRole("button", { name: "Keep draft" }).click();
  await expect(
    file.getByRole("textbox", { name: "Edit README.md" }),
  ).toHaveValue(EDITED_FILE);
  await expect(file.getByRole("status")).toContainText("Unsaved changes");

  await page.reload();
  file = page.getByRole("article", {
    name: "File response artifact: README.md",
  });
  await file.getByRole("button", { name: "Reload current" }).click();
  await expect(file).toContainText("externally updated capability");
  await expect(
    file.getByRole("textbox", { name: "Edit README.md" }),
  ).toHaveCount(0);
  expectNoBrowserFailures(failures);
});

test("production Folder presents and navigates a nested bounded listing before File conversion", async ({
  page,
}) => {
  const failures = captureBrowserFailures(page);
  await installNativeArtifactFixture(page, "folder");
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("/#/chat");

  let folder = page.getByRole("article", {
    name: "Folder response artifact: project",
  });
  await expect(
    folder.getByRole("list", { name: "Folder contents" }),
  ).toBeVisible();
  await expect(folder.getByRole("button", { name: "project" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await expect(
    folder.getByRole("button", { name: "Open folder docs" }),
  ).toContainText("2 items");
  await page.screenshot({
    path: "output/playwright/task-00022-folder-inline.png",
  });
  await folder.getByRole("button", { name: "Open folder docs" }).focus();
  await page.keyboard.press("Enter");

  folder = page.getByRole("article", {
    name: "Folder response artifact: docs",
  });
  await expect(folder.getByRole("button", { name: "docs" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await expect(
    folder.getByRole("button", { name: "Open folder reference" }),
  ).toBeVisible();
  await expect(
    folder.getByRole("button", { name: "Open file guide.md" }),
  ).toContainText("1 KB · Markdown");
  await folder.getByRole("button", { name: "Open file guide.md" }).click();
  const converted = page.getByRole("article", {
    name: "File response artifact: guide.md",
  });
  await expect(converted).toContainText("Nested File conversion is active.");
  await expectNoDocumentOverflow(page);
  expectNoBrowserFailures(failures);
});

test("production focus moves one real Chat DOM while File draft state remains composed", async ({
  page,
}) => {
  const failures = captureBrowserFailures(page);
  await installNativeArtifactFixture(page, "file");
  await page.goto("/#/chat");

  const inlineFile = page.getByRole("article", {
    name: "File response artifact: README.md",
  });
  await inlineFile.getByRole("button", { name: "Edit" }).click();
  await inlineFile
    .getByRole("textbox", { name: "Edit README.md" })
    .fill(EDITED_FILE);
  await page.waitForTimeout(300);
  const transcript = page.getByRole("region", {
    name: "Conversation",
    exact: true,
  });
  await transcript.evaluate((node) => {
    node.setAttribute("data-e2e-stable-transcript", "task-00008");
  });
  await inlineFile.getByRole("button", { name: "Expand" }).focus();
  await page.keyboard.press("Enter");

  const focused = page.getByRole("region", { name: "Focused README.md" });
  const contextual = page.getByRole("region", {
    name: "Contextual conversation",
  });
  await expect(
    focused.getByRole("textbox", { name: "Edit README.md" }),
  ).toHaveValue(EDITED_FILE);
  await expect(contextual).toHaveAttribute(
    "data-e2e-stable-transcript",
    "task-00008",
  );
  await expect(contextual.getByRole("feed")).toHaveCount(1);
  const contextualFile = contextual.getByRole("article", {
    name: "File response artifact: README.md",
  });
  await expect(
    contextualFile.getByRole("textbox", { name: "Edit README.md" }),
  ).toHaveCount(0);
  await expect(contextualFile).toContainText(
    "This retained draft follows the focused File.",
  );
  await expect(
    contextualFile.getByRole("button", { name: "docs" }),
  ).toBeDisabled();
  await expect(
    page.getByRole("button", { name: "Composer mode" }),
  ).toBeDisabled();
  await page.screenshot({
    path: "output/playwright/task-00022-file-focused-contextual.png",
  });

  await page.getByRole("button", { name: "Restore Chat" }).click();
  await expect(
    page.getByRole("region", { name: "Conversation", exact: true }),
  ).toHaveAttribute("data-e2e-stable-transcript", "task-00008");
  await expect(
    page
      .getByRole("article", {
        name: "File response artifact: README.md",
      })
      .getByRole("textbox", { name: "Edit README.md" }),
  ).toHaveValue(EDITED_FILE);
  expectNoBrowserFailures(failures);
});

test("production Folder stays accessible and contained in the responsive focused layout", async ({
  page,
}) => {
  const failures = captureBrowserFailures(page);
  await installNativeArtifactFixture(page, "folder");
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/#/chat");

  const folder = page.getByRole("article", {
    name: "Folder response artifact: project",
  });
  await expect(folder).toBeVisible();
  await expectNoDocumentOverflow(page);
  const body = folder.locator('[data-artifact-scroll-region="body"]');
  await body.focus();
  await expect(body).toBeFocused();
  await folder.getByRole("button", { name: "Expand" }).focus();
  await page.keyboard.press("Enter");
  await expect(
    page.getByRole("region", { name: "Focused project" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Show project panel" }).click();
  await expect(page.getByLabel("Contextual conversation")).toBeVisible();
  await expectNoDocumentOverflow(page);
  await page.screenshot({
    path: "output/playwright/task-00022-folder-focused-overlay.png",
  });
  expectNoBrowserFailures(failures);
});

test("production Browser shares the focused/contextual shell and restores Chat", async ({
  page,
}) => {
  const failures = captureBrowserFailures(page);
  await installNativeArtifactFixture(page, "browser");
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("/#/chat");

  const browser = page.getByRole("article", {
    name: "Browser response artifact: C4OS documentation",
  });
  await expect(browser).toContainText("https://docs.example.test/c4os");
  await browser.getByRole("button", { name: "Expand" }).click();

  const focused = page.getByRole("region", {
    name: "Focused C4OS documentation",
  });
  await expect(
    focused.getByRole("group", {
      name: "Native Browser viewport: C4OS documentation",
    }),
  ).toBeVisible();
  await expect(page.getByLabel("Contextual conversation")).toBeVisible();
  await page.screenshot({
    path: "output/playwright/task-00022-browser-focused-contextual.png",
  });

  await page.setViewportSize({ width: 700, height: 840 });
  await expectNoDocumentOverflow(page);
  await page.screenshot({
    path: "output/playwright/task-00022-browser-focused-overlay.png",
  });

  await focused.getByRole("button", { name: "Close" }).click();
  await expect(
    page.getByRole("region", { name: "Conversation", exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("article", {
      name: "Browser response artifact: C4OS documentation",
    }),
  ).toBeVisible();
  expectNoBrowserFailures(failures);
});
