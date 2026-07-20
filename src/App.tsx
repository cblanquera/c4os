import { RouterProvider } from "react-router";

import { appRouter } from "./app/router";

export function App() {
  return <RouterProvider router={appRouter} />;
}
