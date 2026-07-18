import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Provider } from "react-redux";

import { App } from "./App";
import { store } from "./app/store";
import "./styles.css";

const root = document.getElementById("root");

if (!root) {
  throw new Error("C4OS renderer root is missing");
}

createRoot(root).render(
  <StrictMode>
    <Provider store={store}>
      <App />
    </Provider>
  </StrictMode>,
);
