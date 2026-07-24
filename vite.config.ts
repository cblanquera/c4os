import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";
import { resolve } from "node:path";

export default defineConfig(() => {
  const qaBuild = process.env.VITE_C4OS_QA_FIXTURES === "1";
  const qaModule = (name: string) =>
    resolve(
      __dirname,
      `src/qa/${name}${qaBuild ? "" : ".production"}${
        name === "build-router" ? ".tsx" : ".ts"
      }`,
    );

  return {
    clearScreen: false,
    plugins: [react()],
    resolve: {
      alias: {
        "#qa-native-transport": qaModule("build-native-transport"),
        "#qa-router": qaModule("build-router"),
        "#qa-store": qaModule("build-store"),
      },
    },
    server: {
      host: "127.0.0.1",
      port: 1420,
      strictPort: true,
    },
    envPrefix: ["VITE_", "TAURI_ENV_"],
    build: {
      target: "safari16",
      // Production app bundles must not ship source maps that retain
      // compile-time-only QA modules or authority markers. QA builds keep maps
      // for deterministic browser/native diagnosis.
      sourcemap: qaBuild,
    },
    test: {
      environment: "jsdom",
      globals: true,
      setupFiles: ["./src/test/setup.ts"],
      include: ["src/**/*.test.{ts,tsx}"],
      coverage: {
        enabled: false,
      },
    },
  };
});
