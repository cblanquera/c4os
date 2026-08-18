import { expect, test, type Page } from "@playwright/test";

type ConversationState =
  "normal" | "pending" | "streaming" | "failure" | "reply";

const IMAGE_REFERENCE = `workspace-blob:sha256:${"a".repeat(64)}:v1`;
const IMAGE_PREVIEW =
  "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAusB9Wl2n1cAAAAASUVORK5CYII=";

async function installNativeConversationFixture(
  page: Page,
  state: ConversationState,
) {
  await page.addInitScript(
    ({ initialSnapshot, imagePreview }) => {
      let snapshot = initialSnapshot as Record<string, unknown>;
      let nextCallbackId = 1;
      const callbacks = new Map<
        number,
        {
          readonly callback: (payload: unknown) => void;
          readonly once: boolean;
        }
      >();
      const envelope = (
        request: Record<string, unknown>,
        generation: number,
        payload: unknown,
      ) => ({
        protocolVersion: 1,
        requestId: request.requestId,
        correlationId: request.correlationId,
        generation,
        payload,
      });
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
        async invoke(
          command: string,
          args: Record<string, unknown> = {},
        ): Promise<unknown> {
          if (command === "plugin:event|listen") return nextCallbackId++;
          if (command === "plugin:event|unlisten") return null;
          const request = args.request as Record<string, unknown>;
          if (command === "platform_snapshot") {
            return envelope(request, 0, {
              contractVersion: 1,
              platform: "macos",
              architecture: "aarch64",
              initialTheme: {
                scheme: "dark",
                source: "macosAppearance",
              },
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
            return envelope(request, snapshot.generation as number, snapshot);
          }
          if (command === "artifact_snapshot") {
            return envelope(request, 0, {
              protocolVersion: 1,
              generation: 0,
              authority: "rust-core",
              workspaceId: snapshot.workspaceId,
              activeProjectId: snapshot.activeProjectId,
              activeSessionId: snapshot.activeSessionId,
              focusedArtifactId: null,
              artifacts: [],
            });
          }
          if (command === "conversation_attachment_preview") {
            const input = args.input as Record<string, unknown>;
            return envelope(request, snapshot.generation as number, {
              attachmentId: input.attachmentId,
              mediaType: "image/png",
              dataUrl: imagePreview,
            });
          }
          if (command === "conversation_update_draft") {
            const input = args.input as Record<string, unknown>;
            const nextGeneration = (snapshot.generation as number) + 1;
            snapshot = {
              ...snapshot,
              generation: nextGeneration,
              draft: {
                ...(snapshot.draft as Record<string, unknown>),
                prompt: input.prompt,
                providerId: input.providerId,
                modelId: input.modelId,
                reasoningMode: input.reasoningMode,
                mode: input.mode,
                replyTargetId: input.replyTargetId,
              },
              models: (snapshot.models as Record<string, unknown>[]).map(
                (model) => ({
                  ...model,
                  selected:
                    model.providerId === input.providerId &&
                    model.modelId === input.modelId,
                }),
              ),
            };
            return envelope(request, nextGeneration, snapshot);
          }
          throw new Error(`Fixture does not provide ${command}`);
        },
      };
      (
        window as unknown as {
          __TAURI_INTERNALS__: typeof internals;
          __C4OS_QA_NATIVE_FIXTURE__: "deterministic-e2e";
        }
      ).__TAURI_INTERNALS__ = internals;
      (
        window as unknown as {
          __C4OS_QA_NATIVE_FIXTURE__: "deterministic-e2e";
        }
      ).__C4OS_QA_NATIVE_FIXTURE__ = "deterministic-e2e";
    },
    {
      initialSnapshot: conversationSnapshot(state),
      imagePreview: IMAGE_PREVIEW,
    },
  );
}

function conversationSnapshot(state: ConversationState) {
  const attachment = {
    attachmentId: "attachment-qa-image",
    displayName: "concept-preview.png",
    mediaType: "image/png",
    byteLength: 68,
    stableReference: IMAGE_REFERENCE,
    originalReference: 1,
  };
  const attemptStatus =
    state === "streaming"
      ? "working"
      : state === "failure"
        ? "failed"
        : "completed";
  const hasAttempt = state !== "pending";
  const attemptId = "attempt-qa-1";
  return {
    protocolVersion: 1,
    generation: 61,
    authority: "rust-core",
    workspaceId: "workspace:qa",
    workspaceName: "C4OS QA Workspace",
    activeProjectId: "project:qa",
    activeSessionId: state === "pending" ? "chat-pending" : "session:qa",
    pending:
      state === "pending"
        ? {
            sessionId: "chat-pending",
            projectId: "project:qa",
            title: "New Chat",
            attachments: [],
          }
        : null,
    draft: {
      prompt:
        state === "normal"
          ? "Review the attached concept"
          : state === "reply"
            ? "Continue this durable Reply after restart"
            : "",
      attachments: state === "normal" ? [attachment] : [],
      nextAttachmentReference: state === "normal" ? 2 : 1,
      providerId: "openai",
      modelId: "openai/gpt-5-vision",
      reasoningMode: "medium",
      mode: "chat",
      replyTargetId: state === "reply" ? "turn-qa-1" : null,
    },
    projects: [
      {
        projectId: "project:qa",
        displayName: "AI Desktop UI",
        pathState: "found",
        position: 0,
        gitVersioned: true,
      },
      {
        projectId: "project:missing",
        displayName: "Archived Prototype",
        pathState: "missing",
        position: 1,
        gitVersioned: false,
      },
    ],
    sessions: [
      {
        sessionId: "session:qa",
        projectId: "project:qa",
        title: "Build attachment previews",
        updatedAtMs: 2_000,
      },
      {
        sessionId: "session:second",
        projectId: "project:qa",
        title: "Refine model controls",
        updatedAtMs: 1_000,
      },
    ],
    activeConversation: hasAttempt
      ? {
          sessionId: "session:qa",
          title: "Build attachment previews",
          turns: [
            {
              turnId: "turn-qa-1",
              prompt: "Build the **image preview** and keep it safe.",
              attachments: [attachment],
              artifactContext: null,
              mcpProvenance: null,
              submittedAtMs: 100,
            },
          ],
          attempts: [
            {
              attemptId,
              turnId: "turn-qa-1",
              status: attemptStatus,
              assistantMarkdown:
                state === "failure"
                  ? ""
                  : state === "streaming"
                    ? "Verifying the immutable blob…"
                    : "The **bounded preview** is ready.",
              activities:
                state === "failure"
                  ? [
                      {
                        sequence: 1,
                        kind: "error",
                        label: "Preview verification failed safely",
                        detail: null,
                      },
                    ]
                  : [
                      {
                        sequence: 1,
                        kind: "work",
                        label: "Verified immutable attachment content",
                        detail: null,
                      },
                      {
                        sequence: 2,
                        kind: "reasoning-summary",
                        label: "Reasoning summary",
                        detail:
                          "Checked the safe preview boundary without exposing private reasoning.",
                      },
                    ],
              runtimeId: "runtime:qa-opencode",
              runtimeKind: "open-code",
              environmentId: "local",
              providerId: "openai",
              modelId: "openai/gpt-5-vision",
              adapterId: "adapter:opencode",
              inputTokens: 1_200,
              outputTokens: state === "streaming" ? 120 : 480,
              durationMs: state === "streaming" ? null : 4_200,
            },
          ],
          activeAttemptId: state === "streaming" ? attemptId : null,
        }
      : null,
    models: [
      {
        providerId: "openai",
        providerName: "OpenAI",
        modelId: "openai/gpt-5-vision",
        selected: true,
        available: true,
        supportsVision: true,
        supportsTools: true,
        supportsReasoning: true,
        supportsAudio: false,
        contextTokens: 400_000,
      },
      {
        providerId: "openai",
        providerName: "OpenAI",
        modelId: "openai/gpt-5-text",
        selected: false,
        available: true,
        supportsVision: false,
        supportsTools: true,
        supportsReasoning: false,
        supportsAudio: false,
        contextTokens: 200_000,
      },
    ],
    branchControl: {
      currentBranch: "main",
      branches: [
        { name: "main", targetOid: "1".repeat(40), selected: true },
        {
          name: "feature/preview",
          targetOid: "2".repeat(40),
          selected: false,
        },
      ],
      pendingApprovalId: null,
      operationStatus: null,
      operationMessage: null,
    },
  };
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

test("production Chat renders the normal, preview, model, information, and search paths", async ({
  page,
}) => {
  const consoleErrors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error") consoleErrors.push(message.text());
  });
  await installNativeConversationFixture(page, "normal");
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/#/chat");

  await expect(
    page.getByRole("banner", { name: "C4OS window title" }),
  ).toContainText("Build attachment previews");
  await expect(
    page.getByRole("img", { name: "Preview of concept-preview.png" }),
  ).toBeVisible();
  await expect(
    page.getByRole("list", { name: "Submitted attachments" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Model", exact: true }).click();
  await expect(
    page.getByRole("option", { name: /openai\/gpt-5-text/i }),
  ).toBeVisible();
  await page.screenshot({
    path: "output/playwright/task-00022-chat-model-browser.png",
  });
  await page.keyboard.press("Escape");
  await page.getByRole("button", { name: "Chat information" }).click();
  await expect(page.getByRole("dialog")).toContainText("Healthy runtime");
  await page.screenshot({
    path: "output/playwright/task-00022-chat-information.png",
  });
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).toBeHidden();
  await expectNoDocumentOverflow(page);
  await page.screenshot({
    path: "tests/results/playwright/task-00007-chat-normal.png",
    fullPage: true,
  });

  await expect(page.getByText("Run activity", { exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "Expand" })).toHaveCount(0);
  await expect(page.getByLabel("Contextual conversation")).toHaveCount(0);
  await expect(
    page.getByRole("region", { name: "Conversation", exact: true }),
  ).toBeVisible();

  const chatSearch = page.getByRole("searchbox", {
    name: "Search chat sessions",
  });
  await chatSearch.fill("model");
  await expect(
    page.getByRole("heading", { name: "Search results", level: 2 }),
  ).toBeVisible();
  await expect(page.getByText("Refine model controls")).toBeVisible();
  await page.screenshot({
    path: "tests/results/playwright/task-00007-chat-search.png",
    fullPage: true,
  });
  await chatSearch.press("Escape");
  await expect(
    page.getByRole("heading", { name: "Projects", level: 2 }),
  ).toBeVisible();
  expect(consoleErrors).toEqual([]);
});

for (const state of ["pending", "streaming", "failure"] as const) {
  test(`production Chat renders the ${state} state without overflow`, async ({
    page,
  }) => {
    await installNativeConversationFixture(page, state);
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/#/chat");
    if (state === "pending") {
      await expect(
        page.getByText("What do you want to build in AI Desktop UI?"),
      ).toBeVisible();
    } else if (state === "streaming") {
      await expect(page.getByLabel("C4OS response")).toHaveAttribute(
        "aria-busy",
        "true",
      );
      await expect(
        page.getByRole("button", { name: "Stop response" }),
      ).toBeVisible();
    } else {
      await expect(
        page.getByRole("button", { name: "Retry response" }),
      ).toBeVisible();
      await page.getByRole("button", { name: /Activity/i }).click();
      await expect(
        page.getByText("Preview verification failed safely"),
      ).toBeVisible();
    }
    await expectNoDocumentOverflow(page);
    await page.screenshot({
      path: `tests/results/playwright/task-00007-chat-${state}.png`,
      fullPage: true,
    });
  });
}

test("production Chat remains contained in the responsive overlay layout", async ({
  page,
}) => {
  await installNativeConversationFixture(page, "normal");
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/#/chat");
  await expect(
    page.getByRole("form", { name: "Message composer" }),
  ).toBeVisible();
  await expectNoDocumentOverflow(page);
  await page.getByRole("button", { name: "Show project panel" }).click();
  await expect(
    page.getByRole("navigation", { name: "Projects and chat sessions" }),
  ).toBeVisible();
  await expectNoDocumentOverflow(page);
  await page.screenshot({
    path: "tests/results/playwright/task-00007-chat-responsive.png",
    fullPage: true,
  });
});

test("production Chat preserves a durable Reply target through autosave and reload", async ({
  page,
}) => {
  await installNativeConversationFixture(page, "reply");
  await page.goto("/#/chat");

  const reply = page.getByRole("region", { name: "Reply reference" });
  await expect(reply).toContainText(
    "Build the **image preview** and keep it safe.",
  );
  await page.waitForTimeout(700);
  await expect(reply).toBeVisible();
  await page.screenshot({
    path: "output/playwright/task-00022-chat-reply.png",
  });

  await page.reload();
  await expect(
    page.getByRole("region", { name: "Reply reference" }),
  ).toContainText("Build the **image preview** and keep it safe.");
});
