import { useCallback, useLayoutEffect, useMemo, useRef } from "react";
import type { KeyboardEvent, PointerEvent } from "react";

import type {
  BrowserViewportFocusIntent,
  BrowserViewportIdentity,
  BrowserViewportLifecycleEvent,
  BrowserViewportRect,
} from "./browser-types";

export interface NativeBrowserViewportProps extends BrowserViewportIdentity {
  readonly accessibleTitle: string;
  readonly onFocusIntent?: (intent: BrowserViewportFocusIntent) => void;
  readonly onLifecycle?: (event: BrowserViewportLifecycleEvent) => void;
}

/**
 * Reserves one renderer-owned rectangle for the Rust-owned native Browser view.
 * It renders no website content and emits only bounded geometry and focus intent.
 */
export function NativeBrowserViewport({
  accessibleTitle,
  artifactId,
  baseRecordRevision,
  controllerGeneration,
  mountGeneration,
  onFocusIntent,
  onLifecycle,
  presentation,
}: NativeBrowserViewportProps) {
  const hostRef = useRef<HTMLDivElement>(null);
  const frameRef = useRef<number | null>(null);
  const lastMeasurementRef = useRef("");
  const lifecycleIdentitiesRef = useRef(
    new Map<string, BrowserViewportIdentity>(),
  );
  const identity = useMemo<BrowserViewportIdentity>(
    () => ({
      artifactId,
      baseRecordRevision,
      controllerGeneration,
      mountGeneration,
      presentation,
    }),
    [
      artifactId,
      baseRecordRevision,
      controllerGeneration,
      mountGeneration,
      presentation,
    ],
  );
  const identityKey = `${artifactId}:${controllerGeneration}:${mountGeneration}:${presentation}`;

  useLayoutEffect(() => {
    lifecycleIdentitiesRef.current.set(identityKey, identity);
  }, [identity, identityKey]);

  const reportMeasurement = useCallback(() => {
    frameRef.current = null;
    const host = hostRef.current;
    if (host === null || onLifecycle === undefined) return;
    const rect = clampedViewportRect(host.getBoundingClientRect());
    const visible =
      document.visibilityState !== "hidden" &&
      rect.width > 0 &&
      rect.height > 0;
    const signature = `${visible}:${rect.x}:${rect.y}:${rect.width}:${rect.height}`;
    if (lastMeasurementRef.current === signature) return;
    const currentIdentity = lifecycleIdentitiesRef.current.get(identityKey);
    if (currentIdentity === undefined) return;
    lastMeasurementRef.current = signature;
    onLifecycle(
      visible
        ? { ...currentIdentity, kind: "geometry", rect, visible: true }
        : { ...currentIdentity, kind: "hidden", rect, visible: false },
    );
  }, [identityKey, onLifecycle]);

  const scheduleMeasurement = useCallback(() => {
    if (frameRef.current !== null) return;
    frameRef.current = window.requestAnimationFrame(reportMeasurement);
  }, [reportMeasurement]);

  useLayoutEffect(() => {
    const host = hostRef.current;
    if (host === null) return;
    const lifecycleIdentities = lifecycleIdentitiesRef.current;
    lastMeasurementRef.current = "";
    const observer =
      typeof ResizeObserver === "undefined"
        ? null
        : new ResizeObserver(scheduleMeasurement);
    const positionObserver =
      typeof MutationObserver === "undefined"
        ? null
        : new MutationObserver(scheduleMeasurement);
    observer?.observe(host);
    const layoutRoot = host.closest("[data-artifact-scroll-region='body']");
    if (layoutRoot !== null) {
      positionObserver?.observe(layoutRoot, {
        attributeFilter: [
          "class",
          "data-artifact-status",
          "data-browser-record-revision",
          "style",
        ],
        attributes: true,
        childList: true,
        subtree: true,
      });
    }
    window.addEventListener("resize", scheduleMeasurement);
    window.addEventListener("scroll", scheduleMeasurement, true);
    document.addEventListener("visibilitychange", scheduleMeasurement);
    scheduleMeasurement();

    return () => {
      observer?.disconnect();
      positionObserver?.disconnect();
      window.removeEventListener("resize", scheduleMeasurement);
      window.removeEventListener("scroll", scheduleMeasurement, true);
      document.removeEventListener("visibilitychange", scheduleMeasurement);
      if (frameRef.current !== null) {
        window.cancelAnimationFrame(frameRef.current);
        frameRef.current = null;
      }
      if (onLifecycle !== undefined) {
        const rect = clampedViewportRect(host.getBoundingClientRect());
        const latestIdentity = lifecycleIdentities.get(identityKey);
        if (latestIdentity !== undefined) {
          onLifecycle({
            ...latestIdentity,
            kind: "hidden",
            rect,
            visible: false,
          });
          onLifecycle({ ...latestIdentity, kind: "detach" });
        }
      }
      lifecycleIdentities.delete(identityKey);
    };
  }, [identityKey, onLifecycle, scheduleMeasurement]);

  const requestFocus = (input: BrowserViewportFocusIntent["input"]) => {
    const currentIdentity = lifecycleIdentitiesRef.current.get(identityKey);
    if (currentIdentity !== undefined) {
      onFocusIntent?.({ ...currentIdentity, input });
    }
  };

  const handleKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (event.key !== "Enter" && event.key !== " ") return;
    event.preventDefault();
    requestFocus("keyboard");
  };

  const handlePointerDown = (event: PointerEvent<HTMLDivElement>) => {
    if (event.button !== 0) return;
    requestFocus("pointer");
  };

  return (
    <div
      aria-label={`Native Browser viewport: ${accessibleTitle}`}
      className="artifact-browser__viewport"
      data-browser-artifact-id={artifactId}
      data-browser-record-revision={baseRecordRevision}
      data-browser-controller-generation={controllerGeneration}
      data-browser-mount-generation={mountGeneration}
      data-browser-presentation={presentation}
      onKeyDown={handleKeyDown}
      onPointerDown={handlePointerDown}
      ref={hostRef}
      role="group"
      tabIndex={0}
    />
  );
}

/** Intersects native DOM geometry with the visible window in CSS pixels. */
function clampedViewportRect(rect: DOMRect): BrowserViewportRect {
  const viewportWidth = finiteNonnegative(window.innerWidth);
  const viewportHeight = finiteNonnegative(window.innerHeight);
  const left = clampFinite(rect.left, 0, viewportWidth);
  const top = clampFinite(rect.top, 0, viewportHeight);
  const right = clampFinite(rect.right, left, viewportWidth);
  const bottom = clampFinite(rect.bottom, top, viewportHeight);
  return {
    x: left,
    y: top,
    width: finiteNonnegative(right - left),
    height: finiteNonnegative(bottom - top),
  };
}

function finiteNonnegative(value: number): number {
  return Number.isFinite(value) ? Math.max(0, value) : 0;
}

function clampFinite(value: number, minimum: number, maximum: number): number {
  if (!Number.isFinite(value)) return minimum;
  return Math.min(maximum, Math.max(minimum, value));
}
