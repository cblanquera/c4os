import type { ReactNode } from "react";
import { Navigate, type RouteObject } from "react-router";

import { PlatformThemeQaSurface } from "../features/platform";
import { RuntimeQaSurface } from "../features/runtime";
import { QaFixtureBoundary } from "./QaFixtureBoundary";
import { QaPolicyRoute } from "./policy-route";
import { QaFoundationRoute } from "./route";
import { QaWorkspaceRoute } from "./workspace-route";
import { resolveBuildGatedQaRootEntry } from "../app/qa-entry";
import type { AppRoutePath } from "../app/route-contract";

export function resolveQaRootElement(candidate: string | undefined): ReactNode {
  const entry = resolveBuildGatedQaRootEntry(true, candidate);
  if (entry === "platform") return <PlatformThemeQaSurface />;
  if (entry === "runtime") return <RuntimeQaSurface />;
  if (entry === "policy") return <QaPolicyRoute />;
  if (entry === "chat") return <Navigate replace to="/chat" />;
  if (entry === "foundation") return <Navigate replace to="/qa/foundation" />;
  return null;
}

export function wrapQaProductElement(
  route: AppRoutePath,
  element: ReactNode,
): ReactNode {
  return <QaFixtureBoundary route={route}>{element}</QaFixtureBoundary>;
}

export const qaOnlyRoutes: readonly RouteObject[] = [
  { path: "/qa/foundation", element: <QaFoundationRoute /> },
  { path: "/qa/workspace", element: <QaWorkspaceRoute /> },
  { path: "/qa/policy", element: <QaPolicyRoute /> },
  { path: "/qa/platform", element: <PlatformThemeQaSurface /> },
  { path: "/qa/runtime", element: <RuntimeQaSurface /> },
];
