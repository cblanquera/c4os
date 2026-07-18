import { Navigate, RouterProvider, createHashRouter } from "react-router";

import { WorkspaceStartRoute } from "./features/workspace/WorkspaceStartRoute";
import { QA_FOUNDATION_PATH, QaFoundationRoute } from "./qa/route";
import { QA_WORKSPACE_PATH, QaWorkspaceRoute } from "./qa/workspace-route";

const routes = [
  {
    path: "/",
    element: <WorkspaceStartRoute />,
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
];

const router = createHashRouter(routes);

export function App() {
  return <RouterProvider router={router} />;
}
