import type { ReactNode } from "react";
import type { RouteObject } from "react-router";

import type { AppRoutePath } from "../app/route-contract";

export function resolveQaRootElement(_candidate: string | undefined): null {
  void _candidate;
  return null;
}

export function wrapQaProductElement(
  _route: AppRoutePath,
  element: ReactNode,
): ReactNode {
  return element;
}

export const qaOnlyRoutes: readonly RouteObject[] = [];
