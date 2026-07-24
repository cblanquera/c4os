import { useEffect, useState } from "react";
import {
  Button,
  Dialog,
  DialogTrigger,
  Heading,
  Modal,
  ModalOverlay,
} from "react-aria-components";

import {
  readRootAppearance,
  subscribeToRootAppearance,
  type RootAppearance,
} from "./theme";
import {
  setPlatformReviewState,
  type PlatformReviewState,
  usePlatformReviewState,
} from "./review-state";
import { NativePlatformSettingsContent } from "./PlatformSettingsFoundation";
import "./platform-theme.css";

interface PlatformThemeQaSurfaceProps {
  readonly appearance?: RootAppearance;
  readonly isEnabled?: boolean;
}

const REVIEW_STATES: readonly PlatformReviewState[] = [
  "Default",
  "Hover",
  "Pressed",
  "Selected",
  "Disabled",
  "Dirty",
  "Loading",
];

const TOKEN_SWATCHES = [
  ["Window", "--surface-window"],
  ["Sidebar", "--surface-sidebar"],
  ["Raised", "--surface-raised"],
  ["Sunken", "--surface-sunken"],
  ["Overlay", "--surface-overlay"],
] as const;

function useObservedAppearance(): RootAppearance {
  const [appearance, setAppearance] = useState(readRootAppearance);

  useEffect(() => subscribeToRootAppearance(setAppearance), []);
  return appearance;
}

function labelPlatform(platform: RootAppearance["platform"]): string {
  if (platform === "macos") return "macOS";
  if (platform === "windows") return "Windows";
  return "Linux";
}

function labelAppearanceSource(source: RootAppearance["source"]): string {
  if (source === "macosAppearance") return "macOS appearance";
  if (source === "webviewPreferredColorScheme") {
    return "Webview launch preference";
  }
  if (source === "webviewPrefersColorScheme") {
    return "Webview live preference";
  }
  return "Semantic fallback";
}

function stateData(state: PlatformReviewState): string | undefined {
  return state === "Default" ? undefined : state.toLowerCase();
}

/** Deterministic semantic-theme surface for Task 00005 renderer review. */
export function PlatformThemeQaSurface({
  appearance: controlledAppearance,
  isEnabled = import.meta.env.VITE_C4OS_QA_FIXTURES === "1",
}: PlatformThemeQaSurfaceProps) {
  const observedAppearance = useObservedAppearance();
  const appearance = controlledAppearance ?? observedAppearance;
  const reviewedState = usePlatformReviewState();

  if (!isEnabled) {
    return (
      <main className="platform-qa" aria-labelledby="platform-qa-unavailable">
        <section className="platform-panel">
          <h1 id="platform-qa-unavailable">
            QA platform fixtures are disabled
          </h1>
          <p>Launch the deterministic QA build to inspect this route.</p>
        </section>
      </main>
    );
  }

  return (
    <main className="platform-qa" aria-labelledby="platform-qa-title">
      <header className="platform-qa__masthead">
        <div className="platform-qa__identity">
          <div className="platform-qa__mark" aria-hidden="true">
            C4
          </div>
          <div>
            <p className="platform-qa__eyebrow">Native semantic foundation</p>
            <h1 id="platform-qa-title">
              System appearance, one source at a time
            </h1>
          </div>
        </div>
        <p>
          The native snapshot resolves the first visible frame. The webview
          independently follows later system changes without a saved override.
        </p>
      </header>

      <section
        className="platform-qa__theme-strip"
        aria-label="Resolved platform appearance"
      >
        <div>
          <strong>Platform</strong>
          <span>{labelPlatform(appearance.platform)}</span>
        </div>
        <div>
          <strong>Color scheme</strong>
          <span>{appearance.colorScheme === "dark" ? "Dark" : "Light"}</span>
        </div>
        <div>
          <strong>Current signal</strong>
          <span>{labelAppearanceSource(appearance.source)}</span>
        </div>
        <div>
          <strong>Manual override</strong>
          <span>Not stored</span>
        </div>
      </section>

      <div className="platform-qa__grid">
        <section className="platform-panel" aria-labelledby="token-title">
          <header>
            <div>
              <p className="platform-qa__label">Surface hierarchy</p>
              <h2 id="token-title">Semantic tokens</h2>
              <p>Named by purpose so every product surface changes together.</p>
            </div>
          </header>
          <div className="platform-token-stack">
            {TOKEN_SWATCHES.map(([label, token]) => (
              <div key={token}>
                <span
                  aria-hidden="true"
                  style={
                    { "--token-swatch": `var(${token})` } as React.CSSProperties
                  }
                />
                <strong>{label}</strong>
                <code>{token}</code>
              </div>
            ))}
          </div>
        </section>

        <section className="platform-panel" aria-labelledby="states-title">
          <header>
            <div>
              <p className="platform-qa__label">Control matrix</p>
              <h2 id="states-title">Stable component states</h2>
              <p>
                Words and symbols preserve meaning when color is unavailable.
              </p>
            </div>
          </header>
          <div className="platform-state-grid">
            {REVIEW_STATES.map((state) => (
              <Button
                className="platform-button"
                data-demo-state={stateData(state)}
                isDisabled={state === "Disabled"}
                aria-pressed={reviewedState === state}
                key={state}
                onPress={() => setPlatformReviewState(state)}
              >
                {state === "Loading" ? "Loading…" : state}
              </Button>
            ))}
          </div>
          <p className="platform-review-status" role="status">
            Last reviewed state: {reviewedState}
          </p>
        </section>

        <section
          className="platform-panel platform-panel--wide"
          aria-labelledby="feedback-title"
        >
          <header>
            <div>
              <p className="platform-qa__label">Feedback and containment</p>
              <h2 id="feedback-title">Meaning survives theme changes</h2>
              <p>
                Notices retain explicit labels, while the modal owns focus,
                Escape dismissal, backdrop containment, and trigger restoration.
              </p>
            </div>
            <DialogTrigger>
              <Button className="platform-button">Review dialog</Button>
              <ModalOverlay className="platform-dialog-overlay" isDismissable>
                <Modal className="platform-dialog">
                  <Dialog>
                    {({ close }) => (
                      <>
                        <Heading slot="title">
                          Keep unsaved appearance work?
                        </Heading>
                        <p>
                          Theme changes update this dialog in place. Closing it
                          returns focus to Review dialog.
                        </p>
                        <div className="platform-dialog__actions">
                          <Button className="platform-button" onPress={close}>
                            Discard
                          </Button>
                          <Button
                            className="platform-button platform-button--primary"
                            onPress={close}
                          >
                            Keep changes
                          </Button>
                        </div>
                      </>
                    )}
                  </Dialog>
                </Modal>
              </ModalOverlay>
            </DialogTrigger>
          </header>
          <div className="platform-notice-grid">
            <div className="platform-notice" data-state="success">
              <strong>✓ Ready</strong>
              Native appearance resolved before reveal.
            </div>
            <div className="platform-notice" data-state="warning">
              <strong>! Attention</strong>
              Unsaved Settings remain visibly marked.
            </div>
            <div className="platform-notice" data-state="error">
              <strong>× Unavailable</strong>
              Native actions fail closed with a next step.
            </div>
          </div>
        </section>

        <NativePlatformSettingsContent />
      </div>
    </main>
  );
}
