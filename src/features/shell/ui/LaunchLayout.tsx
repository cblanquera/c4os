import type { ReactNode } from "react";

import { BrandMark } from "../../../components/accessible";
import type { LaunchRoutePath } from "./shell-routes";
import { RouteSurface } from "./RouteSurface";

interface LaunchLayoutProps {
  readonly route: LaunchRoutePath;
  readonly children?: ReactNode;
}

/** Renders onboarding or workspace start without introducing another document. */
export function LaunchLayout({ route, children }: LaunchLayoutProps) {
  return (
    <div className="shell-view shell-launch" data-shell-layout="launch">
      <header className="shell-title-header" aria-label="C4OS window title">
        <BrandMark label="C4OS" />
      </header>
      <main className="shell-launch__main">
        <RouteSurface route={route}>{children}</RouteSurface>
      </main>
    </div>
  );
}
