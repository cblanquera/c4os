import type { ReactNode } from "react";

import { Button } from "../../../components/accessible";
import type { SettingsRoutePath, ShellRoutePath } from "./shell-routes";
import { SETTINGS_DESTINATIONS } from "./shell-routes";
import { RouteSurface } from "./RouteSurface";

interface SettingsLayoutProps {
  readonly route: SettingsRoutePath;
  readonly children?: ReactNode;
  readonly onNavigate: (route: ShellRoutePath) => void;
  readonly onBack: () => void;
}

/** Renders the ordered, responsive Settings shell around service-owned pages. */
export function SettingsLayout({
  route,
  children,
  onNavigate,
  onBack,
}: SettingsLayoutProps) {
  const selectedPath =
    route === "/settings/advanced-policies" ? "/settings/configuration" : route;

  return (
    <div className="shell-view shell-settings" data-shell-layout="settings">
      <div className="shell-settings__body">
        <nav className="shell-settings__navigation" aria-label="Settings">
          <Button
            className="shell-settings__back"
            onPress={onBack}
            variant="quiet"
          >
            <span aria-hidden="true">‹</span>
            <span className="shell-settings__back-label">Back to C4OS</span>
          </Button>
          <p className="shell-settings__navigation-label">Settings</p>
          <ul>
            {SETTINGS_DESTINATIONS.map((destination, index) => (
              <li
                key={destination.path}
                className={index === 4 ? "shell-settings__divider" : undefined}
              >
                <Button
                  aria-label={destination.label}
                  data-compressed-label={destination.symbol}
                  {...(destination.path === selectedPath
                    ? { "aria-current": "page" as const }
                    : {})}
                  onPress={() => onNavigate(destination.path)}
                  variant="quiet"
                >
                  <span
                    className="shell-settings__nav-symbol"
                    aria-hidden="true"
                  >
                    {destination.symbol}
                  </span>
                  <span className="shell-settings__nav-label">
                    {destination.label}
                  </span>
                </Button>
              </li>
            ))}
          </ul>
        </nav>

        <main className="shell-settings__scroll">
          <div className="shell-settings__content">
            {route === "/settings/advanced-policies" ? (
              <PolicyLayout>
                <RouteSurface route={route}>{children}</RouteSurface>
              </PolicyLayout>
            ) : (
              <RouteSurface route={route}>{children}</RouteSurface>
            )}
          </div>
        </main>
      </div>
    </div>
  );
}

interface PolicyLayoutProps {
  readonly children: ReactNode;
}

/** Provides the responsive container reserved for Advanced Policies. */
export function PolicyLayout({ children }: PolicyLayoutProps) {
  return (
    <section
      className="shell-policy-layout"
      aria-label="Advanced policy configuration"
      data-shell-layout="policy"
    >
      {children}
    </section>
  );
}
