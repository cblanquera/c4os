import { useState } from "react";
import { Button } from "react-aria-components";
import { useNavigate } from "react-router";

import {
  getNativeSettingsReturnRoute,
  pickNative,
  type PickerOutcome,
} from "../../platform/platform-service";

interface PlatformSettingsFoundationProps {
  readonly chooseProjectFolder?: () => Promise<PickerOutcome>;
  readonly resolveBackRoute?: () => string | null;
}

type PickerState =
  | { readonly status: "idle" }
  | { readonly status: "selecting" }
  | { readonly status: "cancelled" }
  | { readonly status: "selected"; readonly displayName: string }
  | { readonly status: "error" };

const SETTINGS_DESTINATIONS = [
  ["Providers", "P"],
  ["Models", "M"],
  ["Runtimes", "R"],
  ["Configuration", "C"],
  ["Plugins", "▧"],
  ["Skills", "S"],
  ["MCP Servers", "⇄"],
] as const;

/**
 * Production Settings entry for the Task 00005 platform foundation. Later
 * service-integration tasks fill the destinations without replacing this
 * stateful route, native entry, navigation geometry, or picker boundary.
 */
export function PlatformSettingsFoundation({
  chooseProjectFolder = () => pickNative("openProjectFolder"),
  resolveBackRoute = getNativeSettingsReturnRoute,
}: PlatformSettingsFoundationProps) {
  const navigate = useNavigate();
  const [pickerState, setPickerState] = useState<PickerState>({
    status: "idle",
  });

  const back = () => {
    navigate(resolveBackRoute() ?? "/");
  };

  const selectProjectFolder = async () => {
    setPickerState({ status: "selecting" });
    try {
      const outcome = await chooseProjectFolder();
      if (outcome.type === "cancelled") {
        setPickerState({ status: "cancelled" });
        return;
      }
      setPickerState({
        status: "selected",
        displayName: outcome.grants[0]?.displayName ?? "Selected folder",
      });
    } catch {
      setPickerState({ status: "error" });
    }
  };

  return (
    <main className="settings-foundation" aria-labelledby="settings-title">
      <nav className="settings-foundation__navigation" aria-label="Settings">
        <Button className="settings-foundation__back" onPress={back}>
          <span aria-hidden="true">‹</span>
          <span className="settings-foundation__back-label">Back to C4OS</span>
        </Button>
        <div className="settings-foundation__brand" aria-hidden="true">
          C4
        </div>
        <ul>
          {SETTINGS_DESTINATIONS.map(([label, symbol], index) => (
            <li
              key={label}
              aria-current={index === 0 ? "page" : undefined}
              className={
                index === 4 ? "settings-foundation__divider" : undefined
              }
            >
              <span
                className="settings-foundation__nav-symbol"
                aria-hidden="true"
              >
                {symbol}
              </span>
              <span className="settings-foundation__nav-label">{label}</span>
            </li>
          ))}
        </ul>
      </nav>

      <section className="settings-foundation__scroll">
        <div className="settings-foundation__content">
          <header className="settings-foundation__header">
            <p>Settings</p>
            <h1 id="settings-title">Providers</h1>
            <span>
              Manage the native platform boundary used by provider and workspace
              setup.
            </span>
          </header>

          <section
            className="settings-foundation__card"
            aria-labelledby="native-title"
          >
            <div>
              <p className="settings-foundation__label">macOS foundation</p>
              <h2 id="native-title">Native project access</h2>
              <p>
                C4OS asks macOS to choose a folder and returns only a one-use,
                opaque access grant to the renderer.
              </p>
            </div>
            <Button
              className="platform-button platform-button--primary"
              isDisabled={pickerState.status === "selecting"}
              onPress={() => void selectProjectFolder()}
            >
              {pickerState.status === "selecting"
                ? "Choosing…"
                : "Choose project folder…"}
            </Button>
            <p
              className="settings-foundation__picker-status"
              data-state={pickerState.status}
              role="status"
            >
              {pickerMessage(pickerState)}
            </p>
          </section>

          <section
            className="settings-foundation__card"
            aria-labelledby="appearance-title"
          >
            <div>
              <p className="settings-foundation__label">System appearance</p>
              <h2 id="appearance-title">Follows macOS</h2>
              <p>
                The first visible frame uses a source-qualified native snapshot.
                Later changes arrive through an independent webview listener.
              </p>
            </div>
            <dl className="settings-foundation__facts">
              <div>
                <dt>Shortcut</dt>
                <dd>⌘,</dd>
              </div>
              <div>
                <dt>Reveal action</dt>
                <dd>Reveal in Finder</dd>
              </div>
              <div>
                <dt>Window</dt>
                <dd>Standard decorations</dd>
              </div>
            </dl>
          </section>
        </div>
      </section>
    </main>
  );
}

function pickerMessage(state: PickerState): string {
  switch (state.status) {
    case "idle":
      return "No native folder has been selected.";
    case "selecting":
      return "Waiting for the macOS folder picker.";
    case "cancelled":
      return "Folder selection cancelled. No access was granted.";
    case "selected":
      return `Access granted to ${state.displayName}.`;
    case "error":
      return "Native folder access is unavailable. Restart C4OS and try again.";
  }
}
