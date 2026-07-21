import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Provider } from "react-redux";

import { App } from "./App";
import { navigateAppRoute } from "./app/router";
import {
  createSettingsVisit,
  readActiveShellFocusTarget,
} from "./app/settings-visit";
import { store } from "./app/store";
import { bootstrapPlatformTheme } from "./features/platform";
import {
  listenForNativeSettings,
  readPlatformSnapshot,
  revealMainWindow,
} from "./platform/platform-service";
import { shellDraftActions } from "./features/shell/state";
import {
  ingestNativeShellProjections,
  nativeResumeRoute,
} from "./features/shell/native-bootstrap";
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
    const currentRoute = window.location.hash.slice(1) || "/start";
    store.dispatch(
      shellDraftActions.settingsVisited(
        createSettingsVisit(
          store.getState(),
          currentRoute,
          readActiveShellFocusTarget(),
        ),
      ),
    );
    void navigateAppRoute(route);
  }).catch(() => undefined);

  createRoot(rendererRoot).render(
    <StrictMode>
      <Provider store={store}>
        <App />
      </Provider>
    </StrictMode>,
  );

  // Snapshot ingestion is independent from first-frame theme authority. Each
  // available native domain publishes atomically; unavailable domains stay
  // fail-closed for their later service-integration owners.
  void ingestNativeShellProjections(store.dispatch).then((result) => {
    const currentRoute = window.location.hash.slice(1) || "/";
    const destination = nativeResumeRoute(
      result,
      store.getState(),
      currentRoute,
    );
    if (destination !== null) void navigateAppRoute(destination);
  });

  requestAnimationFrame(() => {
    void revealMainWindow().catch(() => undefined);
  });
}

void start();
