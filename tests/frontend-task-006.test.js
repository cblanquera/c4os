import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { chromium } from "playwright";
import { startFrontendServer } from "./support/browser-harness/server.js";

describe("TASK-006 provider and model management frontend slice", () => {
  let browser;
  let page;
  let server;

  before(async () => {
    server = await startFrontendServer();
    browser = await chromium.launch();
  });

  after(async () => {
    await browser?.close();
    await server?.close();
  });

  it("starts provider and model settings empty before connector records exist", async () => {
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });

    await page.goto(`${server.origin}/#settings-providers`);
    assert.equal(await page.locator(".data-list .data-row").count(), 0);
    assert.doesNotMatch(await page.locator(".settings-main").innerText(), /OpenRouter Review/);

    await page.goto(`${server.origin}/#settings-models`);
    assert.equal(await page.locator(".data-list .data-row").count(), 0);
    assert.doesNotMatch(await page.locator(".settings-main").innerText(), /manual\/review-model|gemini-2\.5-flash-lite/);
  });

  it("returns from start-screen settings back to the start screen even when no provider exists", async () => {
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installProviderModelTauri(page, { noProviders: true });

    await page.goto(`${server.origin}/#app-start`);
    await page.getByText("provider-model-repo").waitFor();
    await page.getByRole("link", { name: "Settings" }).click();
    await page.getByRole("link", { name: "Back to app" }).click();

    await page.waitForFunction(() => window.location.hash === "#app-start");
    await page.getByText("Open a folder to start working").waitFor();
  });

  it("hydrates provider/model settings from connector records and hides raw key material", async () => {
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installProviderModelTauri(page);

    await page.goto(`${server.origin}/#settings-providers`);
    await page.getByText("OpenRouter Review").waitFor();

    const providerText = await page.locator(".settings-main").innerText();
    assert.match(providerText, /https:\/\/openrouter\.ai\/api\/v1/);
    assert.match(providerText, /Key configured/);
    assert.doesNotMatch(providerText, /sk-or-v1/);
    assert.doesNotMatch(providerText, /Mock OpenRouter/);

    await page.goto(`${server.origin}/#settings-models`);
    const modelText = await page.locator(".settings-main").innerText();
    assert.match(modelText, /google\/gemini-2\.5-flash-lite/);
    assert.match(modelText, /manual\/review-model/);
    assert.doesNotMatch(modelText, /mock-fast-coder/);
  });

  it("saves an OpenRouter provider from the accepted add-provider form without keeping the raw key in the UI", async () => {
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installProviderModelTauri(page);

    await page.goto(`${server.origin}/#settings-add-provider`);
    await page.locator("#provider-type").selectOption("OpenRouter");
    await page.locator("#provider-label").fill("OpenRouter Personal");
    await page.locator("#api-key").fill("sk-or-v1-user-entered-secret");
    await page.getByRole("button", { name: "Save Provider" }).click();
    await page.getByText("OpenRouter Personal").waitFor();

    const providerText = await page.locator(".settings-main").innerText();
    assert.match(providerText, /OpenRouter Personal/);
    assert.match(providerText, /Key configured/);
    assert.doesNotMatch(providerText, /OpenRouter Review/);
    assert.doesNotMatch(providerText, /sk-or-v1-user-entered-secret/);
    assert.equal(
      await page.evaluate(() => document.querySelector("#api-key")?.value || ""),
      ""
    );
    assert.deepEqual(await page.evaluate(() => {
      const { providerId, ...saved } = window.__task006SavedProvider;
      return saved;
    }), {
      label: "OpenRouter Personal",
      kind: "OpenRouter",
      apiKey: "sk-or-v1-user-entered-secret"
    });

    await page.goto(`${server.origin}/#settings-models`);
    await page.getByText("openai/gpt-5.2").waitFor();
    const modelText = await page.locator(".settings-main").innerText();
    assert.match(modelText, /anthropic\/claude-sonnet-4.5/);
    assert.doesNotMatch(modelText, /manual\/review-model/);
  });

  it("edits and disables provider records from the existing provider row controls", async () => {
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installProviderModelTauri(page);

    await page.goto(`${server.origin}/#settings-providers`);
    await page.getByRole("button", { name: "Edit OpenRouter Review" }).click();
    await page.locator("#provider-label").waitFor();

    assert.equal(await page.locator("#provider-label").inputValue(), "OpenRouter Review");

    await page.goto(`${server.origin}/#settings-providers`);
    await page.getByRole("button", { name: "OpenRouter Review enabled" }).click();
    await page.getByRole("button", { name: "OpenRouter Review disabled" }).waitFor();
    assert.deepEqual(await page.evaluate(() => window.__task006ProviderEnabled), {
      providerId: "openrouter-review",
      enabled: false
    });
  });

  it("updates and removes provider records from the edit form instead of creating duplicate providers", async () => {
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installProviderModelTauri(page);

    await page.goto(`${server.origin}/#settings-providers`);
    await page.getByRole("button", { name: "Edit OpenRouter Review" }).click();
    await page.waitForFunction(() => document.querySelector(".provider-form")?.dataset.boundProviderSave === "true");
    await page.locator("#provider-label").fill("OpenRouter - Personal");
    await page.getByRole("button", { name: "Save Provider" }).click();
    await page.getByText("OpenRouter - Personal").waitFor();

    let providerLabels = await page.locator(".data-list .data-row strong").evaluateAll((nodes) => nodes.map((node) => node.textContent));
    assert.deepEqual(providerLabels, ["OpenRouter - Personal"]);
    assert.deepEqual(await page.evaluate(() => window.__task006SavedProvider.providerId), "openrouter-review");

    await page.getByRole("button", { name: "Edit OpenRouter - Personal" }).click();
    await page.getByRole("button", { name: "Remove Provider" }).click();
    await page.getByText("OpenRouter - Personal").waitFor({ state: "detached" });

    providerLabels = await page.locator(".data-list .data-row strong").evaluateAll((nodes) => nodes.map((node) => node.textContent));
    assert.deepEqual(providerLabels, []);
    assert.deepEqual(await page.evaluate(() => window.__task006DeletedProvider), {
      providerId: "openrouter-review"
    });
  });

  it("models use enabled or disabled status, support search/provider filtering, and only enabled models appear in the composer picker", async () => {
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installProviderModelTauri(page);

    await page.goto(`${server.origin}/#settings-models`);
    const initialText = await page.locator(".settings-main").innerText();
    assert.doesNotMatch(initialText, /Current/);

    await page.getByLabel("Search models").fill("manual");
    await expectVisibleModels(page, ["manual/review-model"]);
    await page.getByLabel("Filter models by provider").selectOption("OpenRouter Review");
    await expectVisibleModels(page, ["manual/review-model"]);

    await page.goto(`${server.origin}/#new-session`);
    await page.locator(".empty-workspace [data-local-picker-trigger='models']").click();
    const pickerText = await page.locator("[data-local-picker='models']").innerText();
    assert.match(pickerText, /google\/gemini-2\.5-flash-lite/);
    assert.doesNotMatch(pickerText, /manual\/review-model/);
  });

  it("enables all discovered provider models by default after discovery refresh", async () => {
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installProviderModelTauri(page);

    await page.goto(`${server.origin}/#settings-add-provider`);
    await page.locator("#provider-type").selectOption("OpenRouter");
    await page.locator("#provider-label").fill("OpenRouter Personal");
    await page.locator("#api-key").fill("sk-or-v1-user-entered-secret");
    await page.getByRole("button", { name: "Save Provider" }).click();
    await page.getByText("OpenRouter Personal").waitFor();

    await page.goto(`${server.origin}/#settings-models`);
    await page.getByText("anthropic/claude-sonnet-4.5").waitFor();
    const statuses = await page.locator(".data-list .status-pill").evaluateAll((nodes) => nodes.map((node) => node.textContent));
    assert.deepEqual(statuses, ["Enabled", "Enabled"]);

    await page.getByRole("button", { name: "anthropic/claude-sonnet-4.5 enabled" }).click();
    await page.getByRole("button", { name: "anthropic/claude-sonnet-4.5 disabled" }).waitFor();
    assert.equal(
      await page.getByRole("button", { name: "anthropic/claude-sonnet-4.5 disabled" }).getAttribute("aria-pressed"),
      "false"
    );
  });

  it("bulk toggles the currently filtered model results and preserves those choices when filters clear", async () => {
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installProviderModelTauri(page);

    await page.goto(`${server.origin}/#settings-add-provider`);
    await page.locator("#provider-type").selectOption("OpenRouter");
    await page.locator("#provider-label").fill("OpenRouter Personal");
    await page.locator("#api-key").fill("sk-or-v1-user-entered-secret");
    await page.getByRole("button", { name: "Save Provider" }).click();
    await page.getByText("OpenRouter Personal").waitFor();

    await page.goto(`${server.origin}/#settings-models`);
    await page.getByLabel("Search models").fill("anthropic");
    await expectVisibleModels(page, ["anthropic/claude-sonnet-4.5"]);
    await page.getByRole("button", { name: "Disable results" }).click();
    await page.getByRole("button", { name: "anthropic/claude-sonnet-4.5 disabled" }).waitFor();

    await page.getByLabel("Search models").fill("");
    await page.getByLabel("Filter models by provider").selectOption("All providers");
    const rows = await page.locator(".data-list .data-row").evaluateAll((nodes) => nodes.map((node) => ({
      label: node.querySelector("strong")?.textContent,
      status: node.querySelector(".status-pill")?.textContent
    })));
    assert.deepEqual(rows, [
      { label: "openai/gpt-5.2", status: "Enabled" },
      { label: "anthropic/claude-sonnet-4.5", status: "Disabled" }
    ]);
  });

  it("keeps the composer model picker bounded when many enabled models are available", async () => {
    page = await browser.newPage({ viewport: { width: 940, height: 760 } });
    await installProviderModelTauri(page, { manyModels: true });

    await page.goto(`${server.origin}/#new-session`);
    await page.locator(".empty-workspace [data-local-picker-trigger='models']").click();

    const scrollArea = page.locator("[data-local-picker='models'] .popover-scroll");
    await scrollArea.waitFor();
    const metrics = await scrollArea.evaluate((node) => ({
      clientHeight: node.clientHeight,
      scrollHeight: node.scrollHeight,
      overflowY: getComputedStyle(node).overflowY
    }));

    assert.equal(metrics.overflowY, "auto");
    assert.ok(metrics.clientHeight < metrics.scrollHeight);
    assert.ok(metrics.clientHeight <= 520);
  });

  it("keeps model selection scoped to the new chat being started", async () => {
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installProviderModelTauri(page, { extraModels: true });

    await page.goto(`${server.origin}/#new-session`);
    await page.getByRole("button", { name: /Model:/ }).click();
    await page.locator("[data-local-model='openrouter/alpha']").click();
    await page.locator(".prompt-box").fill("First chat");
    await page.getByRole("button", { name: "Send Prompt" }).click();
    await page.getByText("Manual model selected.").waitFor();
    await page.waitForFunction(() => window.__task006Calls?.sentPromptModel === "openrouter/alpha");
    await page.waitForTimeout(450);

    await page.goto(`${server.origin}/#new-session`);
    await page.getByRole("button", { name: /Model:/ }).click();
    await page.locator("[data-local-model='openrouter/beta']").click();
    await page.locator(".prompt-box").fill("Second chat");
    await page.getByRole("button", { name: "Send Prompt" }).click();
    await page.getByText("Manual model selected.").last().waitFor();

    assert.deepEqual(await page.evaluate(() => window.__task006Calls.createdSessionModels), [
      "openrouter/alpha",
      "openrouter/beta"
    ]);

    await page.locator("[data-session-target='First chat']").click();
    await page.locator(".topbar strong", { hasText: "First chat" }).waitFor();
    let sessionState = await page.evaluate(() => ({
      model: document.querySelector(".readonly-chip[aria-label='Model locked for this chat'] span:last-child")?.textContent,
      thread: document.querySelector(".thread-list")?.textContent
    }));
    assert.equal(sessionState.model, "openrouter/alpha");
    assert.match(sessionState.thread, /First chat/);
    assert.doesNotMatch(sessionState.thread, /Second chat/);

    await page.locator("[data-session-target='Second chat']").click();
    await page.locator(".topbar strong", { hasText: "Second chat" }).waitFor();
    sessionState = await page.evaluate(() => ({
      model: document.querySelector(".readonly-chip[aria-label='Model locked for this chat'] span:last-child")?.textContent,
      thread: document.querySelector(".thread-list")?.textContent
    }));
    assert.equal(sessionState.model, "openrouter/beta");
    assert.match(sessionState.thread, /Second chat/);
    assert.doesNotMatch(sessionState.thread, /First chat/);
  });

  it("keeps provider and model picker headers fixed and aligned outside the scroll area", async () => {
    page = await browser.newPage({ viewport: { width: 940, height: 760 } });
    await installProviderModelTauri(page, { manyModels: true });

    await page.goto(`${server.origin}/#new-session`);
    await page.getByRole("button", { name: /Model:/ }).click();
    await page.getByRole("button", { name: "Back to providers" }).click();

    const providerHeader = await page.locator("[data-local-picker='providers'] header").evaluate((node) => ({
      text: node.textContent.trim(),
      left: Math.round(node.querySelector("strong").getBoundingClientRect().left),
      firstRowLeft: Math.round(node.parentElement.querySelector(".popover-row span").getBoundingClientRect().left),
      top: node.getBoundingClientRect().top,
      afterScrollTop: (() => {
        node.parentElement.querySelector(".popover-scroll")?.scrollTo(0, 240);
        return node.getBoundingClientRect().top;
      })()
    }));
    assert.equal(providerHeader.text, "Providers");
    assert.equal(providerHeader.left, providerHeader.firstRowLeft);
    assert.equal(providerHeader.top, providerHeader.afterScrollTop);

    await page.getByRole("button", { name: "OpenRouter Review" }).click();
    const modelHeader = await page.locator("[data-local-picker='models'] header").evaluate((node) => {
      const row = node.parentElement.querySelector(".popover-row span");
      const back = node.querySelector(".popover-back");
      const label = node.querySelector("strong");
      const top = node.getBoundingClientRect().top;
      node.parentElement.querySelector(".popover-scroll")?.scrollTo(0, 320);
      return {
        left: Math.round(back.querySelector("svg").getBoundingClientRect().left),
        rowLeft: Math.round(row.getBoundingClientRect().left),
        gap: Math.round(label.getBoundingClientRect().left - back.querySelector("svg").getBoundingClientRect().right),
        top,
        afterScrollTop: node.getBoundingClientRect().top
      };
    });
    assert.equal(modelHeader.left, modelHeader.rowLeft);
    assert.ok(modelHeader.gap >= 6 && modelHeader.gap <= 16);
    assert.equal(modelHeader.top, modelHeader.afterScrollTop);
  });


  it("enables a manual model and sends the selected per-session model through the composer path", async () => {
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installProviderModelTauri(page);

    await page.goto(`${server.origin}/#settings-models`);
    await page.getByRole("button", { name: "manual/review-model disabled" }).click();
    await page.getByText("Enabled").last().waitFor();

    await page.goto(`${server.origin}/#new-session`);
    await page.getByRole("button", { name: /Model:/ }).click();
    await page.getByRole("button", { name: "manual/review-model" }).click();
    await page.locator(".prompt-box").fill("Use the manual model for this session");
    await page.getByRole("button", { name: "Send Prompt" }).click();
    await page.getByText("Manual model selected.").waitFor();

    assert.deepEqual(await page.evaluate(() => {
      const { enabledModelIds, createdSessionModels, ...calls } = window.__task006Calls;
      return calls;
    }), {
      enabledModelId: "manual-review-model",
      createdSessionModel: "manual/review-model",
      sentPromptModel: "manual/review-model"
    });
    await expectNoSecretLeak(page);
  });

  it("keeps TASK-005A scoped composer picker behavior while replacing fixture records", async () => {
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installProviderModelTauri(page);

    await page.goto(`${server.origin}/#new-session`);
    await page.getByText("provider-model-repo").first().waitFor();
    await page.waitForTimeout(50);
    assert.equal(await markNode(page, ".app-shell", "task006Shell"), true);
    assert.equal(await markNode(page, ".composer", "task006Composer"), true);

    await page.getByRole("button", { name: /Model:/ }).click();
    await page.getByRole("button", { name: "Back to providers" }).click();

    assert.deepEqual(
      await page.evaluate(() => ({
        hash: location.hash,
        shellKept: document.querySelector(".app-shell")?.task006Shell === true,
        composerKept: document.querySelector(".composer")?.task006Composer === true,
        providerRows: Array.from(document.querySelectorAll("[data-local-provider]")).map((row) => row.textContent.trim())
      })),
      {
        hash: "#new-session",
        shellKept: true,
        composerKept: true,
      providerRows: ["OpenRouter Review"]
      }
    );
  });
});

