import type { SVGProps } from "react";

export type IconName =
  | "add"
  | "alert"
  | "chat"
  | "check"
  | "chevron-down"
  | "chevron-right"
  | "close"
  | "file"
  | "folder"
  | "globe"
  | "info"
  | "more"
  | "project"
  | "search"
  | "settings"
  | "terminal";

export interface IconProps extends Omit<SVGProps<SVGSVGElement>, "children"> {
  readonly name: IconName;
  readonly size?: number;
}

export interface BrandMarkProps extends Omit<
  SVGProps<SVGSVGElement>,
  "children"
> {
  readonly label?: string;
  readonly size?: number;
}

const ICON_PATHS: Record<IconName, React.ReactNode> = {
  add: <path d="M9 3.75v10.5M3.75 9h10.5" />,
  alert: (
    <>
      <path d="M7.4 3.2 2.1 12.4a1.1 1.1 0 0 0 1 1.6h10.6a1.1 1.1 0 0 0 1-1.6L9.4 3.2a1.15 1.15 0 0 0-2 0Z" />
      <path d="M8.4 6.4v3.1M8.4 12.2h.01" />
    </>
  ),
  chat: (
    <path d="M3.1 3.5h10.8a1.6 1.6 0 0 1 1.6 1.6v6.3a1.6 1.6 0 0 1-1.6 1.6H8l-3.6 2.2V13H3.1a1.6 1.6 0 0 1-1.6-1.6V5.1a1.6 1.6 0 0 1 1.6-1.6Z" />
  ),
  check: <path d="m3.4 9.1 3.1 3.1 7.9-8" />,
  "chevron-down": <path d="m4 6.5 5 5 5-5" />,
  "chevron-right": <path d="m6.5 4 5 5-5 5" />,
  close: <path d="M4.2 4.2 13.8 13.8M13.8 4.2 4.2 13.8" />,
  file: (
    <>
      <path d="M4 1.8h6l4 4v10.4H4Z" />
      <path d="M10 1.8v4h4M6.5 9h5M6.5 12h5" />
    </>
  ),
  folder: (
    <path d="M2 5.3h5l1.4 1.5H16v6.7a1.5 1.5 0 0 1-1.5 1.5h-11A1.5 1.5 0 0 1 2 13.5V5.3Z" />
  ),
  globe: (
    <>
      <circle cx="9" cy="9" r="6.8" />
      <path d="M2.5 9h13M9 2.2c2 2 3 4.2 3 6.8s-1 4.8-3 6.8M9 2.2C7 4.2 6 6.4 6 9s1 4.8 3 6.8" />
    </>
  ),
  info: (
    <>
      <circle cx="9" cy="9" r="7" />
      <path d="M9 8v4M9 5.5h.01" />
    </>
  ),
  more: (
    <>
      <circle cx="3.5" cy="9" r=".7" fill="currentColor" stroke="none" />
      <circle cx="9" cy="9" r=".7" fill="currentColor" stroke="none" />
      <circle cx="14.5" cy="9" r=".7" fill="currentColor" stroke="none" />
    </>
  ),
  project: (
    <path d="M2 5.3h5l1.4 1.5H16v6.7a1.5 1.5 0 0 1-1.5 1.5h-11A1.5 1.5 0 0 1 2 13.5V5.3Z" />
  ),
  search: (
    <>
      <circle cx="7.8" cy="7.8" r="5.2" />
      <path d="m11.6 11.6 3.8 3.8" />
    </>
  ),
  settings: (
    <>
      <circle cx="9" cy="9" r="2.5" />
      <path d="M7.4 2.2h3.2l.5 2a5.3 5.3 0 0 1 1.2.7l2-.6 1.6 2.8-1.5 1.4a5.5 5.5 0 0 1 0 1.4l1.5 1.4-1.6 2.8-2-.6a5.3 5.3 0 0 1-1.2.7l-.5 2H7.4l-.5-2a5.3 5.3 0 0 1-1.2-.7l-2 .6-1.6-2.8 1.5-1.4a5.5 5.5 0 0 1 0-1.4L2.1 7.1l1.6-2.8 2 .6a5.3 5.3 0 0 1 1.2-.7l.5-2Z" />
    </>
  ),
  terminal: (
    <>
      <rect height="12.5" rx="2" width="14" x="2" y="2.75" />
      <path d="m5 6 2.2 2.2L5 10.4M9.2 11h3.4" />
    </>
  ),
};

/** Renders one coherent 18px outline icon using the current text color. */
export function Icon({ name, size = 18, ...props }: IconProps) {
  return (
    <svg
      {...props}
      aria-hidden={props["aria-label"] ? undefined : true}
      className={["c4-icon", props.className].filter(Boolean).join(" ")}
      fill="none"
      height={size}
      role={props["aria-label"] ? "img" : undefined}
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth="1.7"
      viewBox="0 0 18 18"
      width={size}
    >
      {ICON_PATHS[name]}
    </svg>
  );
}

/** Renders the compact C4OS product mark without competing with task content. */
export function BrandMark({ label, size = 28, ...props }: BrandMarkProps) {
  return (
    <svg
      {...props}
      aria-hidden={label ? undefined : true}
      aria-label={label}
      className={["c4-brand-mark", props.className].filter(Boolean).join(" ")}
      height={size}
      role={label ? "img" : undefined}
      viewBox="0 0 28 28"
      width={size}
    >
      <rect height="24" rx="7" width="24" x="2" y="2" />
      <path d="M12.2 9.2a5.1 5.1 0 1 0 0 9.6M17.2 8.8v10.4M15.1 15.5h5.3" />
    </svg>
  );
}
