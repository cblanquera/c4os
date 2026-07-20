export const APPEARANCE_CHANGE_EVENT = "c4os:appearance-change";
export const DARK_SCHEME_QUERY = "(prefers-color-scheme: dark)";
export const DEFAULT_NATIVE_SNAPSHOT_TIMEOUT_MS = 4_000;

export type HostPlatform = "macos" | "windows" | "linux";
export type ColorScheme = "light" | "dark";
export type InitialAppearanceSource =
  "macosAppearance" | "webviewPreferredColorScheme" | "semanticFallback";
export type AppearanceSource =
  InitialAppearanceSource | "webviewPrefersColorScheme";

export interface NativeAppearanceSnapshot {
  readonly platform: HostPlatform;
  readonly initialTheme: {
    readonly scheme: ColorScheme;
    readonly source: InitialAppearanceSource;
  };
}

export interface RootAppearance {
  readonly platform: HostPlatform;
  readonly colorScheme: ColorScheme;
  readonly source: AppearanceSource;
}

type MatchMedia = (query: string) => MediaQueryList;

export interface PlatformThemeBootstrapOptions {
  readonly document?: Document;
  readonly fallbackPlatform?: HostPlatform;
  readonly matchMedia?: MatchMedia;
  readonly readNativeSnapshot?: () => Promise<unknown>;
  readonly nativeSnapshotTimeoutMs?: number;
  readonly revealElement?: HTMLElement | null;
}

export interface PlatformThemeController {
  readonly initialAppearance: RootAppearance;
  dispose(): void;
}

function isHostPlatform(value: unknown): value is HostPlatform {
  return value === "macos" || value === "windows" || value === "linux";
}

function isColorScheme(value: unknown): value is ColorScheme {
  return value === "light" || value === "dark";
}

function isInitialAppearanceSource(
  value: unknown,
): value is InitialAppearanceSource {
  return (
    value === "macosAppearance" ||
    value === "webviewPreferredColorScheme" ||
    value === "semanticFallback"
  );
}

function readNativeAppearance(value: unknown): RootAppearance | null {
  if (typeof value !== "object" || value === null) return null;

  const candidate = value as Record<string, unknown>;
  if (
    !isHostPlatform(candidate.platform) ||
    typeof candidate.initialTheme !== "object" ||
    candidate.initialTheme === null
  ) {
    return null;
  }

  const initialTheme = candidate.initialTheme as Record<string, unknown>;
  return isColorScheme(initialTheme.scheme) &&
    isInitialAppearanceSource(initialTheme.source)
    ? {
        platform: candidate.platform,
        colorScheme: initialTheme.scheme,
        source: initialTheme.source,
      }
    : null;
}

function createStaticMediaQueryList(): MediaQueryList {
  return {
    matches: false,
    media: DARK_SCHEME_QUERY,
    onchange: null,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
    addListener: () => undefined,
    removeListener: () => undefined,
    dispatchEvent: () => false,
  };
}

function getDarkSchemeQuery(
  document: Document,
  matchMedia?: MatchMedia,
): MediaQueryList {
  const query =
    matchMedia ?? document.defaultView?.matchMedia?.bind(document.defaultView);
  return query?.(DARK_SCHEME_QUERY) ?? createStaticMediaQueryList();
}

function appearanceFromWebview(
  platform: HostPlatform,
  mediaQuery: MediaQueryList,
): RootAppearance {
  return {
    platform,
    colorScheme: mediaQuery.matches ? "dark" : "light",
    source: "webviewPreferredColorScheme",
  };
}

function applyRootAppearance(
  document: Document,
  appearance: RootAppearance,
): void {
  const root = document.documentElement;
  root.dataset.platform = appearance.platform;
  root.dataset.colorScheme = appearance.colorScheme;
  root.dataset.appearanceSource = appearance.source;
  root.style.colorScheme = appearance.colorScheme;
  document.dispatchEvent(
    new CustomEvent<RootAppearance>(APPEARANCE_CHANGE_EVENT, {
      detail: appearance,
    }),
  );
}