async function installProviderModelTauri(page, options = {}) {
  await page.addInitScript((nextOptions) => {
    window.__task006Options = nextOptions;
  }, options);
  await page.addInitScript(() => {
    const provider = {
      id: "openrouter-review",
      label: "OpenRouter Review",
      kind: "OpenAI-compatible",
      baseUrl: "https://openrouter.ai/api/v1",
      endpoint: "https://openrouter.ai/api/v1",
      status: "Key configured",
      keyStatus: { state: "present", source: "environment" },
      enabled: true,
      supportsDiscovery: true
    };
    let models = [
      { id: "gemini-flash-lite", label: "google/gemini-2.5-flash-lite", providerId: "openrouter-review", provider: "OpenRouter Review", enabled: true, active: true, source: "discovered" },
      { id: "manual-review-model", label: "manual/review-model", providerId: "openrouter-review", provider: "OpenRouter Review", enabled: false, active: false, source: "manual" }
    ];
    if (window.__task006Options?.extraModels) {
      models = [
        { id: "alpha", label: "openrouter/alpha", providerId: "openrouter-review", provider: "OpenRouter Review", enabled: true, active: false, source: "discovered" },
        { id: "beta", label: "openrouter/beta", providerId: "openrouter-review", provider: "OpenRouter Review", enabled: true, active: false, source: "discovered" }
      ];
    }
    if (window.__task006Options?.manyModels) {
      models = Array.from({ length: 28 }, (_, index) => ({
        id: `model-${index}`,
        label: `openrouter/model-${index}`,
        providerId: "openrouter-review",
        provider: "OpenRouter Review",
        enabled: true,
        active: index === 0,
        source: "discovered"
      }));
    }
    const shellPayload = {
      workspace: {
        project: "provider-model-repo",
        session: "Provider model review",
        branch: "main",
        model: "google/gemini-2.5-flash-lite"
      },
      projects: [{ name: "provider-model-repo", sessions: ["Provider model review"] }],
      providers: window.__task006Options?.noProviders ? [] : [provider],
      models,
      pluginCatalog: [],
      pluginMarketplaces: [{ label: "Mock C4OS Marketplace", summary: "Mock configured marketplace", active: true }],
      skillCatalog: [],
      mcpServers: [],
      browser: { url: "http://127.0.0.1/mock", title: "Mock rendered page", summary: "Mock Browser state" },
      files: { roots: [], breadcrumbs: ["provider-model-repo"], lines: [] },
      terminal: { output: "mock terminal", title: "Mock terminal", summary: "Mock output" },
      thread: {
        user: "Existing accepted turn.",
        agent: "Existing assistant answer.",
        extra: "Existing extra detail.",
        tool: "Previous reasoning",
        run: "Previous run complete"
      }
    };
    window.__task006Calls = {};
    window.__TAURI__ = {
      core: {
        invoke: async (command, payload = {}) => {
          if (command === "load_workspace") return shellPayload;
          if (command === "list_provider_profiles") return shellPayload.providers;
          if (command === "list_provider_models") return models;
          if (command === "save_provider_profile") {
            window.__task006SavedProvider = {
              providerId: payload.request.providerId,
              label: payload.request.label,
              kind: payload.request.kind,
              apiKey: payload.request.apiKey
            };
            provider.id = payload.request.providerId || "openrouter-personal";
            provider.label = payload.request.label;
            provider.status = "Key configured";
            provider.keyStatus = { state: "present", source: "session" };
            shellPayload.providers = [provider];
            models = [
              { id: "openai-gpt-5-2", label: "openai/gpt-5.2", providerId: provider.id, provider: payload.request.label, enabled: true, active: false, source: "discovered" },
              { id: "anthropic-claude-sonnet-4-5", label: "anthropic/claude-sonnet-4.5", providerId: provider.id, provider: payload.request.label, enabled: true, active: false, source: "discovered" }
            ];
            return provider;
          }
          if (command === "delete_provider_profile") {
            window.__task006DeletedProvider = payload.request;
            shellPayload.providers = shellPayload.providers.filter((record) => record.id !== payload.request.providerId);
            return { providerId: payload.request.providerId, deleted: true };
          }
          if (command === "set_provider_enabled") {
            provider.enabled = payload.request.enabled;
            window.__task006ProviderEnabled = payload.request;
            return provider;
          }
          if (command === "set_model_enabled") {
            const record = models.find((model) => model.id === payload.request.modelId);
            record.enabled = payload.request.enabled;
            window.__task006Calls.enabledModelId = payload.request.modelId;
            window.__task006Calls.enabledModelIds ||= [];
            window.__task006Calls.enabledModelIds.push(payload.request.modelId);
            return record;
          }
          if (command === "select_session_model") {
            window.__task006Calls.selectedSessionModel = payload.request.model;
            shellPayload.workspace.model = payload.request.model;
            models.forEach((model) => model.active = model.label === payload.request.model);
            return { session: payload.request.session, model: payload.request.model };
          }
          if (command === "create_session") {
            window.__task006Calls.createdSessionModel = payload.model;
            window.__task006Calls.createdSessionModels ||= [];
            window.__task006Calls.createdSessionModels.push(payload.model);
            return { id: "session-manual", project: payload.project, status: "created" };
          }
          if (command === "send_prompt") {
            window.__task006Calls.sentPromptModel = payload.model;
            return {
              agent: "Manual model selected.",
              prompt: payload.prompt,
              run: "Provider/model run complete",
              events: [{ kind: "activity", text: `Selected ${payload.model}`, sequence: 1 }]
            };
          }
          return { ok: true };
        }
      },
      event: {
        listen: async () => () => {}
      }
    };
  });
}

async function expectVisibleModels(page, labels) {
  const rows = await page.locator(".data-list .data-row strong").evaluateAll((nodes) => nodes.map((node) => node.textContent));
  assert.deepEqual(rows, labels);
}

async function markNode(page, selector, property) {
  return page.evaluate(({ selector, property }) => {
    const node = document.querySelector(selector);
    if (!node) return false;
    node[property] = true;
    return true;
  }, { selector, property });
}

async function expectNoSecretLeak(page) {
  const content = await page.content();
  assert.doesNotMatch(content, /sk-or-v1/);
  const frontendData = await readFile(new URL("../frontend/data.js", import.meta.url), "utf8");
  assert.doesNotMatch(frontendData, /sk-(or-v1|proj)-[A-Za-z0-9]/);
}
