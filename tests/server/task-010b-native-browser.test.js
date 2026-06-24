import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

describe("TASK-010B raw Wry native Browser boundary", () => {
  it("uses raw Wry for the public Browser host and does not expose an external-open fallback", async () => {
    const cargoToml = await readFile(new URL("../../backend/Cargo.toml", import.meta.url), "utf8");
    const libSource = await readFile(new URL("../../backend/src/lib.rs", import.meta.url), "utf8");
    const connectorContract = await readFile(new URL("../../frontend/connector-contract.js", import.meta.url), "utf8");
    const connectorSource = await readFile(new URL("../../frontend/connectors.js", import.meta.url), "utf8");
    const appSource = await readFile(new URL("../../frontend/app.js", import.meta.url), "utf8");
    const nativeBrowserSource = await readFile(new URL("../../backend/src/native_browser.rs", import.meta.url), "utf8");

    assert.match(cargoToml, /^wry = "0\.55\.1"$/m);
    assert.match(cargoToml, /^raw-window-handle = "0\.6"$/m);
    assert.match(nativeBrowserSource, /WebViewBuilder::new\(\)/);
    assert.match(nativeBrowserSource, /native_browser_host\(&window, bounds\)/);
    assert.match(nativeBrowserSource, /\.contentView\(\)/);
    assert.match(nativeBrowserSource, /NSView::initWithFrame/);
    assert.match(nativeBrowserSource, /setClipsToBounds\(true\)/);
    assert.match(nativeBrowserSource, /native_browser_child_bounds\(bounds\)/);
    assert.match(nativeBrowserSource, /RawWindowHandle::AppKit/);
    assert.match(nativeBrowserSource, /\.build_as_child\(&host\.parent\)/);
    assert.doesNotMatch(nativeBrowserSource, /\.build_as_child\(&window\)/);
    assert.match(nativeBrowserSource, /\.with_navigation_handler/);
    assert.match(nativeBrowserSource, /NewWindowResponse::Deny/);
    assert.doesNotMatch(nativeBrowserSource, /with_ipc_handler/);
    assert.match(libSource, /native_browser::sync_native_browser/);
    assert.match(connectorContract, /"syncNativeBrowser"/);
    assert.match(connectorSource, /sync_native_browser/);
    assert.match(appSource, /data-native-browser-frame/);
    assert.doesNotMatch(appSource, /Open externally/);
    assert.doesNotMatch(connectorContract, /openExternalBrowser/);
    assert.doesNotMatch(connectorSource, /open_external_browser/);
    assert.doesNotMatch(libSource, /open_external_browser/);
  });
});
