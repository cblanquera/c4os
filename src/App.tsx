import { RouterProvider, createHashRouter } from "react-router";

import { FoundationScreen } from "./features/foundation/FoundationScreen";
import { QA_FOUNDATION_PATH, QaFoundationRoute } from "./qa/route";

const routes = [
  {
    path: "/",
    element: <FoundationScreen />,
  },
  {
    path: QA_FOUNDATION_PATH,
    element: <QaFoundationRoute />,
  },
];

const router = createHashRouter(routes);

export function App() {
  return <RouterProvider router={router} />;
}
