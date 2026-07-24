import { invokeQaProductRoute as invokeDeterministicProductRoute } from "./product-route-native";

interface ExplicitQaNativeFixture {
  readonly invoke: (
    command: string,
    args: Readonly<Record<string, unknown>>,
  ) => unknown;
}

type ExplicitQaNativeFixtureGlobal = typeof globalThis & {
  readonly __C4OS_QA_NATIVE_FIXTURE__?: "deterministic-e2e";
  readonly __TAURI_INTERNALS__?: ExplicitQaNativeFixture;
};

/**
 * Real Tauri shells keep native menu/window events enabled in fixture builds.
 * The explicit Playwright transport remains isolated from those subscriptions.
 */
export function shouldSubscribeToNativeShellEvents(): boolean {
  const fixtureGlobal = globalThis as ExplicitQaNativeFixtureGlobal;
  return (
    fixtureGlobal.__C4OS_QA_NATIVE_FIXTURE__ !== "deterministic-e2e" &&
    typeof fixtureGlobal.__TAURI_INTERNALS__?.invoke === "function"
  );
}

/**
 * QA builds never fall through to a real native runtime. Playwright may install
 * one explicit in-page fixture transport; every other call uses the build-owned
 * deterministic product adapter.
 */
export function invokeQaProductRoute(
  command: string,
  args: Readonly<Record<string, unknown>>,
): Promise<unknown> {
  const fixtureGlobal = globalThis as ExplicitQaNativeFixtureGlobal;
  if (
    fixtureGlobal.__C4OS_QA_NATIVE_FIXTURE__ === "deterministic-e2e" &&
    typeof fixtureGlobal.__TAURI_INTERNALS__?.invoke === "function"
  ) {
    return Promise.resolve(
      fixtureGlobal.__TAURI_INTERNALS__.invoke(command, args),
    );
  }
  return invokeDeterministicProductRoute(command, args);
}
