import type { ReactNode } from "react";

import { Button } from "../../../components/accessible";
import type { ShellRoutePath } from "./shell-routes";
import { getShellRouteCopy } from "./shell-routes";

interface RouteSurfaceProps {
  readonly route: ShellRoutePath;
  readonly children?: ReactNode;
}

/** Provides the stable title/support boundary for every direct route. */
export function RouteSurface({ route, children }: RouteSurfaceProps) {
  const copy = getShellRouteCopy(route);
  const titleId = `shell-route-${route.replaceAll("/", "-").slice(1)}-title`;

  return (
    <section
      className="shell-route-surface"
      aria-labelledby={titleId}
      data-route={route}
    >
      <div className="shell-route-surface__header">
        <h1 id={titleId}>{copy.title}</h1>
        <p>{copy.support}</p>
      </div>
      {children === undefined ? (
        <div className="shell-route-surface__placeholder" aria-hidden="true" />
      ) : (
        children
      )}
    </section>
  );
}

interface UnavailableGateProps {
  readonly feature: string;
  readonly children: ReactNode;
}

/** Renders an honest, disabled boundary for explicitly deferred behavior. */
export function UnavailableGate({ feature, children }: UnavailableGateProps) {
  const descriptionId = `unavailable-${feature
    .toLocaleLowerCase()
    .replaceAll(/[^a-z0-9]+/gu, "-")}`;

  return (
    <div className="shell-unavailable" data-feature-gate={feature}>
      <Button isDisabled aria-describedby={descriptionId} variant="quiet">
        {children}
      </Button>
      <span id={descriptionId}>Not available: {feature}.</span>
    </div>
  );
}
