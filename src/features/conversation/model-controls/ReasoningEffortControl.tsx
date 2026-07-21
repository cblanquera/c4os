import type { ChangeEvent } from "react";

import type { ReasoningEffort } from "./types";

export interface ReasoningEffortControlProps {
  readonly isDisabled?: boolean;
  readonly onChange: (effort: ReasoningEffort) => void;
  readonly options?: readonly ReasoningEffort[];
  readonly value: ReasoningEffort | null;
}

const EFFORT_ORDER: readonly ReasoningEffort[] = [
  "off",
  "low",
  "medium",
  "high",
];

/** Renders the controlled reasoning selector only for supported routes. */
export function ReasoningEffortControl({
  isDisabled = false,
  onChange,
  options = [],
  value,
}: ReasoningEffortControlProps) {
  const supportedOptions = EFFORT_ORDER.filter((effort) =>
    options.includes(effort),
  );
  if (supportedOptions.length === 0) {
    return null;
  }

  const selectedValue =
    value && supportedOptions.includes(value) ? value : supportedOptions[0];

  /** Reports only an allowed reasoning effort from the native selector. */
  const handleChange = (event: ChangeEvent<HTMLSelectElement>) => {
    const effort = event.currentTarget.value as ReasoningEffort;
    if (supportedOptions.includes(effort)) {
      onChange(effort);
    }
  };

  return (
    <label className="conversation-reasoning-control">
      <span className="conversation-reasoning-control__label">Reasoning</span>
      <select
        aria-label="Reasoning effort"
        disabled={isDisabled}
        onChange={handleChange}
        value={selectedValue}
      >
        {supportedOptions.map((effort) => (
          <option key={effort} value={effort}>
            {effortLabel(effort)}
          </option>
        ))}
      </select>
    </label>
  );
}

/** Formats the accepted reasoning effort labels. */
function effortLabel(effort: ReasoningEffort) {
  return `${effort.charAt(0).toUpperCase()}${effort.slice(1)}`;
}
