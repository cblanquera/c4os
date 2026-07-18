import { Navigate, RouterProvider, createHashRouter } from "react-router";

import { WorkspaceStartRoute } from "./features/workspace/WorkspaceStartRoute";
import { QA_FOUNDATION_PATH, QaFoundationRoute } from "./qa/route";
import { QA_POLICY_PATH, QaPolicyRoute } from "./qa/policy-route";
import { QA_WORKSPACE_PATH, QaWorkspaceRoute } from "./qa/workspace-route";

const rootElement =
  import.meta.env.VITE_C4OS_QA_FIXTURES === "1" &&
  import.meta.env.VITE_C4OS_QA_ENTRY === "policy" ? (
    <QaPolicyRoute />
  ) : (
    <WorkspaceStartRoute />
  );

const routes = [
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
];

const router = createHashRouter(routes);

export function App() {
  return <RouterProvider router={router} />;
}
