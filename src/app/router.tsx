import { Navigate, createHashRouter } from "react-router";

import { PlatformThemeQaSurface } from "../features/platform";
import { RuntimeQaSurface } from "../features/runtime";
import { ShellRouteController } from "../features/shell/ShellRouteController";
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
      qaRootEntry !== null ? rootElement : <Navigate replace to="/start" />,
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
  ...APP_ROUTE_DEFINITIONS.map((route) => ({
    path: route.path,
    element: <ShellRouteController route={route.path} />,
  })),
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
