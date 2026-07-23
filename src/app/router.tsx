import { Navigate, createHashRouter } from "react-router";

import { PlatformThemeQaSurface } from "../features/platform";
import { RuntimeQaSurface } from "../features/runtime";
import { ShellRouteController } from "../features/shell/ShellRouteController";
import {
  LaunchRoute,
  ProviderOnboardingRoute,
  ProviderRouteGate,
} from "../features/shell/LaunchRoute";
import { WorkspaceStartRoute } from "../features/workspace/WorkspaceStartRoute";
import { QA_POLICY_PATH, QaPolicyRoute } from "../qa/policy-route";
import { QA_FOUNDATION_PATH, QaFoundationRoute } from "../qa/route";
import { QA_WORKSPACE_PATH, QaWorkspaceRoute } from "../qa/workspace-route";
import { resolveBuildGatedQaRootEntry } from "./qa-entry";
import { APP_ROUTE_DEFINITIONS } from "./route-contract";

const qaRootEntry = resolveBuildGatedQaRootEntry(
  import.meta.env.VITE_C4OS_QA_FIXTURES === "1",
  import.meta.env.VITE_C4OS_QA_ENTRY,
);
const qaFixturesEnabled = import.meta.env.VITE_C4OS_QA_FIXTURES === "1";

const rootElement =
  qaRootEntry !== null ? (
    qaRootEntry === "platform" ? (
      <PlatformThemeQaSurface />
    ) : qaRootEntry === "runtime" ? (
      <RuntimeQaSurface />
    ) : qaRootEntry === "policy" ? (
      <QaPolicyRoute />
    ) : (
      <Navigate replace to="/chat" />
    )
  ) : (
    <WorkspaceStartRoute />
  );

export const appRouter = createHashRouter([
  {
    path: "/",
    element:
      qaRootEntry !== null ? (
        rootElement
      ) : qaFixturesEnabled ? (
        <Navigate replace to="/start" />
      ) : (
        <LaunchRoute />
      ),
  },
  {
    path: "/foundation",
    element: <Navigate replace to="/start" />,
  },
  {
    path: QA_FOUNDATION_PATH,
    element: <QaFoundationRoute />,
  },
  {
    path: QA_WORKSPACE_PATH,
    element: <QaWorkspaceRoute />,
  },
  {
    path: QA_POLICY_PATH,
    element: <QaPolicyRoute />,
  },
  ...APP_ROUTE_DEFINITIONS.map((route) => {
    const productElement =
      route.path === "/onboarding" ? (
        <ProviderOnboardingRoute />
      ) : route.path === "/start" ? (
        <WorkspaceStartRoute />
      ) : (
        <ShellRouteController route={route.path} />
      );
    return {
      path: route.path,
      element: qaFixturesEnabled ? (
        route.path === "/onboarding" || route.path === "/start" ? (
          <div data-qa-product-adapter="deterministic" data-route={route.path}>
            {productElement}
          </div>
        ) : (
          <ShellRouteController route={route.path} />
        )
      ) : (
        <ProviderRouteGate routePath={route.path}>
          {productElement}
        </ProviderRouteGate>
      ),
    };
  }),
  {
    path: "/qa/platform",
    element: <PlatformThemeQaSurface />,
  },
  {
    path: "/qa/runtime",
    element: <RuntimeQaSurface />,
  },
]);

/** Route native app commands through the live router without reloading the webview. */
export function navigateAppRoute(route: string): Promise<void> {
  return appRouter.navigate(route);
}
