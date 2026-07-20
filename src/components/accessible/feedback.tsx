import type { HTMLAttributes, ReactNode } from "react";

import { Icon } from "./icons";

export type NoticeTone = "neutral" | "success" | "warning" | "danger";

export interface NoticeProps extends Omit<
  HTMLAttributes<HTMLElement>,
  "title"
> {
  readonly children?: ReactNode;
  readonly title: ReactNode;
  readonly tone?: NoticeTone;
}

export interface StatusRegionProps extends HTMLAttributes<HTMLDivElement> {
  readonly politeness?: "polite" | "assertive";
}

export interface LiveRegionProps extends StatusRegionProps {
  readonly isVisuallyHidden?: boolean;
}

/** Renders a semantic notice whose icon and label duplicate its color state. */
export function Notice({
  children,
  className,
  title,
  tone = "neutral",
  ...props
}: NoticeProps) {
  const iconName =
    tone === "success" ? "check" : tone === "neutral" ? "info" : "alert";

  return (
    <section
      {...props}
      className={["c4-notice", `c4-notice--${tone}`, className]
        .filter(Boolean)
        .join(" ")}
      role={tone === "danger" ? "alert" : "status"}
    >
      <Icon name={iconName} />
      <div>
        <strong>{title}</strong>
        {children ? <div className="c4-notice__detail">{children}</div> : null}
      </div>
    </section>
  );
}

/** Renders visible status copy that announces complete state changes. */
export function StatusRegion({
  children,
  className,
  politeness = "polite",
  ...props
}: StatusRegionProps) {
  return (
    <div
      {...props}
      aria-atomic="true"
      aria-live={politeness}
      className={["c4-status-region", className].filter(Boolean).join(" ")}
      role="status"
    >
      {children}
    </div>
  );
}

/** Renders an atomic live region, hidden visually by default. */
export function LiveRegion({
  className,
  isVisuallyHidden = true,
  ...props
}: LiveRegionProps) {
  return (
    <StatusRegion
      {...props}
      className={[
        "c4-live-region",
        isVisuallyHidden ? "c4-visually-hidden" : null,
        className,
      ]
        .filter(Boolean)
        .join(" ")}
    />
  );
}
