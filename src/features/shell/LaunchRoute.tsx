import { type ReactNode, useEffect, useState } from "react";
import { Navigate, useNavigate } from "react-router";

import { ProviderOnboarding } from "../settings/providers";
import { readProviderSnapshot } from "../../platform/provider-service";
import { readWorkspaceStartSnapshot } from "../../platform/workspace-start";
import { StartupRecoveryGate } from "../recovery";

type LaunchDecision = "loading" | "onboarding" | "workspace-start" | "error";

type LaunchDecisionState = {
  readonly decision: LaunchDecision;
  readonly routePath: string;
};

/** Resolves first launch from the durable Rust provider authority. */
export function LaunchRoute() {
  return (
    <ProviderRouteGate routePath="/">
      <Navigate replace to="/start" />
    </ProviderRouteGate>
  );
}

/**
 * Enforces the durable provider launch gate for every production deep route.
 * QA builds bypass this wrapper in the router so their direct reconstruction
 * surfaces remain addressable without fabricating native provider authority.
 */
export function ProviderRouteGate({
  children,
  routePath,
}: {
  readonly children: ReactNode;
  readonly routePath: string;
}) {
  return (
    <StartupRecoveryGate>
      <ProviderSetupRouteGate routePath={routePath}>
        <ActiveRecoveryRouteGate routePath={routePath}>
          {children}
        </ActiveRecoveryRouteGate>
      </ProviderSetupRouteGate>
    </StartupRecoveryGate>
  );
}

function ActiveRecoveryRouteGate({
  children,
  routePath,
}: {
  readonly children: ReactNode;
  readonly routePath: string;
}) {
  const [state, setState] = useState<{
    readonly decision: "loading" | "allow" | "recovery" | "error";
    readonly routePath: string;
  }>({ decision: "loading", routePath });
  const [attempt, setAttempt] = useState(0);

  useEffect(() => {
    let active = true;
    if (
      routePath === "/" ||
      routePath === "/start" ||
      routePath === "/onboarding"
    ) {
      return () => {
        active = false;
      };
    }
    void readWorkspaceStartSnapshot().then(
      (snapshot) => {
        if (active) {
          setState({
            decision:
              snapshot.activeRecoveryNotice === null ? "allow" : "recovery",
            routePath,
          });
        }
      },
      () => {
        if (active) setState({ decision: "error", routePath });
      },
    );
    return () => {
      active = false;
    };
  }, [attempt, routePath]);

  const passThrough =
    routePath === "/" || routePath === "/start" || routePath === "/onboarding";
  const decision = passThrough
    ? "allow"
    : state.routePath === routePath
      ? state.decision
      : "loading";
  if (decision === "recovery") return <Navigate replace to="/start" />;
  if (decision === "allow") return children;
  return (
    <main className="launch-gate" aria-labelledby="launch-gate-title">
      <h1 id="launch-gate-title">C4OS</h1>
      <p role={decision === "error" ? "alert" : "status"}>
        {decision === "error"
          ? "C4OS could not read Workspace recovery state. Retry before opening a product route."
          : "Checking Workspace recovery state…"}
      </p>
      {decision === "error" ? (
        <button
          onClick={() => {
            setState({ decision: "loading", routePath });
            setAttempt((value) => value + 1);
          }}
          type="button"
        >
          Try again
        </button>
      ) : null}
    </main>
  );
}

function ProviderSetupRouteGate({
  children,
  routePath,
}: {
  readonly children: ReactNode;
  readonly routePath: string;
}) {
  const [decisionState, setDecisionState] = useState<LaunchDecisionState>({
    decision: "loading",
    routePath,
  });
  const [attempt, setAttempt] = useState(0);

  useEffect(() => {
    let active = true;
    void readProviderSnapshot().then(
      (snapshot) => {
        if (active) {
          setDecisionState({
            decision: snapshot.onboardingCompleted
              ? "workspace-start"
              : "onboarding",
            routePath,
          });
        }
      },
      () => {
        if (active) setDecisionState({ decision: "error", routePath });
      },
    );
    return () => {
      active = false;
    };
  }, [attempt, routePath]);

  // React Router can preserve this gate while changing between sibling product
  // routes. Never apply a decision read for the previous path to the new one:
  // onboarding completion must be re-read before either redirect can fire.
  const decision =
    decisionState.routePath === routePath ? decisionState.decision : "loading";

  if (decision === "onboarding" && routePath !== "/onboarding") {
    return <Navigate replace to="/onboarding" />;
  }
  if (decision === "workspace-start" && routePath === "/onboarding") {
    return <Navigate replace to="/start" />;
  }
  if (decision === "onboarding" || decision === "workspace-start") {
    return children;
  }
  return (
    <main className="launch-gate" aria-labelledby="launch-gate-title">
      <h1 id="launch-gate-title">C4OS</h1>
      <p role={decision === "error" ? "alert" : "status"}>
        {decision === "error"
          ? "C4OS could not read provider setup. Retry before opening a product route."
          : "Checking provider setup…"}
      </p>
      {decision === "error" ? (
        <button
          onClick={() => {
            setDecisionState({ decision: "loading", routePath });
            setAttempt((value) => value + 1);
          }}
          type="button"
        >
          Try again
        </button>
      ) : null}
    </main>
  );
}

/** Composes first-provider onboarding outside the Workspace shell. */
export function ProviderOnboardingRoute() {
  const navigate = useNavigate();
  return (
    <main className="launch-provider-route">
      <ProviderOnboarding
        onComplete={() => void navigate("/start", { replace: true })}
      />
    </main>
  );
}
