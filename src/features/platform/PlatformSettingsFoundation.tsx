import { useState } from "react";
import { Button } from "react-aria-components";

import {
  pickNative,
  type PickerOutcome,
} from "../../platform/platform-service";

export interface NativePlatformSettingsContentProps {
  readonly chooseProjectFolder?: () => Promise<PickerOutcome>;
}

type PickerState =
  | { readonly status: "idle" }
  | { readonly status: "selecting" }
  | { readonly status: "cancelled" }
  | { readonly status: "selected"; readonly displayName: string }
  | { readonly status: "error" };

/**
 * Preserves the Task 00005 native picker boundary as content composed into
 * the single application Settings shell.
 */
export function NativePlatformSettingsContent({
  chooseProjectFolder = () => pickNative("openProjectFolder"),
}: NativePlatformSettingsContentProps) {
  const [pickerState, setPickerState] = useState<PickerState>({
    status: "idle",
  });

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
    <>
      <section className="native-settings__card" aria-labelledby="native-title">
        <div>
          <p className="native-settings__label">macOS foundation</p>
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
          className="native-settings__picker-status"
          data-state={pickerState.status}
          role="status"
        >
          {pickerMessage(pickerState)}
        </p>
      </section>

      <section
        className="native-settings__card"
        aria-labelledby="appearance-title"
      >
        <div>
          <p className="native-settings__label">System appearance</p>
          <h2 id="appearance-title">Follows macOS</h2>
          <p>
            The first visible frame uses a source-qualified native snapshot.
            Later changes arrive through an independent webview listener.
          </p>
        </div>
        <dl className="native-settings__facts">
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
    </>
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
