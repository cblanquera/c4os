import { Navigate, createHashRouter } from "react-router";

import { ShellRouteController } from "../features/shell/ShellRouteController";
import {
  LaunchRoute,
  ProviderOnboardingRoute,
  ProviderRouteGate,
} from "../features/shell/LaunchRoute";
import { WorkspaceStartRoute } from "../features/workspace/WorkspaceStartRoute";
import { APP_ROUTE_DEFINITIONS } from "./route-contract";
import {
  qaOnlyRoutes,
  resolveQaRootElement,
  wrapQaProductElement,
} from "#qa-router";

const qaRootElement = resolveQaRootElement(import.meta.env.VITE_C4OS_QA_ENTRY);
const qaFixturesEnabled = import.meta.env.VITE_C4OS_QA_FIXTURES === "1";

const rootElement =
  qaRootElement !== null ? qaRootElement : <Navigate replace to="/chat" />;

export const appRouter = createHashRouter([
  {
    path: "/",
    element:
      qaRootElement !== null ? (
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
  ...qaOnlyRoutes,
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
        wrapQaProductElement(route.path, productElement)
      ) : (
        <ProviderRouteGate routePath={route.path}>
          {productElement}
        </ProviderRouteGate>
      ),
    };
  }),
]);

/** Route native app commands through the live router without reloading the webview. */
export function navigateAppRoute(route: string): Promise<void> {
  return appRouter.navigate(route);
}