function guardReveal(document: Document, revealElement?: HTMLElement | null) {
  const target = revealElement ?? document.getElementById("root");
  const previousVisibility = target?.style.visibility ?? "";

  document.documentElement.dataset.appearanceReady = "false";
  if (target) target.style.visibility = "hidden";

  return () => {
    document.documentElement.dataset.appearanceReady = "true";
    if (!target) return;

    if (previousVisibility) {
      target.style.visibility = previousVisibility;
    } else {
      target.style.removeProperty("visibility");
    }
  };
}

async function readNativeSnapshotWithin(
  readNativeSnapshot: () => Promise<unknown>,
  timeoutMs: number,
): Promise<unknown> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  try {
    return await Promise.race([
      readNativeSnapshot(),
      new Promise<never>((_, reject) => {
        timer = setTimeout(
          () => reject(new Error("native appearance snapshot timed out")),
          timeoutMs,
        );
      }),
    ]);
  } finally {
    if (timer !== undefined) clearTimeout(timer);
  }
}

/**
 * Resolves the first visible scheme from the native snapshot when valid, then
 * keeps a separate webview media-query listener for live system changes.
 * Theme state is root-only and is never read from or written to browser storage.
 */
export async function bootstrapPlatformTheme({
  document = globalThis.document,
  fallbackPlatform = "macos",
  matchMedia,
  readNativeSnapshot,
  nativeSnapshotTimeoutMs = DEFAULT_NATIVE_SNAPSHOT_TIMEOUT_MS,
  revealElement,
}: PlatformThemeBootstrapOptions = {}): Promise<PlatformThemeController> {
  const reveal = guardReveal(document, revealElement);
  const mediaQuery = getDarkSchemeQuery(document, matchMedia);

  let nativeSnapshot: RootAppearance | null = null;
  if (readNativeSnapshot) {
    try {
      nativeSnapshot = readNativeAppearance(
        await readNativeSnapshotWithin(
          readNativeSnapshot,
          Math.max(1, nativeSnapshotTimeoutMs),
        ),
      );
    } catch {
      nativeSnapshot = null;
    }
  }

  const initialAppearance =
    nativeSnapshot ?? appearanceFromWebview(fallbackPlatform, mediaQuery);

  applyRootAppearance(document, initialAppearance);

  const onSchemeChange = (event: MediaQueryListEvent) => {
    applyRootAppearance(document, {
      platform: initialAppearance.platform,
      colorScheme: event.matches ? "dark" : "light",
      source: "webviewPrefersColorScheme",
    });
  };

  mediaQuery.addEventListener("change", onSchemeChange);
  reveal();

  return {
    initialAppearance,
    dispose() {
      mediaQuery.removeEventListener("change", onSchemeChange);
    },
  };
}

export function readRootAppearance(
  document: Document = globalThis.document,
): RootAppearance {
  const { appearanceSource, colorScheme, platform } =
    document.documentElement.dataset;

  return {
    platform: isHostPlatform(platform) ? platform : "macos",
    colorScheme: isColorScheme(colorScheme) ? colorScheme : "light",
    source:
      appearanceSource === "macosAppearance" ||
      appearanceSource === "webviewPreferredColorScheme" ||
      appearanceSource === "semanticFallback" ||
      appearanceSource === "webviewPrefersColorScheme"
        ? appearanceSource
        : "semanticFallback",
  };
}

export function subscribeToRootAppearance(
  listener: (appearance: RootAppearance) => void,
  document: Document = globalThis.document,
): () => void {
  const onAppearanceChange = () => listener(readRootAppearance(document));
  document.addEventListener(APPEARANCE_CHANGE_EVENT, onAppearanceChange);
  return () =>
    document.removeEventListener(APPEARANCE_CHANGE_EVENT, onAppearanceChange);
}
