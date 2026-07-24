export type QaAwareAuthority<ProductionAuthority extends string> =
  ProductionAuthority | "qa-fixture-only";

/**
 * Accepts fixture-tagged payloads only in the compile-time QA build.
 *
 * Production builds continue to require the exact service-owned authority.
 */
export function isAcceptedQaAwareAuthority<ProductionAuthority extends string>(
  value: unknown,
  productionAuthority: ProductionAuthority,
): value is QaAwareAuthority<ProductionAuthority> {
  if (value === productionAuthority) return true;
  if (
    import.meta.env.VITE_C4OS_QA_FIXTURES !== "1" &&
    import.meta.env.MODE !== "test"
  ) {
    return false;
  }
  return value === "qa-fixture-only";
}
