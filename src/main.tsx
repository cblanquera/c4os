import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Provider } from "react-redux";

import { App } from "./App";
import { navigateAppRoute } from "./app/router";
import { store } from "./app/store";
import { bootstrapPlatformTheme } from "./features/platform";
import {
  listenForNativeSettings,
  readPlatformSnapshot,
  revealMainWindow,
} from "./platform/platform-service";
import "./styles.css";
import "./features/platform/platform-theme.css";

const root = document.getElementById("root");

if (!root) {
  throw new Error("C4OS renderer root is missing");
}
const rendererRoot = root;

async function start() {
  await bootstrapPlatformTheme({ readNativeSnapshot: readPlatformSnapshot });
  void listenForNativeSettings((route) => {
    void navigateAppRoute(route);
  }).catch(() => undefined);

  createRoot(rendererRoot).render(
    <StrictMode>
      <Provider store={store}>
        <App />
      </Provider>
    </StrictMode>,
  );

  requestAnimationFrame(() => {
    void revealMainWindow().catch(() => undefined);
  });
}

void start();
