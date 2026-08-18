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

async function installNativeMcpFixture(page: Page) {
  await page.addInitScript(() => {
    type JsonRecord = Record<string, unknown>;

    let nextCallbackId = 1;
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
    const platformSnapshot = {
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
        revealFallbackTimeoutMs: 1_500,
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
    };
    const digest = `sha256:${"a".repeat(64)}`;
    const mcpSnapshot = {
      schemaVersion: 1,
      generation: 4,
      servers: [
        {
          serverId: "docs.server",
          displayName: "Documentation server",
          source: { kind: "user" },
          scope: { kind: "application" },
          transport: {
            kind: "stdio",
            command: "/usr/bin/docs-mcp",
            arguments: ["--stdio"],
            environment: [
              { name: "LANG", source: { kind: "passthrough" } },
              {
                name: "DOCS_TOKEN",
                source: {
                  kind: "secret",
                  reference: {
                    kind: "environment",
                    variable: "DOCS_MCP_TOKEN",
                  },
                },
              },
            ],
            workingDirectory: { kind: "c4osHome" },
            executableSha256: digest,
          },
          trust: "trusted",
          trustedDefinitionSha256: digest,
          pendingTrustApproval: null,
          lifecycle: "ready",
          timeoutMs: 30_000,
          maxOutputBytes: 1_048_576,
          lifecycleGeneration: 3,
          restartAttempts: 0,
          nextRestartAtMs: null,
          protocolVersion: "2025-11-25",
          serverName: "docs-fixture",
          serverVersion: "1.0.0",
          instructionsPresent: false,
          capabilities: {
            tools: true,
            toolListChanged: false,
            resources: true,
            resourceListChanged: false,
            resourceSubscribe: false,
            prompts: false,
            logging: false,
            completions: false,
            tasks: false,
            experimentalKeys: [],
          },
          tools: [
            {
              name: "echo",
              title: null,
              description: "Echo a bounded input.",
              inputSchema: {
                type: "object",
                properties: { text: { type: "string" } },
                required: ["text"],
                additionalProperties: false,
              },
              inputSchemaSha256: digest,
              outputSchemaSha256: null,
            },
          ],
          resources: [
            {
              uri: "c4os://fixture/readme",
              name: "Fixture readme",
              title: null,
              description: null,
              mimeType: "text/plain",
              size: 32,
            },
          ],
          activeRequests: 0,
          lastConnectedAtMs: 12_000,
          lastFailureCode: null,
          lastFailureDetail: null,
          lastEventId: 7,
        },
      ],
      activeWorkers: 1,
      lastEventId: 7,
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
      async invoke(
        command: string,
        args: Record<string, unknown> = {},
      ): Promise<unknown> {
        if (command === "plugin:event|listen") return nextCallbackId++;
        if (command === "plugin:event|unlisten") return null;
        const request = args.request as JsonRecord;
        if (command === "platform_snapshot") {
          return envelope(request, 0, platformSnapshot);
        }
        if (command === "platform_reveal_main") {
          return envelope(request, 0, { revealed: true, fallback: false });
        }
        if (command === "mcp_snapshot") {
          return envelope(request, 4, mcpSnapshot);
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
  });
}

test("contains production MCP Settings without renderer failures", async ({
  page,
}) => {
  const failures = captureBrowserFailures(page);
  await installNativeMcpFixture(page);
  await page.setViewportSize({ width: 1_100, height: 720 });
  await page.goto("/#/settings/mcp");

  await expect(
    page.getByRole("heading", { name: "MCP Servers", level: 1 }),
  ).toBeVisible();
  await expect(page.getByText("1 active server process")).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Documentation server" }),
  ).toBeVisible();
  await expectNoDocumentOverflow(page);

  const details = page.getByRole("button", { name: "Details" });
  await details.focus();
  await details.click();
  const dialog = page.getByRole("dialog", { name: "Documentation server" });
  await expect(dialog).toContainText("Environment: DOCS_MCP_TOKEN");
  await expect(dialog).not.toContainText("must-not-render");
  await expectNoDocumentOverflow(page);
  await page.keyboard.press("Escape");
  await expect(details).toBeFocused();

  await page.setViewportSize({ width: 622, height: 720 });
  await expect(
    page.getByRole("heading", { name: "Documentation server" }),
  ).toBeVisible();
  await expectNoDocumentOverflow(page);
  expect(failures.console).toEqual([]);
  expect(failures.page).toEqual([]);
});
