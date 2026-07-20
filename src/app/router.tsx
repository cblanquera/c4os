import { Navigate, createHashRouter } from "react-router";

import {
  PlatformSettingsFoundation,
  PlatformThemeQaSurface,
} from "../features/platform";
import { RuntimeQaSurface } from "../features/runtime";
import { WorkspaceStartRoute } from "../features/workspace/WorkspaceStartRoute";
import { QA_POLICY_PATH, QaPolicyRoute } from "../qa/policy-route";
import { QA_FOUNDATION_PATH, QaFoundationRoute } from "../qa/route";
import { QA_WORKSPACE_PATH, QaWorkspaceRoute } from "../qa/workspace-route";

const rootElement =
  import.meta.env.VITE_C4OS_QA_FIXTURES === "1" &&
  ["platform", "policy", "runtime"].includes(
    import.meta.env.VITE_C4OS_QA_ENTRY ?? "",
  ) ? (
    import.meta.env.VITE_C4OS_QA_ENTRY === "platform" ? (
      <PlatformThemeQaSurface />
    ) : import.meta.env.VITE_C4OS_QA_ENTRY === "runtime" ? (
      <RuntimeQaSurface />
    ) : (
      <QaPolicyRoute />
    )
  ) : (
    <WorkspaceStartRoute />
  );

export const appRouter = createHashRouter([
  {
    path: "/",
    element: rootElement,
  },
  {
    path: "/start",
    element: <WorkspaceStartRoute />,
  },
  {
    path: "/foundation",
    element: <Navigate replace to="/" />,
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
  {
    path: "/settings/providers",
    element: <PlatformSettingsFoundation />,
  },
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
