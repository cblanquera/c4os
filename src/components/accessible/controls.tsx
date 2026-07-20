import type { ReactNode } from "react";
import type {
  ButtonProps as AriaButtonProps,
  SwitchProps as AriaSwitchProps,
} from "react-aria-components";
import {
  Button as AriaButton,
  Switch as AriaSwitch,
  Tooltip,
  TooltipTrigger,
} from "react-aria-components";

export type ButtonVariant = "primary" | "secondary" | "quiet" | "danger";

export interface ButtonProps extends Omit<AriaButtonProps, "className"> {
  readonly className?: string;
  readonly variant?: ButtonVariant;
}

export interface IconButtonProps extends Omit<
  ButtonProps,
  "aria-label" | "children"
> {
  readonly icon: ReactNode;
  readonly label: string;
  readonly tooltip?: string;
  readonly tooltipPlacement?: "top" | "bottom" | "left" | "right";
}

export interface SwitchProps extends Omit<AriaSwitchProps, "className"> {
  readonly className?: string;
  readonly description?: ReactNode;
  readonly label: ReactNode;
}

/** Renders the shared keyboard and pointer button surface. */
export function Button({
  className,
  variant = "secondary",
  ...props
}: ButtonProps) {
  return (
    <AriaButton
      {...props}
      className={["c4-button", `c4-button--${variant}`, className]
        .filter(Boolean)
        .join(" ")}
    />
  );
}

/** Renders a named icon action with a focus and hover accessible tooltip. */
export function IconButton({
  className,
  icon,
  label,
  tooltip = label,
  tooltipPlacement = "top",
  variant = "quiet",
  ...props
}: IconButtonProps) {
  return (
    <TooltipTrigger closeDelay={0} delay={0}>
      <Button
        {...props}
        aria-label={label}
        className={["c4-icon-button", className].filter(Boolean).join(" ")}
        variant={variant}
      >
        {icon}
      </Button>
      <Tooltip className="c4-tooltip" placement={tooltipPlacement}>
        {tooltip}
      </Tooltip>
    </TooltipTrigger>
  );
}

/** Renders a binary setting with a visible label and optional description. */
export function Switch({
  className,
  description,
  label,
  ...props
}: SwitchProps) {
  return (
    <AriaSwitch
      {...props}
      className={["c4-switch", className].filter(Boolean).join(" ")}
    >
      <span aria-hidden="true" className="c4-switch__track">
        <span className="c4-switch__thumb" />
      </span>
      <span className="c4-switch__copy">
        <span className="c4-switch__label">{label}</span>
        {description ? (
          <span className="c4-switch__description">{description}</span>
        ) : null}
      </span>
    </AriaSwitch>
  );
}
