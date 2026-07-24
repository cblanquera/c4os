import type { PropsWithChildren } from "react";

import type { AppRoutePath } from "../app/route-contract";

export interface QaFixtureBoundaryProps extends PropsWithChildren {
  readonly route: AppRoutePath;
}

/**
 * Keeps a visible authority label on every product-composed QA destination.
 * This component is mounted only by the compile-time fixture branch.
 */
export function QaFixtureBoundary({ children, route }: QaFixtureBoundaryProps) {
  return (
    <div
      className="qa-fixture-boundary"
      data-qa-product-adapter="deterministic"
      data-qa-route={route}
    >
      <div
        className="qa-fixture-boundary__identity"
        aria-label="QA fixture identity"
        role="note"
      >
        Deterministic QA fixture data · not production state
      </div>
      {children}
    </div>
  );
}
