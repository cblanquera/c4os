import { useLayoutEffect, useRef, useState } from "react";
import {
  Button,
  Menu,
  MenuItem,
  MenuTrigger,
  Popover,
} from "react-aria-components";

import { Icon } from "../../../components/accessible";
import type { ComposerApprovalPreset } from "./types";

export interface ApprovalPresetControlProps {
  readonly isDisabled?: boolean;
  readonly onChange: (preset: ComposerApprovalPreset) => void;
  readonly value: ComposerApprovalPreset;
}

const PRESETS: readonly {
  readonly label: string;
  readonly shortLabel: string;
  readonly value: ComposerApprovalPreset;
}[] = [
  { label: "Ask for approval", shortLabel: "Ask", value: "ask" },
  {
    label: "Approve safe actions",
    shortLabel: "Safe",
    value: "approve-safe",
  },
  {
    label: "Approve for me",
    shortLabel: "For me",
    value: "approve-for-me",
  },
  { label: "Custom", shortLabel: "Custom", value: "custom" },
];

/** Changes the Rust-owned default approval preset from the compact Chat toolbar. */
export function ApprovalPresetControl({
  isDisabled = false,
  onChange,
  value,
}: ApprovalPresetControlProps) {
  const triggerRef = useRef<HTMLButtonElement>(null);
  const shouldRestoreFocus = useRef(false);
  const [isOpen, setIsOpen] = useState(false);
  const selected =
    PRESETS.find((preset) => preset.value === value) ?? PRESETS[0];

  const handleOpenChange = (nextIsOpen: boolean) => {
    shouldRestoreFocus.current = !nextIsOpen;
    setIsOpen(nextIsOpen);
  };

  useLayoutEffect(() => {
    if (!isOpen && shouldRestoreFocus.current) {
      shouldRestoreFocus.current = false;
      triggerRef.current?.focus();
    }
  }, [isOpen]);

  return (
    <MenuTrigger isOpen={isOpen} onOpenChange={handleOpenChange}>
      <Button
        aria-label="Approval preset"
        className="conversation-approval-control__trigger"
        isDisabled={isDisabled}
        ref={triggerRef}
      >
        <Icon aria-hidden="true" name="check" size={14} />
        <span>{selected?.shortLabel ?? "Ask"}</span>
        <Icon aria-hidden="true" name="chevron-down" size={14} />
      </Button>
      <Popover
        className="conversation-approval-control__popover"
        offset={6}
        placement="top start"
      >
        <Menu
          aria-label="Approval presets"
          onAction={(key) => onChange(key as ComposerApprovalPreset)}
          selectedKeys={[value]}
          selectionMode="single"
        >
          {PRESETS.map((preset) => (
            <MenuItem
              className="conversation-approval-control__option"
              id={preset.value}
              key={preset.value}
              textValue={preset.label}
            >
              <span>{preset.label}</span>
              {preset.value === value ? (
                <Icon aria-hidden="true" name="check" size={14} />
              ) : null}
            </MenuItem>
          ))}
        </Menu>
        <p className="conversation-approval-control__note">
          Sandbox and managed limits still apply.
        </p>
      </Popover>
    </MenuTrigger>
  );
}
