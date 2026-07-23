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

/** Provides an explicit, non-mutating check of the native folder picker. */
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
    <section className="native-settings__card" aria-labelledby="native-title">
      <div>
        <p className="native-settings__label">macOS integration</p>
        <h2 id="native-title">Folder picker check</h2>
        <p>
          Confirm that the native folder picker is available. This check does
          not open, add, or change a Project.
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
  );
}

function pickerMessage(state: PickerState): string {
  switch (state.status) {
    case "idle":
      return "The native folder picker has not been checked.";
    case "selecting":
      return "Waiting for the macOS folder picker.";
    case "cancelled":
      return "Folder selection cancelled. No access was granted.";
    case "selected":
      return `The native picker returned ${state.displayName}. No Project was changed.`;
    case "error":
      return "Native folder access is unavailable. Restart C4OS and try again.";
  }
}
